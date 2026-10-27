//! The queue against a fake chain, a fake Storage and a fake wallet. Nothing is sent anywhere.

use super::*;
use crate::chain::{b58, AccountId};
use crate::fetch::Fetched;
use crate::rpc::Chain;
use pukaar_sites_rules::{apply_create, CreateSite, GateConfig, Instruction};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

pub(crate) const PROGRAM: AccountId = [7; 32];
pub(crate) const T0: u64 = 1_000_000;
pub(crate) const CLOCK_NOW: u64 = 5_000_000;

pub(crate) fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}
pub(crate) fn pk(n: u8) -> Key {
    key(n).verifying_key().to_bytes()
}
pub(crate) fn acct(n: u8) -> AccountId {
    lez_account_id(&derive_lez_secret(&[n; 32]))
}

/// `getAccount` from a map. An account that is not in it holds nothing.
#[derive(Default)]
pub(crate) struct FakeRpc {
    /// base58 id -> (program of the shard, bytes)
    accounts: Mutex<HashMap<String, (AccountId, Vec<u8>)>>,
}

impl FakeRpc {
    fn put(&self, id: &AccountId, program: &AccountId, bytes: Vec<u8>) {
        self.accounts.lock().unwrap().insert(b58(id), (*program, bytes));
    }
}

impl Rpc for FakeRpc {
    fn call(&self, method: &str, params: Value) -> Result<String, String> {
        match method {
            "getLastBlockId" => Ok(r#"{"result":100}"#.into()),
            "getAccount" => {
                let id = params[0].as_str().unwrap();
                let shards = match self.accounts.lock().unwrap().get(id) {
                    Some((p, bytes)) => Value::Object([(b58(p), json!(bytes))].into_iter().collect()),
                    None => json!({}),
                };
                Ok(json!({ "result": { "nonce": 0, "data": { "shards": shards } } }).to_string())
            }
            _ => Err("no such method".into()),
        }
    }
}

#[derive(Default)]
pub(crate) struct FakeStore {
    blobs: Mutex<HashMap<String, Vec<u8>>>,
    down: std::sync::atomic::AtomicBool,
    /// Every download fails, and is counted in `refused`: a remote node that cannot be reached.
    pub(crate) no_download: std::sync::atomic::AtomicBool,
    pub(crate) refused: AtomicUsize,
}

impl FakeStore {
    pub(crate) fn put(&self, bytes: &[u8]) -> String {
        let cid = format!("cid{}", &hex::encode(sha256(bytes))[..16]);
        self.blobs.lock().unwrap().insert(cid.clone(), bytes.to_vec());
        cid
    }
    pub(crate) fn get(&self, cid: &str) -> Vec<u8> {
        self.blobs.lock().unwrap()[cid].clone()
    }
}

impl Store for FakeStore {
    fn upload(&self, bytes: &[u8]) -> Result<String, String> {
        if self.down.load(Ordering::Relaxed) {
            return Err("no peers".into());
        }
        Ok(self.put(bytes))
    }
    fn download(&self, cid: &str) -> Result<Vec<u8>, String> {
        if self.no_download.load(Ordering::Relaxed) {
            self.refused.fetch_add(1, Ordering::Relaxed);
            return Err("timed out waiting for Storage".into());
        }
        self.blobs.lock().unwrap().get(cid).cloned().ok_or_else(|| "not found".into())
    }
}

pub(crate) struct FakeWriter {
    import: Mutex<Result<(), String>>,
    imports: AtomicU32,
    sent: Mutex<Vec<Tx>>,
    polls: Mutex<VecDeque<Poll>>,
    on_send: Mutex<Option<Box<dyn Fn(&Tx) + Send>>>,
}

impl ChainWriter for FakeWriter {
    fn import_key(&self, secret_hex: &str) -> Result<(), String> {
        assert_eq!(secret_hex.len(), 64);
        self.imports.fetch_add(1, Ordering::Relaxed);
        self.import.lock().unwrap().clone()
    }
    fn send(&self, tx: &Tx) -> Result<String, String> {
        if let Some(f) = &*self.on_send.lock().unwrap() {
            f(tx);
        }
        let mut s = self.sent.lock().unwrap();
        s.push(tx.clone());
        Ok(hex::encode([s.len() as u8; 32]))
    }
    fn poll(&self, _: &str) -> Poll {
        self.polls.lock().unwrap().pop_front().unwrap_or(Poll::Pending)
    }
}

static DIRS: AtomicUsize = AtomicUsize::new(0);

pub(crate) struct World {
    pub(crate) rpc: Arc<FakeRpc>,
    pub(crate) store: Arc<FakeStore>,
    pub(crate) writer: Arc<FakeWriter>,
    pub(crate) chain: SharedChain,
    pub(crate) fetched: SharedFetched,
    pub(crate) key: SigningKey,
    pub(crate) site: Site,
    pub(crate) creator: Member,
    pub(crate) out: Mutex<Vec<Wire>>,
    pub(crate) q: Mutex<Queue>,
    pub(crate) dir: std::path::PathBuf,
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

impl World {
    /// A site of creator 1 (super admin, us) with its three files in Storage.
    pub(crate) fn new() -> World {
        let dir = std::env::temp_dir().join(format!("pukaar-queue-{}-{}", std::process::id(), DIRS.fetch_add(1, Ordering::Relaxed)));
        let (rpc, store) = (Arc::new(FakeRpc::default()), Arc::new(FakeStore::default()));
        let details = Details {
            schema: crate::fetch::DETAILS_SCHEMA.into(),
            name: "Dhun".into(),
            categories: vec!["water".into()],
            sla_ack_h: 24,
            sla_fix_h: 72,
            max_open_per_author: 5,
            revoke_reasons: Default::default(),
        };
        let places = Places::new(vec![crate::event::Location { code: "W-01".into(), label: "Tap".into(), group: "Water".into() }]);
        let salt = [5; 32];
        let site_id = pukaar_sites_rules::seed::site_id(&acct(1), &salt);
        let ring = profile::rotate(&site_id, &mut std::collections::BTreeMap::new(), &[(acct(1), pk(1))]).unwrap();
        let (d, p, ring) = (details.to_bytes(), places.to_bytes(), profile::keyring_to_bytes(&ring));
        let args = CreateSite {
            creator: acct(1),
            salt,
            site_id,
            pukaar_key: pk(1),
            details_cid: store.put(&d),
            details_sha256: sha256(&d),
            places_cid: store.put(&p),
            places_sha256: sha256(&p),
            keyring_cid: store.put(&ring),
            keyring_sha256: sha256(&ring),
            gates: GateConfig::default(),
            access_duration_ms: 0,
            renew_window_ms: 0,
        };
        let (site, creator) = apply_create(&args, 1_000).unwrap();
        let w = World {
            rpc,
            store,
            writer: Arc::new(FakeWriter {
                import: Mutex::new(Ok(())),
                imports: AtomicU32::new(0),
                sent: Mutex::new(vec![]),
                polls: Mutex::new(VecDeque::new()),
                on_send: Mutex::new(None),
            }),
            chain: Chain::shared(PROGRAM, site_id, acct(1)),
            fetched: Fetched::shared(),
            key: key(1),
            site,
            creator,
            out: Mutex::new(vec![]),
            q: Mutex::new(Queue::load(&dir)),
            dir,
        };
        w.put_site(&w.site.clone());
        w.put_member(&w.creator.clone());
        w.rpc.put(&writes::CLOCK_ACCOUNT, &writes::clock_program(), [&[0u8; 8][..], &CLOCK_NOW.to_le_bytes()].concat());
        w
    }

    pub(crate) fn site_id(&self) -> Key {
        self.site.site_id
    }

    pub(crate) fn put_site(&self, s: &Site) {
        self.rpc.put(&site_pda(&PROGRAM, &s.site_id), &PROGRAM, s.encode());
    }

    pub(crate) fn put_member(&self, m: &Member) {
        self.rpc.put(&member_pda(&PROGRAM, &m.site_id, &m.user), &PROGRAM, m.encode());
    }

    pub(crate) fn member(&self, n: u8, role: pukaar_sites_rules::Role) -> Member {
        let mut m = self.creator.clone();
        (m.user, m.pukaar_key, m.role, m.granted_by) = (acct(n), pk(n), role, acct(1));
        m
    }

    pub(crate) fn read_member(&self, n: u8) -> Member {
        let id = member_pda(&PROGRAM, &self.site_id(), &acct(n));
        Member::decode(&self.rpc.accounts.lock().unwrap()[&b58(&id)].1).unwrap()
    }

    pub(crate) fn push(&self, intent: Intent, state: St, blobs: Vec<Blob>) -> u64 {
        self.q.lock().unwrap().push(self.site_id(), intent, state, blobs, T0)
    }

    pub(crate) fn tick(&self, now: u64) {
        let broadcast = |m: Wire| self.out.lock().unwrap().push(m);
        let env = Env {
            writer: &*self.writer,
            rpc: &*self.rpc,
            store: &*self.store,
            chain: &self.chain,
            fetched: &self.fetched,
            key: &self.key,
            program: PROGRAM,
            now_ms: now,
            broadcast: &broadcast,
        };
        tick(&self.q, &env);
    }

    pub(crate) fn entry(&self, id: u64) -> Entry {
        self.q.lock().unwrap().entries.iter().find(|e| e.id == id).cloned().unwrap()
    }

    pub(crate) fn sent(&self) -> Vec<Tx> {
        self.writer.sent.lock().unwrap().clone()
    }

    pub(crate) fn on_send(&self, f: impl Fn(&Tx) + Send + 'static) {
        *self.writer.on_send.lock().unwrap() = Some(Box::new(f));
    }

    pub(crate) fn poll_next(&self, p: Poll) {
        self.writer.polls.lock().unwrap().push_back(p);
    }
}

pub(crate) fn role_item(n: u8, r: pukaar_sites_rules::Role) -> Intent {
    Intent::SetRole { items: vec![(acct(n), writes::role_code(r))] }
}

#[test]
fn a_role_change_goes_sent_landed_done() {
    use pukaar_sites_rules::Role;
    let w = World::new();
    w.put_member(&w.member(2, Role::Resident));
    let id = w.push(role_item(2, Role::Steward), St::Queued, vec![]);

    w.tick(T0);
    let e = w.entry(id);
    assert_eq!((e.state, e.tx.as_deref()), (St::Sent, Some(&*hex::encode([1u8; 32]))));
    // The instruction is the builder's, with the chain clock and the rank we hold (super admin).
    let ctx = Ctx { program: PROGRAM, site_id: w.site_id(), signer: acct(1), claimed: Rank::SuperAdmin, now_ms: CLOCK_NOW };
    assert_eq!(w.sent()[0], writes::set_role_batch(&ctx, vec![(acct(2), Role::Steward)]));
    assert!(matches!(&w.out.lock().unwrap()[..], [Wire::ChainPending { tx, kind: WriteKind::SetRole }] if *tx == [1u8; 32]));

    w.tick(T0 + 5_000); // still pending
    assert_eq!(w.entry(id).state, St::Sent);
    w.poll_next(Poll::Included);
    w.tick(T0 + 10_000);
    assert_eq!(w.entry(id).state, St::Landed);
    w.tick(T0 + 15_000); // included, but the chain does not show the role yet
    assert_eq!(w.entry(id).state, St::Landed);
    let mut m = w.read_member(2);
    m.role = Role::Steward;
    w.put_member(&m);
    w.tick(T0 + 20_000);
    assert_eq!(w.entry(id).state, St::Done);
    assert_eq!(w.sent().len(), 1, "one send");
}

#[test]
fn a_rejected_write_keeps_its_reason_and_the_next_one_goes() {
    use pukaar_sites_rules::Role;
    let w = World::new();
    w.put_member(&w.member(2, Role::Resident));
    w.put_member(&w.member(3, Role::Resident));
    let a = w.push(role_item(2, Role::Steward), St::Queued, vec![]);
    let b = w.push(role_item(3, Role::Steward), St::Queued, vec![]);
    w.tick(T0);
    assert_eq!((w.entry(a).state, w.entry(b).state), (St::Sent, St::Queued), "one write per tick");
    w.poll_next(Poll::Rejected("balance too low".into()));
    w.tick(T0 + 5_000);
    let e = w.entry(a);
    assert_eq!((e.state, e.reason.as_deref()), (St::Rejected, Some("balance too low")));
    assert_eq!(w.entry(b).state, St::Queued);
    w.tick(T0 + 10_000);
    assert_eq!(w.entry(b).state, St::Sent);
    assert_eq!(w.sent().len(), 2);
    let s = w.q.lock().unwrap().status_json(&w.site_id(), T0);
    assert_eq!(s["writes"][0]["state"], "rejected");
    assert_eq!(s["writes"][0]["reason"], "balance too low");
    assert_eq!(s["writes"][1]["state"], "sent");
    assert_eq!(s["writes"][1]["tx"], hex::encode([2u8; 32])[..10]);
}

#[test]
fn timeout_comes_only_after_ten_minutes() {
    use pukaar_sites_rules::Role;
    let w = World::new();
    w.put_member(&w.member(2, Role::Resident));
    let id = w.push(role_item(2, Role::Steward), St::Queued, vec![]);
    w.tick(T0);
    w.tick(T0 + SEND_WINDOW_MS - 1_000);
    assert_eq!(w.entry(id).state, St::Sent, "not yet");
    w.tick(T0 + SEND_WINDOW_MS + 1_000);
    let e = w.entry(id);
    assert_eq!((e.state, e.reason.as_deref()), (St::Rejected, Some("timeout")));
    assert_eq!(w.sent().len(), 1, "no resend");
}

#[test]
fn included_but_not_applied_is_rejected_after_a_minute() {
    use pukaar_sites_rules::Role;
    let w = World::new();
    w.put_member(&w.member(2, Role::Resident));
    let id = w.push(role_item(2, Role::Steward), St::Queued, vec![]);
    w.tick(T0);
    w.poll_next(Poll::Included);
    w.tick(T0 + 1_000);
    assert_eq!(w.entry(id).state, St::Landed);
    w.tick(T0 + 1_000 + LAND_WINDOW_MS + 1);
    let e = w.entry(id);
    assert_eq!(e.state, St::Rejected);
    assert!(e.reason.unwrap().contains("did not apply"));
}

#[test]
fn an_effect_that_is_on_the_chain_is_not_sent() {
    use pukaar_sites_rules::Role;
    let w = World::new();
    w.put_member(&w.member(2, Role::Steward));
    let id = w.push(role_item(2, Role::Steward), St::Queued, vec![]);
    w.tick(T0);
    assert_eq!(w.entry(id).state, St::Done);
    assert!(w.sent().is_empty());
}

#[test]
fn a_restart_never_sends_twice() {
    use pukaar_sites_rules::Role;
    let w = World::new();
    w.put_member(&w.member(2, Role::Resident));
    // "Sending" is on disk before the wallet is called.
    let dir = w.dir.clone();
    w.on_send(move |_| {
        let on_disk = std::fs::read_to_string(dir.join("writes.json")).unwrap();
        assert!(on_disk.contains("\"Sending\""), "the state is saved before the send");
    });
    let id = w.push(role_item(2, Role::Steward), St::Queued, vec![]);
    w.tick(T0);
    assert_eq!(w.sent().len(), 1);

    // The module dies between the send and the save of the hash: the file says Sending.
    {
        let mut q = w.q.lock().unwrap();
        q.entries[0].state = St::Sending;
        q.entries[0].tx = None;
        q.save();
    }
    *w.q.lock().unwrap() = Queue::load(&w.dir);
    let e = w.entry(id);
    assert_eq!((e.state, e.tx.clone()), (St::Sent, None));
    w.tick(T0 + 5_000);
    assert_eq!((w.entry(id).state, w.sent().len()), (St::Sent, 1), "no new send, the chain is watched");
    // The effect shows up: done.
    let mut m = w.read_member(2);
    m.role = Role::Steward;
    w.put_member(&m);
    w.tick(T0 + 10_000);
    assert_eq!((w.entry(id).state, w.sent().len()), (St::Done, 1));

    // Another entry whose effect never shows: it ends in a timeout and is still not sent again.
    let id2 = w.push(role_item(2, Role::Admin), St::Sending, vec![]);
    w.q.lock().unwrap().save();
    let loaded = Queue::load(&w.dir);
    *w.q.lock().unwrap() = loaded;
    w.tick(T0);
    w.tick(T0 + SEND_WINDOW_MS + 1);
    assert_eq!(w.entry(id2).reason.as_deref(), Some("timeout"));
    assert_eq!(w.sent().len(), 1);
}

#[test]
fn without_key_import_every_write_fails_cleanly() {
    use pukaar_sites_rules::Role;
    let w = World::new();
    *w.writer.import.lock().unwrap() = Err("key import not available: no such method".into());
    w.put_member(&w.member(2, Role::Resident));
    // At start, with nothing queued, the key is imported once.
    w.tick(T0);
    w.tick(T0 + 1);
    assert_eq!(w.writer.imports.load(Ordering::Relaxed), 1, "one import at start");
    assert!(w.q.lock().unwrap().status_json(&w.site_id(), T0)["key"].as_str().unwrap().starts_with("key import not available"));

    let id = w.push(role_item(2, Role::Steward), St::Queued, vec![]);
    w.tick(T0 + 2);
    let e = w.entry(id);
    assert_eq!(e.state, St::Rejected);
    assert!(e.reason.unwrap().starts_with("key import not available"));
    assert!(w.sent().is_empty(), "nothing was sent");
    // The user installs a good lez_core: the next write imports again and goes.
    *w.writer.import.lock().unwrap() = Ok(());
    w.push(role_item(2, Role::Steward), St::Queued, vec![]);
    w.tick(T0 + 3);
    assert_eq!(w.sent().len(), 1);
    assert_eq!(w.q.lock().unwrap().status_json(&w.site_id(), T0)["key"], "ok");
}

pub(crate) fn decode(tx: &Tx) -> Instruction {
    borsh::from_slice(&tx.ix).unwrap()
}

#[test]
fn a_revoke_writes_the_reason_text_then_rotates_the_keyring() {
    use pukaar_sites_rules::Role;
    let w = World::new();
    // Members 2 and 3 are in the keyring and on the chain.
    let mut keys = std::collections::BTreeMap::new();
    let ring = profile::rotate(&w.site_id(), &mut keys, &[(acct(1), pk(1)), (acct(2), pk(2)), (acct(3), pk(3))]).unwrap();
    let ring_bytes = profile::keyring_to_bytes(&ring);
    let mut site = w.site.clone();
    (site.keyring_cid, site.keyring_sha256) = (w.store.put(&ring_bytes), sha256(&ring_bytes));
    w.put_site(&site);
    for n in [2, 3] {
        w.put_member(&w.member(n, Role::Resident));
    }
    let r = |n: u8, why: &str| RevokeReq { user: acct(n), key: pk(n), reason: why.into() };
    let items = vec![r(2, "spam"), r(3, "abuse")];
    let mut q = w.q.lock().unwrap();
    q.push(w.site_id(), Intent::Revoke { items: items.clone() }, St::Held, vec![], T0);
    let ids = q.make_batches(&w.site_id(), vec![], T0);
    drop(q);
    assert_eq!(ids.len(), 2, "details first, then the revoke");
    assert!(matches!(w.entry(ids[0]).intent, Intent::Details { .. }));

    // SetDetails: the chain pointer moves in the closure, as the program would do.
    let rpc = w.rpc.clone();
    let (s2, sid) = (site.clone(), w.site_id());
    w.on_send(move |tx| {
        if let Instruction::SetDetails(p) = borsh::from_slice(&tx.ix).unwrap() {
            let mut s = s2.clone();
            (s.details_cid, s.details_sha256) = (p.cid, p.sha256);
            rpc.put(&site_pda(&PROGRAM, &sid), &PROGRAM, s.encode());
        }
    });
    w.tick(T0); // prepare the details: reads details.json, adds the texts, uploads
    assert_eq!(w.entry(ids[0]).state, St::Queued);
    w.tick(T0 + 1);
    let Instruction::SetDetails(p) = decode(&w.sent()[0]) else { panic!("not set_details") };
    let d = Details::from_bytes(&w.store.get(&p.cid)).unwrap();
    assert_eq!(d.revoke_reasons[&hex::encode(sha256(b"spam"))], "spam");
    assert_eq!(d.reasons().len(), 2, "both texts match their hashes");
    assert_eq!(sha256(&w.store.get(&p.cid)), p.sha256);
    w.tick(T0 + 2);
    assert_eq!(w.entry(ids[0]).state, St::Done, "the pointer is on the chain");

    // The revoke: a new epoch, without 2 and 3.
    w.tick(T0 + 3);
    assert_eq!(w.entry(ids[1]).state, St::Queued);
    let b = w.entry(ids[1]).blobs[0].clone();
    let next = profile::keyring_from_bytes(&b.bytes).unwrap();
    assert_eq!(next.epoch, 2);
    assert_eq!(profile::entries(&next).unwrap(), vec![(acct(1), pk(1))]);
    assert!(profile::open_keyring(&next, &key(2)).is_err(), "a revoked member cannot read the new key");
    assert_eq!(profile::open_keyring(&next, &key(1)).unwrap().len(), 2, "the admin reads both epochs");
    w.tick(T0 + 4);
    let Instruction::RevokeBatch(rv) = decode(&w.sent()[1]) else { panic!("not revoke_batch") };
    assert_eq!(rv.keyring_cid, b.cid.unwrap());
    assert_eq!(rv.items.iter().map(|i| i.reason_sha256).collect::<Vec<_>>(), vec![sha256(b"spam"), sha256(b"abuse")]);
}

/// A joiner (key 4) with a profile sealed to the admins, announced to us.
pub(crate) fn joiner(w: &World, n: u8, role: u8) -> ApproveReq {
    joiner_with(w, n, role, true)
}

/// `cached` false: only the announce arrived, so an approval must download the file from Storage.
pub(crate) fn joiner_with(w: &World, n: u8, role: u8, cached: bool) -> ApproveReq {
    let request = joinreq::build(&key(n), &derive_lez_secret(&[n; 32]), w.site_id(), 1, T0 + REQUEST_VALID_MS).to_vec();
    let body = profile::Body { display_name: "Ravi".into(), bio: String::new(), experience: String::new() };
    let file = profile::build(&key(n), &w.site_id(), &acct(n), 1, 1, &body, None, &[pk(1)]).unwrap();
    let bytes = profile::to_bytes(&file);
    let cid = w.store.put(&bytes);
    let a = Announce::sign(&key(n), w.site_id(), 1, &cid, sha256(&bytes));
    let mut f = w.fetched.lock().unwrap();
    assert!(if cached { f.note_blob(a, bytes) } else { f.note_announce(a) });
    ApproveReq { request, role, name: String::new(), profile: None }
}

#[test]
fn an_approval_adds_the_seat_and_the_site_wrap() {
    let w = World::new();
    let req = joiner(&w, 4, 0);
    w.push(Intent::Approve { roster_len: 0, items: vec![req] }, St::Preparing, vec![]);
    w.tick(T0);
    let e = w.q.lock().unwrap().entries[0].clone();
    assert_eq!(e.state, St::Queued);
    let Intent::Approve { roster_len, items } = &e.intent else { panic!() };
    assert_eq!(*roster_len, 1, "the roster length that the keyring was made against");
    // The joiner can read the new keyring, so it holds the site key and can open its own profile.
    let ring = profile::keyring_from_bytes(&e.blobs[0].bytes).unwrap();
    assert_eq!(ring.epoch, 1);
    assert_eq!(profile::entries(&ring).unwrap().len(), 2);
    let keys = profile::open_keyring(&ring, &key(4)).unwrap();
    let p = items[0].profile.clone().unwrap();
    let file = profile::from_bytes(&w.store.get(&p.cid)).unwrap();
    assert_eq!(sha256(&w.store.get(&p.cid)), p.sha);
    assert_eq!(profile::open(&file, &key(4), &keys).unwrap().display_name, "Ravi");
    assert_eq!(p.version, 1);

    w.tick(T0 + 1);
    let Instruction::ApproveBatch(ix) = decode(&w.sent()[0]) else { panic!("not approve_batch") };
    assert_eq!((ix.roster_len, ix.items.len(), ix.items[0].is_new), (1, 1, true));
    assert_eq!(ix.keyring_cid, e.blobs[0].cid.clone().unwrap());
    assert_eq!(ix.keyring_sha256, sha256(&e.blobs[0].bytes));
    assert_eq!(ix.items[0].item.profile_cid, p.cid);
    assert_eq!(ix.auth.claimed, Rank::SuperAdmin);
    assert_eq!(ix.auth.now_ms, CLOCK_NOW);
}

#[test]
fn another_admin_approving_first_makes_the_keyring_again() {
    let w = World::new();
    let req = joiner(&w, 4, 0);
    let id = w.push(Intent::Approve { roster_len: 0, items: vec![req] }, St::Preparing, vec![]);
    w.tick(T0);
    assert_eq!(w.entry(id).state, St::Queued);
    // Another admin added a member meanwhile: roster_len is 2 now.
    let mut s = w.site.clone();
    s.roster_len = 2;
    w.put_site(&s);
    w.tick(T0 + 1);
    let e = w.entry(id);
    assert_eq!((e.state, e.blobs.len()), (St::Preparing, 0));
    assert!(w.sent().is_empty(), "a stale roster is never sent");
}

#[test]
fn an_approval_without_a_profile_waits_then_times_out() {
    let w = World::new();
    // The announce of key 5 never came.
    let other = joiner(&w, 5, 0);
    *w.fetched.lock().unwrap() = Fetched::default();
    let id = w.push(Intent::Approve { roster_len: 0, items: vec![other] }, St::Preparing, vec![]);
    w.tick(T0 + 1);
    let e = w.entry(id);
    assert_eq!(e.state, St::Preparing);
    assert!(e.reason.unwrap().starts_with("no profile from"));
    w.tick(T0 + PREP_WINDOW_MS + 10);
    assert_eq!(w.entry(id).state, St::Rejected);
}

#[test]
fn places_are_edited_on_the_fresh_file() {
    let w = World::new();
    let l = |c: &str| crate::event::Location { code: c.into(), label: "Bin".into(), group: "Bins".into() };
    let id = w.push(Intent::Places { ops: vec![PlaceOp::Add(l("B-01")), PlaceOp::Retire("W-01".into())] }, St::Preparing, vec![]);
    w.tick(T0);
    let e = w.entry(id);
    assert_eq!(e.state, St::Queued);
    let p = Places::from_bytes(&e.blobs[0].bytes).unwrap();
    assert_eq!((p.places.len(), p.retired.len()), (1, 1));
    // A duplicate code is refused when the edit meets the real file.
    let dup = w.push(Intent::Places { ops: vec![PlaceOp::Add(l("W-01"))] }, St::Preparing, vec![]);
    w.q.lock().unwrap().entries[0].state = St::Done;
    w.tick(T0 + 1);
    let e = w.entry(dup);
    assert_eq!(e.state, St::Rejected);
    assert!(e.reason.unwrap().contains("used already"));
}

#[test]
fn what_we_upload_goes_in_the_cache_so_that_we_can_answer_for_it() {
    let w = World::new();
    let l = |c: &str| crate::event::Location { code: c.into(), label: "Bin".into(), group: "Bins".into() };
    let id = w.push(Intent::Places { ops: vec![PlaceOp::Add(l("B-01"))] }, St::Preparing, vec![]);
    w.tick(T0);
    let b = &w.entry(id).blobs[0];
    let sha = sha256(&b.bytes);
    let cid = b.cid.clone().expect("uploaded");
    let held = w.fetched.lock().unwrap().cached(&cid, &sha);
    assert_eq!(held.as_deref(), Some(&b.bytes[..]));
    assert!(w.fetched.lock().unwrap().answer(&cid, &sha).len() == 1, "and it answers a BlobReq");
}

#[test]
fn a_dead_storage_gives_up_after_five_tries() {
    let w = World::new();
    let id = w.push(Intent::Places { ops: vec![PlaceOp::Retire("W-01".into())] }, St::Preparing, vec![]);
    w.store.down.store(true, Ordering::Relaxed);
    for i in 0..4 {
        w.tick(T0 + i);
        assert_eq!(w.entry(id).state, St::Preparing);
    }
    w.tick(T0 + 5);
    let e = w.entry(id);
    assert_eq!(e.state, St::Rejected);
    assert!(e.reason.unwrap().contains("no peers"));
}

#[test]
fn a_profile_is_sealed_uploaded_and_announced() {
    let w = World::new();
    crate::rpc::refresh(&w.chain, &*w.rpc);
    let id = w.push(Intent::Publish { name: "Asha".into(), version: 3, bio: "nurse".into(), experience: "10 years".into() }, St::Preparing, vec![]);
    // We are a member, so we wait up to a minute for the site key.
    w.tick(T0 + 1_000);
    let e = w.entry(id);
    assert_eq!(e.state, St::Preparing);
    assert_eq!(e.reason.as_deref(), Some("waiting for the site key"));
    w.tick(T0 + 61_000);
    assert_eq!(w.entry(id).state, St::Done);
    let out = w.out.lock().unwrap();
    let [Wire::ProfileBlob { announce: a, file }] = &out[..] else { panic!("one announce with its file") };
    assert!(a.verify().is_ok());
    assert_eq!(sha256(file), a.sha256, "the file that goes inline is the one that was uploaded");
    assert_eq!((a.version, a.site, a.author), (3, w.site_id(), pk(1)));
    let bytes = w.store.get(&a.cid);
    assert_eq!(sha256(&bytes), a.sha256);
    let file = profile::from_bytes(&bytes).unwrap();
    let body = profile::open(&file, &key(1), &Default::default()).unwrap();
    assert_eq!((body.display_name.as_str(), body.bio.as_str(), body.experience.as_str()), ("Asha", "nurse", "10 years"));
    assert!(w.fetched.lock().unwrap().announce_of(&pk(1)).is_some(), "we keep our own announce");
    assert!(w.sent().is_empty(), "a profile is no chain write");
}

#[test]
fn a_big_profile_goes_by_cid_only() {
    let w = World::new();
    crate::rpc::refresh(&w.chain, &*w.rpc);
    // 3-byte characters: 500 + 500 of them, hex-encoded and sealed, are far over the inline cap.
    let (bio, experience) = ("ह".repeat(500), "ह".repeat(500));
    let id = w.push(Intent::Publish { name: "Asha".into(), version: 3, bio, experience }, St::Preparing, vec![]);
    w.tick(T0 + 61_000);
    assert_eq!(w.entry(id).state, St::Done);
    let out = w.out.lock().unwrap();
    let [Wire::ProfileAnnounce(a)] = &out[..] else { panic!("the announce alone") };
    assert!(w.store.get(&a.cid).len() > MAX_INLINE_PROFILE);
}

#[test]
fn a_failed_download_keeps_the_entry_preparing_with_its_reason_and_wait() {
    let w = World::new();
    let req = joiner_with(&w, 4, 0, false); // only the announce came: the file must be downloaded
    let id = w.push(Intent::Approve { roster_len: 0, items: vec![req] }, St::Preparing, vec![]);
    w.store.no_download.store(true, Ordering::Relaxed);
    w.tick(T0 + 7_000);
    let e = w.entry(id);
    assert_eq!(e.state, St::Preparing);
    assert_eq!(e.reason.as_deref(), Some("storage: timed out waiting for Storage"));
    let s = w.q.lock().unwrap().status_json(&w.site_id(), T0 + 7_500);
    assert_eq!((s["writes"][0]["state"].as_str(), s["writes"][0]["waited_s"].as_u64()), (Some("preparing"), Some(7)));
    assert_eq!(s["writes"][0]["reason"], "storage: timed out waiting for Storage");
    // The node answers again: the same entry goes on to Queued with no new click, and the wait is null.
    w.store.no_download.store(false, Ordering::Relaxed);
    w.tick(T0 + 12_000);
    assert_eq!(w.entry(id).state, St::Queued);
    assert!(w.q.lock().unwrap().status_json(&w.site_id(), T0 + 13_000)["writes"][0]["waited_s"].is_null());
}

#[test]
fn batches_hold_ten() {
    use pukaar_sites_rules::Role;
    let w = World::new();
    let approve = |n: u8| Intent::Approve { roster_len: 0, items: vec![ApproveReq { request: vec![n; 290], role: 0, name: String::new(), profile: None }] };
    let mut q = w.q.lock().unwrap();
    for n in 0..25 {
        q.push(w.site_id(), approve(n), St::Held, vec![], T0);
    }
    for n in 0..3 {
        q.push(w.site_id(), role_item(10 + n, Role::Steward), St::Held, vec![], T0);
    }
    let other_site = q.push([1; 32], role_item(2, Role::Steward), St::Held, vec![], T0);
    let refresh = |n: u8| ProfReq { user: acct(n), prof: Prof { cid: "c".into(), sha: [0; 32], version: 2 } };
    let ids = q.make_batches(&w.site_id(), (0..12).map(refresh).collect(), T0);
    let kinds: Vec<(&str, usize, St)> = ids.iter().map(|i| q.entries.iter().find(|e| e.id == *i).unwrap()).map(|e| (e.intent.name(), e.intent.len(), e.state)).collect();
    assert_eq!(
        kinds,
        vec![
            ("approve", 10, St::Preparing),
            ("approve", 10, St::Preparing),
            ("approve", 5, St::Preparing),
            ("set_role", 3, St::Queued),
            ("set_profile", 10, St::Queued),
            ("set_profile", 2, St::Queued),
        ]
    );
    assert_eq!(q.held(&w.site_id()).count(), 0, "the held ones were turned into batches");
    assert_eq!(q.entries.iter().find(|e| e.id == other_site).unwrap().state, St::Held, "another site is untouched");
}

#[test]
fn join_requests_are_kept_when_they_hold_up() {
    let w = World::new();
    let make = |n: u8, site: Key, nonce: u64, valid: u64| joinreq::build(&key(n), &derive_lez_secret(&[n; 32]), site, nonce, valid);
    let mut q = w.q.lock().unwrap();
    let site = w.site_id();
    assert!(q.keep_request(&site, &make(4, site, 5, T0 + 100), T0));
    assert!(!q.keep_request(&site, &make(4, site, 5, T0 + 100), T0), "same nonce: nothing new");
    assert!(!q.keep_request(&site, &make(4, site, 4, T0 + 100), T0), "older nonce");
    assert!(q.keep_request(&site, &make(4, site, 6, T0 + 200), T0), "a newer one replaces");
    assert!(!q.keep_request(&site, &make(5, [3; 32], 1, T0 + 100), T0), "another site");
    assert!(!q.keep_request(&site, &make(5, site, 1, T0 - 1), T0), "expired");
    assert!(!q.keep_request(&site, &[0; 290], T0), "junk");
    assert!(q.keep_request(&site, &make(1, site, 1, T0 + 100), T0));
    assert_eq!(q.requests(&pk(1), T0).len(), 1, "ours is not listed");
    let (b, _) = q.request_for(&pk(4), T0).unwrap();
    assert_eq!(b.nonce, 6);
    assert!(q.request_for(&pk(4), T0 + 300).is_none(), "expired");
    // It survives a restart.
    drop(q);
    let again = Queue::load(&w.dir);
    assert!(again.request_for(&pk(4), T0).is_some());
}

#[test]
fn finished_entries_are_pruned_and_the_queue_reloads() {
    let w = World::new();
    let mut q = w.q.lock().unwrap();
    for _ in 0..40 {
        let id = q.push(w.site_id(), Intent::SuperAdd(acct(2)), St::Done, vec![], T0);
        assert!(id > 0);
    }
    let live = q.push(w.site_id(), Intent::SuperAdd(acct(3)), St::Queued, vec![], T0);
    q.prune();
    assert_eq!(q.entries.iter().filter(|e| e.terminal()).count(), KEEP_DONE);
    q.save();
    drop(q);
    let again = Queue::load(&w.dir);
    assert_eq!(again.entries.len(), KEEP_DONE + 1);
    assert_eq!(again.entries.last().unwrap().id, live);
    assert_eq!(again.entries.last().unwrap().intent, Intent::SuperAdd(acct(3)));
}

#[test]
fn super_admin_writes_name_the_target() {
    use pukaar_sites_rules::Role;
    let w = World::new();
    w.put_member(&w.member(2, Role::Admin));
    let id = w.push(Intent::SuperAdd(acct(2)), St::Queued, vec![]);
    w.tick(T0);
    assert_eq!(w.entry(id).state, St::Sent);
    let Instruction::SuperAdminAdd(ix) = decode(&w.sent()[0]) else { panic!() };
    assert_eq!(ix.target, acct(2));
    // The target slot is on the chain: done.
    let mut s = w.site.clone();
    s.super_admins[1] = Some(acct(2));
    w.put_site(&s);
    w.tick(T0 + 1);
    assert_eq!(w.entry(id).state, St::Done);
}

#[test]
fn create_site_waits_for_nothing_and_ends_when_the_site_shows() {
    let w = World::new();
    // A new site (salt 6) that the chain does not hold yet.
    let salt = [6; 32];
    let site_id = pukaar_sites_rules::seed::site_id(&acct(1), &salt);
    let blobs: Vec<Blob> = ["details", "places", "keyring"].iter().map(|n| Blob::new(n.as_bytes().to_vec())).collect();
    let id = w.q.lock().unwrap().push(site_id, Intent::CreateSite { salt, pukaar_key: pk(1) }, St::Preparing, blobs, T0);
    w.tick(T0);
    assert_eq!(w.entry(id).state, St::Queued, "the three files are in Storage");
    assert!(w.entry(id).blobs.iter().all(|b| b.cid.is_some()));
    w.tick(T0 + 1);
    let Instruction::CreateSite(ix) = decode(&w.sent()[0]) else { panic!("not create_site") };
    assert_eq!((ix.args.site_id, ix.args.creator, ix.now_ms), (site_id, acct(1), CLOCK_NOW));
    assert_eq!(ix.args.keyring_sha256, sha256(b"keyring"));
    assert_eq!(w.entry(id).state, St::Sent);
}

#[test]
fn a_saved_profile_job_without_bio_still_loads() {
    // writes.json of an earlier build: no bio, no experience.
    let old = r#"{"Publish":{"name":"Asha","version":3}}"#;
    let i: Intent = serde_json::from_str(old).unwrap();
    assert_eq!(i, Intent::Publish { name: "Asha".into(), version: 3, bio: String::new(), experience: String::new() });
}
