//! Table tests of the Site and Member rules. `W` plays the program: for each instruction it runs the
//! apply of every shard (signer `Member`, `Site`, targets) and commits only if all pass.
//! Role cases are ported from `pukaar_core/rust-lib/logic/tests/{super_admin,lifecycle}.rs`.
//! The notice cases there wait for P4.
#![allow(clippy::type_complexity)]

use pukaar_sites_rules::*;
use std::collections::BTreeMap;

const NOW: u64 = 1_000_000;
// People of the ported tests: the genesis super admin, the deputy (admin), the steward, residents.
const A: u8 = 1;
const D: u8 = 2;
const S: u8 = 3;
const ASHA: u8 = 4;
const RAVI: u8 = 5;
const MEERA: u8 = 6;
const OUT: u8 = 7;

fn acct(n: u8) -> AccountId {
    lez_account_id(&[n; 32])
}

fn h(n: u8) -> [u8; 32] {
    [n; 32]
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn create_args() -> CreateSite {
    CreateSite {
        creator: acct(A),
        salt: h(9),
        site_id: seed::site_id(&acct(A), &h(9)),
        pukaar_key: h(0xA1),
        details_cid: "details1".into(),
        details_sha256: h(11),
        places_cid: "places1".into(),
        places_sha256: h(12),
        keyring_cid: "keyring1".into(),
        keyring_sha256: h(13),
        gates: GateConfig::default(),
        access_duration_ms: 0,
        renew_window_ms: 0,
    }
}

fn body(n: u8, site_id: [u8; 32], nonce: u64) -> JoinBody {
    JoinBody {
        site_id,
        user_account: acct(n),
        user_schnorr_pk: [n; 32],
        pukaar_ed_pk: [n + 100; 32],
        nonce,
        valid_until_ms: NOW + 1000,
    }
}

/// Body ‖ two unchecked signatures (O11 off).
fn req(b: &JoinBody) -> [u8; JOIN_REQ_LEN] {
    [&b.encode()[..], &[0xEE; 128]].concat().try_into().unwrap()
}

type R = Result<(), RuleError>;

struct W {
    site: Site,
    m: BTreeMap<AccountId, Member>,
    nonce: u64,
}

impl W {
    fn new() -> W {
        let (site, creator) = apply_create(&create_args(), NOW).unwrap();
        W { site, m: BTreeMap::from([(acct(A), creator)]), nonce: 0 }
    }

    /// `Site::new()` of the ported tests, with the steward promoted to admin "Deputy".
    fn deputy_site() -> W {
        let mut w = W::new();
        let a = w.me(A);
        w.approve(a, &[(D, Role::Admin), (S, Role::Steward), (ASHA, Role::Resident), (RAVI, Role::Resident), (MEERA, Role::Resident)])
            .unwrap();
        w
    }

    /// The true rank, as an honest client claims it.
    fn rank(&self, n: u8) -> Rank {
        if self.site.is_super(&acct(n)) { Rank::SuperAdmin } else { self.m[&acct(n)].role.into() }
    }

    fn me(&self, n: u8) -> Actor {
        Actor { account: acct(n), claimed: self.rank(n) }
    }

    fn get(&self, n: u8) -> Result<&Member, RuleError> {
        self.m.get(&acct(n)).ok_or(RuleError::NotActive)
    }

    fn signer(&self, a: &Actor) -> R {
        let m = self.m.get(&a.account).ok_or(RuleError::NotActive)?;
        check_signer(m, &self.site.site_id, a, NOW)
    }

    fn approve(&mut self, a: Actor, items: &[(u8, Role)]) -> R {
        let len = self.site.roster_len;
        self.approve_at(a, items, len)
    }

    fn approve_at(&mut self, a: Actor, items: &[(u8, Role)], expected_roster_len: u32) -> R {
        self.signer(&a)?;
        let sid = self.site.site_id;
        let (mut out, mut new) = (vec![], 0);
        for &(n, role) in items {
            let pre = self.m.get(&acct(n));
            new += u32::from(pre.is_none());
            self.nonce += 1;
            let item = ApproveItem {
                request: req(&body(n, sid, self.nonce)),
                role,
                profile_cid: "profile1".into(),
                profile_sha256: h(20),
                profile_version: pre.map_or(1, |m| m.profile_version + 1),
            };
            out.push(apply_approve(pre, &item, &sid, &a, NOW)?);
        }
        let site = apply_approve_site(&self.site, &a, expected_roster_len, new, "keyring2", h(14), NOW)?;
        self.site = site;
        out.into_iter().for_each(|m| drop(self.m.insert(m.user, m)));
        Ok(())
    }

    fn set_role(&mut self, a: Actor, n: u8, role: Role) -> R {
        self.signer(&a)?;
        check_site_claim(&self.site, &a)?;
        check_not_super(&self.site, &[acct(n)])?;
        let m = apply_set_role(self.get(n)?, role, &a, NOW)?;
        self.m.insert(m.user, m);
        Ok(())
    }

    fn revoke(&mut self, a: Actor, n: u8) -> R {
        self.revoke_why(a, n, h(5))
    }

    fn revoke_why(&mut self, a: Actor, n: u8, reason: [u8; 32]) -> R {
        self.signer(&a)?;
        let site = apply_revoke_site(&self.site, &a, &[acct(n)], "keyring3", h(15), NOW)?;
        let m = apply_revoke(self.get(n)?, reason, &a, NOW)?;
        self.site = site;
        self.m.insert(m.user, m);
        Ok(())
    }

    fn sup_add(&mut self, a: Actor, n: u8) -> R {
        self.signer(&a)?;
        let site = apply_super_admin_add(&self.site, &a, &acct(n), NOW)?;
        let m = apply_super_admin_promote(self.get(n)?, &a, NOW)?;
        self.site = site;
        self.m.insert(m.user, m);
        Ok(())
    }

    fn sup_remove(&mut self, a: Actor, n: u8) -> R {
        self.signer(&a)?;
        self.site = apply_super_admin_remove(&self.site, &a, &acct(n), NOW)?;
        Ok(())
    }

    fn sup_transfer(&mut self, a: Actor, n: u8) -> R {
        self.signer(&a)?;
        let site = apply_super_admin_transfer(&self.site, &a, &acct(n), NOW)?;
        let m = apply_super_admin_promote(self.get(n)?, &a, NOW)?;
        self.site = site;
        self.m.insert(m.user, m);
        Ok(())
    }

    fn set_profile(&mut self, a: Actor, n: u8, version: u64) -> R {
        self.signer(&a)?;
        check_site_claim(&self.site, &a)?;
        let m = apply_set_profile(self.get(n)?, "profile2", h(21), version, &a, NOW)?;
        self.m.insert(m.user, m);
        Ok(())
    }

    fn anchor(&self, n: u8) -> R {
        let rec = CheckpointRecord {
            site_id: self.site.site_id,
            heads_root: h(30),
            n_events: 1,
            signer_pk: self.get(n)?.pukaar_key,
            sig_r: h(32),
            sig_s: h(33),
        };
        check_anchor_signer(self.get(n)?, &rec, &acct(n), NOW)?;
        apply_anchor(&rec, &[]).map(|_| ())
    }

    fn supers(&self) -> Vec<AccountId> {
        self.site.super_admins.iter().flatten().copied().collect()
    }
}

fn claim(n: u8, claimed: Rank) -> Actor {
    Actor { account: acct(n), claimed }
}

/// Run each step in order on one world; each row says what it must return.
fn run(w: &mut W, steps: &[(&str, fn(&mut W) -> R, R)]) {
    for (what, step, want) in steps {
        assert_eq!(&step(w), want, "{what}");
    }
}

// ---------- the rights table ----------

#[test]
fn can_table() {
    use Action::*;
    use Rank::{Admin as Ad, Resident as Re, Steward as St, SuperAdmin as Su};
    let rows: &[(Rank, Action, Option<Rank>, bool)] = &[
        // approve: admin or higher; only a super admin gives the admin role
        (Re, Approve(Role::Resident), None, false),
        (St, Approve(Role::Resident), None, false),
        (Ad, Approve(Role::Resident), None, true),
        (Ad, Approve(Role::Steward), None, true),
        (Ad, Approve(Role::Admin), None, false),
        (Su, Approve(Role::Admin), None, true),
        (Su, Approve(Role::Resident), Some(Re), false), // a record exists: not an approval
        // set_role: admins change residents and stewards; the admin role is the super admins'; a super admin never changes
        (St, SetRole(Role::Steward), Some(Re), false),
        (Ad, SetRole(Role::Steward), Some(Re), true),
        (Ad, SetRole(Role::Resident), Some(St), true),
        (Ad, SetRole(Role::Admin), Some(Re), false),
        (Ad, SetRole(Role::Steward), Some(Ad), false),
        (Ad, SetRole(Role::Resident), Some(Su), false),
        (Su, SetRole(Role::Admin), Some(Re), true),
        (Su, SetRole(Role::Steward), Some(Ad), true),
        (Su, SetRole(Role::Steward), Some(Su), false),
        (Ad, SetRole(Role::Steward), None, false),
        // revoke: never a super admin; an admin only by a super admin
        (Re, Revoke, Some(Re), false),
        (St, Revoke, Some(Re), false),
        (Ad, Revoke, Some(Re), true),
        (Ad, Revoke, Some(St), true),
        (Ad, Revoke, Some(Ad), false),
        (Ad, Revoke, Some(Su), false),
        (Su, Revoke, Some(Ad), true),
        (Su, Revoke, Some(Su), false),
        (Ad, Revoke, None, false),
        // super admin slots
        (Ad, SuperAdminAdd, Some(Re), false),
        (Su, SuperAdminAdd, Some(Re), true),
        (Su, SuperAdminAdd, None, true),
        (Su, SuperAdminAdd, Some(Su), false),
        (Ad, SuperAdminRemove, Some(Su), false),
        (Su, SuperAdminRemove, Some(Su), true),
        (Su, SuperAdminRemove, Some(Ad), false),
        (Ad, SuperAdminTransfer, None, false),
        (Su, SuperAdminTransfer, Some(St), true),
        (Su, SuperAdminTransfer, Some(Su), false),
        // pointers: admins only
        (St, SetProfile, Some(Re), false),
        (Ad, SetProfile, Some(Su), true),
        (St, SetSite, None, false),
        (Ad, SetSite, None, true),
        // anchor: any active member
        (Re, Anchor, None, true),
    ];
    for &(actor, action, target, want) in rows {
        assert_eq!(can(actor, action, target), want, "{actor:?} {action:?} {target:?}");
    }
}

// ---------- ported: super_admin.rs ----------

#[test]
fn role_change_and_revoke_cannot_touch_the_genesis_super_admin() {
    let mut w = W::deputy_site();
    run(&mut w, &[
        ("deputy demotes the super admin", |w| w.set_role(w.me(D), A, Role::Resident), Err(RuleError::SuperAdminTarget)),
        ("deputy revokes the super admin", |w| w.revoke(w.me(D), A), Err(RuleError::SuperAdminTarget)),
        ("the super admin changes own role", |w| w.set_role(w.me(A), A, Role::Steward), Err(RuleError::SuperAdminTarget)),
        ("the super admin revokes self", |w| w.revoke(w.me(A), A), Err(RuleError::SuperAdminTarget)),
    ]);
    assert_eq!(w.rank(A), Rank::SuperAdmin);
    assert_eq!(effective_status(&w.m[&acct(A)], NOW), Effective::Active(Role::Admin));
}

#[test]
fn only_a_super_admin_grants_or_changes_the_admin_role() {
    let mut w = W::deputy_site();
    run(&mut w, &[
        ("admin makes a resident admin", |w| w.set_role(w.me(D), ASHA, Role::Admin), Err(RuleError::Forbidden)),
        ("admin approves a new admin", |w| w.approve(w.me(D), &[(OUT, Role::Admin)]), Err(RuleError::Forbidden)),
        ("an admin keeps every other power", |w| w.set_role(w.me(D), ASHA, Role::Steward), Ok(())),
        ("super admin makes Ravi admin", |w| w.set_role(w.me(A), RAVI, Role::Admin), Ok(())),
        ("a second admin demotes the other admin", |w| w.set_role(w.me(D), RAVI, Role::Steward), Err(RuleError::Forbidden)),
        ("a second admin revokes the other admin", |w| w.revoke(w.me(D), RAVI), Err(RuleError::Forbidden)),
        ("the super admin demotes the deputy", |w| w.set_role(w.me(A), D, Role::Steward), Ok(())),
        ("the super admin approves a new admin", |w| w.approve(w.me(A), &[(OUT, Role::Admin)]), Ok(())),
    ]);
    assert_eq!(w.m[&acct(D)].role, Role::Steward);
    assert_eq!(w.m[&acct(ASHA)].role, Role::Steward);
    assert_eq!(w.m[&acct(RAVI)].role, Role::Admin);
    assert_eq!(w.m[&acct(OUT)].role, Role::Admin);
}

#[test]
fn a_super_admin_adds_one_more_member_and_no_third() {
    let mut w = W::deputy_site();
    run(&mut w, &[
        ("a plain admin adds", |w| w.sup_add(w.me(D), D), Err(RuleError::Forbidden)),
        ("a plain admin claims super admin", |w| w.sup_add(claim(D, Rank::SuperAdmin), D), Err(RuleError::ClaimMismatch)),
        ("a non-member", |w| w.sup_add(w.me(A), OUT), Err(RuleError::NotActive)),
        ("the super admin adds Ravi", |w| w.sup_add(w.me(A), RAVI), Ok(())),
        ("Ravi again", |w| w.sup_add(w.me(A), RAVI), Err(RuleError::AlreadySuperAdmin)),
        ("a third", |w| w.sup_add(w.me(A), S), Err(RuleError::SuperAdminsFull)),
    ]);
    assert_eq!(w.supers(), [acct(A), acct(RAVI)]);
    assert_eq!(w.m[&acct(RAVI)].role, Role::Admin);
}

/// On chain the two removals are ordered: the second signer is no longer a super admin.
#[test]
fn mutual_removal_keeps_the_first() {
    let mut w = W::deputy_site();
    run(&mut w, &[
        ("A adds the deputy", |w| w.sup_add(w.me(A), D), Ok(())),
        ("A removes the deputy", |w| w.sup_remove(w.me(A), D), Ok(())),
        ("the deputy, with its stale claim, removes A", |w| w.sup_remove(claim(D, Rank::SuperAdmin), A), Err(RuleError::ClaimMismatch)),
        ("the deputy, as a plain admin, removes A", |w| w.sup_remove(w.me(D), A), Err(RuleError::Forbidden)),
        ("A removes a plain admin", |w| w.sup_remove(w.me(A), D), Err(RuleError::NotSuperAdmin)),
    ]);
    assert_eq!(w.supers(), [acct(A)]);
    // both stay admins
    assert_eq!((w.m[&acct(A)].role, w.m[&acct(D)].role), (Role::Admin, Role::Admin));
}

/// Two super admins each remove the other and add a new one: only the first fits.
#[test]
fn adds_over_the_limit_keep_the_first() {
    let mut w = W::deputy_site();
    run(&mut w, &[
        ("A adds the deputy", |w| w.sup_add(w.me(A), D), Ok(())),
        ("A removes the deputy", |w| w.sup_remove(w.me(A), D), Ok(())),
        ("A adds Ravi", |w| w.sup_add(w.me(A), RAVI), Ok(())),
        ("the deputy's stale add of Meera", |w| w.sup_add(claim(D, Rank::SuperAdmin), MEERA), Err(RuleError::ClaimMismatch)),
        ("a third by A", |w| w.sup_add(w.me(A), MEERA), Err(RuleError::SuperAdminsFull)),
    ]);
    assert_eq!(w.supers(), [acct(A), acct(RAVI)]);
}

#[test]
fn transfer_hands_over_and_works_with_two() {
    let mut w = W::deputy_site();
    run(&mut w, &[
        ("a plain admin transfers", |w| w.sup_transfer(w.me(D), RAVI), Err(RuleError::Forbidden)),
        ("A transfers to the deputy", |w| w.sup_transfer(w.me(A), D), Ok(())),
        ("A is a plain admin: no admin grants", |w| w.set_role(w.me(A), ASHA, Role::Admin), Err(RuleError::Forbidden)),
        ("A still claims super admin", |w| w.set_role(claim(A, Rank::SuperAdmin), ASHA, Role::Admin), Err(RuleError::ClaimMismatch)),
        ("A keeps the other admin powers", |w| w.set_role(w.me(A), ASHA, Role::Steward), Ok(())),
        ("the deputy adds A back", |w| w.sup_add(w.me(D), A), Ok(())),
        ("A transfers to a super admin", |w| w.sup_transfer(w.me(A), D), Err(RuleError::AlreadySuperAdmin)),
        ("A transfers to Ravi", |w| w.sup_transfer(w.me(A), RAVI), Ok(())),
    ]);
    assert_eq!(w.supers(), [acct(D), acct(RAVI)], "each takes the slot it was given");
    assert_eq!(w.rank(A), Rank::Admin);
    assert_eq!(w.m[&acct(RAVI)].role, Role::Admin);
}

#[test]
fn there_is_always_a_super_admin() {
    let mut w = W::deputy_site();
    run(&mut w, &[
        ("remove self", |w| w.sup_remove(w.me(A), A), Err(RuleError::SelfTarget)),
        ("transfer to self", |w| w.sup_transfer(w.me(A), A), Err(RuleError::SelfTarget)),
    ]);
    assert_eq!(w.supers(), [acct(A)]);
}

// ---------- ported: lifecycle.rs ----------

#[test]
fn revoke_needs_an_admin_and_a_reason_and_keeps_who_when_why() {
    let mut w = W::deputy_site();
    run(&mut w, &[
        ("no reason", |w| w.revoke_why(w.me(A), S, [0; 32]), Err(RuleError::EmptyReason)),
        ("a resident revokes", |w| w.revoke(w.me(ASHA), D), Err(RuleError::Forbidden)),
        ("a steward revokes", |w| w.revoke(w.me(S), ASHA), Err(RuleError::Forbidden)),
        ("a resident claims admin", |w| w.revoke(claim(ASHA, Rank::Admin), RAVI), Err(RuleError::ClaimMismatch)),
        ("the admin revokes the steward", |w| w.revoke(w.me(D), S), Ok(())),
        ("revoked twice", |w| w.revoke(w.me(D), S), Err(RuleError::NotActive)),
        ("a non-member", |w| w.revoke(w.me(D), OUT), Err(RuleError::NotActive)),
        ("the genesis admin cannot be revoked", |w| w.revoke(w.me(A), A), Err(RuleError::SuperAdminTarget)),
    ]);
    let s = &w.m[&acct(S)];
    assert_eq!(effective_status(s, NOW), Effective::Removed);
    assert_eq!(s.revoked, Some(Revocation { by: acct(D), at_ms: NOW, reason_sha256: h(5) }));
    assert_eq!(w.site.key_epoch, 2, "one revoke, one rotation");
    assert_eq!((w.site.keyring_cid.as_str(), w.site.keyring_sha256), ("keyring3", h(15)));
}

#[test]
fn a_removed_member_cannot_act_until_approved_again() {
    let mut w = W::deputy_site();
    w.revoke(w.me(A), D).unwrap();
    let len = w.site.roster_len;
    run(&mut w, &[
        ("removed admin approves", |w| w.approve(claim(D, Rank::Admin), &[(OUT, Role::Resident)]), Err(RuleError::NotActive)),
        ("removed admin anchors", |w| w.anchor(D), Err(RuleError::NotActive)),
        ("a non-member anchors", |w| w.anchor(OUT), Err(RuleError::NotActive)),
        ("a resident anchors", |w| w.anchor(ASHA), Ok(())),
        ("approved again with a new request", |w| w.approve(w.me(A), &[(D, Role::Resident)]), Ok(())),
        ("approved twice", |w| w.approve(w.me(A), &[(D, Role::Resident)]), Err(RuleError::AlreadyMember)),
        ("back as a resident", |w| w.anchor(D), Ok(())),
    ]);
    let d = &w.m[&acct(D)];
    assert_eq!((d.revoked.clone(), d.status, d.role), (None, MemberStatus::Active, Role::Resident));
    assert_eq!(w.site.roster_len, len, "the same record: the roster does not grow");
}

#[test]
fn non_admins_cannot_approve() {
    let mut w = W::deputy_site();
    run(&mut w, &[
        ("a resident", |w| w.approve(w.me(ASHA), &[(OUT, Role::Resident)]), Err(RuleError::Forbidden)),
        ("a steward", |w| w.approve(w.me(S), &[(OUT, Role::Resident)]), Err(RuleError::Forbidden)),
        ("a non-member", |w| w.approve(claim(OUT, Rank::Admin), &[(OUT, Role::Resident)]), Err(RuleError::NotActive)),
        ("a steward claims admin", |w| w.approve(claim(S, Rank::Admin), &[(OUT, Role::Resident)]), Err(RuleError::ClaimMismatch)),
        ("the admin", |w| w.approve(w.me(D), &[(OUT, Role::Steward)]), Ok(())),
    ]);
}

// ---------- approve_batch ----------

#[test]
fn create_makes_the_creator_super_admin_and_first_member() {
    let w = W::new();
    let m = &w.m[&acct(A)];
    assert_eq!(w.site.super_admins, [Some(acct(A)), None]);
    assert_eq!((m.role, m.status, m.granted_by), (Role::Admin, MemberStatus::Active, acct(A)));
    assert_eq!((w.site.roster_len, w.site.key_epoch, w.site.created_at_ms), (1, 1, NOW));
    assert_eq!(w.site.site_id, seed::site_id(&acct(A), &h(9)));
    assert_eq!(w.rank(A), Rank::SuperAdmin);
}

#[test]
fn create_table() {
    let rows: &[(&str, fn(&mut CreateSite), R)] = &[
        ("ok", |_| {}, Ok(())),
        ("site_id of another salt", |a| a.salt = h(8), Err(RuleError::WrongSiteId)),
        ("site_id of another creator", |a| a.creator = acct(D), Err(RuleError::WrongSiteId)),
        ("empty details CID", |a| a.details_cid.clear(), Err(RuleError::BadCid)),
        ("129-byte places CID", |a| a.places_cid = "a".repeat(129), Err(RuleError::BadCid)),
        ("128-byte places CID", |a| a.places_cid = "a".repeat(CID_MAX), Ok(())),
        ("keyring CID with a slash", |a| a.keyring_cid = "ab/cd".into(), Err(RuleError::BadCid)),
        ("native gate on", |a| a.gates.native_min = Some(1), Err(RuleError::GatesNotInP1)),
        ("token program set", |a| a.gates.token_program = Some(h(1)), Err(RuleError::GatesNotInP1)),
        ("access duration", |a| a.access_duration_ms = 1, Err(RuleError::ExpiryNotInP1)),
    ];
    for (what, edit, want) in rows {
        let mut a = create_args();
        edit(&mut a);
        assert_eq!(&apply_create(&a, NOW).map(|_| ()), want, "{what}");
    }
}

#[test]
fn two_admins_approving_at_once_cannot_both_win() {
    let mut w = W::deputy_site();
    let seen = w.site.roster_len;
    w.approve_at(w.me(A), &[(OUT, Role::Resident)], seen).unwrap();
    assert_eq!(w.site.roster_len, seen + 1);
    assert_eq!(w.approve_at(w.me(D), &[(8, Role::Resident)], seen), Err(RuleError::StaleRoster));
    assert_eq!(w.approve_at(w.me(D), &[(8, Role::Resident)], seen + 1), Ok(()));
}

#[test]
fn approve_item_table() {
    let w = W::new();
    let sid = w.site.site_id;
    let admin = w.me(A);
    let good = || ApproveItem {
        request: req(&body(OUT, sid, 5)),
        role: Role::Resident,
        profile_cid: "profile1".into(),
        profile_sha256: h(20),
        profile_version: 1,
    };
    let rows: &[(&str, fn(&mut ApproveItem), R)] = &[
        ("ok", |_| {}, Ok(())),
        ("request for another site", |i| i.request = req(&body(OUT, h(99), 5)), Err(RuleError::JoinWrongSite)),
        ("account not bound to the key", |i| i.request[18 + 32] ^= 1, Err(RuleError::KeyBinding)),
        ("schnorr key not bound to the account", |i| i.request[18 + 64] ^= 1, Err(RuleError::KeyBinding)),
        ("bad domain", |i| i.request[0] = b'x', Err(RuleError::BadJoinRequest)),
        ("nonce 0", |i| i.request[146..154].copy_from_slice(&0u64.to_be_bytes()), Err(RuleError::StaleNonce)),
        ("empty profile CID", |i| i.profile_cid.clear(), Err(RuleError::BadCid)),
        ("profile version 0", |i| i.profile_version = 0, Err(RuleError::StaleProfile)),
    ];
    for (what, edit, want) in rows {
        let mut i = good();
        edit(&mut i);
        assert_eq!(&apply_approve(None, &i, &sid, &admin, NOW).map(|_| ()), want, "{what}");
    }
    let m = apply_approve(None, &good(), &sid, &admin, NOW).unwrap();
    let b = body(OUT, sid, 5);
    assert_eq!((m.user, m.pukaar_key, m.last_req_nonce), (acct(OUT), b.pukaar_ed_pk, 5));
    assert_eq!(m.request_id, request_id(&good().request));
    assert_eq!((m.granted_by, m.granted_at_ms, m.profile_version), (acct(A), NOW, 1));
}

// ---------- join request ----------

#[test]
fn join_request_is_162_bytes_and_290_with_both_signatures() {
    // domain + site_id + user_account + user_schnorr_pk + pukaar_ed_pk + nonce + valid_until_ms
    assert_eq!(JOIN_DOMAIN.len() + 32 + 32 + 32 + 32 + 8 + 8, 162);
    assert_eq!(JOIN_BODY_LEN, 162);
    assert_eq!(JOIN_BODY_LEN + 64 + 64, JOIN_REQ_LEN);
    assert_eq!(JOIN_REQ_LEN, 290);
    let b = body(OUT, h(1), 0x0102030405060708);
    let e = b.encode();
    assert_eq!(&e[146..154], &[1, 2, 3, 4, 5, 6, 7, 8], "big-endian nonce");
    assert_eq!(JoinBody::parse(&e), Some(b));
}

#[test]
fn check_join_request_table() {
    let sid = h(1);
    let mut old = W::new().m[&acct(A)].clone();
    old.site_id = sid;
    old.user = acct(OUT);
    old.last_req_nonce = 7;
    let other = Member { user: acct(D), ..old.clone() };
    let at = |valid_until_ms| req(&JoinBody { valid_until_ms, ..body(OUT, sid, 8) });
    let rows: Vec<(&str, [u8; JOIN_REQ_LEN], Option<&Member>, u64, R)> = vec![
        ("new record", req(&body(OUT, sid, 1)), None, NOW, Ok(())),
        ("nonce above the last", req(&body(OUT, sid, 8)), Some(&old), NOW, Ok(())),
        ("nonce equal to the last", req(&body(OUT, sid, 7)), Some(&old), NOW, Err(RuleError::StaleNonce)),
        ("nonce below the last", req(&body(OUT, sid, 2)), Some(&old), NOW, Err(RuleError::StaleNonce)),
        ("wrong site_id", req(&body(OUT, h(2), 8)), Some(&old), NOW, Err(RuleError::JoinWrongSite)),
        ("record of another user", req(&body(OUT, sid, 8)), Some(&other), NOW, Err(RuleError::WrongRecord)),
        ("valid until now", at(NOW), Some(&old), NOW, Ok(())),
        ("valid until 1 ms ago", at(NOW - 1), Some(&old), NOW, Err(RuleError::JoinExpired)),
    ];
    for (what, r, m, now, want) in rows {
        assert_eq!(check_join_request(&r, &sid, m, now).map(|_| ()), want, "{what}");
    }
}

#[test]
fn account_id_matches_lez_v0_3_1() {
    // lezkey.rs fixed test key 6f4cd00b…394f23: the wallet printed Public/AVi7Fm67WkAqJwLVD7PeA7KisBNuA2o6ew1CxmdtGrGP
    let pk: [u8; 32] = unhex("71a8c978d972b086f1a5b028df17b817bc4e5dccb170ede29fa18b14d3e22eda");
    assert_eq!(hex(&lez_account_id(&pk)), "8d13ecff3bf91d2ad05a8f68aa6be75d63dd1fc0008db38f20f7965ff2c383e8");
}

// ---------- set_role, set_profile, site pointers ----------

#[test]
fn set_role_table() {
    let mut w = W::deputy_site();
    run(&mut w, &[
        ("resident to steward by admin", |w| w.set_role(w.me(D), ASHA, Role::Steward), Ok(())),
        ("steward to resident by admin", |w| w.set_role(w.me(D), S, Role::Resident), Ok(())),
        ("by a steward", |w| w.set_role(w.me(ASHA), RAVI, Role::Steward), Err(RuleError::Forbidden)),
        ("admin on self", |w| w.set_role(w.me(D), D, Role::Steward), Err(RuleError::Forbidden)),
        ("removed target", |w| {
            w.revoke(w.me(A), MEERA)?;
            w.set_role(w.me(A), MEERA, Role::Steward)
        }, Err(RuleError::NotActive)),
        ("a super admin's role, by the other super admin", |w| {
            w.sup_add(w.me(A), D)?;
            w.set_role(w.me(D), A, Role::Steward)
        }, Err(RuleError::SuperAdminTarget)),
    ]);
    assert_eq!((w.m[&acct(ASHA)].role, w.m[&acct(S)].role), (Role::Steward, Role::Resident));
    assert_eq!(w.m[&acct(ASHA)].granted_by, acct(D));
}

#[test]
fn profile_version_must_grow() {
    let mut w = W::deputy_site();
    let v = w.m[&acct(ASHA)].profile_version;
    assert_eq!(v, 1);
    run(&mut w, &[
        ("same version", |w| w.set_profile(w.me(A), ASHA, 1), Err(RuleError::StaleProfile)),
        ("older version", |w| w.set_profile(w.me(A), ASHA, 0), Err(RuleError::StaleProfile)),
        ("a resident refreshes own", |w| w.set_profile(w.me(ASHA), ASHA, 2), Err(RuleError::Forbidden)),
        ("a steward refreshes", |w| w.set_profile(w.me(S), ASHA, 2), Err(RuleError::Forbidden)),
        ("an admin refreshes", |w| w.set_profile(w.me(D), ASHA, 2), Ok(())),
        ("an admin refreshes a super admin", |w| w.set_profile(w.me(D), A, 3), Ok(())),
        ("the same version again", |w| w.set_profile(w.me(D), ASHA, 2), Err(RuleError::StaleProfile)),
        ("a version jump", |w| w.set_profile(w.me(D), ASHA, 10), Ok(())),
    ]);
    let m = &w.m[&acct(ASHA)];
    assert_eq!((m.profile_cid.as_str(), m.profile_sha256, m.profile_version), ("profile2", h(21), 10));
}

#[test]
fn site_pointer_table() {
    type Set = fn(&Site, &Actor, &str, [u8; 32], u64) -> Result<Site, RuleError>;
    let w = W::deputy_site();
    let sets: [(&str, Set, fn(&Site) -> (&str, [u8; 32])); 3] = [
        ("details", apply_set_details, |s| (&s.details_cid, s.details_sha256)),
        ("places", apply_set_places, |s| (&s.places_cid, s.places_sha256)),
        ("keyring", apply_set_keyring, |s| (&s.keyring_cid, s.keyring_sha256)),
    ];
    for (what, set, get) in sets {
        let rows: [(Actor, &str, R); 6] = [
            (w.me(A), "new1", Ok(())),
            (w.me(D), "new1", Ok(())),
            (w.me(S), "new1", Err(RuleError::Forbidden)),
            (w.me(ASHA), "new1", Err(RuleError::Forbidden)),
            (claim(D, Rank::SuperAdmin), "new1", Err(RuleError::ClaimMismatch)),
            (w.me(D), "", Err(RuleError::BadCid)),
        ];
        for (actor, cid, want) in rows {
            let got = set(&w.site, &actor, cid, h(40), NOW + 1);
            assert_eq!(got.as_ref().map(|_| ()).map_err(|e| *e), want, "{what} {actor:?}");
            if let Ok(s) = got {
                assert_eq!(get(&s), ("new1", h(40)), "{what}");
                assert_eq!((s.updated_at_ms, s.key_epoch), (NOW + 1, w.site.key_epoch), "{what}");
            }
        }
    }
}

// ---------- signer checks and effective status ----------

#[test]
fn signer_check_table() {
    let mut w = W::deputy_site();
    w.revoke(w.me(A), RAVI).unwrap();
    let sid = w.site.site_id;
    let rows: &[(&str, u8, Actor, [u8; 32], R)] = &[
        ("admin claims admin", D, claim(D, Rank::Admin), sid, Ok(())),
        ("super admin claims super admin", A, claim(A, Rank::SuperAdmin), sid, Ok(())),
        ("steward claims admin", S, claim(S, Rank::Admin), sid, Err(RuleError::ClaimMismatch)),
        ("admin claims steward", D, claim(D, Rank::Steward), sid, Err(RuleError::ClaimMismatch)),
        ("removed resident", RAVI, claim(RAVI, Rank::Resident), sid, Err(RuleError::NotActive)),
        ("record of another account", D, claim(S, Rank::Admin), sid, Err(RuleError::WrongRecord)),
        ("record of another site", D, claim(D, Rank::Admin), h(99), Err(RuleError::WrongRecord)),
    ];
    for (what, rec, actor, site_id, want) in rows {
        assert_eq!(&check_signer(&w.m[&acct(*rec)], site_id, actor, NOW), want, "{what}");
    }
    // the Site half of the claim: a plain admin claiming super, a super admin claiming plain admin
    assert_eq!(check_site_claim(&w.site, &claim(D, Rank::SuperAdmin)), Err(RuleError::ClaimMismatch));
    assert_eq!(check_site_claim(&w.site, &claim(A, Rank::Admin)), Err(RuleError::ClaimMismatch));
    assert_eq!(check_site_claim(&w.site, &claim(A, Rank::SuperAdmin)), Ok(()));
    // anchor: only for the checkpoint's own site
    let rec = CheckpointRecord { site_id: h(99), heads_root: h(1), n_events: 1, signer_pk: h(2), sig_r: h(3), sig_s: h(4) };
    assert_eq!(check_anchor_signer(&w.m[&acct(ASHA)], &rec, &acct(ASHA), NOW), Err(RuleError::WrongRecord));
}

#[test]
fn effective_status_table() {
    let w = W::deputy_site();
    let m = |n| w.m[&acct(n)].clone();
    let removed = Member { status: MemberStatus::Removed, ..m(S) };
    let revoked = Member { revoked: Some(Revocation { by: acct(A), at_ms: 1, reason_sha256: h(1) }), ..m(S) };
    let rows = [
        (m(A), Effective::Active(Role::Admin)),
        (m(S), Effective::Active(Role::Steward)),
        (m(ASHA), Effective::Active(Role::Resident)),
        (removed, Effective::Removed),
        (revoked, Effective::Removed),
    ];
    for (rec, want) in rows {
        assert_eq!(effective_status(&rec, NOW), want);
    }
}

// ---------- clock claim ----------

#[test]
fn clock_claim_table() {
    let clock = |ts: u64| [7u64.to_le_bytes(), ts.to_le_bytes()].concat();
    let ts = 10_000_000;
    let rows = [
        ("claim equals the clock", ts, Ok(())),
        ("at the edge: 900 000 ms behind", ts - CLOCK_TOLERANCE_MS, Ok(())),
        ("too old: 900 001 ms behind", ts - CLOCK_TOLERANCE_MS - 1, Err(RuleError::ClockStale)),
        ("too new: 1 ms ahead", ts + 1, Err(RuleError::ClockBehind)),
    ];
    for (what, now, want) in rows {
        assert_eq!(check_clock(&clock(ts), now), want, "{what}");
    }
    assert_eq!(CLOCK_TOLERANCE_MS, 900_000);
}

// ---------- records ----------

fn max_cid() -> String {
    "z".repeat(CID_MAX)
}

fn max_site() -> Site {
    let (mut s, _) = apply_create(&create_args(), NOW).unwrap();
    s.super_admins = [Some(h(1)), Some(h(2))];
    (s.details_cid, s.places_cid, s.keyring_cid) = (max_cid(), max_cid(), max_cid());
    s.gates = GateConfig {
        native_min: Some(u128::MAX),
        token: Some(TokenGate { definition: h(3), min: u128::MAX, mode: GateMode::Attested }),
        nft: Some(NftGate { definition: h(4), mode: GateMode::OnChain }),
        token_program: Some(h(5)),
        ata_program: Some(h(6)),
    };
    s
}

fn max_member() -> Member {
    let mut m = W::new().m[&acct(A)].clone();
    m.notice = Some(Notice { outcome: NoticeOutcome::Remove, deadline_ms: 9, by: h(1), reason_sha256: h(2) });
    m.revoked = Some(Revocation { by: h(3), at_ms: 4, reason_sha256: h(5) });
    m.profile_cid = max_cid();
    m.gate_attestation = Some(Attestation { gate_id: GATE_NFT, at_ms: 6, by: h(7) });
    m
}

#[test]
fn records_round_trip() {
    let w = W::deputy_site();
    let sites = [w.site.clone(), max_site()];
    let members = [w.m[&acct(ASHA)].clone(), max_member()];
    for s in sites {
        let b = s.encode();
        assert_eq!(Site::decode(&b), Some(s));
        assert_eq!(Site::decode(&b[..b.len() - 1]), None);
        assert_eq!(Site::decode(&[b.clone(), vec![0]].concat()), None);
    }
    for m in members {
        let b = m.encode();
        assert_eq!(Member::decode(&b), Some(m));
        assert_eq!(Member::decode(&b[..b.len() - 1]), None);
    }
}

#[test]
fn records_fit_the_shard_limit() {
    // Site: 1 + 32*3 + 2*33 + 3*(4+128+32) + 8 + gates(17+50+34+33+33) + 8+8+4+8+8
    // Member: 1 + 32*3 + 1+1 + 32+8+8 + notice 74 + revoked 73 + 4 + (4+128+32) + 8 + attestation 42 + 32+8+8
    let (site, member) = (max_site().encode().len(), max_member().encode().len());
    assert_eq!((site, member), (866, 560));
    assert!(site <= SHARD_MAX && member <= SHARD_MAX);
    assert_eq!(SHARD_MAX, 102_400);
}

#[test]
fn seed_vectors() {
    // python3: hashlib.sha256 over the same bytes
    let sid = seed::site_id(&h(1), &h(2));
    let rows = [
        ("site_id", sid, "90718851df4ca0db581828abba56be7e3c452da6adcb51d692b5f9ff02aaac10"),
        ("site", seed::site(&sid), "3ecf057b22f93b0b7e99f2b01df20815434437957fb5173f82794ee628068c54"),
        ("member", seed::member(&sid, &h(3)), "0d77d948b0dda96d1c0b4002b0812601d6f63bf1ae06c7c6f751d4e40eba520f"),
        ("roster page 1", seed::roster(&sid, 1), "4f0eafeeab3047dd8858caa348620b525d0a57a4c9554091e2a4ceea02d75fcd"),
        ("join batch", seed::join_batch(&sid, &h(4)), "aea68a4f46a504fa606b5d4f62c89da71a48e50237a32100756c25a51e198607"),
        ("place removal", seed::place_removal(&sid, "W-03"), "76827b746a9e9bd39bc5514e4958c56598136c2b3a6d8a78fa523bd6104c374c"),
        ("checkpoint", seed::checkpoint(&h(1), &h(2)), "48a1501a8ccc11f268170a5236679b76b7709876158fab21ac79b96e801eb0fc"),
    ];
    for (what, got, want) in rows {
        assert_eq!(hex(&got), want, "{what}");
    }
}
