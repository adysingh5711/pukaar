//! The arguments of the site and member instructions, and the apply of their effects.
//!
//! The plan (in the guest) turns each instruction into one effect per shard. `apply_site_effect` runs
//! the matching rule on that shard. A shard that holds a record is decoded here; an empty shard is
//! `None` for the one effect that may create it.

use crate::records::*;
use crate::roles::*;
use crate::{CheckpointRecord, Effect, RuleError};
use borsh::{BorshDeserialize, BorshSerialize};

/// Who signs, for which site, at which claimed time and with which claimed rank.
/// The signer account is the one authorized account of the call, so it is not repeated here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Auth {
    pub site_id: [u8; 32],
    pub now_ms: u64,
    pub claimed: Rank,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct AnchorIx {
    pub record: CheckpointRecord,
    pub now_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CreateSiteIx {
    pub args: CreateSite,
    pub now_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum Pointer {
    Details,
    Places,
    Keyring,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct SetPointerIx {
    pub auth: Auth,
    pub cid: String,
    pub sha256: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct ApproveEntry {
    pub item: ApproveItem,
    /// Claim: the member account holds no record yet. The member shard compares it with the chain.
    pub is_new: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct ApproveBatchIx {
    pub auth: Auth,
    /// The `roster_len` the client read. A second admin who approves at once fails on it.
    pub roster_len: u32,
    pub keyring_cid: String,
    pub keyring_sha256: [u8; 32],
    pub items: Vec<ApproveEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct RoleItem {
    pub user: AccountId,
    pub role: Role,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct SetRoleBatchIx {
    pub auth: Auth,
    pub items: Vec<RoleItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct RevokeItem {
    pub user: AccountId,
    pub reason_sha256: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct RevokeBatchIx {
    pub auth: Auth,
    /// The new keyring (the site key rotates with every revoke).
    pub keyring_cid: String,
    pub keyring_sha256: [u8; 32],
    pub items: Vec<RevokeItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct ProfileItem {
    pub user: AccountId,
    pub cid: String,
    pub sha256: [u8; 32],
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct SetProfileBatchIx {
    pub auth: Auth,
    pub items: Vec<ProfileItem>,
}

/// `super_admin_add`, `super_admin_remove` and `super_admin_transfer`.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct SuperAdminIx {
    pub auth: Auth,
    pub target: AccountId,
}

fn member(pre: &[u8]) -> Result<Member, RuleError> {
    Member::decode(pre).ok_or(RuleError::NoRecord)
}

fn site(pre: &[u8], site_id: &[u8; 32]) -> Result<Site, RuleError> {
    let s = Site::decode(pre).ok_or(RuleError::NoRecord)?;
    if s.site_id == *site_id { Ok(s) } else { Err(RuleError::WrongSiteId) }
}

/// The new shard bytes of one Site or Member effect, or `None` to keep the shard.
/// Every other effect is in `apply_effect`.
pub fn apply_site_effect(effect: &Effect, pre: &[u8]) -> Result<Option<Vec<u8>>, RuleError> {
    let write_site = |r: Result<Site, RuleError>| r.map(|s| Some(s.encode()));
    let write_member = |r: Result<Member, RuleError>| r.map(|m| Some(m.encode()));
    match effect {
        Effect::Signer { site_id, actor, now_ms } => check_signer(&member(pre)?, site_id, actor, *now_ms).map(|()| None),
        Effect::AnchorSigner { record, signer, now_ms } => check_anchor_signer(&member(pre)?, record, signer, *now_ms).map(|()| None),
        Effect::SiteGuard { site_id, actor, not_super } => {
            let s = site(pre, site_id)?;
            check_site_claim(&s, actor)?;
            check_not_super(&s, not_super).map(|()| None)
        }
        Effect::CreateSite { args, now_ms } | Effect::CreateMember { args, now_ms } => {
            if !pre.is_empty() {
                return Err(RuleError::RecordExists);
            }
            let (s, m) = apply_create(args, *now_ms)?;
            Ok(Some(if matches!(effect, Effect::CreateSite { .. }) { s.encode() } else { m.encode() }))
        }
        Effect::SetPointer { kind, site_id, actor, cid, sha256, now_ms } => {
            let f: fn(&Site, &Actor, &str, [u8; 32], u64) -> Result<Site, RuleError> = match kind {
                Pointer::Details => apply_set_details,
                Pointer::Places => apply_set_places,
                Pointer::Keyring => apply_set_keyring,
            };
            write_site(f(&site(pre, site_id)?, actor, cid, *sha256, *now_ms))
        }
        Effect::Approve { site_id, actor, item, is_new, now_ms } => {
            let old = if pre.is_empty() { None } else { Some(member(pre)?) };
            if old.is_none() != *is_new {
                return Err(RuleError::ClaimMismatch);
            }
            write_member(apply_approve(old.as_ref(), item, site_id, actor, *now_ms))
        }
        Effect::ApproveSite { site_id, actor, roster_len, new_members, keyring_cid, keyring_sha256, now_ms } => write_site(
            apply_approve_site(&site(pre, site_id)?, actor, *roster_len, *new_members, keyring_cid, *keyring_sha256, *now_ms),
        ),
        Effect::SetRole { actor, role, now_ms } => write_member(apply_set_role(&member(pre)?, *role, actor, *now_ms)),
        Effect::Revoke { actor, reason_sha256, now_ms } => write_member(apply_revoke(&member(pre)?, *reason_sha256, actor, *now_ms)),
        Effect::RevokeSite { site_id, actor, targets, keyring_cid, keyring_sha256, now_ms } => {
            write_site(apply_revoke_site(&site(pre, site_id)?, actor, targets, keyring_cid, *keyring_sha256, *now_ms))
        }
        Effect::SetProfile { actor, cid, sha256, version, now_ms } => {
            write_member(apply_set_profile(&member(pre)?, cid, *sha256, *version, actor, *now_ms))
        }
        Effect::SuperAdd { site_id, actor, target, now_ms } => write_site(apply_super_admin_add(&site(pre, site_id)?, actor, target, *now_ms)),
        Effect::SuperRemove { site_id, actor, target, now_ms } => {
            write_site(apply_super_admin_remove(&site(pre, site_id)?, actor, target, *now_ms))
        }
        Effect::SuperTransfer { site_id, actor, target, now_ms } => {
            write_site(apply_super_admin_transfer(&site(pre, site_id)?, actor, target, *now_ms))
        }
        Effect::Promote { actor, now_ms } => write_member(apply_super_admin_promote(&member(pre)?, actor, *now_ms)),
        // The other effects are handled by `apply_effect`.
        _ => unreachable!("apply_effect handles the probe and anchor effects"),
    }
}
