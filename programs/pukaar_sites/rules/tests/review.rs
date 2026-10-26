//! Security review findings (F1, F2). Each test shows the attack and proves the fix.
use pukaar_sites_rules::*;

const NOW: u64 = 1_000_000;

fn acct(n: u8) -> AccountId {
    lez_account_id(&[n; 32])
}

fn h(n: u8) -> [u8; 32] {
    [n; 32]
}

fn site() -> Site {
    let a = CreateSite {
        creator: acct(1),
        salt: h(9),
        site_id: seed::site_id(&acct(1), &h(9)),
        pukaar_key: h(0xA1),
        details_cid: "d1".into(),
        details_sha256: h(1),
        places_cid: "p1".into(),
        places_sha256: h(2),
        keyring_cid: "k1".into(),
        keyring_sha256: h(3),
        gates: GateConfig::default(),
        access_duration_ms: 0,
        renew_window_ms: 0,
    };
    apply_create(&a, NOW).unwrap().0
}

fn item(n: u8, site_id: &[u8; 32], nonce: u64, version: u64, role: Role) -> ApproveItem {
    let b = JoinBody { site_id: *site_id, user_account: acct(n), user_schnorr_pk: [n; 32], pukaar_ed_pk: [n + 100; 32], nonce, valid_until_ms: NOW + 1000 };
    let request = [&b.encode()[..], &[0xEE; 128]].concat().try_into().unwrap();
    ApproveItem { request, role, profile_cid: "profile1".into(), profile_sha256: h(n), profile_version: version }
}

/// F1: an admin cannot set a huge profile version or a huge join nonce to lock a member out.
#[test]
fn an_admin_cannot_lock_a_member_out_with_a_huge_profile_version() {
    let s = site();
    let sid = s.site_id;
    let sup = Actor { account: acct(1), claimed: Rank::SuperAdmin };
    let d = Actor { account: acct(2), claimed: Rank::Admin };
    let resident = apply_approve(None, &item(5, &sid, 1, 1, Role::Resident), &sid, &sup, NOW).unwrap();
    // the jump is refused, by set_profile and by approve
    assert_eq!(apply_set_profile(&resident, "profile2", h(1), u64::MAX, &d, NOW), Err(RuleError::VersionJump));
    assert_eq!(apply_set_profile(&resident, "profile2", h(1), 1 + MAX_VERSION_STEP + 1, &d, NOW), Err(RuleError::VersionJump));
    let removed = apply_revoke(&resident, h(7), &d, NOW).unwrap();
    assert_eq!(apply_approve(Some(&removed), &item(5, &sid, 2, u64::MAX, Role::Resident), &sid, &sup, NOW), Err(RuleError::VersionJump));
    // the largest allowed step still works
    let ok = apply_set_profile(&resident, "profile2", h(1), 1 + MAX_VERSION_STEP, &d, NOW).unwrap();
    let removed = apply_revoke(&ok, h(7), &d, NOW).unwrap();
    // a forged nonce is refused, so the member is never locked
    assert_eq!(apply_approve(Some(&removed), &item(5, &sid, u64::MAX, 2 + MAX_VERSION_STEP, Role::Resident), &sid, &sup, NOW), Err(RuleError::NonceTooHigh));
    let back = apply_approve(Some(&removed), &item(5, &sid, 2, 2 + MAX_VERSION_STEP, Role::Resident), &sid, &sup, NOW);
    assert!(back.is_ok(), "re-approval is blocked: {back:?}");
}

/// F2: `anchor` ties `signer_pk` to `Member.pukaar_key`, so a junk record is refused.
#[test]
fn an_anchor_signer_key_must_be_the_key_of_the_member() {
    let s = site();
    let sid = s.site_id;
    let sup = Actor { account: acct(1), claimed: Rank::SuperAdmin };
    let resident = apply_approve(None, &item(5, &sid, 1, 1, Role::Resident), &sid, &sup, NOW).unwrap();
    let mut rec = CheckpointRecord { site_id: sid, heads_root: h(30), n_events: 999, signer_pk: h(77), sig_r: h(0), sig_s: h(0) };
    assert_ne!(rec.signer_pk, resident.pukaar_key);
    assert_eq!(check_anchor_signer(&resident, &rec, &acct(5), NOW), Err(RuleError::SignerKey));
    rec.signer_pk = resident.pukaar_key;
    assert_eq!(check_anchor_signer(&resident, &rec, &acct(5), NOW), Ok(()));
}
