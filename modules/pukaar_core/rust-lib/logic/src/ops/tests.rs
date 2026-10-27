//! The checks and the entries of the module methods, against the fake world of the queue tests.

use super::*;
use crate::event::Body;
use crate::fetch::{Details, Places, DETAILS_SCHEMA};
use crate::node::Node;
use crate::profile::Announce;
use crate::queue::tests::{acct, joiner_with, key, joiner, pk, World, PROGRAM, T0};
use crate::sync::Wire;
use crate::reducer::Config;
use crate::rpc::{refresh, Chain, SharedChain};
use pukaar_sites_rules::MemberStatus;
use crate::event::Key;
use pukaar_sites_rules::{check_join_request, Role};

const NOW: u64 = T0;

/// The node of member `n` on the site of the world, with the chain read.
fn node(w: &World, n: u8) -> Node {
    for u in [2, 3, 4] {
        w.chain.lock().unwrap().track(acct(u));
    }
    refresh(&w.chain, &*w.rpc);
    let mut nd = Node::join(key(n), w.site_id());
    nd.attach_chain(w.chain.clone());
    nd.attach_fetched(w.fetched.clone());
    nd.config = Config { categories: vec!["water".into()], max_open_per_author: 5, ..Default::default() };
    nd.locations = vec![Location { code: "W-01".into(), label: "Tap".into(), group: "Water".into() }];
    nd
}

fn hexk(n: u8) -> String {
    hex::encode(pk(n))
}

fn locked(w: &World) -> std::sync::MutexGuard<'_, Queue> {
    w.q.lock().unwrap()
}

#[test]
fn a_role_change_is_held_until_send_batch() {
    let w = World::new();
    w.put_member(&w.member(2, Role::Resident));
    let n = node(&w, 1);
    let mut q = locked(&w);
    let id = grant(&n, &mut q, &hexk(2), "steward", "Ravi", NOW).unwrap();
    let e = q.entries.iter().find(|e| e.id == id).unwrap();
    assert_eq!(e.state, St::Held);
    assert_eq!(e.intent, Intent::SetRole { items: vec![(acct(2), writes::role_code(Role::Steward))] });
    assert_eq!(grant(&n, &mut q, &hexk(2), "resident", "", NOW).unwrap_err(), "they have this role already");
    assert!(grant(&n, &mut q, &hexk(2), "boss", "", NOW).unwrap_err().contains("unknown role"));
    assert!(grant(&n, &mut q, "zz", "steward", "x", NOW).is_err());
    // A newer choice for the same person replaces the held one.
    let id2 = grant(&n, &mut q, &hexk(2), "admin", "Ravi", NOW).unwrap();
    let held: Vec<u64> = q.held(&w.site_id()).map(|e| e.id).collect();
    assert_eq!(held, vec![id2]);
    // Nothing was sent or prepared: the worker has nothing to do.
    assert!(q.entries.iter().all(|e| e.state == St::Held));
}

#[test]
fn only_an_admin_may_grant_and_the_rules_are_checked_first() {
    let w = World::new();
    w.put_member(&w.member(2, Role::Resident));
    w.put_member(&w.member(3, Role::Steward));
    let mut q = locked(&w);
    // A resident cannot.
    let resident = node(&w, 2);
    assert!(grant(&resident, &mut q, &hexk(3), "resident", "", NOW).unwrap_err().starts_with("not allowed"));
    assert!(revoke(&resident, &mut q, &hexk(3), "spam", NOW).unwrap_err().starts_with("not allowed"));
    assert!(send_batch(&resident, &mut q, NOW).unwrap_err().starts_with("not allowed"));
    assert!(place_edit(&resident, &mut q, Place::Add { code: "B-01", label: "Bin", group: "Bins" }, NOW).unwrap_err().starts_with("not allowed"));
    // A stranger has no role at all.
    let stranger = node(&w, 9);
    assert!(grant(&stranger, &mut q, &hexk(3), "resident", "", NOW).unwrap_err().contains("not an approved member"));
    assert!(q.entries.is_empty(), "nothing queued by a refused request");
}

#[test]
fn an_approval_needs_the_request_and_the_profile() {
    let w = World::new();
    let n = node(&w, 1);
    let site = w.site_id();
    let mut q = locked(&w);
    assert!(grant(&n, &mut q, &hexk(4), "resident", "", NOW).unwrap_err().contains("no join request"));
    let req = crate::joinreq::build(&key(4), &crate::lezkey::derive_lez_secret(&[4; 32]), site, NOW, NOW + REQUEST_VALID_MS);
    assert!(q.keep_request(&site, &req, NOW));
    assert!(grant(&n, &mut q, &hexk(4), "resident", "", NOW).unwrap_err().contains("profile has not arrived"));
    // The announce comes (the joiner helper makes one).
    drop(q);
    let _ = joiner(&w, 4, 0);
    let mut q = locked(&w);
    assert!(grant(&n, &mut q, &hexk(4), "steward", " ", NOW).unwrap_err().contains("needs a name"));
    let rows = pending_rows(&n, &q, NOW);
    assert_eq!((rows.len(), rows[0]["queued"].clone(), rows[0]["key"].clone()), (1, false.into(), hexk(4).into()));
    let id = grant(&n, &mut q, &hexk(4), "resident", "", NOW).unwrap();
    let e = q.entries.iter().find(|e| e.id == id).unwrap();
    let Intent::Approve { items, .. } = &e.intent else { panic!() };
    assert_eq!((e.state, items[0].request.as_slice()), (St::Held, &req[..]));
    assert_eq!(pending_rows(&n, &q, NOW)[0]["queued"], true);
    // A double click replaces, it does not stack.
    grant(&n, &mut q, &hexk(4), "resident", "", NOW).unwrap();
    assert_eq!(q.held(&site).count(), 1);
    // The request expires: it is gone from the list and cannot be granted.
    assert!(pending_rows(&n, &q, NOW + REQUEST_VALID_MS + 1).is_empty());
}

#[test]
fn an_approval_waits_for_the_profile_bytes_and_queues_nothing_before() {
    let w = World::new();
    let n = node(&w, 1);
    let site = w.site_id();
    // The announce is here, the file is not (Delivery brought only the pointer).
    let req = joiner_with(&w, 4, 0, false);
    let mut q = locked(&w);
    assert!(q.keep_request(&site, &req.request, NOW));
    assert_eq!(grant(&n, &mut q, &hexk(4), "resident", "", NOW).unwrap_err(), "This person's profile has not arrived yet. Try again in a minute.");
    assert!(q.entries.is_empty(), "nothing is queued, so nothing waits in the write thread");
    // The file comes in a ProfileBlob message: the same click now works.
    let a = w.fetched.lock().unwrap().announce_of(&pk(4)).unwrap();
    let msg = Wire::ProfileBlob { announce: a.clone(), file: w.store.get(&a.cid) }.encode();
    let Some(Wire::ProfileBlob { announce, file }) = Wire::decode(&msg) else { panic!() };
    assert!(n.on_profile_blob(announce, file));
    grant(&n, &mut q, &hexk(4), "resident", "", NOW).unwrap();
    assert_eq!(q.held(&site).count(), 1);
}

#[test]
fn a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download() {
    use pukaar_sites_rules::Instruction;
    let w = World::new();
    let n = node(&w, 1);
    let site = w.site_id();
    // The joiner (key 4) sends a signed request and its profile file. Its Storage CID is in no store.
    let request = joinreq::build(&key(4), &derive_lez_secret(&[4; 32]), site, NOW, NOW + REQUEST_VALID_MS).to_vec();
    let body = profile::Body { display_name: "Ravi".into(), bio: "nurse".into(), experience: "10 years".into() };
    let file = profile::build(&key(4), &site, &acct(4), 1, 1, &body, None, &[pk(1)]).unwrap();
    let bytes = profile::to_bytes(&file);
    let announce = Announce::sign(&key(4), site, 1, "zNotInAnyStore", crate::event::sha256(&bytes));
    let wire = [Wire::JoinReq(request.clone()).encode(), Wire::ProfileBlob { announce, file: bytes }.encode()];
    // The admin node handles them as the module does.
    for m in &wire {
        match Wire::decode(m) {
            Some(Wire::JoinReq(r)) => {
                n.on_join_req(&r);
                assert!(locked(&w).keep_request(&site, &r, NOW));
            }
            Some(Wire::ProfileBlob { announce, file }) => assert!(n.on_profile_blob(announce, file)),
            _ => panic!("unexpected message"),
        }
    }
    // The fetcher runs once with a working Storage (it reads the keyring of the admin's own node). Then
    // every download fails: no remote node can be reached.
    crate::fetch::fetch_cycle(&*w.store, &w.chain, &w.fetched, &w.key);
    w.store.no_download.store(true, std::sync::atomic::Ordering::Relaxed);

    let mut q = locked(&w);
    grant(&n, &mut q, &hexk(4), "resident", "", NOW).unwrap();
    assert_eq!(send_batch(&n, &mut q, NOW).unwrap().len(), 1);
    drop(q);
    w.tick(T0);
    let e = w.q.lock().unwrap().entries[0].clone();
    assert_eq!((e.state, e.reason), (St::Queued, None));
    assert_eq!(w.store.refused.load(std::sync::atomic::Ordering::Relaxed), 0, "no download was tried");
    let Intent::Approve { items, .. } = &e.intent else { panic!() };
    let p = items[0].profile.clone().unwrap();
    // The file in Storage has the site wrap and the joiner opens it with the new keyring.
    let ring = profile::keyring_from_bytes(&e.blobs[0].bytes).unwrap();
    let keys = profile::open_keyring(&ring, &key(4)).unwrap();
    let out = profile::from_bytes(&w.store.get(&p.cid)).unwrap();
    assert_eq!(profile::open(&out, &key(4), &keys).unwrap().display_name, "Ravi");
    // The approval instruction is built.
    w.tick(T0 + 1);
    let Instruction::ApproveBatch(ix) = crate::queue::tests::decode(&w.sent()[0]) else { panic!("not approve_batch") };
    assert_eq!((ix.items.len(), ix.items[0].item.profile_cid.as_str()), (1, p.cid.as_str()));
    assert_eq!(w.store.refused.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[test]
fn a_revoke_needs_a_reason_and_never_hits_a_super_admin() {
    let w = World::new();
    w.put_member(&w.member(2, Role::Resident));
    let n = node(&w, 1);
    let mut q = locked(&w);
    assert!(revoke(&n, &mut q, &hexk(2), "  ", NOW).unwrap_err().contains("reason"));
    assert!(revoke(&n, &mut q, &hexk(2), &"x".repeat(501), NOW).unwrap_err().contains("too long"));
    assert!(revoke(&n, &mut q, &hexk(1), "bye", NOW).unwrap_err().starts_with("not allowed"), "a super admin cannot be revoked");
    assert!(revoke(&n, &mut q, &hexk(7), "bye", NOW).unwrap_err().contains("not an active member"));
    let id = revoke(&n, &mut q, &hexk(2), " spam ", NOW).unwrap();
    let e = q.entries.iter().find(|e| e.id == id).unwrap();
    assert_eq!(e.intent, Intent::Revoke { items: vec![RevokeReq { user: acct(2), key: pk(2), reason: "spam".into() }] });
    assert_eq!(e.state, St::Held);
}

#[test]
fn super_admin_changes_are_checked_against_the_rules() {
    let w = World::new();
    w.put_member(&w.member(2, Role::Admin));
    w.put_member(&w.member(3, Role::Resident));
    let n = node(&w, 1);
    let mut q = locked(&w);
    let id = super_admin(&n, &mut q, &hexk(2), SuperKind::Add, NOW).unwrap();
    let e = q.entries.iter().find(|e| e.id == id).unwrap();
    assert_eq!((e.state, &e.intent), (St::Queued, &Intent::SuperAdd(acct(2))));
    assert_eq!(super_admin(&n, &mut q, &hexk(2), SuperKind::Transfer, NOW).map(|_| ()), Ok(()));
    assert!(super_admin(&n, &mut q, &hexk(2), SuperKind::Remove, NOW).unwrap_err().starts_with("not allowed"), "2 is not a super admin");
    assert!(super_admin(&n, &mut q, &hexk(1), SuperKind::Remove, NOW).unwrap_err().contains("yourself"));
    assert!(super_admin(&n, &mut q, &hexk(1), SuperKind::Transfer, NOW).unwrap_err().starts_with("not allowed"), "not to a super admin");
    // An admin who is no super admin cannot.
    let admin = node(&w, 2);
    assert!(super_admin(&admin, &mut q, &hexk(3), SuperKind::Add, NOW).unwrap_err().starts_with("not allowed"));
    // With two super admins on the chain, a third is refused.
    let mut s = w.site.clone();
    s.super_admins[1] = Some(acct(2));
    w.put_site(&s);
    let n = node(&w, 1);
    refresh(&w.chain, &*w.rpc);
    assert!(super_admin(&n, &mut q, &hexk(3), SuperKind::Add, NOW).unwrap_err().contains("two super admins"));
    // With a second super admin, we may remove it, not ourselves.
    assert!(super_admin(&n, &mut q, &hexk(2), SuperKind::Remove, NOW).is_ok());
}

#[test]
fn place_edits_are_checked_on_the_places_that_we_know() {
    let w = World::new();
    let mut n = node(&w, 1);
    // One open report at W-01 (our own event, accepted because we have a role).
    n.publish(Body::Report { category: "water".into(), location: "W-01".into(), landmark: String::new(), text: "leak".into() }, 10).unwrap();
    let mut q = locked(&w);
    assert!(place_edit(&n, &mut q, Place::Add { code: "W-01", label: "x", group: "y" }, NOW).unwrap_err().contains("used already"));
    assert!(place_edit(&n, &mut q, Place::Add { code: "", label: "x", group: "y" }, NOW).is_err());
    assert!(place_edit(&n, &mut q, Place::Retire { code: "W-01", reason: "gone" }, NOW).unwrap_err().contains("1 open issue"));
    assert!(place_edit(&n, &mut q, Place::Remove { code: "W-01", reason: "gone" }, NOW).unwrap_err().contains("has had reports"));
    assert!(place_edit(&n, &mut q, Place::Retire { code: "W-02", reason: "gone" }, NOW).unwrap_err().contains("no active location"));
    assert!(place_edit(&n, &mut q, Place::Retire { code: "W-01", reason: " " }, NOW).unwrap_err().contains("reason"));
    assert!(place_edit(&n, &mut q, Place::Edit { code: "W-09", label: "x", group: "y" }, NOW).is_err());
    assert!(q.entries.is_empty());
    let id = place_edit(&n, &mut q, Place::Add { code: " B-01 ", label: "Bin", group: "Bins" }, NOW).unwrap();
    let e = q.entries.iter().find(|e| e.id == id).unwrap();
    assert_eq!(e.state, St::Preparing);
    let Intent::Places { ops } = &e.intent else { panic!() };
    assert_eq!(ops, &vec![PlaceOp::Add(Location { code: "B-01".into(), label: "Bin".into(), group: "Bins".into() })]);
    assert!(place_edit(&n, &mut q, Place::Restore { code: "W-01", reason: "back" }, NOW).is_ok());
}

#[test]
fn send_batch_takes_the_held_writes_and_the_member_refreshes() {
    let w = World::new();
    w.put_member(&w.member(2, Role::Resident));
    let n = node(&w, 1);
    let mut q = locked(&w);
    assert_eq!(send_batch(&n, &mut q, NOW).unwrap_err(), "nothing waits for a batch");
    // Member 2 announces a profile (version 2) that the chain does not hold.
    let body = profile::Body { display_name: "Ravi".into(), bio: String::new(), experience: String::new() };
    let file = profile::build(&key(2), &w.site_id(), &acct(2), 2, 1, &body, None, &[pk(1)]).unwrap();
    let bytes = profile::to_bytes(&file);
    let cid = w.store.put(&bytes);
    assert!(w.fetched.lock().unwrap().note_announce(Announce::sign(&key(2), w.site_id(), 2, &cid, crate::event::sha256(&bytes))));
    assert_eq!(status(&n, &q, NOW)["refresh_waiting"], 1);
    grant(&n, &mut q, &hexk(2), "steward", "Ravi", NOW).unwrap();
    let ids = send_batch(&n, &mut q, NOW).unwrap();
    let names: Vec<_> = ids.iter().map(|i| q.entries.iter().find(|e| e.id == *i).unwrap().intent.name()).collect();
    assert_eq!(names, vec!["set_role", "set_profile"]);
    let p = q.entries.iter().find_map(|e| match &e.intent {
        Intent::SetProfile { items } => Some(items[0].clone()),
        _ => None,
    });
    let p = p.unwrap();
    assert_eq!((p.user, p.prof.cid, p.prof.version), (acct(2), cid, 2));
    // Asked again: the refresh is on its way, nothing new.
    assert_eq!(send_batch(&n, &mut q, NOW).unwrap_err(), "nothing waits for a batch");
    assert_eq!(status(&n, &q, NOW)["refresh_waiting"], 0);
}

#[test]
fn profile_versions_only_grow() {
    let w = World::new();
    let n = node(&w, 1);
    let mut q = locked(&w);
    let v = |q: &Queue, id: u64| match &q.entries.iter().find(|e| e.id == id).unwrap().intent {
        Intent::Publish { name, version, .. } => (name.clone(), *version),
        _ => panic!(),
    };
    let a = set_profile(&n, &mut q, " Asha\u{7} ", "", "", NOW).unwrap();
    let b = set_profile(&n, &mut q, "Asha K", "", "", NOW).unwrap();
    assert_eq!((v(&q, a), v(&q, b)), (("Asha".into(), 1), ("Asha K".into(), 2)));
    assert!(set_profile(&n, &mut q, &"x".repeat(65), "", "", NOW).unwrap_err().contains("too long"));
    // The bio and the experience travel with the name, cleaned, and have a cap of their own.
    let c = set_profile(&n, &mut q, "Asha", " nurse\u{7}\nnights ", "10 years", NOW).unwrap();
    let e = q.entries.iter().find(|e| e.id == c).unwrap();
    assert!(matches!(&e.intent, Intent::Publish { bio, experience, .. } if bio == "nurse\nnights" && experience == "10 years"));
    assert!(set_profile(&n, &mut q, "Asha", &"x".repeat(501), "", NOW).unwrap_err().contains("bio"));
    assert!(set_profile(&n, &mut q, "Asha", "", &"x".repeat(501), NOW).unwrap_err().contains("experience"));
    q.entries.retain(|e| e.id != c);
    assert_eq!(q.entries.len(), 2);
    assert_eq!(q.entries[0].state, St::Preparing);
}

#[test]
fn a_join_request_is_signed_twice_and_kept() {
    let w = World::new();
    let n = node(&w, 4);
    let mut q = locked(&w);
    let (req, id) = join(&n, &mut q, NOW);
    assert_eq!(req.len(), 290);
    let (body, _) = crate::joinreq::parse(&req).unwrap();
    assert_eq!((body.user_account, body.pukaar_ed_pk, body.site_id), (acct(4), pk(4), w.site_id()));
    assert!(check_join_request(req.as_slice().try_into().unwrap(), &w.site_id(), None, NOW).is_ok());
    assert_eq!(q.mine.as_deref(), Some(&req[..]));
    assert!(matches!(&q.entries.iter().find(|e| e.id == id).unwrap().intent, Intent::Publish { name, version: 1, .. } if name.is_empty()));
    // It survives a restart: `my_identity.pending` stays "requested".
    q.save();
    assert!(Queue::load(&w.dir).mine.is_some());
}

#[test]
fn a_new_site_has_its_files_and_the_first_profile_behind_it() {
    let w = World::new();
    let g = writes::genesis(r#"{"Genesis":{"name":"Dhun","admin_name":"Asha","categories":["water"],"locations":[{"code":"W-01","label":"Tap","group":"Water"}],"sla_ack_h":24,"sla_fix_h":72,"max_open_per_author":5}}"#).unwrap();
    let ns = new_site(&key(1), &g).unwrap();
    assert_eq!(ns.site_id, seed::site_id(&acct(1), &ns.salt));
    assert_eq!(Details::from_bytes(&ns.blobs[0].bytes).unwrap().name, "Dhun");
    assert_eq!(Details::from_bytes(&ns.blobs[0].bytes).unwrap().schema, DETAILS_SCHEMA);
    assert_eq!(Places::from_bytes(&ns.blobs[1].bytes).unwrap().places.len(), 1);
    // The creator opens the first keyring and holds the site key of epoch 1.
    let ring = profile::keyring_from_bytes(&ns.blobs[2].bytes).unwrap();
    assert_eq!(profile::open_keyring(&ring, &key(1)).unwrap().len(), 1);
    assert_eq!(profile::entries(&ring).unwrap(), vec![(acct(1), pk(1))]);
    let n = Node::join(key(1), ns.site_id);
    let site_id = ns.site_id;
    let id = create_site(&n, &mut locked(&w), ns, "Asha", NOW);
    let q = locked(&w);
    let mine: Vec<_> = q.entries.iter().filter(|e| e.site == site_id).collect();
    assert_eq!((mine[0].id, mine[0].intent.name(), mine[0].blobs.len()), (id, "create_site", 3));
    assert_eq!((mine[1].intent.name(), mine[1].state), ("profile", St::Preparing));
    assert!(matches!(&mine[1].intent, Intent::Publish { name, version: 1, .. } if name == "Asha"));
}

#[test]
fn status_has_the_shape_of_chain_status() {
    let w = World::new();
    w.put_member(&w.member(2, Role::Resident));
    let n = node(&w, 1);
    let mut q = locked(&w);
    grant(&n, &mut q, &hexk(2), "steward", "R", NOW).unwrap();
    let s = status(&n, &q, NOW);
    assert_eq!((s["key"].as_str(), s["waiting"].as_u64(), s["refresh_waiting"].as_u64()), (Some("not imported yet"), Some(1), Some(0)));
    let w0 = &s["writes"][0];
    for f in ["id", "kind", "state", "tx", "reason", "items", "accounts", "waited_s"] {
        assert!(w0.get(f).is_some(), "writes[0] has {f}");
    }
    assert_eq!((w0["kind"].as_str(), w0["state"].as_str()), (Some("set_role"), Some("waiting")));
    assert_eq!(w0["accounts"], serde_json::json!([crate::chain::b58(&acct(2))]), "the row of the member is found by its account");
    let _ = PROGRAM;
}

/// Member `n` that the chain shows as removed by member 1.
fn removed(w: &World, n: u8) -> Member {
    let mut m = w.member(n, Role::Admin);
    m.status = MemberStatus::Removed;
    m.revoked = Some(pukaar_sites_rules::Revocation { by: acct(1), at_ms: 5_000, reason_sha256: [8; 32] });
    m
}

/// The keyring entries of members 1 and 2, then the next epoch that leaves 2 out (what a revoke makes).
fn rotate_keyring(w: &World, chain: &SharedChain, keep: &[u8]) {
    let ents: Vec<_> = keep.iter().map(|n| (acct(*n), pk(*n))).collect();
    chain.lock().unwrap().set_keyring(&ents);
    refresh(chain, &*w.rpc);
}

fn removed_keys(c: &SharedChain) -> Vec<Value> {
    c.lock().unwrap().people_json(&Default::default())["revoked"].as_array().unwrap().iter().map(|r| r["key"].clone()).collect()
}

#[test]
fn a_removed_member_stays_in_the_removed_list_after_the_keyring_rotates() {
    let w = World::new();
    w.put_member(&removed(&w, 2));
    let c = Chain::shared(PROGRAM, w.site_id(), acct(1));
    c.lock().unwrap().persist_in(&w.dir);
    rotate_keyring(&w, &c, &[1, 2]);
    assert_eq!(removed_keys(&c), vec![json!(hexk(2))]);
    // The revoke made a new keyring without member 2: the list keeps them.
    rotate_keyring(&w, &c, &[1]);
    assert_eq!(removed_keys(&c), vec![json!(hexk(2))], "left out of the keyring, still listed");
    // A restart: the accounts come back from the file, with no keyring of the old epoch.
    let again = Chain::shared(PROGRAM, w.site_id(), acct(1));
    again.lock().unwrap().persist_in(&w.dir);
    rotate_keyring(&w, &again, &[1]);
    assert_eq!(removed_keys(&again), vec![json!(hexk(2))], "after a restart");
}

#[test]
fn the_queue_brings_back_a_removed_member_that_no_keyring_lists() {
    let w = World::new();
    w.put_member(&removed(&w, 2));
    w.push(Intent::Revoke { items: vec![RevokeReq { user: acct(2), key: pk(2), reason: "spam".into() }] }, St::Done, vec![]);
    let c = Chain::shared(PROGRAM, w.site_id(), acct(1));
    let mut n = Node::join(key(1), w.site_id());
    n.attach_chain(c.clone());
    rotate_keyring(&w, &c, &[1]);
    assert!(removed_keys(&c).is_empty(), "nothing knows member 2 yet");
    remember_known(&n, &locked(&w));
    refresh(&c, &*w.rpc);
    assert_eq!(removed_keys(&c), vec![json!(hexk(2))]);
}

#[test]
fn a_removed_member_gets_a_reason_and_no_storage_error() {
    let w = World::new();
    w.put_member(&removed(&w, 2));
    // Epoch 2 of the keyring leaves member 2 out.
    let ring = profile::rotate(&w.site_id(), &mut std::collections::BTreeMap::from([(1, [9; 32])]), &[(acct(1), pk(1))]).unwrap();
    let bytes = profile::keyring_to_bytes(&ring);
    let mut s = w.site.clone();
    (s.key_epoch, s.keyring_cid, s.keyring_sha256) = (2, w.store.put(&bytes), crate::event::sha256(&bytes));
    w.put_site(&s);
    let c = Chain::shared(PROGRAM, w.site_id(), acct(2));
    refresh(&c, &*w.rpc);
    let fx = crate::fetch::Fetched::shared();
    crate::fetch::fetch_cycle(&*w.store, &c, &fx, &key(2));
    assert!(fx.lock().unwrap().status_json()["error"].is_null(), "not in the keyring is not a storage fault");
    let mut n = Node::join(key(2), w.site_id());
    n.attach_chain(c);
    let v = status(&n, &locked(&w), NOW);
    assert!(v["reason"].as_str().unwrap().contains("role was removed"));
    // A member with a role has no reason.
    w.put_member(&w.member(3, Role::Resident));
    let mut ok = Node::join(key(3), w.site_id());
    let c3 = Chain::shared(PROGRAM, w.site_id(), acct(3));
    refresh(&c3, &*w.rpc);
    ok.attach_chain(c3);
    assert!(status(&ok, &locked(&w), NOW)["reason"].is_null());
}

#[test]
fn a_spent_request_is_not_waiting_so_a_removed_member_sees_the_removal() {
    let w = World::new();
    let mut q = locked(&w);
    let n = node(&w, 2);
    assert!(!request_waits(&n, &q), "no request sent");
    let (req, _) = join(&n, &mut q, NOW);
    assert!(request_waits(&n, &q), "sent, the chain has no record");
    // The chain used it (the member was approved, then removed): the record keeps its nonce.
    let mut m = removed(&w, 2);
    m.last_req_nonce = crate::joinreq::parse(&req).unwrap().0.nonce;
    w.put_member(&m);
    refresh(&w.chain, &*w.rpc);
    assert!(!request_waits(&n, &q));
    // A new request has a higher nonce and waits again.
    join(&n, &mut q, NOW + 1);
    assert!(request_waits(&n, &q));
}

fn request_of(n: u8, w: &World, nonce: u64) -> Vec<u8> {
    joinreq::build(&key(n), &derive_lez_secret(&[n; 32]), w.site_id(), nonce, NOW + REQUEST_VALID_MS).to_vec()
}

fn decline_of(wire: &[u8]) -> (Key, Key, Key, Vec<u8>) {
    let Some(Wire::JoinDecline { site, request_id, admin, sig }) = Wire::decode(wire) else { panic!("not a decline") };
    (site, request_id, admin, sig)
}

#[test]
fn reject_hides_a_request_for_good_and_a_new_one_shows_again() {
    let w = World::new();
    let admin = node(&w, 1);
    let site = w.site_id();
    let mut q = locked(&w);
    let req = request_of(4, &w, NOW);
    assert!(q.keep_request(&site, &req, NOW));
    assert_eq!(pending_rows(&admin, &q, NOW).len(), 1);
    // Reject writes nothing to the chain and queues nothing.
    reject(&admin, &mut q, &hexk(4), NOW).unwrap();
    assert!(q.entries.is_empty() && w.sent().is_empty());
    assert!(pending_rows(&admin, &q, NOW).is_empty());
    assert!(grant(&admin, &mut q, &hexk(4), "resident", "", NOW).unwrap_err().contains("no join request"));
    assert!(reject(&admin, &mut q, &hexk(4), NOW).unwrap_err().contains("no join request"));
    // Delivery brings the same request again, and an older one: both stay hidden.
    assert!(!q.keep_request(&site, &req, NOW));
    assert!(!q.keep_request(&site, &request_of(4, &w, NOW - 5), NOW));
    // A restart keeps it hidden.
    let again = Queue::load(&w.dir);
    assert!(pending_rows(&admin, &again, NOW).is_empty());
    let mut again = again;
    assert!(!again.keep_request(&site, &req, NOW));
    // A new request of the same key (a new nonce) shows, and only it.
    assert!(again.keep_request(&site, &request_of(4, &w, NOW + 1), NOW));
    assert_eq!(pending_rows(&admin, &again, NOW).len(), 1);
    // Only an admin may reject.
    w.put_member(&w.member(3, Role::Resident));
    let resident = node(&w, 3);
    assert!(reject(&resident, &mut again, &hexk(4), NOW).unwrap_err().starts_with("not allowed"));
    // An approval that waits for the batch is not undone by a reject.
    let item = ApproveReq { request: request_of(4, &w, NOW + 1), role: 0, name: String::new(), profile: None };
    again.push(site, Intent::Approve { roster_len: 0, items: vec![item] }, St::Held, vec![], NOW);
    assert!(reject(&admin, &mut again, &hexk(4), NOW).unwrap_err().contains("waits to be sent"));
}

#[test]
fn the_joiner_hears_the_decline_and_can_send_again_also_with_no_admin_online() {
    let w = World::new();
    let admin = node(&w, 1);
    let joiner = node(&w, 4);
    let site = w.site_id();
    let mut aq = locked(&w);
    let mut jq = Queue::load(&w.dir.join("joiner"));
    let (req, _) = join(&joiner, &mut jq, NOW);
    assert!(request_waits(&joiner, &jq));
    assert!(aq.keep_request(&site, &req, NOW));
    let wire = reject(&admin, &mut aq, &hexk(4), NOW).unwrap();
    let (s, id, a, sig) = decline_of(&wire);
    assert_eq!((s, a, id), (site, pk(1), jq.mine_id().unwrap()));
    // The joiner takes it once. A replay finds no request that it sent.
    assert!(on_decline(&joiner, &mut jq, &s, &id, &a, &sig));
    assert!((jq.mine.is_none(), jq.declined) == (true, true) && !request_waits(&joiner, &jq));
    assert!(!on_decline(&joiner, &mut jq, &s, &id, &a, &sig));
    // The state survives a restart.
    assert!(Queue::load(&w.dir.join("joiner")).declined);
    // Send again: a new request, a new profile file, and the declined state is gone.
    let (req2, id2) = join_again(&joiner, &mut jq, NOW + 10).unwrap();
    assert_ne!(req, req2);
    assert!(request_waits(&joiner, &jq) && !jq.declined);
    assert!(jq.entries.iter().any(|e| e.id == id2 && matches!(e.intent, Intent::Publish { .. })));
    assert!(aq.keep_request(&site, &req2, NOW), "the admin sees the new request");
    assert_eq!(pending_rows(&admin, &aq, NOW).len(), 1);
    // Cancel and send again while the request waits, with the admin away: it needs nobody.
    let (req3, _) = join_again(&joiner, &mut jq, NOW + 20).unwrap();
    assert!(req3 != req2 && jq.mine.as_deref() == Some(&req3[..]));
    // The old request is still valid on the admin side, and the newest one wins there.
    assert!(aq.keep_request(&site, &req3, NOW));
    assert!(!aq.keep_request(&site, &req2, NOW));
    assert_eq!(pending_rows(&admin, &aq, NOW).len(), 1);
    // A member with a role cannot use it.
    assert!(join_again(&admin, &mut jq, NOW).unwrap_err().contains("role"));
}

#[test]
fn a_forged_or_unsolicited_decline_is_ignored() {
    let w = World::new();
    w.put_member(&w.member(3, Role::Resident));
    let admin = node(&w, 1);
    let joiner = node(&w, 4);
    let mut aq = locked(&w);
    let mut jq = Queue::load(&w.dir.join("joiner"));
    let (req, _) = join(&joiner, &mut jq, NOW);
    let id = jq.mine_id().unwrap();
    let site = w.site_id();
    let good = joinreq::sign_decline(&key(1), &site, &id);
    let mut flipped = good.clone();
    flipped[5] ^= 1;
    let others = request_id(&request_of(5, &w, NOW).try_into().unwrap());
    let cases: Vec<(&str, Key, Key, Key, Vec<u8>)> = vec![
        ("bad signature", site, id, pk(1), flipped),
        ("signed by a stranger", site, id, pk(5), joinreq::sign_decline(&key(5), &site, &id)),
        ("signed by a resident", site, id, pk(3), joinreq::sign_decline(&key(3), &site, &id)),
        ("a request that we did not send", site, others, pk(1), joinreq::sign_decline(&key(1), &site, &others)),
        ("another site", [9; 32], id, pk(1), joinreq::sign_decline(&key(1), &[9; 32], &id)),
        ("signature of another request", site, id, pk(1), joinreq::sign_decline(&key(1), &site, &others)),
        ("short signature", site, id, pk(1), good[..63].to_vec()),
    ];
    for (what, s, i, a, sig) in cases {
        assert!(!on_decline(&joiner, &mut jq, &s, &i, &a, &sig), "{what}");
        assert!(jq.mine.is_some() && !jq.declined, "{what}: nothing changed");
    }
    assert!(Wire::decode(&Wire::JoinDecline { site, request_id: id, admin: pk(1), sig: good[..63].to_vec() }.encode()).is_none());
    // The real one works, so the checks above were not too strict.
    assert!(on_decline(&joiner, &mut jq, &site, &id, &pk(1), &good));
    // An admin cannot decline a request that is not stored or already spent.
    assert!(reject(&admin, &mut aq, &hexk(4), NOW).is_err());
    let _ = req;
}

#[test]
fn a_spent_request_does_not_show_as_waiting_and_cannot_be_approved() {
    let w = World::new();
    let site = w.site_id();
    // Member 2 was approved with this request, then removed: the chain keeps its nonce.
    let req = request_of(2, &w, NOW);
    let mut m = removed(&w, 2);
    m.last_req_nonce = NOW;
    w.put_member(&m);
    let admin = node(&w, 1);
    let _ = joiner(&w, 2, 0);
    let mut q = locked(&w);
    assert!(q.keep_request(&site, &req, NOW));
    assert!(pending_rows(&admin, &q, NOW).is_empty(), "the chain used this request");
    assert!(grant(&admin, &mut q, &hexk(2), "resident", "", NOW).unwrap_err().contains("no new join request"));
    assert!(q.entries.is_empty());
    // Their new request has a higher nonce: it shows and can be approved.
    assert!(q.keep_request(&site, &request_of(2, &w, NOW + 1), NOW));
    assert_eq!(pending_rows(&admin, &q, NOW).len(), 1);
}

#[test]
fn the_new_wire_message_is_appended_and_old_ones_keep_their_bytes() {
    let tag = |w: Wire| w.encode()[0];
    assert_eq!(tag(Wire::Event(vec![])), 0);
    assert_eq!(tag(Wire::Heads(vec![])), 1);
    assert_eq!(tag(Wire::JoinReq(vec![])), 2);
    assert_eq!(tag(Wire::BlobReq { cid: "c".into(), sha256: [0; 32] }), 6);
    assert_eq!(tag(Wire::BlobChunk { cid: "c".into(), sha256: [0; 32], index: 0, total: 1, bytes: vec![1] }), 7);
    let d = Wire::JoinDecline { site: [1; 32], request_id: [2; 32], admin: [3; 32], sig: vec![4; 64] }.encode();
    assert_eq!((d[0], d.len()), (8, 1 + 3 * 32 + 1 + 64));
}
