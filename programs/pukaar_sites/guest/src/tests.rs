//! Plan and apply as plain functions. `Chain` plays the executor: it runs `plan`, then `apply` for each
//! effect in order on a copy of the shards, and keeps the copy only if every step passed (a panic in
//! either step fails the call and leaves the state as it was).

use super::*;
use clock_core::clock_account_id;
use lee_core::program::{PlanInput, validate_plan};
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

const PROGRAM: AccountId = AccountId::new([0x77; 32]);
const NOW: u64 = 1_000_000;
// People, as in the rules tests.
const A: u8 = 1; // creator, super admin
const D: u8 = 2; // deputy admin
const S: u8 = 3; // steward
const R1: u8 = 4;
const R2: u8 = 5;
const OUT: u8 = 9; // not a member

fn user(n: u8) -> AccountId {
    AccountId::new(lez_account_id(&[n; 32]))
}

fn h(n: u8) -> [u8; 32] {
    [n; 32]
}

fn site_id() -> [u8; 32] {
    seed::site_id(&user(A).into_value(), &h(9))
}

fn derive(seed: [u8; 32]) -> AccountId {
    AccountId::for_public_pda(&PROGRAM, &PdaSeed::new(seed))
}

fn site_acct() -> AccountId {
    derive(seed::site(&site_id()))
}

fn member_acct(n: u8) -> AccountId {
    derive(seed::member(&site_id(), &user(n).into_value()))
}

fn prog(id: AccountId) -> AccountMeta {
    AccountMeta::new(id, false, PROGRAM)
}

fn clock_meta() -> AccountMeta {
    AccountMeta::new(CLOCK_01_PROGRAM_ACCOUNT_ID, false, clock_account_id())
}

fn signer_meta(n: u8) -> AccountMeta {
    AccountMeta::new(user(n), true, PROGRAM)
}

/// signer, Site, signer Member, clock, then the targets' Members (the signer's own is not repeated).
fn role_accounts(signer: u8, targets: &[u8]) -> Vec<AccountMeta> {
    let mut v = vec![signer_meta(signer), prog(site_acct()), prog(member_acct(signer)), clock_meta()];
    for t in targets {
        if *t != signer && !v.iter().any(|m| m.account_id == member_acct(*t)) {
            v.push(prog(member_acct(*t)));
        }
    }
    v
}

fn auth(claimed: Rank) -> Auth {
    Auth { site_id: site_id(), now_ms: NOW, claimed }
}

fn create_args() -> CreateSite {
    CreateSite {
        creator: user(A).into_value(),
        salt: h(9),
        site_id: site_id(),
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

fn entry(n: u8, role: Role, nonce: u64, is_new: bool) -> ApproveEntry {
    let body = JoinBody {
        site_id: site_id(),
        user_account: user(n).into_value(),
        user_schnorr_pk: [n; 32],
        pukaar_ed_pk: [n + 100; 32],
        nonce,
        valid_until_ms: NOW + 1000,
    };
    let request = [&body.encode()[..], &[0xEE; 128]].concat().try_into().unwrap();
    ApproveEntry {
        item: ApproveItem { request, role, profile_cid: format!("profile{n}"), profile_sha256: h(n), profile_version: 1 },
        is_new,
    }
}

type Key = (AccountId, AccountId);

struct Chain {
    shards: HashMap<Key, Vec<u8>>,
    clock_ms: u64,
}

impl Chain {
    fn new() -> Self {
        let clock = [5u64.to_le_bytes(), NOW.to_le_bytes()].concat();
        Self { shards: HashMap::from([((CLOCK_01_PROGRAM_ACCOUNT_ID, clock_account_id()), clock)]), clock_ms: NOW }
    }

    /// A chain with the site created by A.
    fn with_site() -> Self {
        let mut c = Self::new();
        let ix = Instruction::CreateSite(CreateSiteIx { args: create_args(), now_ms: NOW });
        c.send(vec![signer_meta(A), prog(site_acct()), prog(member_acct(A)), clock_meta()], ix).unwrap();
        c
    }

    /// The site with D (admin), S (steward), R1 and R2 (residents), approved by A.
    fn full() -> Self {
        let mut c = Self::with_site();
        let items = vec![entry(D, Role::Admin, 1, true), entry(S, Role::Steward, 1, true), entry(R1, Role::Resident, 1, true), entry(R2, Role::Resident, 1, true)];
        c.approve(A, Rank::SuperAdmin, 1, items).unwrap();
        c
    }

    fn send(&mut self, accounts: Vec<AccountMeta>, ix: Instruction) -> Result<(), String> {
        let input = PlanInput {
            self_account_id: PROGRAM,
            caller_account_id: None,
            accounts,
            instruction_data: borsh::to_vec(&ix).unwrap(),
        };
        let mut next = self.shards.clone();
        catch_unwind(AssertUnwindSafe(|| {
            let p = plan(&input, ix);
            validate_plan(&input, p.output()).expect("the plan stays inside its inputs");
            for e in &p.output().effects {
                let key = (e.selector.account_id, e.selector.program_account_id);
                let effect: Effect = borsh::from_slice(&e.data).unwrap();
                if let Some(post) = apply(effect, next.get(&key).map_or(&[][..], |v| v)) {
                    next.insert(key, post);
                }
            }
        }))
        .map_err(|e| e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| (*s).into())).unwrap_or_default())?;
        self.shards = next;
        Ok(())
    }

    fn approve(&mut self, signer: u8, claimed: Rank, roster_len: u32, items: Vec<ApproveEntry>) -> Result<(), String> {
        let targets: Vec<AccountMeta> = items
            .iter()
            .map(|e| {
                let body = JoinBody::parse(e.item.request[..JOIN_BODY_LEN].try_into().unwrap()).unwrap();
                prog(derive(seed::member(&site_id(), &body.user_account)))
            })
            .collect();
        let mut accounts = role_accounts(signer, &[]);
        accounts.extend(targets);
        let ix = Instruction::ApproveBatch(ApproveBatchIx {
            auth: auth(claimed),
            roster_len,
            keyring_cid: "keyring2".into(),
            keyring_sha256: h(23),
            items,
        });
        self.send(accounts, ix)
    }

    fn site(&self) -> Site {
        Site::decode(&self.shards[&(site_acct(), PROGRAM)]).unwrap()
    }

    fn member(&self, n: u8) -> Member {
        Member::decode(&self.shards[&(member_acct(n), PROGRAM)]).unwrap()
    }

    fn snapshot(&self) -> HashMap<Key, Vec<u8>> {
        self.shards.clone()
    }

    fn set_clock(&mut self, ts: u64) {
        self.clock_ms = ts;
        self.shards.insert((CLOCK_01_PROGRAM_ACCOUNT_ID, clock_account_id()), [5u64.to_le_bytes(), ts.to_le_bytes()].concat());
    }
}

/// The call fails and the shards are as before.
fn fails(c: &mut Chain, accounts: Vec<AccountMeta>, ix: Instruction, why: &str) {
    let before = c.snapshot();
    let err = c.send(accounts, ix).expect_err("the call must fail");
    assert!(err.contains(why), "wanted {why:?}, got {err:?}");
    assert_eq!(c.shards, before, "a failed call leaves the state as it was");
}

#[test]
fn instruction_indices_are_stable() {
    let b = |ix: &Instruction| borsh::to_vec(ix).unwrap()[0];
    let rec = CheckpointRecord { site_id: h(1), heads_root: h(2), n_events: 0, signer_pk: h(0), sig_r: h(0), sig_s: h(0) };
    assert_eq!(b(&Instruction::Anchor(AnchorIx { record: rec, now_ms: 0 })), 0);
    assert_eq!(b(&Instruction::CreateSite(CreateSiteIx { args: create_args(), now_ms: 0 })), 2);
    let sa = SuperAdminIx { auth: auth(Rank::SuperAdmin), target: h(1) };
    assert_eq!(b(&Instruction::SuperAdminAdd(sa.clone())), 10);
    assert_eq!(b(&Instruction::SuperAdminTransfer(sa)), 12);
    assert_eq!(b(&Instruction::SetBackup(SetBackupIx { site_id: site_id(), now_ms: 0, cid: "c".into(), sha256: h(1) })), 13);
}

#[test]
fn create_site_writes_site_and_creator() {
    let c = Chain::with_site();
    let (s, m) = (c.site(), c.member(A));
    assert_eq!((s.site_id, s.super_admins[0], s.roster_len, s.key_epoch, s.created_at_ms), (site_id(), Some(user(A).into_value()), 1, 1, NOW));
    assert_eq!((m.role, m.user, m.status), (Role::Admin, user(A).into_value(), MemberStatus::Active));
}

#[test]
fn create_site_negatives() {
    let mut c = Chain::with_site();
    let ix = Instruction::CreateSite(CreateSiteIx { args: create_args(), now_ms: NOW });
    let accts = vec![signer_meta(A), prog(site_acct()), prog(member_acct(A)), clock_meta()];
    fails(&mut c, accts.clone(), ix.clone(), "already written");
    // somebody else signs for the creator
    let mut other = accts.clone();
    other[0] = signer_meta(D);
    fails(&mut Chain::new(), other, ix.clone(), "the creator must sign");
    // wrong Site PDA
    let mut wrong = accts.clone();
    wrong[1] = prog(derive(h(5)));
    fails(&mut Chain::new(), wrong, ix.clone(), "missing account: Site PDA");
    // a claimed site_id that is not sha256(creator, salt)
    let mut args = create_args();
    args.site_id = h(7);
    let bad = Instruction::CreateSite(CreateSiteIx { args, now_ms: NOW });
    let accts7 = vec![signer_meta(A), prog(derive(seed::site(&h(7)))), prog(derive(seed::member(&h(7), &user(A).into_value()))), clock_meta()];
    fails(&mut Chain::new(), accts7, bad, "site_id is not");
    // gates must be off and the access duration 0 in P1
    let mut args = create_args();
    args.access_duration_ms = 5;
    fails(&mut Chain::new(), accts.clone(), Instruction::CreateSite(CreateSiteIx { args, now_ms: NOW }), "access_duration_ms stays 0");
    let mut args = create_args();
    args.gates.native_min = Some(1);
    fails(&mut Chain::new(), accts.clone(), Instruction::CreateSite(CreateSiteIx { args, now_ms: NOW }), "gates stay off");
    // an extra account
    let mut extra = accts;
    extra.push(prog(member_acct(D)));
    fails(&mut Chain::new(), extra, ix, "too many accounts");
}

#[test]
fn approve_batch_writes_members_and_site() {
    let c = Chain::full();
    let s = c.site();
    assert_eq!((s.roster_len, s.keyring_cid.as_str(), s.keyring_sha256, s.key_epoch), (5, "keyring2", h(23), 1));
    let m = c.member(S);
    assert_eq!((m.role, m.granted_by, m.profile_cid.as_str(), m.profile_version, m.last_req_nonce), (Role::Steward, user(A).into_value(), "profile3", 1, 1));
    assert_eq!(c.member(D).role, Role::Admin);
}

#[test]
fn approve_batch_negatives() {
    let mut c = Chain::with_site();
    let ok = |n, role| entry(n, role, 1, true);
    // a stale roster_len
    assert!(c.approve(A, Rank::SuperAdmin, 7, vec![ok(D, Role::Admin)]).unwrap_err().contains("roster_len changed"));
    // a good first item and a bad second one: nothing is written
    let before = c.snapshot();
    let err = c.approve(A, Rank::SuperAdmin, 1, vec![ok(S, Role::Steward), entry(R1, Role::Resident, 1, false)]).unwrap_err();
    assert!(err.contains("the claimed role is not the stored one"), "{err}");
    assert_eq!(c.shards, before);
    // a lie about the rank: A is the super admin, so claiming plain Admin is refused by the Site
    assert!(c.approve(A, Rank::Admin, 1, vec![ok(R1, Role::Resident)]).unwrap_err().contains("claimed role"));
    assert_eq!(c.shards, before);
    // a member who claims to be an admin
    let mut c = Chain::full();
    let before = c.snapshot();
    assert!(c.approve(S, Rank::Admin, 5, vec![ok(OUT, Role::Resident)]).unwrap_err().contains("claimed role"));
    // a steward who tells the truth cannot approve
    assert!(c.approve(S, Rank::Steward, 5, vec![ok(OUT, Role::Resident)]).unwrap_err().contains("does not allow"));
    // a resident cannot either
    assert!(c.approve(R1, Rank::Resident, 5, vec![ok(OUT, Role::Resident)]).unwrap_err().contains("does not allow"));
    // a plain admin cannot give the admin role
    assert!(c.approve(D, Rank::Admin, 5, vec![ok(OUT, Role::Admin)]).unwrap_err().contains("does not allow"));
    // someone who is not a member signs
    assert!(c.approve(OUT, Rank::Admin, 5, vec![ok(OUT + 1, Role::Resident)]).unwrap_err().contains("no record"));
    // an active member again
    assert!(c.approve(A, Rank::SuperAdmin, 5, vec![entry(S, Role::Steward, 2, false)]).unwrap_err().contains("already an active member"));
    // a client that says "new" for an existing member
    assert!(c.approve(A, Rank::SuperAdmin, 5, vec![entry(S, Role::Steward, 2, true)]).unwrap_err().contains("claimed role"));
    assert_eq!(c.shards, before);
    // an empty batch
    assert!(c.approve(A, Rank::SuperAdmin, 5, vec![]).unwrap_err().contains("no item"));
}

#[test]
fn a_wrong_member_pda_is_refused() {
    let mut c = Chain::with_site();
    let before = c.snapshot();
    let mut accts = role_accounts(A, &[]);
    accts.push(prog(derive(h(1)))); // not the PDA of D
    let ix = Instruction::ApproveBatch(ApproveBatchIx {
        auth: auth(Rank::SuperAdmin),
        roster_len: 1,
        keyring_cid: "k2".into(),
        keyring_sha256: h(1),
        items: vec![entry(D, Role::Admin, 1, true)],
    });
    fails(&mut c, accts.clone(), ix.clone(), "missing account: Member PDA");
    // the right PDA, plus one more account
    accts.pop();
    accts.push(prog(member_acct(D)));
    accts.push(prog(member_acct(S)));
    fails(&mut c, accts, ix, "too many accounts");
    assert_eq!(c.shards, before);
}

#[test]
fn set_role_batch() {
    let mut c = Chain::full();
    let call = |claimed, items: Vec<(u8, Role)>| {
        (
            role_accounts(D, &items.iter().map(|i| i.0).collect::<Vec<_>>()),
            Instruction::SetRoleBatch(SetRoleBatchIx {
                auth: auth(claimed),
                items: items.iter().map(|(n, r)| RoleItem { user: user(*n).into_value(), role: *r }).collect(),
            }),
        )
    };
    let (a, ix) = call(Rank::Admin, vec![(S, Role::Resident), (R1, Role::Steward)]);
    c.send(a, ix).unwrap();
    assert_eq!((c.member(S).role, c.member(R1).role, c.member(S).granted_by), (Role::Resident, Role::Steward, user(D).into_value()));
    // an admin cannot give the admin role, and cannot touch an admin or the super admin
    for (n, r) in [(R2, Role::Admin), (A, Role::Resident)] {
        let (a, ix) = call(Rank::Admin, vec![(n, r)]);
        fails(&mut c, a, ix, if n == A { "cannot be revoked or have the role changed" } else { "does not allow" });
    }
    // the super admin changes an admin
    let ix = Instruction::SetRoleBatch(SetRoleBatchIx { auth: auth(Rank::SuperAdmin), items: vec![RoleItem { user: user(D).into_value(), role: Role::Steward }] });
    c.send(role_accounts(A, &[D]), ix).unwrap();
    assert_eq!(c.member(D).role, Role::Steward);
    // the batch is atomic: the second item fails, the first is not written
    let ix = Instruction::SetRoleBatch(SetRoleBatchIx {
        auth: auth(Rank::SuperAdmin),
        items: vec![RoleItem { user: user(R2).into_value(), role: Role::Steward }, RoleItem { user: user(A).into_value(), role: Role::Resident }],
    });
    fails(&mut c, role_accounts(A, &[R2]), ix, "cannot be revoked or have the role changed");
    assert_eq!(c.member(R2).role, Role::Resident);
}

#[test]
fn revoke_batch_rotates_the_key() {
    let mut c = Chain::full();
    let revoke = |claimed, signer, items: Vec<(u8, u8)>| {
        (
            role_accounts(signer, &items.iter().map(|i| i.0).collect::<Vec<_>>()),
            Instruction::RevokeBatch(RevokeBatchIx {
                auth: auth(claimed),
                keyring_cid: "keyring3".into(),
                keyring_sha256: h(33),
                items: items.iter().map(|(n, why)| RevokeItem { user: user(*n).into_value(), reason_sha256: h(*why) }).collect(),
            }),
        )
    };
    // the super admin is never revoked, not even by themselves; an admin cannot revoke an admin
    let (a, ix) = revoke(Rank::Admin, D, vec![(A, 1)]);
    fails(&mut c, a, ix, "cannot be revoked");
    let (a, ix) = revoke(Rank::SuperAdmin, A, vec![(A, 1)]);
    fails(&mut c, a, ix, "cannot be revoked");
    // an empty reason
    let (a, ix) = revoke(Rank::Admin, D, vec![(R1, 0)]);
    fails(&mut c, a, ix, "needs a reason");
    // a steward cannot revoke
    let (a, ix) = revoke(Rank::Steward, S, vec![(R1, 1)]);
    fails(&mut c, a, ix, "does not allow");
    // the happy path
    let (a, ix) = revoke(Rank::Admin, D, vec![(R1, 1), (R2, 2)]);
    c.send(a, ix).unwrap();
    let (s, m) = (c.site(), c.member(R2));
    assert_eq!((s.key_epoch, s.keyring_cid.as_str(), s.keyring_sha256, s.roster_len), (2, "keyring3", h(33), 5));
    assert_eq!((m.status, m.revoked.unwrap().reason_sha256), (MemberStatus::Removed, h(2)));
    // a removed member can no longer sign
    let (a, ix) = revoke(Rank::Resident, R1, vec![(R2, 1)]);
    fails(&mut c, a, ix, "not active");
}

#[test]
fn set_profile_batch() {
    let mut c = Chain::full();
    let item = |n: u8, version| ProfileItem { user: user(n).into_value(), cid: format!("newprofile{n}"), sha256: h(50 + n), version };
    let call = |claimed, signer, items: Vec<ProfileItem>| {
        let targets: Vec<u8> = (0..=9).filter(|n| items.iter().any(|i| i.user == user(*n).into_value())).collect();
        (role_accounts(signer, &targets), Instruction::SetProfileBatch(SetProfileBatchIx { auth: auth(claimed), items }))
    };
    // the signer's own profile is in the same batch: its Member account is listed once
    let (a, ix) = call(Rank::Admin, D, vec![item(R1, 2), item(D, 2)]);
    c.send(a, ix).unwrap();
    assert_eq!((c.member(R1).profile_cid.as_str(), c.member(R1).profile_version, c.member(D).profile_version), ("newprofile4", 2, 2));
    // the version does not grow
    let (a, ix) = call(Rank::Admin, D, vec![item(R2, 2), item(R1, 2)]);
    fails(&mut c, a, ix, "does not grow");
    let (a, ix) = call(Rank::Admin, D, vec![item(R1, 1)]);
    fails(&mut c, a, ix, "does not grow");
    // only an admin
    let (a, ix) = call(Rank::Steward, S, vec![item(R1, 3)]);
    fails(&mut c, a, ix, "does not allow");
}

#[test]
fn set_details_places_keyring() {
    let mut c = Chain::full();
    let ix = |k: fn(SetPointerIx) -> Instruction, claimed| k(SetPointerIx { auth: auth(claimed), cid: "newcid".into(), sha256: h(60) });
    c.send(role_accounts(D, &[]), ix(Instruction::SetDetails, Rank::Admin)).unwrap();
    c.send(role_accounts(D, &[]), ix(Instruction::SetPlaces, Rank::Admin)).unwrap();
    c.send(role_accounts(A, &[]), ix(Instruction::SetKeyring, Rank::SuperAdmin)).unwrap();
    let s = c.site();
    assert_eq!((s.details_cid.as_str(), s.places_cid.as_str(), s.keyring_cid.as_str(), s.key_epoch), ("newcid", "newcid", "newcid", 1));
    fails(&mut c, role_accounts(S, &[]), ix(Instruction::SetDetails, Rank::Steward), "does not allow");
    let mut bad = ix(Instruction::SetDetails, Rank::Admin);
    if let Instruction::SetDetails(p) = &mut bad {
        p.cid = "not a cid!".into();
    }
    fails(&mut c, role_accounts(D, &[]), bad, "CID");
}

#[test]
fn super_admin_add_remove_transfer() {
    let mut c = Chain::full();
    let sa = |claimed, target| SuperAdminIx { auth: auth(claimed), target: user(target).into_value() };
    // only a super admin
    fails(&mut c, role_accounts(D, &[S]), Instruction::SuperAdminAdd(sa(Rank::Admin, S)), "does not allow");
    // add D (an admin) and then S: no third
    c.send(role_accounts(A, &[R1]), Instruction::SuperAdminAdd(sa(Rank::SuperAdmin, R1))).unwrap();
    assert_eq!((c.site().super_admins[1], c.member(R1).role), (Some(user(R1).into_value()), Role::Admin));
    fails(&mut c, role_accounts(A, &[S]), Instruction::SuperAdminAdd(sa(Rank::SuperAdmin, S)), "already two");
    // never remove self; remove the other one
    fails(&mut c, role_accounts(A, &[]), Instruction::SuperAdminRemove(sa(Rank::SuperAdmin, A)), "never removes");
    c.send(role_accounts(A, &[]), Instruction::SuperAdminRemove(sa(Rank::SuperAdmin, R1))).unwrap();
    assert_eq!(c.site().super_admins, [Some(user(A).into_value()), None]);
    // transfer: D takes the slot, A stays an admin
    c.send(role_accounts(A, &[D]), Instruction::SuperAdminTransfer(sa(Rank::SuperAdmin, D))).unwrap();
    assert_eq!(c.site().super_admins, [Some(user(D).into_value()), None]);
    assert_eq!(c.member(A).role, Role::Admin);
    // A now claims super admin and is refused
    fails(&mut c, role_accounts(A, &[S]), Instruction::SuperAdminAdd(sa(Rank::SuperAdmin, S)), "claimed role");
}

#[test]
fn a_stale_or_future_clock_claim_fails() {
    let mut c = Chain::full();
    let ix = |now_ms| {
        Instruction::SetDetails(SetPointerIx { auth: Auth { site_id: site_id(), now_ms, claimed: Rank::Admin }, cid: "c".into(), sha256: h(1) })
    };
    c.set_clock(NOW + 2_000_000);
    c.send(role_accounts(D, &[]), ix(NOW + 2_000_000 - CLOCK_TOLERANCE_MS)).unwrap();
    fails(&mut c, role_accounts(D, &[]), ix(NOW + 2_000_000 - CLOCK_TOLERANCE_MS - 1), "too far behind");
    fails(&mut c, role_accounts(D, &[]), ix(NOW + 2_000_001), "ahead of the clock");
}

#[test]
fn anchor_needs_an_active_member() {
    let mut c = Chain::full();
    let (pk1, pk2) = (c.member(R1).pukaar_key, c.member(R2).pukaar_key);
    let rec = |signer_pk| CheckpointRecord { site_id: site_id(), heads_root: h(8), n_events: 3, signer_pk, sig_r: h(0), sig_s: h(0) };
    let cp = derive(checkpoint_seed(&site_id(), &h(8)));
    let ix = |pk| Instruction::Anchor(AnchorIx { record: rec(pk), now_ms: NOW });
    let accts = |n| vec![signer_meta(n), prog(member_acct(n)), prog(cp), clock_meta()];
    let (ix_r1, ix_r2, junk) = (ix(pk1), ix(pk2), ix(h(0)));
    // the member shard does not exist for a stranger
    fails(&mut c, accts(OUT), ix_r1.clone(), "no record");
    // a record that carries a key other than the key of the signer
    fails(&mut c, accts(R1), junk, "signer_pk");
    c.send(accts(R1), ix_r1).unwrap();
    assert_eq!(CheckpointRecord::decode(&c.shards[&(cp, PROGRAM)]), Some(rec(pk1)));
    // the same (site, root) twice
    fails(&mut c, accts(R2), ix_r2, "already anchored");
    // a removed member
    let revoke = Instruction::RevokeBatch(RevokeBatchIx {
        auth: auth(Rank::Admin),
        keyring_cid: "k3".into(),
        keyring_sha256: h(1),
        items: vec![RevokeItem { user: user(R2).into_value(), reason_sha256: h(1) }],
    });
    c.send(role_accounts(D, &[R2]), revoke).unwrap();
    let other = Instruction::Anchor(AnchorIx { record: CheckpointRecord { heads_root: h(9), ..CheckpointRecord::decode(&c.shards[&(cp, PROGRAM)]).unwrap() }, now_ms: NOW });
    let cp2 = derive(checkpoint_seed(&site_id(), &h(9)));
    fails(&mut c, vec![signer_meta(R2), prog(member_acct(R2)), prog(cp2), clock_meta()], other, "not active");
}

fn backup_acct(n: u8) -> AccountId {
    derive(seed::backup(&site_id(), &user(n).into_value()))
}

/// signer, signer Member, backup pointer, clock (the order of `anchor`).
fn backup_call(signer: u8, pointer: u8, cid: &str, sha: u8) -> (Vec<AccountMeta>, Instruction) {
    let accounts = vec![signer_meta(signer), prog(member_acct(signer)), prog(backup_acct(pointer)), clock_meta()];
    (accounts, Instruction::SetBackup(SetBackupIx { site_id: site_id(), now_ms: NOW, cid: cid.into(), sha256: h(sha) }))
}

#[test]
fn a_backup_overwrites_the_same_record_of_the_member() {
    let mut c = Chain::full();
    let rec = |c: &Chain, n: u8| BackupPointer::decode(&c.shards[&(backup_acct(n), PROGRAM)]).unwrap();
    let (a, ix) = backup_call(R1, R1, "cidone", 1);
    c.send(a, ix).unwrap();
    assert_eq!((rec(&c, R1).cid.as_str(), rec(&c, R1).rev, rec(&c, R1).user), ("cidone", 1, user(R1).into_value()));
    // the second backup: same address, new CID, the count grows
    let (a, ix) = backup_call(R1, R1, "cidtwo", 2);
    c.send(a, ix).unwrap();
    assert_eq!((rec(&c, R1).cid.as_str(), rec(&c, R1).sha256, rec(&c, R1).rev), ("cidtwo", h(2), 2));
    // one record for each (site, user): R2 starts its own, R1 is untouched
    let (a, ix) = backup_call(R2, R2, "cidthree", 3);
    c.send(a, ix).unwrap();
    assert_eq!((rec(&c, R1).cid.as_str(), rec(&c, R2).cid.as_str()), ("cidtwo", "cidthree"));
}

#[test]
fn a_backup_has_negatives() {
    let mut c = Chain::full();
    // the pointer of another member is not the signer's: the plan finds no such account
    let (a, ix) = backup_call(R1, R2, "cidone", 1);
    fails(&mut c, a, ix, "missing account");
    // a stranger has no Member record
    let (a, ix) = backup_call(OUT, OUT, "cidone", 1);
    fails(&mut c, a, ix, "no record");
    // a CID is letters and digits
    let (a, ix) = backup_call(R1, R1, "bad cid!", 1);
    fails(&mut c, a, ix, "CID");
    // a removed member
    let revoke = Instruction::RevokeBatch(RevokeBatchIx {
        auth: auth(Rank::Admin),
        keyring_cid: "k3".into(),
        keyring_sha256: h(1),
        items: vec![RevokeItem { user: user(R2).into_value(), reason_sha256: h(1) }],
    });
    c.send(role_accounts(D, &[R2]), revoke).unwrap();
    let (a, ix) = backup_call(R2, R2, "cidone", 1);
    fails(&mut c, a, ix, "not active");
    // a claim far ahead of the clock
    let (a, mut ix) = backup_call(R1, R1, "cidone", 1);
    if let Instruction::SetBackup(b) = &mut ix {
        b.now_ms = NOW + 10 * 60 * 1000;
    }
    fails(&mut c, a, ix, "ahead of the clock");
}

#[cfg(feature = "probe")]
#[test]
fn the_probe_still_plans() {
    let p = sample_probe(PROBE_CLOCK, NOW, 0, 0, h(1));
    let mut c = Chain::new();
    c.send(vec![clock_meta()], Instruction::Probe(p)).unwrap();
}

// ---- security review ----

/// F3: a signer's authority reaches every chained call of the same transaction, so a role instruction
/// refuses a call that another program makes.
#[test]
fn a_call_from_another_program_is_refused() {
    let ix = Instruction::SetDetails(SetPointerIx { auth: Auth { site_id: site_id(), now_ms: NOW, claimed: Rank::Admin }, cid: "c".into(), sha256: h(1) });
    let input = PlanInput {
        self_account_id: PROGRAM,
        caller_account_id: Some(AccountId::new([0x55; 32])),
        accounts: role_accounts(D, &[]),
        instruction_data: borsh::to_vec(&ix).unwrap(),
    };
    let r = catch_unwind(AssertUnwindSafe(|| drop(plan(&input, ix.clone()))));
    assert!(r.is_err(), "plan accepted a chained call");
    // the same call from a user directly plans
    let direct = PlanInput { caller_account_id: None, ..input };
    drop(plan(&direct, ix));
}

/// F4: the default build has no probe. Index 1 stays reserved and the plan refuses it, with no signer needed to try.
#[cfg(not(feature = "probe"))]
#[test]
fn the_probe_is_not_callable_by_anyone() {
    let p = Probe {
        flags: PROBE_MEMBERS,
        now_ms: 0,
        min_balance: 0,
        leaf_data: vec![],
        index: 0,
        n_leaves: 0,
        proof: vec![],
        root: h(0),
        pk: [0; 32],
        sig: [0; 64],
        msg: vec![],
        salt: h(3),
        n_members: 3,
    };
    let ix = Instruction::Reserved(p);
    assert_eq!(borsh::to_vec(&ix).unwrap()[0], 1, "the slot keeps index 1");
    let accounts: Vec<AccountMeta> = (0..3).map(|i| prog(AccountId::new([i; 32]))).collect();
    let input = PlanInput { self_account_id: PROGRAM, caller_account_id: None, accounts, instruction_data: borsh::to_vec(&ix).unwrap() };
    let err = catch_unwind(AssertUnwindSafe(|| drop(plan(&input, ix)))).expect_err("an unsigned call planned 3 writes");
    let msg = err.downcast_ref::<&str>().map(|s| s.to_string()).or(err.downcast_ref::<String>().cloned()).unwrap_or_default();
    assert!(msg.contains("reserved"), "{msg}");
}
