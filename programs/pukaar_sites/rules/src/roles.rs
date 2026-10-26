//! The role rules, ported from `pukaar_core` `reducer.rs` (RoleGrant, RoleRevoke, SuperAdmin*).
//!
//! Claim and compare: the instruction carries `Actor { account, claimed }`. Each apply sees one shard:
//! - the signer's `Member` (inspect) runs `check_signer`: the stored role equals the claimed one;
//! - the `Site` runs `check_site_claim`: the signer holds a super admin slot iff it claims `SuperAdmin`,
//!   and `check_not_super` for the targets of a role change or a revoke;
//! - each target runs its `apply_*` with the claim.
//!
//! The transaction is atomic, so a role instruction must touch the signer `Member` and the `Site`.

use crate::records::*;
use crate::{CheckpointRecord, RuleError};
use borsh::{BorshDeserialize, BorshSerialize};

/// A stored `Role` plus the super admin slot of `Site`. Ordered: each rank holds every right of the ranks below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, BorshSerialize, BorshDeserialize)]
pub enum Rank {
    Resident,
    Steward,
    Admin,
    SuperAdmin,
}

impl Rank {
    /// The role stored in `Member`: a super admin holds `Admin`.
    pub fn role(self) -> Role {
        match self {
            Rank::Resident => Role::Resident,
            Rank::Steward => Role::Steward,
            Rank::Admin | Rank::SuperAdmin => Role::Admin,
        }
    }
}

impl From<Role> for Rank {
    fn from(r: Role) -> Self {
        match r {
            Role::Resident => Rank::Resident,
            Role::Steward => Rank::Steward,
            Role::Admin => Rank::Admin,
        }
    }
}

/// The signer of an instruction and the rank it claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Actor {
    pub account: AccountId,
    pub claimed: Rank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Approve a join request into this role.
    Approve(Role),
    /// Change an existing member to this role.
    SetRole(Role),
    Revoke,
    SuperAdminAdd,
    SuperAdminRemove,
    SuperAdminTransfer,
    /// Refresh a profile pointer (Decision 7: admins only).
    SetProfile,
    /// `set_details`, `set_places`, `set_keyring` (O6).
    SetSite,
    Anchor,
}

/// The rights table. `target` is the rank of the subject, `None` when it has no active record
/// (approve) or when it is not a super admin and the shard cannot tell more (super admin ops on `Site`).
pub fn can(actor: Rank, action: Action, target: Option<Rank>) -> bool {
    use Rank::*;
    let admin = actor >= Admin;
    let sup = actor == SuperAdmin;
    match action {
        Action::Anchor => true,
        Action::SetProfile | Action::SetSite => admin,
        // Only a super admin gives the admin role.
        Action::Approve(r) => admin && target.is_none() && (sup || r != Role::Admin),
        // A super admin's role never changes here; the admin role (giving it, or changing an admin's) is the super admins' alone.
        Action::SetRole(r) => admin && target.is_some_and(|t| t != SuperAdmin && (sup || (t != Admin && r != Role::Admin))),
        // Never a super admin; an admin only by a super admin.
        Action::Revoke => admin && target.is_some_and(|t| t != SuperAdmin && (sup || t != Admin)),
        Action::SuperAdminAdd | Action::SuperAdminTransfer => sup && target != Some(SuperAdmin),
        Action::SuperAdminRemove => sup && target == Some(SuperAdmin),
    }
}

fn allow(actor: &Actor, action: Action, target: Option<Rank>) -> Result<(), RuleError> {
    if can(actor.claimed, action, target) { Ok(()) } else { Err(RuleError::Forbidden) }
}

/// The status of a record at chain time `now_ms`. P1: the role, or removed. P4 adds the notice
/// deadline and the expiry, so the signature stays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effective {
    Active(Role),
    Removed,
}

pub fn effective_status(m: &Member, now_ms: u64) -> Effective {
    let _ = now_ms; // P4: notice deadline and expires_at_ms
    if m.status == MemberStatus::Removed || m.revoked.is_some() { Effective::Removed } else { Effective::Active(m.role) }
}

fn active_rank(m: &Member, now_ms: u64) -> Result<Rank, RuleError> {
    match effective_status(m, now_ms) {
        Effective::Active(r) => Ok(r.into()),
        Effective::Removed => Err(RuleError::NotActive),
    }
}

// ---- signer checks (claim and compare) ----

/// Apply of the signer's `Member` (inspect): it belongs to the signer in this site, it is active,
/// and its stored role is the claimed one.
pub fn check_signer(m: &Member, site_id: &[u8; 32], actor: &Actor, now_ms: u64) -> Result<(), RuleError> {
    if m.site_id != *site_id || m.user != actor.account {
        return Err(RuleError::WrongRecord);
    }
    if active_rank(m, now_ms)?.role() != actor.claimed.role() {
        return Err(RuleError::ClaimMismatch);
    }
    Ok(())
}

/// Apply of `Site`: the signer holds a super admin slot iff it claims `SuperAdmin`.
pub fn check_site_claim(site: &Site, actor: &Actor) -> Result<(), RuleError> {
    if site.is_super(&actor.account) != (actor.claimed == Rank::SuperAdmin) {
        return Err(RuleError::ClaimMismatch);
    }
    Ok(())
}

/// Apply of `Site` for `set_role_batch` and `revoke_batch`: no target is a super admin.
pub fn check_not_super(site: &Site, targets: &[AccountId]) -> Result<(), RuleError> {
    if targets.iter().any(|t| site.is_super(t)) { Err(RuleError::SuperAdminTarget) } else { Ok(()) }
}

/// Apply of the signer's `Member` for `anchor`: any active member of the checkpoint's site.
pub fn check_anchor_signer(m: &Member, rec: &CheckpointRecord, signer: &AccountId, now_ms: u64) -> Result<(), RuleError> {
    if m.site_id != rec.site_id || m.user != *signer {
        return Err(RuleError::WrongRecord);
    }
    active_rank(m, now_ms).map(|_| ())
}

// ---- create_site ----

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CreateSite {
    /// The signer.
    pub creator: AccountId,
    pub salt: [u8; 32],
    /// Claimed; must equal `seed::site_id(creator, salt)`.
    pub site_id: [u8; 32],
    pub pukaar_key: [u8; 32],
    pub details_cid: String,
    pub details_sha256: [u8; 32],
    pub places_cid: String,
    pub places_sha256: [u8; 32],
    pub keyring_cid: String,
    pub keyring_sha256: [u8; 32],
    pub gates: GateConfig,
    pub access_duration_ms: u64,
    pub renew_window_ms: u64,
}

/// The new `Site` and the creator's `Member`: an admin in super admin slot 0, roster entry 0, key epoch 1.
pub fn apply_create(a: &CreateSite, now_ms: u64) -> Result<(Site, Member), RuleError> {
    if a.site_id != seed::site_id(&a.creator, &a.salt) {
        return Err(RuleError::WrongSiteId);
    }
    for cid in [&a.details_cid, &a.places_cid, &a.keyring_cid] {
        check_cid(cid)?;
    }
    if a.gates != GateConfig::default() {
        return Err(RuleError::GatesNotInP1);
    }
    if a.access_duration_ms != 0 {
        return Err(RuleError::ExpiryNotInP1);
    }
    let site = Site {
        version: RECORD_VERSION,
        site_id: a.site_id,
        creator: a.creator,
        salt: a.salt,
        super_admins: [Some(a.creator), None],
        details_cid: a.details_cid.clone(),
        details_sha256: a.details_sha256,
        places_cid: a.places_cid.clone(),
        places_sha256: a.places_sha256,
        key_epoch: 1,
        keyring_cid: a.keyring_cid.clone(),
        keyring_sha256: a.keyring_sha256,
        gates: a.gates.clone(),
        access_duration_ms: 0,
        renew_window_ms: a.renew_window_ms,
        roster_len: 1,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };
    let member = Member {
        version: RECORD_VERSION,
        site_id: a.site_id,
        user: a.creator,
        pukaar_key: a.pukaar_key,
        role: Role::Admin,
        status: MemberStatus::Active,
        granted_by: a.creator,
        granted_at_ms: now_ms,
        expires_at_ms: 0,
        notice: None,
        revoked: None,
        rights_override: 0,
        profile_cid: String::new(),
        profile_sha256: [0; 32],
        profile_version: 0,
        gate_attestation: None,
        request_id: [0; 32],
        requested_at_ms: 0,
        last_req_nonce: 0,
    };
    Ok((site, member))
}

// ---- approve_batch ----

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct ApproveItem {
    pub request: [u8; JOIN_REQ_LEN],
    pub role: Role,
    pub profile_cid: String,
    pub profile_sha256: [u8; 32],
    pub profile_version: u64,
}

/// Apply of one target `Member`. `pre` is `None` for a new record, or a removed record that comes back.
pub fn apply_approve(
    pre: Option<&Member>,
    item: &ApproveItem,
    site_id: &[u8; 32],
    actor: &Actor,
    now_ms: u64,
) -> Result<Member, RuleError> {
    allow(actor, Action::Approve(item.role), None)?;
    let body = check_join_request(&item.request, site_id, pre, now_ms)?;
    if pre.is_some_and(|m| effective_status(m, now_ms) != Effective::Removed) {
        return Err(RuleError::AlreadyMember);
    }
    check_cid(&item.profile_cid)?;
    if item.profile_version <= pre.map_or(0, |m| m.profile_version) {
        return Err(RuleError::StaleProfile);
    }
    Ok(Member {
        version: RECORD_VERSION,
        site_id: *site_id,
        user: body.user_account,
        pukaar_key: body.pukaar_ed_pk,
        role: item.role,
        status: MemberStatus::Active,
        granted_by: actor.account,
        granted_at_ms: now_ms,
        expires_at_ms: 0,
        notice: None,
        revoked: None,
        rights_override: 0,
        profile_cid: item.profile_cid.clone(),
        profile_sha256: item.profile_sha256,
        profile_version: item.profile_version,
        gate_attestation: None,
        request_id: request_id(&item.request),
        requested_at_ms: 0,
        last_req_nonce: body.nonce,
    })
}

/// Apply of `Site` in `approve_batch`: the claimed `roster_len` is the stored one, so of two admins
/// who approve at once only the first wins. `new_members` counts the items with no record before.
pub fn apply_approve_site(
    site: &Site,
    actor: &Actor,
    expected_roster_len: u32,
    new_members: u32,
    keyring_cid: &str,
    keyring_sha256: [u8; 32],
    now_ms: u64,
) -> Result<Site, RuleError> {
    check_site_claim(site, actor)?;
    if actor.claimed < Rank::Admin {
        return Err(RuleError::Forbidden);
    }
    if site.roster_len != expected_roster_len {
        return Err(RuleError::StaleRoster);
    }
    check_cid(keyring_cid)?;
    let mut s = site.clone();
    s.roster_len = s.roster_len.checked_add(new_members).ok_or(RuleError::RosterFull)?;
    s.keyring_cid = keyring_cid.into();
    s.keyring_sha256 = keyring_sha256;
    s.updated_at_ms = now_ms;
    Ok(s)
}

// ---- set_role_batch, revoke_batch ----

/// Apply of one target `Member`. The `Site` apply runs `check_not_super` on the targets.
pub fn apply_set_role(target: &Member, role: Role, actor: &Actor, now_ms: u64) -> Result<Member, RuleError> {
    allow(actor, Action::SetRole(role), Some(active_rank(target, now_ms)?))?;
    let mut m = target.clone();
    m.role = role;
    m.granted_by = actor.account;
    m.granted_at_ms = now_ms;
    m.notice = None;
    Ok(m)
}

/// Apply of one target `Member`: removed, with who, when and the SHA-256 of the reason text.
pub fn apply_revoke(target: &Member, reason_sha256: [u8; 32], actor: &Actor, now_ms: u64) -> Result<Member, RuleError> {
    if reason_sha256 == [0; 32] {
        return Err(RuleError::EmptyReason);
    }
    allow(actor, Action::Revoke, Some(active_rank(target, now_ms)?))?;
    let mut m = target.clone();
    m.status = MemberStatus::Removed;
    m.notice = None;
    m.revoked = Some(Revocation { by: actor.account, at_ms: now_ms, reason_sha256 });
    Ok(m)
}

/// Apply of `Site` in `revoke_batch`: never a super admin; the site key rotates.
pub fn apply_revoke_site(
    site: &Site,
    actor: &Actor,
    targets: &[AccountId],
    keyring_cid: &str,
    keyring_sha256: [u8; 32],
    now_ms: u64,
) -> Result<Site, RuleError> {
    check_site_claim(site, actor)?;
    if actor.claimed < Rank::Admin {
        return Err(RuleError::Forbidden);
    }
    if targets.is_empty() {
        return Err(RuleError::EmptyBatch);
    }
    check_not_super(site, targets)?;
    check_cid(keyring_cid)?;
    let mut s = site.clone();
    s.key_epoch += 1;
    s.keyring_cid = keyring_cid.into();
    s.keyring_sha256 = keyring_sha256;
    s.updated_at_ms = now_ms;
    Ok(s)
}

// ---- super admins: at most 2, never remove self, transfer steps the author down to plain admin ----

pub fn apply_super_admin_add(site: &Site, actor: &Actor, target: &AccountId, now_ms: u64) -> Result<Site, RuleError> {
    check_site_claim(site, actor)?;
    allow(actor, Action::SuperAdminAdd, None)?;
    if site.is_super(target) {
        return Err(RuleError::AlreadySuperAdmin);
    }
    let mut s = site.clone();
    *s.super_admins.iter_mut().find(|x| x.is_none()).ok_or(RuleError::SuperAdminsFull)? = Some(*target);
    s.updated_at_ms = now_ms;
    Ok(s)
}

/// The other super admin loses the slot and stays an admin (its `Member` is not written).
pub fn apply_super_admin_remove(site: &Site, actor: &Actor, target: &AccountId, now_ms: u64) -> Result<Site, RuleError> {
    check_site_claim(site, actor)?;
    if *target == actor.account {
        return Err(RuleError::SelfTarget);
    }
    if !site.is_super(target) {
        return Err(RuleError::NotSuperAdmin);
    }
    allow(actor, Action::SuperAdminRemove, Some(Rank::SuperAdmin))?;
    let mut s = site.clone();
    s.super_admins.iter_mut().filter(|x| **x == Some(*target)).for_each(|x| *x = None);
    s.updated_at_ms = now_ms;
    Ok(s)
}

/// `to` takes the signer's slot. The signer stays an admin (its `Member` is not written).
pub fn apply_super_admin_transfer(site: &Site, actor: &Actor, to: &AccountId, now_ms: u64) -> Result<Site, RuleError> {
    check_site_claim(site, actor)?;
    if *to == actor.account {
        return Err(RuleError::SelfTarget);
    }
    allow(actor, Action::SuperAdminTransfer, None)?;
    if site.is_super(to) {
        return Err(RuleError::AlreadySuperAdmin);
    }
    let mut s = site.clone();
    s.super_admins.iter_mut().filter(|x| **x == Some(actor.account)).for_each(|x| *x = Some(*to));
    s.updated_at_ms = now_ms;
    Ok(s)
}

/// Apply of the target `Member` in `super_admin_add` and `super_admin_transfer`: any active member, now an admin.
pub fn apply_super_admin_promote(target: &Member, actor: &Actor, now_ms: u64) -> Result<Member, RuleError> {
    allow(actor, Action::SuperAdminAdd, Some(active_rank(target, now_ms)?))?;
    let mut m = target.clone();
    m.role = Role::Admin;
    m.granted_by = actor.account;
    m.granted_at_ms = now_ms;
    m.notice = None;
    Ok(m)
}

// ---- pointers: profile (per member), details, places, keyring (per site) ----

/// Apply of one target `Member` in `set_profile_batch`: an admin signs, the version must grow.
pub fn apply_set_profile(
    target: &Member,
    cid: &str,
    sha256: [u8; 32],
    version: u64,
    actor: &Actor,
    now_ms: u64,
) -> Result<Member, RuleError> {
    allow(actor, Action::SetProfile, Some(active_rank(target, now_ms)?))?;
    check_cid(cid)?;
    if version <= target.profile_version {
        return Err(RuleError::StaleProfile);
    }
    let mut m = target.clone();
    m.profile_cid = cid.into();
    m.profile_sha256 = sha256;
    m.profile_version = version;
    Ok(m)
}

fn set_site_pointer(
    site: &Site,
    actor: &Actor,
    cid: &str,
    sha256: [u8; 32],
    now_ms: u64,
    field: fn(&mut Site) -> (&mut String, &mut [u8; 32]),
) -> Result<Site, RuleError> {
    check_site_claim(site, actor)?;
    allow(actor, Action::SetSite, None)?;
    check_cid(cid)?;
    let mut s = site.clone();
    let (c, h) = field(&mut s);
    *c = cid.into();
    *h = sha256;
    s.updated_at_ms = now_ms;
    Ok(s)
}

pub fn apply_set_details(site: &Site, actor: &Actor, cid: &str, sha256: [u8; 32], now_ms: u64) -> Result<Site, RuleError> {
    set_site_pointer(site, actor, cid, sha256, now_ms, |s| (&mut s.details_cid, &mut s.details_sha256))
}

pub fn apply_set_places(site: &Site, actor: &Actor, cid: &str, sha256: [u8; 32], now_ms: u64) -> Result<Site, RuleError> {
    set_site_pointer(site, actor, cid, sha256, now_ms, |s| (&mut s.places_cid, &mut s.places_sha256))
}

/// Replaces the keyring pointer only; `key_epoch` moves with `revoke_batch`.
pub fn apply_set_keyring(site: &Site, actor: &Actor, cid: &str, sha256: [u8; 32], now_ms: u64) -> Result<Site, RuleError> {
    set_site_pointer(site, actor, cid, sha256, now_ms, |s| (&mut s.keyring_cid, &mut s.keyring_sha256))
}
