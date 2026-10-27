//! Chain reads over JSON-RPC, and the cached copy of one site that the rest of the app uses.
//!
//! The HTTP call sits behind the `Rpc` trait, so tests use a fake. The program has no member list:
//! `Site.roster_len` is a count, and the `Roster` page seed of the rules crate is unused. So the
//! app reads the members it knows by account: the creator, the super admins, its own account and
//! the accounts it learned from the keyring (`set_keyring`) and from hints (`hint`: an event, a join
//! request or a profile announce). The chain `Member` record is the truth. Reads only, no transaction.

use crate::chain::{b58, member_pda, shard_of, site_pda, AccountId, Roster};
use crate::event::Key;
use crate::node::fingerprint;
use pukaar_sites_rules::{Member, MemberStatus, Rank};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// At most one refresh in this window.
pub const REFRESH_EVERY: Duration = Duration::from_secs(10);

pub trait Rpc: Send + Sync {
    /// The raw response text of one JSON-RPC call.
    fn call(&self, method: &str, params: Value) -> Result<String, String>;
}

/// Blocking HTTP, five seconds for a whole call.
#[cfg(feature = "http")]
pub struct HttpRpc {
    agent: ureq::Agent,
    url: String,
}

#[cfg(feature = "http")]
impl HttpRpc {
    pub fn new(url: &str) -> Self {
        let cfg = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(5))).build();
        HttpRpc { agent: cfg.into(), url: url.to_string() }
    }
}

#[cfg(feature = "http")]
impl Rpc for HttpRpc {
    fn call(&self, method: &str, params: Value) -> Result<String, String> {
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
        let mut resp = self
            .agent
            .post(&self.url)
            .header("content-type", "application/json")
            .send(body)
            .map_err(|e| format!("{method}: {e}"))?;
        resp.body_mut().read_to_string().map_err(|e| format!("{method}: {e}"))
    }
}

pub fn last_block(rpc: &dyn Rpc) -> Result<u64, String> {
    let text = rpc.call("getLastBlockId", json!([]))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("bad JSON: {e}"))?;
    if let Some(e) = v.get("error") {
        return Err(format!("rpc error: {e}"));
    }
    v.get("result").and_then(Value::as_u64).ok_or_else(|| "no block id in the response".to_string())
}

/// The error of a read when the Site record does not exist (a site not created yet, or a wrong id).
pub const NOT_ON_CHAIN: &str = "the site is not on the chain";

/// The bytes of this program's shard in account `id`. Empty when the account holds no record.
pub fn get_account(rpc: &dyn Rpc, program: &AccountId, id: &AccountId) -> Result<Vec<u8>, String> {
    shard_of(&rpc.call("getAccount", json!([b58(id)]))?, program)
}

/// Read the Site, then the Member of every account in `users` (plus the creator and the super
/// admins). Returns the roster and the block id read before the accounts.
pub fn read_roster(
    rpc: &dyn Rpc,
    program: &AccountId,
    site_id: &[u8; 32],
    users: &BTreeSet<AccountId>,
) -> Result<(Roster, u64), String> {
    // The block comes first: the accounts below are at least this new.
    let block = last_block(rpc)?;
    let mut roster = Roster::new(*program, *site_id);
    let site_at = site_pda(program, site_id);
    let data = get_account(rpc, program, &site_at)?;
    if data.is_empty() {
        return Err(NOT_ON_CHAIN.into());
    }
    roster.apply(&site_at, &data)?;
    let site = roster.site.clone().ok_or("no Site record")?;
    let mut who = users.clone();
    who.insert(site.creator);
    who.extend(site.super_admins.iter().flatten());
    for u in &who {
        let at = member_pda(program, site_id, u);
        roster.apply(&at, &get_account(rpc, program, &at)?)?;
    }
    Ok((roster, block))
}

/// The chain time in ms. The clock account `CLOCK_01` keeps the block id and the timestamp (8 bytes
/// little endian each) in the shard of the clock program. A write claims `claim_ms`, not this time.
pub fn clock_ms(rpc: &dyn Rpc) -> Result<u64, String> {
    let data = get_account(rpc, &crate::writes::clock_program(), &crate::writes::CLOCK_ACCOUNT)?;
    let t = data.get(8..16).ok_or("the clock account holds no time")?;
    Ok(u64::from_le_bytes(t.try_into().expect("8 bytes")))
}

/// How far below the chain clock every write claims its `now_ms`. The program rejects a claim that is
/// ahead of the clock it sees (`ClockBehind`) and one that is more than 900_000 ms behind (`ClockStale`).
/// A tx in block N sees the clock of block N-1, and the RPC can show a block that the sequencer later
/// replaces (it did once: the read was 60_000 ms ahead and a paid write was lost). 120_000 covers that.
pub const CLAIM_MARGIN_MS: u64 = 120_000;

/// The `now_ms` claim of a write, from the clock that was just read.
pub fn claim_of(clock: u64) -> u64 {
    clock.saturating_sub(CLAIM_MARGIN_MS)
}

/// The `now_ms` claim of every write. It reads the clock again each time (never reuse an older read, and
/// never use the wall clock), so a clock that went backwards gives a lower claim.
pub fn claim_ms(rpc: &dyn Rpc) -> Result<u64, String> {
    clock_ms(rpc).map(claim_of)
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// The role name that `Main.qml` knows: a super admin is an "Admin" with `super_admin` set.
pub fn rank_name(r: Rank) -> &'static str {
    match r {
        Rank::Resident => "Resident",
        Rank::Steward => "Steward",
        Rank::Admin | Rank::SuperAdmin => "Admin",
    }
}

/// The last good read of one site, with the error of the last failed one.
pub struct Chain {
    program: AccountId,
    site_id: [u8; 32],
    roster: Roster,
    /// The block id read before the newest good read. 0 until the first one.
    pub block: u64,
    pub ok: bool,
    pub error: Option<String>,
    /// The last read found no Site record: not an outage, the site does not exist (yet).
    pub missing: bool,
    /// When the newest good read ended (ms since 1970). 0 until the first one.
    pub read_ms: u64,
    /// Accounts we trust enough to keep: our own and the ones `track` got.
    users: BTreeSet<AccountId>,
    /// The (account, Pukaar key) pairs of the keyring. The chain record at an account must hold
    /// the key listed here, or the record is ignored.
    expect: BTreeMap<AccountId, Key>,
    /// Accounts that an event, a join request or an announce named. At most `MAX_HINTS`, and one
    /// that holds no record after a read is dropped, so a flood of fake accounts cannot grow it.
    hints: BTreeSet<AccountId>,
    last_try: Option<Instant>,
    busy: bool,
}

/// How many unverified accounts one site may ask us to read.
pub const MAX_HINTS: usize = 256;

pub type SharedChain = Arc<Mutex<Chain>>;

impl Chain {
    /// `me` is our own LEZ account: its Member record gives us our role.
    pub fn shared(program: AccountId, site_id: [u8; 32], me: AccountId) -> SharedChain {
        Arc::new(Mutex::new(Chain {
            program,
            site_id,
            roster: Roster::new(program, site_id),
            block: 0,
            ok: false,
            error: None,
            missing: false,
            read_ms: 0,
            users: BTreeSet::from([me]),
            expect: BTreeMap::new(),
            hints: BTreeSet::new(),
            last_try: None,
            busy: false,
        }))
    }

    /// Read this account from now on.
    pub fn track(&mut self, user: AccountId) {
        self.users.insert(user);
    }

    /// The keyring of the site: read every account in it. `entries` are (account, Pukaar key). Call
    /// it only with a keyring whose SHA-256 matched `Site.keyring_sha256`. A `Member` record whose
    /// key differs from the entry is ignored at the next read. A newer keyring replaces the older.
    /// True when it names an account that the last keyring did not.
    pub fn set_keyring(&mut self, entries: &[(AccountId, Key)]) -> bool {
        let new: BTreeMap<AccountId, Key> = entries.iter().copied().collect();
        let grew = new.keys().any(|a| !self.expect.contains_key(a));
        self.expect = new;
        grew
    }

    /// An event, a join request or an announce names this account (with this Pukaar key when the
    /// message names one). Read it unless we know it already. True when it was added.
    pub fn hint(&mut self, user: AccountId, key: Option<&Key>) -> bool {
        let known = self.users.contains(&user) || self.expect.contains_key(&user) || self.hints.contains(&user);
        let seen = key.is_some_and(|k| self.roster.has_key(k));
        if known || seen || self.hints.len() >= MAX_HINTS {
            return false;
        }
        self.hints.insert(user)
    }

    /// The Site record and the records of every member the chain gave us.
    pub fn snapshot(&self) -> (Option<pukaar_sites_rules::Site>, Vec<Member>) {
        (self.roster.site.clone(), self.roster.members().cloned().collect())
    }

    /// True when the caller must run `refresh` now: the last try is over 10 s old and none runs.
    pub fn start(&mut self) -> bool {
        if self.busy || self.last_try.is_some_and(|t| t.elapsed() < REFRESH_EVERY) {
            return false;
        }
        self.busy = true;
        self.last_try = Some(Instant::now());
        true
    }

    /// The chain role of an author at `block`. ponytail: the roster holds only the newest state
    /// and the app has no block times, so every block gets the newest known role. A member who is
    /// revoked now loses the old events too. The anchor-bound rule of O10 (2.5) fixes that.
    pub fn role_at(&self, who: &Key, _block: u64) -> Option<Rank> {
        self.roster.role_of(who, now_ms())
    }

    /// `role_at` for the author of an event: the `Member` record at `account` must hold `who`.
    pub fn role_of_author(&self, who: &Key, account: Option<&AccountId>, _block: u64) -> Option<Rank> {
        self.roster.role_for(who, account, now_ms())
    }

    /// The Pukaar keys of the active admins and super admins: they can read a first profile.
    pub fn admin_keys(&self) -> Vec<Key> {
        self.roster.admins(now_ms()).into_iter().map(|m| m.pukaar_key).collect()
    }

    /// Active members and super admins by Pukaar key: they elect the Heads answerers.
    pub fn approved_keys(&self) -> BTreeSet<Key> {
        let now = now_ms();
        self.roster.members().filter(|m| self.roster.role_of(&m.user, now).is_some()).map(|m| m.pukaar_key).collect()
    }

    pub fn status_json(&self) -> Value {
        let site = self.roster.site.as_ref();
        json!({
            "block": self.block, "ok": self.ok, "error": self.error, "missing": self.missing, "read_ms": self.read_ms,
            "roster_len": site.map(|s| s.roster_len), "known": self.roster.members().count(),
            "details_cid": site.map(|s| &s.details_cid), "places_cid": site.map(|s| &s.places_cid),
        })
    }

    fn row(&self, m: &Member, names: &BTreeMap<Key, String>) -> Value {
        let sup = self.roster.site.as_ref().is_some_and(|s| s.is_super(&m.user));
        json!({
            "key": hex::encode(m.pukaar_key), "fingerprint": fingerprint(&m.pukaar_key),
            "name": names.get(&m.pukaar_key), "super_admin": sup, "demotions": [],
            "account": b58(&m.user),
        })
    }

    /// The removal of a member, as `Main.qml` reads it. The reason text lives in Storage (2.3b):
    /// the record holds only its hash.
    fn revoked_row(&self, m: &Member, names: &BTreeMap<Key, String>) -> Option<Value> {
        let r = m.revoked.as_ref()?;
        let by = self.roster.member(&r.by).map_or(r.by, |b| b.pukaar_key);
        let mut v = self.row(m, names);
        v["by"] = hex::encode(by).into();
        v["by_name"] = json!(names.get(&by));
        v["ts"] = (r.at_ms / 1000).into();
        v["reason"] = "".into();
        v["reason_sha256"] = hex::encode(r.reason_sha256).into();
        Some(v)
    }

    /// The `members`, `super_admins` and `revoked` lists of `site_info`, in a fixed order.
    pub fn people_json(&self, names: &BTreeMap<Key, String>) -> Value {
        let now = now_ms();
        let mut all: Vec<&Member> = self.roster.members().collect();
        all.sort_by_key(|m| (m.granted_at_ms, m.pukaar_key));
        let (mut members, mut supers) = (vec![], vec![]);
        for m in &all {
            let Some(rank) = self.roster.role_of(&m.user, now) else { continue };
            let mut v = self.row(m, names);
            v["role"] = rank_name(rank).into();
            if rank == Rank::SuperAdmin {
                supers.push(v.clone());
            }
            members.push(v);
        }
        let mut gone: Vec<&&Member> = all.iter().filter(|m| m.status == MemberStatus::Removed || m.revoked.is_some()).collect();
        gone.sort_by_key(|m| std::cmp::Reverse(m.revoked.as_ref().map_or(0, |r| r.at_ms)));
        let revoked: Vec<Value> = gone.iter().filter_map(|m| self.revoked_row(m, names)).collect();
        json!({ "members": members, "super_admins": supers, "revoked": revoked })
    }

    /// Our own removal, if the chain holds one.
    pub fn revocation_of(&self, key: &Key, names: &BTreeMap<Key, String>) -> Option<Value> {
        let m = self.roster.members().find(|m| m.pukaar_key == *key)?;
        self.revoked_row(m, names)
    }
}

/// Read the chain now and store the result. The lock is held only to take the inputs and to store
/// the result, never during the network calls. A failed read keeps the old cache and sets `error`.
pub fn refresh(chain: &Mutex<Chain>, rpc: &dyn Rpc) {
    let (program, site_id, users) = {
        let c = chain.lock().unwrap();
        let all = c.users.iter().chain(c.expect.keys()).chain(&c.hints);
        (c.program, c.site_id, all.copied().collect())
    };
    let r = read_roster(rpc, &program, &site_id, &users);
    let mut c = chain.lock().unwrap();
    c.busy = false;
    match r {
        Ok((mut roster, block)) => {
            roster.retain_matching(&c.expect);
            // A hint with a record becomes a kept account. One without is dropped: the next message
            // that names it adds it again.
            let found: Vec<AccountId> = c.hints.iter().copied().filter(|a| roster.member(a).is_some()).collect();
            c.hints.clear();
            c.users.extend(found);
            (c.roster, c.block, c.ok, c.error, c.missing, c.read_ms) = (roster, block, true, None, false, now_ms());
        }
        Err(e) => (c.ok, c.missing, c.error) = (false, e == NOT_ON_CHAIN, Some(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Answers `getAccount` from saved real responses and `getLastBlockId` from a counter.
    struct Fake {
        accounts: Mutex<HashMap<String, String>>,
        block: Mutex<Option<u64>>,
        calls: Mutex<u32>,
    }

    impl Rpc for Fake {
        fn call(&self, method: &str, params: Value) -> Result<String, String> {
            *self.calls.lock().unwrap() += 1;
            match method {
                "getLastBlockId" => self.block.lock().unwrap().map(|b| format!(r#"{{"jsonrpc":"2.0","id":1,"result":{b}}}"#)).ok_or("offline".into()),
                "getAccount" => {
                    let id = params[0].as_str().unwrap();
                    // An account the chain does not know answers with no shards.
                    Ok(self.accounts.lock().unwrap().get(id).cloned().unwrap_or_else(|| r#"{"jsonrpc":"2.0","id":1,"result":{"nonce":0,"data":{"shards":{}}}}"#.into()))
                }
                _ => Err("no such method".into()),
            }
        }
    }

    const PROGRAM: &str = "Gs1EWbwJKSBGRBiR7xasYtkrfCyUGJ52Vx4ASsDLooVK";
    const SITE_ID: &str = "89169de759a670a419cbe3d179e972c2cd03513f020570b296bd64e5b7904dc0";
    const SITE: &str = include_str!("../tests/fixtures/getaccount_site.json");

    fn program() -> AccountId {
        bs58::decode(PROGRAM).into_vec().unwrap().try_into().unwrap()
    }
    fn site_id() -> [u8; 32] {
        hex::decode(SITE_ID).unwrap().try_into().unwrap()
    }
    /// The LEZ account of test user `n` (seed `[n; 32]`), as in the testnet scenario.
    fn acct(n: u8) -> AccountId {
        crate::lezkey::lez_account_id(&crate::lezkey::derive_lez_secret(&[n; 32]))
    }
    fn key(n: u8) -> Key {
        ed25519_dalek::SigningKey::from_bytes(&[n; 32]).verifying_key().to_bytes()
    }

    fn fake() -> Fake {
        let (p, s) = (program(), site_id());
        let mut m = HashMap::new();
        m.insert(b58(&site_pda(&p, &s)), SITE.to_string());
        for (n, f) in [
            (1u8, include_str!("../tests/fixtures/getaccount_member_t1.json")),
            (2, include_str!("../tests/fixtures/getaccount_member_t2.json")),
            (3, include_str!("../tests/fixtures/getaccount_member_t3_revoked.json")),
        ] {
            m.insert(b58(&member_pda(&p, &s, &acct(n))), f.to_string());
        }
        Fake { accounts: Mutex::new(m), block: Mutex::new(Some(100)), calls: Mutex::new(0) }
    }

    /// A chain that tracks t1, t2 and t3, as the join requests of 2.3b will make it.
    fn chain() -> SharedChain {
        let c = Chain::shared(program(), site_id(), acct(2));
        for n in [1, 3] {
            c.lock().unwrap().track(acct(n));
        }
        c
    }

    #[test]
    fn refresh_fills_the_roster() {
        let (c, f) = (chain(), fake());
        refresh(&c, &f);
        let c = c.lock().unwrap();
        assert!(c.ok && c.error.is_none());
        assert_eq!(c.block, 100);
        assert!(c.read_ms > 0 && !c.missing, "a good read is stamped");
        assert_eq!(c.role_at(&key(2), 100), Some(Rank::Admin));
        assert_eq!(c.role_at(&key(3), 100), None, "t3 was revoked");
        assert_eq!(c.role_at(&[9; 32], 100), None, "a stranger");
        assert_eq!(c.approved_keys().len(), 2);
        let s = c.status_json();
        assert_eq!((s["roster_len"].as_u64(), s["known"].as_u64()), (Some(4), Some(3)));
    }

    #[test]
    fn role_is_the_newest_known_at_any_block() {
        let (c, f) = (chain(), fake());
        refresh(&c, &f);
        let c = c.lock().unwrap();
        for block in [0, 1, 99, 100, 101, u64::MAX] {
            assert_eq!(c.role_at(&key(2), block), Some(Rank::Admin), "block {block}");
            assert_eq!(c.role_at(&key(3), block), None, "block {block}");
        }
    }

    #[test]
    fn a_failed_read_keeps_the_old_cache() {
        let (c, f) = (chain(), fake());
        refresh(&c, &f);
        // The node goes away, then a member's record turns out to be junk.
        *f.block.lock().unwrap() = None;
        refresh(&c, &f);
        {
            let g = c.lock().unwrap();
            assert!(!g.ok && g.error.as_deref().is_some_and(|e| e.contains("offline")));
            assert_eq!((g.block, g.role_at(&key(2), 0)), (100, Some(Rank::Admin)), "old cache stays");
        }
        *f.block.lock().unwrap() = Some(105);
        let t2 = b58(&member_pda(&program(), &site_id(), &acct(2)));
        f.accounts.lock().unwrap().insert(t2, r#"{"result":{"data":{"shards":{"Gs1EWbwJKSBGRBiR7xasYtkrfCyUGJ52Vx4ASsDLooVK":[1,2,3]}}}}"#.into());
        refresh(&c, &f);
        let g = c.lock().unwrap();
        assert!(!g.ok && g.error.as_deref().is_some_and(|e| e.contains("Member")));
        assert_eq!((g.block, g.role_at(&key(2), 0)), (100, Some(Rank::Admin)));
        assert_eq!(g.status_json()["ok"], false);
    }

    #[test]
    fn a_site_that_is_not_on_the_chain_is_an_error() {
        let c = Chain::shared(program(), [7; 32], acct(1));
        refresh(&c, &fake());
        let g = c.lock().unwrap();
        assert!(!g.ok && g.error.as_deref().is_some_and(|e| e.contains("not on the chain")));
        assert_eq!(g.status_json()["missing"], true);
    }

    #[test]
    fn at_most_one_refresh_per_ten_seconds() {
        let c = chain();
        let mut g = c.lock().unwrap();
        assert!(g.start());
        assert!(!g.start(), "one is running");
        g.busy = false;
        assert!(!g.start(), "the last try is under 10 s old");
        g.last_try = Some(Instant::now() - REFRESH_EVERY);
        assert!(g.start());
    }

    #[test]
    fn people_lists_have_the_shape_of_main_qml() {
        let (c, f) = (chain(), fake());
        refresh(&c, &f);
        let names = BTreeMap::from([(key(2), "Beta".to_string())]);
        let p = c.lock().unwrap().people_json(&names);
        let members = p["members"].as_array().unwrap();
        assert_eq!(members.len(), 2);
        let t2 = members.iter().find(|m| m["key"] == hex::encode(key(2))).unwrap();
        assert_eq!((t2["role"].as_str(), t2["name"].as_str(), t2["super_admin"].as_bool()), (Some("Admin"), Some("Beta"), Some(false)));
        assert_eq!(t2["fingerprint"].as_str().unwrap().len(), 6);
        assert!(t2["demotions"].as_array().unwrap().is_empty());
        let t1 = members.iter().find(|m| m["super_admin"] == true).unwrap();
        assert_eq!(t1["role"], "Admin", "a super admin is an Admin with the flag");
        assert_eq!(p["super_admins"].as_array().unwrap().len(), 1);
        let gone = p["revoked"].as_array().unwrap();
        assert_eq!(gone.len(), 1);
        assert_eq!(gone[0]["key"], hex::encode(key(3)));
        for f in ["by", "by_name", "ts", "reason", "fingerprint", "name"] {
            assert!(gone[0].get(f).is_some(), "revoked row has {f}");
        }
        assert!(gone[0]["ts"].as_u64().unwrap() > 0);
        let me = c.lock().unwrap().revocation_of(&key(3), &names);
        assert!(me.is_some());
        assert!(c.lock().unwrap().revocation_of(&key(2), &names).is_none());
    }

    fn idj(n: &crate::node::Node) -> Value {
        serde_json::from_str(&n.identity_json()).unwrap()
    }

    /// A node of t2 (an admin) on the real site: the chain feeds its roles, lists and identity.
    #[test]
    fn a_node_reads_roles_lists_and_identity_from_the_cache() {
        use crate::event::Body;
        let c = chain();
        let mut n = crate::node::Node::join(ed25519_dalek::SigningKey::from_bytes(&[2; 32]), site_id());
        n.attach_chain(c.clone());
        // details.json and places.json come in 2.3b: stand in for them here.
        n.config = crate::reducer::Config { categories: vec!["water".into()], max_open_per_author: 5, ..Default::default() };
        n.locations = vec![crate::event::Location { code: "W-01".into(), label: "Tap".into(), group: "Water".into() }];
        let report = Body::Report { category: "water".into(), location: "W-01".into(), landmark: String::new(), text: "leak".into() };
        // Before the first read nobody has a role, so the event is rejected.
        let e = n.publish(report.clone(), 10).unwrap();
        assert_eq!(n.chain_block, 0);
        assert!(n.state().rejected.contains_key(&e.id));
        assert_eq!(idj(&n)["pending"], "none");
        refresh(&c, &fake());
        assert!(!n.state().rejected.contains_key(&e.id), "accepted once the chain is read");
        let e2 = n.publish(report, 11).unwrap();
        assert_eq!(n.chain_block, 100, "our events name the newest block read");
        assert_eq!(e2.u.chain_block, 100);
        assert_eq!(n.approved_keys().len(), 2);

        let me = idj(&n);
        assert_eq!((me["role"].as_str(), me["super_admin"].as_bool()), (Some("Admin"), Some(false)));
        assert!(me["revoked"].is_null());
        let info: Value = serde_json::from_str(&n.site_info_json()).unwrap();
        assert_eq!(info["members"].as_array().unwrap().len(), 2);
        assert_eq!(info["revoked"].as_array().unwrap().len(), 1);
        assert_eq!(info["super_admins"].as_array().unwrap().len(), 1);
        assert_eq!(info["pending"], json!([]));
        let ch = &info["chain"];
        assert_eq!((ch["block"].as_u64(), ch["ok"].as_bool(), ch["error"].is_null()), (Some(100), Some(true), true));
        assert!(ch["details_cid"].is_string());
        // The chain goes away: the lists stay, the error shows.
        let gone = Fake { block: Mutex::new(None), ..fake() };
        refresh(&c, &gone);
        let info: Value = serde_json::from_str(&n.site_info_json()).unwrap();
        assert_eq!(info["members"].as_array().unwrap().len(), 2);
        assert_eq!((info["chain"]["ok"].as_bool(), info["chain"]["block"].as_u64()), (Some(false), Some(100)));
        assert!(info["chain"]["error"].as_str().unwrap().contains("offline"));

        // t3 sees its own removal.
        let mut n3 = crate::node::Node::join(ed25519_dalek::SigningKey::from_bytes(&[3; 32]), site_id());
        n3.attach_chain(c);
        let me = idj(&n3);
        assert!(me["role"].is_null() && me["revoked"]["by"].is_string() && me["revoked"]["ts"].as_u64() > Some(0));
    }
}
