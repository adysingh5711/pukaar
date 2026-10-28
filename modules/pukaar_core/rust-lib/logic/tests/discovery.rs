//! Step 2.3b-1: the Storage fetchers and the member discovery, against a fake Store and a fake RPC.
//! The program has no member list, so the app learns the accounts from the keyring and from the
//! accounts that events, join requests and announces name. The chain `Member` record is the truth.
use pukaar_logic::chain::{b58, member_pda, site_pda, AccountId};
use pukaar_logic::event::{sha256, Body, Key, Location};
use pukaar_logic::fetch::{fetch_cycle, sync_once, Details, Fetched, Places, SharedFetched, Store, DETAILS_SCHEMA};
use pukaar_logic::lezkey::{derive_lez_secret, lez_account_id};
use pukaar_logic::node::Node;
use pukaar_logic::profile::{self, Announce};
use pukaar_logic::reducer::Rank;
use pukaar_logic::sync::{Wire, BLOB_CHUNK, MAX_CHUNKS};
use std::time::Duration;
use pukaar_logic::rpc::{Chain, Rpc, SharedChain, MAX_HINTS};
use pukaar_sites_rules::{GateConfig, Member, MemberStatus, Revocation, Role, Site, RECORD_VERSION};
use ed25519_dalek::SigningKey;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

const PROGRAM: AccountId = [1; 32];
const SITE: Key = [0xAA; 32];

fn sk(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}
fn key(n: u8) -> Key {
    sk(n).verifying_key().to_bytes()
}
fn acct(n: u8) -> AccountId {
    lez_account_id(&derive_lez_secret(&[n; 32]))
}
/// The keyring entry of user `n`: (LEZ account, Pukaar key).
fn seat(n: u8) -> (Key, Key) {
    (acct(n), key(n))
}

#[derive(Default)]
struct FakeStore {
    blobs: Mutex<HashMap<String, Vec<u8>>>,
    down: AtomicBool,
    downloads: AtomicUsize,
}

impl Store for FakeStore {
    fn upload(&self, bytes: &[u8]) -> Result<String, String> {
        let cid = format!("zb{}", hex::encode(&sha256(bytes)[..16]));
        self.blobs.lock().unwrap().insert(cid.clone(), bytes.to_vec());
        Ok(cid)
    }
    fn download(&self, cid: &str) -> Result<Vec<u8>, String> {
        if self.down.load(Ordering::SeqCst) {
            return Err("the storage node is not running".into());
        }
        self.downloads.fetch_add(1, Ordering::SeqCst);
        self.blobs.lock().unwrap().get(cid).cloned().ok_or_else(|| format!("{cid} not found"))
    }
}

#[derive(Default)]
struct FakeRpc {
    accounts: Mutex<HashMap<String, String>>,
}

impl FakeRpc {
    fn put(&self, at: &AccountId, record: &[u8]) {
        let body = json!({"jsonrpc":"2.0","id":1,"result":{"nonce":0,"data":{"shards":{ b58(&PROGRAM): record }}}});
        self.accounts.lock().unwrap().insert(b58(at), body.to_string());
    }
}

impl Rpc for FakeRpc {
    fn call(&self, method: &str, params: Value) -> Result<String, String> {
        match method {
            "getLastBlockId" => Ok(r#"{"jsonrpc":"2.0","id":1,"result":100}"#.into()),
            "getAccount" => {
                let id = params[0].as_str().unwrap();
                Ok(self.accounts.lock().unwrap().get(id).cloned().unwrap_or_else(|| r#"{"jsonrpc":"2.0","id":1,"result":{"nonce":0,"data":{"shards":{}}}}"#.into()))
            }
            _ => Err("no such method".into()),
        }
    }
}

fn details() -> Details {
    Details {
        schema: DETAILS_SCHEMA.into(),
        name: "Dhun Camp".into(),
        categories: vec!["water".into()],
        sla_ack_h: 24,
        sla_fix_h: 72,
        max_open_per_author: 5,
        revoke_reasons: BTreeMap::new(),
    }
}

fn places() -> Places {
    let hidden = Location { code: "W-02".into(), label: "Old tap".into(), group: "Water".into() };
    Places { retired: vec![hidden], ..Places::new(vec![Location { code: "W-01".into(), label: "Tap".into(), group: "Water".into() }]) }
}

struct World {
    store: FakeStore,
    rpc: FakeRpc,
    keys: BTreeMap<u64, Key>,
}

fn member(n: u8, role: Role, profile: &(String, [u8; 32])) -> Member {
    Member {
        version: RECORD_VERSION,
        site_id: SITE,
        user: acct(n),
        pukaar_key: key(n),
        role,
        status: MemberStatus::Active,
        granted_by: acct(1),
        granted_at_ms: u64::from(n) * 1000,
        expires_at_ms: 0,
        notice: None,
        revoked: None,
        rights_override: 0,
        profile_cid: profile.0.clone(),
        profile_sha256: profile.1,
        profile_version: 1,
        gate_attestation: None,
        request_id: [0; 32],
        requested_at_ms: 0,
        last_req_nonce: 0,
    }
}

impl World {
    fn put(&self, bytes: &[u8]) -> (String, [u8; 32]) {
        (self.store.upload(bytes).unwrap(), sha256(bytes))
    }

    /// A profile of user `n` named "User n", sealed under the site key of epoch 1.
    fn profile(&self, n: u8) -> (String, [u8; 32]) {
        let body = profile::Body { display_name: format!("User{n}"), bio: format!("bio{n}"), experience: format!("exp{n}"), ..Default::default() };
        let e = profile::build(&sk(n), &SITE, &acct(n), 1, 1, &body, Some(&self.keys[&1]), &[]).unwrap();
        self.put(&profile::to_bytes(&e))
    }

    fn put_member(&self, m: &Member) {
        self.rpc.put(&member_pda(&PROGRAM, &SITE, &m.user), &m.encode());
    }

    /// User 1 created the site and is its super admin. `members` are the chain records (user, role),
    /// `ring` the entries of the keyring file, `details` the site settings.
    fn new(members: &[(u8, Role)], ring: &[(Key, Key)], details: &Details) -> World {
        let mut w = World { store: FakeStore::default(), rpc: FakeRpc::default(), keys: BTreeMap::new() };
        let keyring = profile::rotate(&SITE, &mut w.keys, ring).unwrap();
        let (d, p, k) = (w.put(&details.to_bytes()), w.put(&places().to_bytes()), w.put(&profile::keyring_to_bytes(&keyring)));
        let site = Site {
            version: RECORD_VERSION,
            site_id: SITE,
            creator: acct(1),
            salt: [0; 32],
            super_admins: [Some(acct(1)), None],
            details_cid: d.0,
            details_sha256: d.1,
            places_cid: p.0,
            places_sha256: p.1,
            key_epoch: 1,
            keyring_cid: k.0,
            keyring_sha256: k.1,
            gates: GateConfig::default(),
            access_duration_ms: 0,
            renew_window_ms: 0,
            roster_len: members.len() as u32,
            created_at_ms: 1,
            updated_at_ms: 1,
        };
        w.rpc.put(&site_pda(&PROGRAM, &SITE), &site.encode());
        for (n, role) in members {
            let pr = w.profile(*n);
            w.put_member(&member(*n, *role, &pr));
        }
        w
    }

    /// The node of user `n` with a chain cache and fetchers, as `wire_chain` builds them.
    fn node(&self, n: u8) -> (Node, SharedChain, SharedFetched) {
        let mut node = Node::join(sk(n), SITE);
        let (c, f) = (Chain::shared(PROGRAM, SITE, acct(n)), Fetched::shared());
        node.attach_chain(c.clone());
        node.attach_fetched(f.clone());
        (node, c, f)
    }

    fn sync(&self, node: &mut Node, c: &SharedChain, f: &SharedFetched) {
        sync_once(&self.rpc, &self.store, c, f, &node.key.clone());
        node.apply_fetched();
    }
}

fn info(n: &Node) -> Value {
    serde_json::from_str(&n.site_info_json()).unwrap()
}

fn names(v: &Value) -> Vec<String> {
    let mut out: Vec<String> = v["members"].as_array().unwrap().iter().map(|m| m["name"].as_str().unwrap_or("").to_string()).collect();
    out.sort();
    out
}

const TRIO: [(u8, Role); 3] = [(1, Role::Admin), (2, Role::Admin), (3, Role::Resident)];

#[test]
fn the_keyring_lists_the_members_and_the_fetchers_fill_site_info() {
    let w = World::new(&TRIO, &[seat(1), seat(2), seat(3)], &details());
    // User 3 knows only itself, the creator and the super admin. The keyring gives the rest.
    let (mut n, c, f) = w.node(3);
    assert!(info(&n)["name"] == "" && info(&n)["members"].as_array().unwrap().is_empty(), "nothing before the first sync");
    w.sync(&mut n, &c, &f);
    let i = info(&n);
    assert_eq!((i["name"].as_str(), i["sla_ack_h"].as_u64(), i["sla_fix_h"].as_u64()), (Some("Dhun Camp"), Some(24), Some(72)));
    assert_eq!(i["categories"], json!(["water"]));
    assert_eq!(i["locations"].as_array().unwrap().len(), 1);
    assert_eq!((i["locations"][0]["ever_used"].as_bool(), i["locations"][0]["open_issues"].as_u64()), (Some(false), Some(0)));
    // A hidden place is listed apart, so the UI can show it and an admin can restore it.
    assert_eq!(i["retired_locations"].as_array().unwrap().iter().map(|l| l["code"].as_str().unwrap()).collect::<Vec<_>>(), ["W-02"]);
    assert_eq!(names(&i), ["User1", "User2", "User3"]);
    assert_eq!(i["chain"]["known"], 3);
    assert_eq!(i["storage"], json!({ "pending": 0, "error": null }));
    assert_eq!(n.role_now(&key(2)), Some(Rank::Admin));
    assert_eq!(n.role_now(&key(1)), Some(Rank::SuperAdmin));
    let me: Value = serde_json::from_str(&n.identity_json()).unwrap();
    assert_eq!((me["name"].as_str(), me["role"].as_str()), (Some("User3"), Some("Resident")));
    assert_eq!((me["bio"].as_str(), me["experience"].as_str()), (Some("bio3"), Some("exp3")));
}

#[test]
fn a_keyring_entry_that_lies_about_its_key_is_ignored() {
    // The keyring says user 3 holds key 9. The chain record of user 3 holds key 3.
    let w = World::new(&TRIO, &[seat(1), seat(2), (acct(3), key(9))], &details());
    let (mut n, c, f) = w.node(1);
    w.sync(&mut n, &c, &f);
    let i = info(&n);
    assert_eq!(names(&i), ["User1", "User2"], "the record at account 3 is dropped");
    assert_eq!(n.role_now(&key(3)), None);
    assert_eq!(n.role_now(&key(9)), None, "the liar's key is nobody either");
    assert_eq!(c.lock().unwrap().role_of_author(&key(3), Some(&acct(3)), 0), None);
}

fn report(n: &mut Node) -> Body {
    let _ = n;
    Body::Report { category: "water".into(), location: "W-01".into(), landmark: String::new(), text: "leak".into() }
}

#[test]
fn an_event_from_an_unknown_author_is_judged_by_the_record_at_its_account() {
    // User 5 is a member on the chain but not in the keyring file (the file is older).
    let w = World::new(&[(1, Role::Admin), (2, Role::Admin), (5, Role::Resident)], &[seat(1), seat(2)], &details());
    let (mut a, c, f) = w.node(1);
    w.sync(&mut a, &c, &f);
    assert_eq!(names(&info(&a)), ["User1", "User2"], "user 5 is unknown");

    let mut x = Node::join(sk(5), SITE);
    let e = {
        let b = report(&mut x);
        x.publish(b, 10).unwrap()
    };
    assert_eq!(x.account, acct(5), "an event names the LEZ account of its author");
    a.receive(&e.bytes).unwrap();
    assert!(a.state().rejected.contains_key(&e.id), "rejected until the account is read");
    w.sync(&mut a, &c, &f);
    assert!(!a.state().rejected.contains_key(&e.id), "accepted once the record at its account is read");
    assert!(a.state().issues.contains_key(&e.id));
    assert_eq!(names(&info(&a)), ["User1", "User2", "User5"]);
}

#[test]
fn a_lying_author_account_is_rejected() {
    let w = World::new(&[(1, Role::Admin), (2, Role::Admin), (5, Role::Resident)], &[seat(1), seat(2)], &details());
    let (mut a, c, f) = w.node(1);
    // Key 7 has no record and claims the account of user 5. Key 5 claims the account of user 2.
    // Key 6 claims an account that holds nothing.
    let mut send = |signer: u8, account: AccountId| {
        let mut x = Node::join(sk(signer), SITE);
        x.account = account;
        let b = report(&mut x);
        let e = x.publish(b, 10).unwrap();
        a.receive(&e.bytes).unwrap();
        e
    };
    let (by7, by5, by6) = (send(7, acct(5)), send(5, acct(2)), send(6, [0x66; 32]));
    let honest = send(5, acct(5));
    w.sync(&mut a, &c, &f);
    let st = a.state();
    for e in [&by7, &by5, &by6] {
        assert!(st.rejected[&e.id].contains("not an approved member"), "a wrong account has no role");
    }
    assert!(!st.rejected.contains_key(&honest.id), "the right account is accepted");
    // The liars cost nothing lasting: the account without a record is dropped after the read.
    assert!(c.lock().unwrap().hint([0x66; 32], None), "a hint without a record was dropped");
}

#[test]
fn storage_not_delivered_shows_empty_fields_and_a_pending_count() {
    let w = World::new(&TRIO, &[seat(1), seat(2), seat(3)], &details());
    w.store.down.store(true, Ordering::SeqCst);
    let (mut n, c, f) = w.node(3);
    w.sync(&mut n, &c, &f);
    let i = info(&n);
    assert_eq!((i["name"].as_str(), i["categories"].as_array().unwrap().len(), i["locations"].as_array().unwrap().len()), (Some(""), 0, 0));
    assert!(i["storage"]["pending"].as_u64().unwrap() >= 3, "keyring, details and places are missing");
    assert!(i["storage"]["error"].as_str().unwrap().contains("not running"));
    // The chain part still works: our own record gives the role without any name.
    assert_eq!(n.role_now(&key(3)), Some(Rank::Resident));

    w.store.down.store(false, Ordering::SeqCst);
    w.sync(&mut n, &c, &f);
    let i = info(&n);
    assert_eq!((i["name"].as_str(), i["storage"]["pending"].as_u64()), (Some("Dhun Camp"), Some(0)));
    assert_eq!(names(&i), ["User1", "User2", "User3"]);
}

#[test]
fn a_blob_that_does_not_match_the_chain_hash_is_not_used_and_is_fetched_again() {
    let w = World::new(&TRIO, &[seat(1), seat(2), seat(3)], &details());
    // Someone serves other bytes under the CID of details.json (the CID follows the content).
    let (cid, _) = w.put(&details().to_bytes());
    let good = w.store.blobs.lock().unwrap().insert(cid.clone(), details_with_name("Fake Camp").to_bytes()).unwrap();
    let (mut n, c, f) = w.node(3);
    w.sync(&mut n, &c, &f);
    let i = info(&n);
    assert_eq!(i["name"], "", "the forged details are not shown");
    assert!(i["storage"]["error"].as_str().unwrap().contains("does not match the SHA-256"));
    assert_eq!(i["storage"]["pending"], 1);
    assert_eq!(names(&i), ["User1", "User2", "User3"], "the other files are fine");
    // The honest copy comes back: the next round shows it.
    w.store.blobs.lock().unwrap().insert(cid, good);
    fetch_cycle(&w.store, &c, &f, &sk(3));
    n.apply_fetched();
    assert_eq!(info(&n)["name"], "Dhun Camp");
    assert_eq!(info(&n)["storage"]["pending"], 0);
}

fn details_with_name(name: &str) -> Details {
    Details { name: name.into(), ..details() }
}

#[test]
fn files_are_cached_by_cid() {
    let w = World::new(&TRIO, &[seat(1), seat(2), seat(3)], &details());
    let (mut n, c, f) = w.node(3);
    w.sync(&mut n, &c, &f);
    let after_first = w.store.downloads.load(Ordering::SeqCst);
    assert_eq!(after_first, 3 + 3, "keyring, details, places and three profiles, each once");
    fetch_cycle(&w.store, &c, &f, &sk(3));
    assert_eq!(w.store.downloads.load(Ordering::SeqCst), after_first, "a second round downloads nothing");
}

#[test]
fn a_removal_reason_comes_from_details_and_must_match_its_hash() {
    let (text, forged) = ("late three times", "this text is not the one that was hashed");
    let mut d = details();
    d.revoke_reasons.insert(hex::encode(sha256(text.as_bytes())), text.into());
    d.revoke_reasons.insert(hex::encode([7u8; 32]), forged.into());
    let w = World::new(&[(1, Role::Admin), (2, Role::Admin), (3, Role::Resident)], &[seat(1), seat(2)], &d);
    let mut m = member(3, Role::Resident, &w.profile(3));
    m.status = MemberStatus::Removed;
    m.revoked = Some(Revocation { by: acct(1), at_ms: 5000, reason_sha256: sha256(text.as_bytes()) });
    w.put_member(&m);
    // User 3 is not in the ring any more, so a node learns of the record only when it is told.
    let (mut n, c, f) = w.node(3);
    w.sync(&mut n, &c, &f);
    let gone = info(&n)["revoked"].as_array().unwrap().clone();
    assert_eq!(gone.len(), 1);
    assert_eq!(gone[0]["reason"], text);
    let me: Value = serde_json::from_str(&n.identity_json()).unwrap();
    assert_eq!(me["revoked"]["reason"], text, "the removed member reads the reason too");
    assert!(n.reasons.values().all(|t| t != forged), "a text under the wrong hash is dropped");
}

#[test]
fn a_join_request_and_an_announce_name_accounts_to_read() {
    let members = [(1, Role::Admin), (2, Role::Admin), (6, Role::Resident), (7, Role::Resident)];
    let w = World::new(&members, &[seat(1), seat(2)], &details());
    let (mut a, c, f) = w.node(1);

    // User 7: a join request names the account. It is read, so an approval gives a role at once.
    let req = pukaar_logic::joinreq::build(&sk(7), &derive_lez_secret(&[7; 32]), SITE, 1, u64::MAX);
    assert!(a.on_join_req(&req));
    assert!(!a.on_join_req(&req), "known already");
    let other_site = pukaar_logic::joinreq::build(&sk(7), &derive_lez_secret(&[7; 32]), [1; 32], 1, u64::MAX);
    assert!(!a.on_join_req(&other_site) && !a.on_join_req(&req[..100]));

    // User 6: an announce has only the Pukaar key. The profile file names the account.
    let pr = w.profile(6);
    let ann = Announce::sign(&sk(6), SITE, 1, &pr.0, pr.1);
    assert!(a.on_announce(ann.clone()));
    assert!(!a.on_announce(Announce { site: [2; 32], ..ann.clone() }), "another site");
    assert!(!a.on_announce(Announce { version: 2, ..ann }), "a bad signature");

    w.sync(&mut a, &c, &f);
    assert_eq!(names(&info(&a)), ["User1", "User2", "User6", "User7"]);
    assert_eq!(a.role_now(&key(6)), Some(Rank::Resident));
}

#[test]
fn an_announce_with_a_higher_version_shows_the_newer_name() {
    let w = World::new(&TRIO, &[seat(1), seat(2), seat(3)], &details());
    let (mut n, c, f) = w.node(1);
    w.sync(&mut n, &c, &f);
    assert_eq!(names(&info(&n)), ["User1", "User2", "User3"]);
    // User 3 edits the name. The chain pointer still holds version 1: the announce is newer.
    let body = profile::Body { display_name: "Asha K".into(), ..Default::default() };
    let e = profile::build(&sk(3), &SITE, &acct(3), 2, 1, &body, Some(&w.keys[&1]), &[]).unwrap();
    let (cid, sha) = w.put(&profile::to_bytes(&e));
    assert!(n.on_announce(Announce::sign(&sk(3), SITE, 2, &cid, sha)));
    w.sync(&mut n, &c, &f);
    assert_eq!(names(&info(&n)), ["Asha K", "User1", "User2"]);
}

#[test]
fn unverified_accounts_are_capped_and_a_known_key_is_not_read_again() {
    let (_, c, _) = World::new(&TRIO, &[seat(1)], &details()).node(1);
    let mut g = c.lock().unwrap();
    let fake = |i: usize| {
        let mut id = [9u8; 32];
        id[..2].copy_from_slice(&(i as u16).to_be_bytes());
        id
    };
    let added = (0..MAX_HINTS + 10).filter(|i| g.hint(fake(*i), None)).count();
    assert_eq!(added, MAX_HINTS);
}

// ---- Delivery fallback: Storage cannot connect, the files come over BlobReq and BlobChunk ----

/// Wire round trip, as the module does it.
fn over_the_wire(w: &Wire) -> Wire {
    Wire::decode(&w.encode()).expect("a valid message")
}

/// Every request that `joiner` made goes to `admin`, every chunk comes back. Returns the requests.
fn pump(admin: &Node, joiner: &Node, jf: &SharedFetched) -> Vec<Wire> {
    let reqs: Vec<Wire> = std::mem::take(&mut jf.lock().unwrap().out);
    for r in &reqs {
        for chunk in admin.on_blob(&over_the_wire(r), 0) {
            joiner.on_blob(&over_the_wire(&chunk), 0);
        }
    }
    reqs
}

/// An admin that holds the files (it read them from its Storage), a joiner whose Storage is dead.
fn admin_and_dead_joiner(w: &World) -> (Node, Node, SharedChain, SharedFetched) {
    let (mut a, ac, af) = w.node(1);
    w.sync(&mut a, &ac, &af);
    w.store.down.store(true, Ordering::SeqCst);
    let (j, jc, jf) = w.node(3);
    (a, j, jc, jf)
}

fn joiner_reaches_the_site(details: &Details, name_len: usize) {
    let w = World::new(&TRIO, &[seat(1), seat(2), seat(3)], details);
    let (a, mut j, jc, jf) = admin_and_dead_joiner(&w);
    let mut asked = 0;
    for _ in 0..4 {
        w.sync(&mut j, &jc, &jf);
        asked += pump(&a, &j, &jf).len();
    }
    w.sync(&mut j, &jc, &jf);
    let i = info(&j);
    assert!(asked >= 6, "keyring, details, places and the profiles were asked for: {asked}");
    assert_eq!(i["name"].as_str().map(str::len), Some(name_len));
    assert_eq!(i["locations"].as_array().unwrap().len(), 1);
    assert_eq!(names(&i), ["User1", "User2", "User3"], "the admin profile came too");
    assert_eq!(i["storage"], json!({ "pending": 0, "error": null }));
    assert!(jf.lock().unwrap().out.is_empty(), "nothing left to ask");
}

#[test]
fn the_site_shows_through_delivery_when_storage_cannot_connect() {
    joiner_reaches_the_site(&details(), "Dhun Camp".len());
}

#[test]
fn a_file_of_100_kb_comes_in_chunks_and_reassembles() {
    let big = Details { name: "x".repeat(100_000), ..details() };
    assert!(big.to_bytes().len() > 100_000 && big.to_bytes().len().div_ceil(BLOB_CHUNK) > 30);
    joiner_reaches_the_site(&big, 100_000);
}

#[test]
fn when_storage_works_no_request_goes_out() {
    let w = World::new(&TRIO, &[seat(1), seat(2), seat(3)], &details());
    let (mut n, c, f) = w.node(3);
    w.sync(&mut n, &c, &f);
    assert!(f.lock().unwrap().out.is_empty());
    assert_eq!(info(&n)["storage"]["pending"], 0);
}

#[test]
fn an_offline_admin_leaves_the_joiner_asking_and_waiting() {
    let w = World::new(&TRIO, &[seat(1), seat(2), seat(3)], &details());
    w.store.down.store(true, Ordering::SeqCst);
    let (mut n, c, f) = w.node(3);
    w.sync(&mut n, &c, &f);
    let first = std::mem::take(&mut f.lock().unwrap().out);
    assert!(first.iter().any(|m| matches!(m, Wire::BlobReq { .. })), "it asks at once");
    w.sync(&mut n, &c, &f);
    assert!(f.lock().unwrap().out.is_empty(), "not again inside the back-off");
    f.lock().unwrap().req_every = Duration::ZERO; // the back-off is over
    w.sync(&mut n, &c, &f);
    let again = std::mem::take(&mut f.lock().unwrap().out);
    assert!(!again.is_empty() && again.iter().all(|m| first.contains(m)), "it asks again for the same files");
    let i = info(&n);
    assert!(i["storage"]["pending"].as_u64().unwrap() >= 3, "still waiting for the site data");
    assert!(i["storage"]["error"].as_str().is_some());
}

#[test]
fn bad_unasked_repeated_and_oversize_chunks_are_refused() {
    let w = World::new(&TRIO, &[seat(1), seat(2), seat(3)], &details());
    let (a, mut j, jc, jf) = admin_and_dead_joiner(&w);
    w.sync(&mut j, &jc, &jf);
    let reqs = std::mem::take(&mut jf.lock().unwrap().out);
    let Some(Wire::BlobReq { cid, sha256 }) = reqs.into_iter().next() else { panic!("a request") };
    let good: Vec<Wire> = a.on_blob(&Wire::BlobReq { cid: cid.clone(), sha256 }, 0);
    let Wire::BlobChunk { bytes, total, .. } = &good[0] else { panic!("a chunk") };
    assert_eq!(*total, 1, "a small file is one chunk");
    let chunk = |cid: &str, sha: [u8; 32], index: u16, total: u16, b: &[u8]| Wire::BlobChunk { cid: cid.into(), sha256: sha, index, total, bytes: b.to_vec() };
    let try_chunk = |c: Wire| {
        j.on_blob(&c, 0);
        jf.lock().unwrap().cached(&cid, &sha256).is_some()
    };
    let mut evil = bytes.clone();
    evil[0] ^= 1;
    assert!(!try_chunk(chunk(&cid, sha256, 0, 1, &evil)), "a bad hash");
    assert!(!try_chunk(chunk("zOther", sha256, 0, 1, bytes)), "a cid that we did not ask for");
    assert!(!try_chunk(chunk(&cid, [9; 32], 0, 1, bytes)), "another sha than the one that we asked with");
    assert!(!try_chunk(chunk(&cid, sha256, 0, 1, &vec![1; BLOB_CHUNK + 1])), "a chunk over the size");
    assert!(!try_chunk(chunk(&cid, sha256, 0, MAX_CHUNKS as u16 + 1, bytes)), "too many chunks");
    assert!(!try_chunk(chunk(&cid, sha256, 1, 1, bytes)), "an index past the total");
    // Chunks that together pass the largest file: the last ones are dropped, the file never completes.
    for i in 0..MAX_CHUNKS as u16 {
        assert!(!try_chunk(chunk(&cid, sha256, i, MAX_CHUNKS as u16, &vec![3; BLOB_CHUNK])));
    }
    // A spoiled start (another total, a half file) does not block the honest file, which completes.
    assert!(!try_chunk(chunk(&cid, sha256, 0, 2, &bytes[..bytes.len() / 2])));
    assert!(!try_chunk(chunk(&cid, sha256, 0, 2, &bytes[..bytes.len() / 2])), "a repeat of a chunk");
    assert!(try_chunk(chunk(&cid, sha256, 0, 1, bytes)), "the honest chunk is accepted");
    assert!(!jf.lock().unwrap().note_chunk(&cid, &sha256, 0, 1, bytes), "a file that we hold needs no chunk");
    assert_eq!(jf.lock().unwrap().cached(&cid, &sha256).as_deref(), Some(&bytes[..]));
}

#[test]
fn a_flood_of_requests_gets_few_answers() {
    let mut f = Fetched::default();
    let blobs: Vec<(String, Vec<u8>)> = (0..8u8).map(|n| (format!("c{n}"), vec![n; 250_000])).collect();
    for (cid, b) in &blobs {
        f.put(cid, b);
    }
    // The same file again and again: one answer. A file that we do not hold, or a wrong sha: none.
    let sha0 = sha256(&blobs[0].1);
    assert_eq!(f.answer("c0", &sha0).len(), 84);
    assert!((0..100).all(|_| f.answer("c0", &sha0).is_empty()));
    assert!(f.answer("nope", &sha0).is_empty() && f.answer("c1", &sha0).is_empty());
    // Many different files: the chunks of one minute stop at the cap.
    let sent: usize = blobs.iter().map(|(c, b)| f.answer(c, &sha256(b)).len()).sum::<usize>() + 84;
    assert!(sent <= 400 && sent > 300, "{sent} chunks in a minute");
}
