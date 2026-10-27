//! The write queue: what the module wants on the chain, one write at a time.
//!
//! A module call only adds an `Entry` and returns. A worker thread calls `tick`, which moves ONE entry
//! one step (the chain and Storage can take seconds, and the UI gives each call 20 seconds).
//!
//! States of an entry: `Held` (waits for `send_batch`), `Preparing` (uploads the files that the chain
//! will point to), `Queued`, `Sending`, `Sent`, `Landed`, `Done` or `Rejected`. The order is strict: the
//! first live entry is the only one that moves, so each write sees the result of the one before it.
//!
//! Safety rules:
//! - `Sending` is saved BEFORE the call to the writer. After a restart it becomes `Sent` without a hash:
//!   the entry is never sent again. It watches the chain for its effect until the window ends.
//! - Before a send, the chain is read. An effect that is on the chain already makes the entry `Done`.
//! - A write is `Rejected("timeout")` only after `SEND_WINDOW` (10 minutes: testnet blocks take 30 s to
//!   2 minutes). `Included` is not `applied` (a program that panics still lands), so the account is read
//!   back before an entry is `Done`.
//! - A failed send, a rejection or a timeout is never retried by the queue. The user acts again.

use crate::chain::{decode_member, decode_site, member_pda, site_pda, AccountId};
use crate::event::{sha256, Key};
use crate::fetch::{Details, Places, SharedFetched, Store};
use crate::joinreq;
use crate::lezkey::{derive_lez_secret, lez_account_id};
use crate::profile::{self, Announce, MAX_INLINE_PROFILE};
use crate::rpc::{clock_ms, get_account, Rpc, SharedChain};
use crate::sync::{Wire, WriteKind};
use crate::writes::{self, Ctx, PlaceOp, Pointed, SuperOp, Tx};
use ed25519_dalek::SigningKey;
use pukaar_sites_rules::{request_id, ApproveItem, JoinBody, Member, MemberStatus, Pointer, ProfileItem, Rank, Site, JOIN_BODY_LEN, JOIN_REQ_LEN};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// From the send to the end of the wait. The testnet makes a block every 30 s to 2 minutes.
pub const SEND_WINDOW_MS: u64 = 10 * 60 * 1000;
/// From "included" to the read-back that must show the effect.
pub const LAND_WINDOW_MS: u64 = 60 * 1000;
/// An entry that cannot get its files ready (no Storage, no chain) is given up after this.
pub const PREP_WINDOW_MS: u64 = 10 * 60 * 1000;
/// One tick at most this often.
pub const TICK_EVERY: Duration = Duration::from_secs(5);
const KEEP_DONE: usize = 30;
const MAX_REQUESTS: usize = 256;
const UPLOAD_TRIES: u32 = 5;
/// A join request is valid for a day.
pub const REQUEST_VALID_MS: u64 = 24 * 60 * 60 * 1000;

pub enum Poll {
    Pending,
    Included,
    Rejected(String),
}

/// The wallet side. `WalletWriter` (the module) uses the LEZ wallet crate, the tests use a fake.
pub trait ChainWriter: Send + Sync {
    /// Put our derived LEZ key (64 hex chars) in the wallet. An already imported key is success.
    fn import_key(&self, secret_hex: &str) -> Result<(), String>;
    /// Sign and send. Gives the transaction hash. May block: only the worker calls it.
    fn send(&self, tx: &Tx) -> Result<String, String>;
    /// May block for seconds: only the worker calls it.
    fn poll(&self, tx_hash: &str) -> Poll;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum KeyState {
    #[default]
    Unknown,
    Ready,
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prof {
    pub cid: String,
    pub sha: [u8; 32],
    pub version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApproveReq {
    /// The 290-byte join request.
    pub request: Vec<u8>,
    pub role: u8,
    /// The name an admin gives to staff (O5). Not signed into a profile yet: 2.3c.
    pub name: String,
    /// The profile file with the site wrap, set in `Preparing`.
    pub profile: Option<Prof>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevokeReq {
    pub user: AccountId,
    pub key: Key,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfReq {
    pub user: AccountId,
    pub prof: Prof,
}

/// What an entry wants. Everything the builder needs besides the chain clock, the claimed rank and
/// the roster state, which are read at the send.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Blobs: details.json, places.json, keyring.json.
    CreateSite { salt: Key, pukaar_key: Key },
    /// Blob: the new keyring. `roster_len` is the one read when it was made.
    Approve { roster_len: u32, items: Vec<ApproveReq> },
    SetRole { items: Vec<(AccountId, u8)> },
    /// Blob: the new keyring (a new epoch, without the revoked members).
    Revoke { items: Vec<RevokeReq> },
    SetProfile { items: Vec<ProfReq> },
    /// Blob: details.json with the reason texts added.
    Details { reasons: Vec<String> },
    /// Blob: places.json with the edits applied.
    Places { ops: Vec<PlaceOp> },
    SuperAdd(AccountId),
    SuperRemove(AccountId),
    SuperTransfer(AccountId),
    /// Not a chain write: seal our profile, upload it and send the announce. Blob: the profile file.
    Publish {
        name: String,
        version: u64,
        /// Older `writes.json` files have none of these two.
        #[serde(default)]
        bio: String,
        #[serde(default)]
        experience: String,
    },
}

impl Intent {
    /// The kind that a `Wire::ChainPending` names. `None` for a job that is no chain write.
    pub fn kind(&self) -> Option<WriteKind> {
        use Intent::*;
        Some(match self {
            CreateSite { .. } => WriteKind::CreateSite,
            Approve { .. } => WriteKind::Approve,
            SetRole { .. } => WriteKind::SetRole,
            Revoke { .. } => WriteKind::Revoke,
            SetProfile { .. } => WriteKind::SetProfile,
            Details { .. } | Places { .. } => WriteKind::SetSite,
            SuperAdd(_) | SuperRemove(_) | SuperTransfer(_) => WriteKind::SuperAdmin,
            Publish { .. } => return None,
        })
    }

    pub fn name(&self) -> &'static str {
        use Intent::*;
        match self {
            CreateSite { .. } => "create_site",
            Approve { .. } => "approve",
            SetRole { .. } => "set_role",
            Revoke { .. } => "revoke",
            SetProfile { .. } => "set_profile",
            Details { .. } => "set_details",
            Places { .. } => "set_places",
            SuperAdd(_) => "super_admin_add",
            SuperRemove(_) => "super_admin_remove",
            SuperTransfer(_) => "super_admin_transfer",
            Publish { .. } => "profile",
        }
    }

    fn len(&self) -> usize {
        use Intent::*;
        match self {
            Approve { items, .. } => items.len(),
            SetRole { items } => items.len(),
            Revoke { items } => items.len(),
            SetProfile { items } => items.len(),
            _ => 1,
        }
    }

    /// The accounts whose Member record the chain read must include.
    fn users(&self) -> Vec<AccountId> {
        use Intent::*;
        match self {
            Approve { items, .. } => items.iter().filter_map(|i| body_of(&i.request).map(|b| b.user_account)).collect(),
            SetRole { items } => items.iter().map(|(u, _)| *u).collect(),
            Revoke { items } => items.iter().map(|i| i.user).collect(),
            SetProfile { items } => items.iter().map(|i| i.user).collect(),
            SuperAdd(t) | SuperTransfer(t) => vec![*t],
            _ => vec![],
        }
    }

    /// Drop the items that the chain shows already. True when nothing is left to send.
    fn settle(&mut self, r: &Reads, me: &AccountId, blob_cid: Option<&str>) -> bool {
        use Intent::*;
        let m = |u: &AccountId| r.members.get(u);
        let site = r.site.as_ref();
        let active = |x: &Member| x.status == MemberStatus::Active && x.revoked.is_none();
        match self {
            CreateSite { .. } => site.is_some(),
            Approve { items, .. } => {
                items.retain(|i| {
                    let req: Option<[u8; JOIN_REQ_LEN]> = i.request.clone().try_into().ok();
                    let user = body_of(&i.request).map(|b| b.user_account);
                    !(req.zip(user).is_some_and(|(q, u)| m(&u).is_some_and(|x| active(x) && x.request_id == request_id(&q))))
                });
                items.is_empty()
            }
            SetRole { items } => {
                items.retain(|(u, c)| !m(u).is_some_and(|x| active(x) && x.role == writes::role_from_code(*c)));
                items.is_empty()
            }
            Revoke { items } => {
                items.retain(|i| !m(&i.user).is_some_and(|x| x.revoked.is_some()));
                items.is_empty()
            }
            SetProfile { items } => {
                items.retain(|i| !m(&i.user).is_some_and(|x| x.profile_version >= i.prof.version));
                items.is_empty()
            }
            Details { .. } => site.zip(blob_cid).is_some_and(|(s, c)| s.details_cid == c),
            Places { .. } => site.zip(blob_cid).is_some_and(|(s, c)| s.places_cid == c),
            SuperAdd(t) => site.is_some_and(|s| s.is_super(t)),
            SuperRemove(t) => site.is_some_and(|s| !s.is_super(t)),
            SuperTransfer(t) => site.is_some_and(|s| s.is_super(t) && !s.is_super(me)),
            Publish { .. } => false,
        }
    }
}

fn body_of(request: &[u8]) -> Option<JoinBody> {
    let b: &[u8; JOIN_BODY_LEN] = request.get(..JOIN_BODY_LEN)?.try_into().ok()?;
    JoinBody::parse(b)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blob {
    pub bytes: Vec<u8>,
    pub cid: Option<String>,
}

impl Blob {
    pub fn new(bytes: Vec<u8>) -> Blob {
        Blob { bytes, cid: None }
    }

    fn pointed(&self) -> Result<Pointed, String> {
        Ok((self.cid.clone().ok_or("a file is not in Storage yet")?, sha256(&self.bytes)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum St {
    Held,
    Preparing,
    Queued,
    Sending,
    Sent,
    Landed,
    Done,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub id: u64,
    pub site: Key,
    pub intent: Intent,
    pub state: St,
    pub blobs: Vec<Blob>,
    /// The transaction hash (hex). `None` after a restart in `Sending`.
    pub tx: Option<String>,
    /// Why it was rejected, or the last soft problem of a step that is retried.
    pub reason: Option<String>,
    /// When the state last changed (ms): the windows count from here.
    pub since_ms: u64,
    pub tries: u32,
}

impl Entry {
    fn to(&mut self, s: St, now: u64) {
        (self.state, self.since_ms) = (s, now);
        if matches!(s, St::Done | St::Queued | St::Sent) {
            self.reason = None;
        }
        if matches!(s, St::Done | St::Rejected) {
            self.blobs.clear(); // a finished entry keeps no files: the queue file stays small
        }
    }

    fn reject(&mut self, why: impl Into<String>, now: u64) {
        self.to(St::Rejected, now);
        self.reason = Some(why.into());
    }

    fn live(&self) -> bool {
        !matches!(self.state, St::Held | St::Done | St::Rejected)
    }

    fn terminal(&self) -> bool {
        matches!(self.state, St::Done | St::Rejected)
    }

    /// The name that the UI shows.
    pub fn state_name(&self) -> &'static str {
        match self.state {
            St::Held => "waiting",
            St::Preparing => "preparing",
            St::Queued => "queued",
            St::Sending | St::Sent => "sent",
            St::Landed => "confirming",
            St::Done => "included",
            St::Rejected => "rejected",
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Queue {
    next_id: u64,
    pub entries: Vec<Entry>,
    /// Join requests that Delivery brought (admins), 290 bytes each.
    requests: Vec<Vec<u8>>,
    /// The join request that we sent.
    pub mine: Option<Vec<u8>>,
    /// The highest profile version that we gave out.
    pub profile_version: u64,
    #[serde(skip)]
    pub key: KeyState,
    #[serde(skip)]
    busy: bool,
    #[serde(skip)]
    last_tick: Option<Instant>,
    #[serde(skip)]
    dir: Option<PathBuf>,
}

pub type SharedQueue = std::sync::Arc<Mutex<Queue>>;

impl Queue {
    /// Read `writes.json`. A file that cannot be read gives an empty queue. A `Sending` entry is turned
    /// into `Sent` without a hash: it may or may not have gone out, so it is never sent again.
    pub fn load(dir: &std::path::Path) -> Queue {
        let mut q: Queue = std::fs::read(dir.join("writes.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        q.dir = Some(dir.to_path_buf());
        for e in q.entries.iter_mut().filter(|e| e.state == St::Sending) {
            e.state = St::Sent;
            e.tx = None;
        }
        q
    }

    pub fn shared(dir: &std::path::Path) -> SharedQueue {
        std::sync::Arc::new(Mutex::new(Queue::load(dir)))
    }

    pub fn save(&self) {
        let Some(dir) = &self.dir else { return };
        let r = serde_json::to_vec(self).map_err(std::io::Error::other).and_then(|b| {
            let (tmp, to) = (dir.join("writes.tmp"), dir.join("writes.json"));
            std::fs::create_dir_all(dir)?;
            std::fs::write(&tmp, b)?;
            std::fs::rename(tmp, to)
        });
        if let Err(e) = r {
            eprintln!("pukaar: could not save the write queue: {e}");
        }
    }

    pub fn push(&mut self, site: Key, intent: Intent, state: St, blobs: Vec<Blob>, now_ms: u64) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        self.entries.push(Entry { id, site, intent, state, blobs, tx: None, reason: None, since_ms: now_ms, tries: 0 });
        self.save();
        id
    }

    /// True when a tick has something to do: a live entry, or the key that is not imported yet.
    pub fn wants_tick(&self) -> bool {
        self.next_live().is_some() || self.key == KeyState::Unknown
    }

    /// True when the caller must run `tick` now: none runs and the last start is over `TICK_EVERY` old.
    pub fn start(&mut self) -> bool {
        if self.busy || self.last_tick.is_some_and(|t| t.elapsed() < TICK_EVERY) {
            return false;
        }
        (self.busy, self.last_tick) = (true, Some(Instant::now()));
        true
    }

    /// Forget one site (the user left it): its entries, its requests and our own request.
    pub fn drop_site(&mut self, site: &Key) {
        self.entries.retain(|e| e.site != *site);
        self.requests.clear();
        self.mine = None;
    }

    /// The worker died in the middle of a tick: let the next one start.
    pub fn unbusy(&mut self) {
        self.busy = false;
    }

    /// The writes that wait for `send_batch`.
    pub fn held(&self, site: &Key) -> impl Iterator<Item = &Entry> {
        let site = *site;
        self.entries.iter().filter(move |e| e.site == site && e.state == St::Held)
    }

    /// Entries that are not finished (held ones too), for the checks of the module methods.
    pub fn open(&self, site: &Key) -> impl Iterator<Item = &Entry> {
        let site = *site;
        self.entries.iter().filter(move |e| e.site == site && !e.terminal())
    }

    /// Keep a join request from Delivery. It must be for this site, signed twice and not expired. A
    /// newer request of the same Pukaar key replaces the older. True when it was kept.
    pub fn keep_request(&mut self, site: &Key, req: &[u8], now_ms: u64) -> bool {
        let Ok((body, _)) = joinreq::parse(req) else { return false };
        if body.site_id != *site || body.valid_until_ms <= now_ms {
            return false;
        }
        let same = |r: &Vec<u8>| body_of(r).is_some_and(|b| b.pukaar_ed_pk == body.pukaar_ed_pk);
        if let Some(i) = self.requests.iter().position(same) {
            if body_of(&self.requests[i]).is_some_and(|b| b.nonce >= body.nonce) {
                return false;
            }
            self.requests.remove(i);
        }
        if self.requests.len() >= MAX_REQUESTS {
            self.requests.remove(0);
        }
        self.requests.push(req.to_vec());
        self.save();
        true
    }

    /// The newest join request of a Pukaar key that has not expired.
    pub fn request_for(&self, key: &Key, now_ms: u64) -> Option<(JoinBody, Vec<u8>)> {
        self.requests.iter().find_map(|r| body_of(r).filter(|b| b.pukaar_ed_pk == *key && b.valid_until_ms > now_ms).map(|b| (b, r.clone())))
    }

    /// Does an approval of this Pukaar key wait or run?
    pub fn is_queued_approval(&self, site: &Key, key: &Key) -> bool {
        self.open(site).any(|e| matches!(&e.intent, Intent::Approve { items, .. } if items.iter().any(|i| body_of(&i.request).is_some_and(|b| b.pukaar_ed_pk == *key))))
    }

    /// The requests that wait: not expired, and not ours.
    pub fn requests(&self, me: &Key, now_ms: u64) -> Vec<JoinBody> {
        self.requests.iter().filter_map(|r| body_of(r)).filter(|b| b.valid_until_ms > now_ms && b.pukaar_ed_pk != *me).collect()
    }

    /// Turn the held writes and the refreshes into batches of `BATCH` (10). Returns the new ids. An approval
    /// and a role change are `Queued` or `Preparing`; a revoke first sends the reason texts (`Details`).
    pub fn make_batches(&mut self, site: &Key, refreshes: Vec<ProfReq>, now_ms: u64) -> Vec<u64> {
        let (mine, rest): (Vec<Entry>, Vec<Entry>) = std::mem::take(&mut self.entries).into_iter().partition(|e| e.site == *site && e.state == St::Held);
        self.entries = rest;
        let (mut approve, mut roles, mut revokes) = (vec![], vec![], vec![]);
        for e in mine {
            match e.intent {
                Intent::Approve { items, .. } => approve.extend(items),
                Intent::SetRole { items } => roles.extend(items),
                Intent::Revoke { items } => revokes.extend(items),
                _ => {}
            }
        }
        let mut ids = vec![];
        let n = writes::BATCH;
        for c in approve.chunks(n) {
            ids.push(self.push(*site, Intent::Approve { roster_len: 0, items: c.to_vec() }, St::Preparing, vec![], now_ms));
        }
        for c in roles.chunks(n) {
            ids.push(self.push(*site, Intent::SetRole { items: c.to_vec() }, St::Queued, vec![], now_ms));
        }
        for c in revokes.chunks(n) {
            let reasons = c.iter().map(|r| r.reason.clone()).collect();
            ids.push(self.push(*site, Intent::Details { reasons }, St::Preparing, vec![], now_ms));
            ids.push(self.push(*site, Intent::Revoke { items: c.to_vec() }, St::Preparing, vec![], now_ms));
        }
        for c in refreshes.chunks(n) {
            ids.push(self.push(*site, Intent::SetProfile { items: c.to_vec() }, St::Queued, vec![], now_ms));
        }
        ids
    }

    /// `chain_status`: the state of each write of this site, and of the key import.
    pub fn status_json(&self, site: &Key, now_ms: u64) -> serde_json::Value {
        let key = match &self.key {
            KeyState::Unknown => "not imported yet".to_string(),
            KeyState::Ready => "ok".to_string(),
            KeyState::Failed(r) => r.clone(),
        };
        let writes: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.site == *site)
            .map(|e| {
                serde_json::json!({
                    "id": e.id, "kind": e.intent.name(), "state": e.state_name(), "items": e.intent.len(),
                    "tx": e.tx.as_ref().map(|t| t.chars().take(10).collect::<String>()), "reason": e.reason,
                    // Seconds in `Preparing` (what the reason says it waits for), else null.
                    "waited_s": (e.state == St::Preparing).then(|| now_ms.saturating_sub(e.since_ms) / 1000),
                    // The members that the write is about (base58), so the UI can mark their rows.
                    "accounts": e.intent.users().iter().map(crate::chain::b58).collect::<Vec<_>>(),
                })
            })
            .collect();
        serde_json::json!({ "key": key, "writes": writes, "waiting": self.held(site).count() })
    }

    fn next_live(&self) -> Option<&Entry> {
        self.entries.iter().find(|e| e.live())
    }

    fn replace(&mut self, e: Entry) {
        if let Some(slot) = self.entries.iter_mut().find(|x| x.id == e.id) {
            *slot = e;
        }
    }

    /// Keep the newest `KEEP_DONE` finished entries.
    fn prune(&mut self) {
        let finished = self.entries.iter().filter(|e| e.terminal()).count();
        let mut drop = finished.saturating_sub(KEEP_DONE);
        self.entries.retain(|e| {
            let old = e.terminal() && drop > 0;
            drop -= usize::from(old);
            !old
        });
    }
}

/// Everything a tick needs from outside.
pub struct Env<'a> {
    pub writer: &'a dyn ChainWriter,
    pub rpc: &'a dyn Rpc,
    pub store: &'a dyn Store,
    pub chain: &'a SharedChain,
    pub fetched: &'a SharedFetched,
    pub key: &'a SigningKey,
    pub program: AccountId,
    pub now_ms: u64,
    /// Put a wire message on the outbox of the module.
    pub broadcast: &'a dyn Fn(Wire),
}

impl Env<'_> {
    fn me(&self) -> AccountId {
        lez_account_id(&derive_lez_secret(&self.key.to_bytes()))
    }

    fn import(&self) -> KeyState {
        match self.writer.import_key(&hex::encode(derive_lez_secret(&self.key.to_bytes()))) {
            Ok(()) => KeyState::Ready,
            Err(e) => KeyState::Failed(e),
        }
    }
}

/// What the chain holds about one entry now.
struct Reads {
    site: Option<Site>,
    members: HashMap<AccountId, Member>,
}

fn read_site(env: &Env, site: &Key) -> Result<Option<Site>, String> {
    let d = get_account(env.rpc, &env.program, &site_pda(&env.program, site))?;
    if d.is_empty() {
        return Ok(None);
    }
    decode_site(&d).map(Some)
}

fn reads(env: &Env, e: &Entry) -> Result<Reads, String> {
    let site = read_site(env, &e.site)?;
    let mut members = HashMap::new();
    for u in e.intent.users().into_iter().chain([env.me()]) {
        if members.contains_key(&u) {
            continue;
        }
        let d = get_account(env.rpc, &env.program, &member_pda(&env.program, &e.site, &u))?;
        if !d.is_empty() {
            members.insert(u, decode_member(&d)?);
        }
    }
    Ok(Reads { site, members })
}

fn claimed(r: &Reads, me: &AccountId) -> Rank {
    if r.site.as_ref().is_some_and(|s| s.is_super(me)) {
        Rank::SuperAdmin
    } else {
        r.members.get(me).map_or(Rank::Resident, |m| m.role.into())
    }
}

/// Move the first live entry one step, then save. The queue lock is never held during a call to the
/// chain, to Storage or to the wallet.
pub fn tick(q: &Mutex<Queue>, env: &Env) {
    let (entry, mut ks) = {
        let g = q.lock().unwrap();
        (g.next_live().cloned(), g.key.clone())
    };
    let commit = |e: &Entry| {
        let mut g = q.lock().unwrap();
        g.replace(e.clone());
        g.save();
    };
    let entry = match entry {
        // Import the key at start, once (an imported key is not an action on the chain).
        None => {
            if ks == KeyState::Unknown {
                ks = env.import();
            }
            None
        }
        Some(mut e) => {
            step(env, &mut e, &mut ks, &commit);
            Some(e)
        }
    };
    let mut g = q.lock().unwrap();
    g.key = ks;
    g.busy = false;
    if let Some(e) = entry {
        g.replace(e);
        g.prune();
        g.save();
    }
}

fn step(env: &Env, e: &mut Entry, ks: &mut KeyState, commit: &dyn Fn(&Entry)) {
    let now = env.now_ms;
    if e.state == St::Preparing && now.saturating_sub(e.since_ms) > PREP_WINDOW_MS {
        let why = e.reason.clone().unwrap_or_else(|| "not ready".into());
        e.reject(format!("timeout while preparing: {why}"), now);
        return;
    }
    let r = match e.state {
        St::Preparing => prepare(env, e),
        St::Queued => send(env, e, ks, commit),
        St::Sending | St::Sent => watch(env, e),
        St::Landed => confirm(env, e),
        St::Held | St::Done | St::Rejected => Ok(()),
    };
    // A soft problem keeps the state: the next tick tries again (a read, an upload).
    if let Err(why) = r {
        e.reason = Some(why);
    }
}

// ---- Preparing ----

/// A file that the chain or an announce points to. What the fetchers or Delivery brought is read first,
/// so a remote node is only asked when we hold nothing. The wait is short (`try_download`): this runs on
/// the write thread.
fn fetch_blob(env: &Env, cid: &str, sha: &[u8; 32]) -> Result<Vec<u8>, String> {
    let held = env.fetched.lock().unwrap().cached(cid, sha);
    if let Some(b) = held {
        return Ok(b);
    }
    let b = env.store.try_download(cid).map_err(|e| format!("storage: {e}"))?;
    if sha256(&b) != *sha {
        return Err(format!("storage: {cid} does not match the SHA-256 on the chain"));
    }
    Ok(b)
}

/// The site, or a soft problem while the chain does not show it yet.
fn need_site(env: &Env, e: &Entry) -> Result<Site, String> {
    read_site(env, &e.site)?.ok_or_else(|| "the site is not on the chain yet".to_string())
}

/// The keyring of the site and our site keys.
fn open_ring(env: &Env, site: &Site) -> Result<(profile::Keyring, std::collections::BTreeMap<u64, Key>), String> {
    let ring = profile::keyring_from_bytes(&fetch_blob(env, &site.keyring_cid, &site.keyring_sha256)?)?;
    if ring.epoch != site.key_epoch {
        return Err("the keyring is not the one of the current epoch".into());
    }
    let keys = profile::open_keyring(&ring, env.key)?;
    Ok((ring, keys))
}

/// Upload what is not uploaded. True when every blob has its CID. Five failures reject the entry.
fn upload_all(env: &Env, e: &mut Entry) -> Result<bool, String> {
    for b in e.blobs.iter_mut().filter(|b| b.cid.is_none()) {
        match env.store.upload(&b.bytes) {
            Ok(cid) => b.cid = Some(cid),
            Err(err) => {
                e.tries += 1;
                if e.tries >= UPLOAD_TRIES {
                    e.reject(format!("storage: {err}"), env.now_ms);
                    return Ok(false);
                }
                return Err(format!("storage: {err}"));
            }
        }
    }
    Ok(true)
}

fn prepare(env: &Env, e: &mut Entry) -> Result<(), String> {
    if e.blobs.is_empty() {
        match e.intent.clone() {
            Intent::Details { reasons } => {
                let site = need_site(env, e)?;
                let mut d = Details::from_bytes(&fetch_blob(env, &site.details_cid, &site.details_sha256)?)?;
                for r in reasons {
                    d.revoke_reasons.insert(hex::encode(writes::reason_hash(&r)), r);
                }
                e.blobs = vec![Blob::new(d.to_bytes())];
            }
            Intent::Places { ops } => {
                let site = need_site(env, e)?;
                let mut p = Places::from_bytes(&fetch_blob(env, &site.places_cid, &site.places_sha256)?)?;
                for op in &ops {
                    if let Err(why) = writes::apply_place_op(&mut p, op) {
                        e.reject(why, env.now_ms);
                        return Ok(());
                    }
                }
                e.blobs = vec![Blob::new(p.to_bytes())];
            }
            Intent::Approve { items, .. } => return prepare_approve(env, e, items),
            Intent::Revoke { items } => {
                let site = need_site(env, e)?;
                let (ring, mut keys) = open_ring(env, &site)?;
                let gone: Vec<AccountId> = items.iter().map(|i| i.user).collect();
                let staying: Vec<(Key, Key)> = profile::entries(&ring)?.into_iter().filter(|(a, _)| !gone.contains(a)).collect();
                let next = profile::rotate(&e.site, &mut keys, &staying)?;
                e.blobs = vec![Blob::new(profile::keyring_to_bytes(&next))];
            }
            Intent::Publish { name, version, bio, experience } => return prepare_publish(env, e, profile::Body { display_name: name, bio, experience }, version),
            // The other writes carry no file, or their files were made with the entry (create_site).
            _ => {}
        }
    }
    if upload_all(env, e)? {
        e.to(St::Queued, env.now_ms);
    }
    Ok(())
}

fn prepare_approve(env: &Env, e: &mut Entry, mut items: Vec<ApproveReq>) -> Result<(), String> {
    let site = need_site(env, e)?;
    let (mut ring, keys) = open_ring(env, &site)?;
    let newest = *keys.get(&site.key_epoch).ok_or("we hold no site key of the current epoch")?;
    let mut seats = vec![];
    for it in &mut items {
        let body = body_of(&it.request).ok_or("bad join request")?;
        let a = env.fetched.lock().unwrap().announce_of(&body.pukaar_ed_pk).ok_or_else(|| format!("no profile from {} yet", crate::node::fingerprint(&body.pukaar_ed_pk)))?;
        let mut file = profile::from_bytes(&fetch_blob(env, &a.cid, &a.sha256)?)?;
        profile::add_site_wrap(&mut file, env.key, &keys)?;
        let out = profile::to_bytes(&file);
        let cid = env.store.upload(&out).map_err(|e| format!("storage: {e}"))?;
        it.profile = Some(Prof { cid, sha: sha256(&out), version: a.version });
        seats.push((body.user_account, body.pukaar_ed_pk));
    }
    profile::add_seats(&mut ring, &newest, &seats)?;
    e.blobs = vec![Blob::new(profile::keyring_to_bytes(&ring))];
    e.intent = Intent::Approve { roster_len: site.roster_len, items };
    if upload_all(env, e)? {
        e.to(St::Queued, env.now_ms);
    }
    Ok(())
}

/// Seal our profile (the site key when we are in the keyring, else the admins) and upload it. Then the
/// announce goes to the site and the entry is done. The chain pointer follows in the next admin batch.
fn prepare_publish(env: &Env, e: &mut Entry, body: profile::Body, version: u64) -> Result<(), String> {
    if e.blobs.is_empty() {
        let (site, _) = env.chain.lock().unwrap().snapshot();
        let site = site.ok_or("waiting for the chain")?;
        let (member, admins) = {
            let c = env.chain.lock().unwrap();
            (c.role_at(&env.key.verifying_key().to_bytes(), 0).is_some(), c.admin_keys())
        };
        let site_key = env.fetched.lock().unwrap().site_keys().get(&site.key_epoch).copied();
        if member && site_key.is_none() && env.now_ms.saturating_sub(e.since_ms) < 60_000 {
            return Err("waiting for the site key".into());
        }
        let readers = if site_key.is_some() { vec![] } else { admins };
        let body = profile::Body { display_name: body.display_name.trim().into(), ..body };
        let file = profile::build(env.key, &e.site, &env.me(), version, site.key_epoch, &body, site_key.as_ref(), &readers)?;
        e.blobs = vec![Blob::new(profile::to_bytes(&file))];
    }
    if upload_all(env, e)? {
        let b = &e.blobs[0];
        let a = Announce::sign(env.key, e.site, version, b.cid.as_deref().unwrap_or(""), sha256(&b.bytes));
        env.fetched.lock().unwrap().note_announce(a.clone());
        // The file goes with the announce when it fits: an admin then approves without Storage.
        (env.broadcast)(if b.bytes.len() <= MAX_INLINE_PROFILE { Wire::ProfileBlob { announce: a, file: b.bytes.clone() } } else { Wire::ProfileAnnounce(a) });
        e.to(St::Done, env.now_ms);
    }
    Ok(())
}

// ---- Queued: build and send ----

fn build(ctx: &Ctx, e: &Entry, r: &Reads) -> Result<Tx, String> {
    let blob = |i: usize| e.blobs.get(i).ok_or_else(|| "a file is missing".to_string())?.pointed();
    Ok(match &e.intent {
        Intent::CreateSite { salt, pukaar_key } => writes::create_site(ctx, *salt, *pukaar_key, blob(0)?, blob(1)?, blob(2)?),
        Intent::Approve { roster_len, items } => {
            let mut v = vec![];
            for i in items {
                let p = i.profile.clone().ok_or("an approval has no profile file")?;
                let request = i.request.clone().try_into().map_err(|_| "bad join request")?;
                let item = ApproveItem { request, role: writes::role_from_code(i.role), profile_cid: p.cid, profile_sha256: p.sha, profile_version: p.version };
                let is_new = !r.members.contains_key(&writes::requester(&item)?);
                v.push((item, is_new));
            }
            writes::approve_batch(ctx, *roster_len, blob(0)?, v)?
        }
        Intent::SetRole { items } => writes::set_role_batch(ctx, items.iter().map(|(u, c)| (*u, writes::role_from_code(*c))).collect()),
        Intent::Revoke { items } => writes::revoke_batch(ctx, blob(0)?, items.iter().map(|i| (i.user, writes::reason_hash(&i.reason))).collect()),
        Intent::SetProfile { items } => {
            writes::set_profile_batch(ctx, items.iter().map(|i| ProfileItem { user: i.user, cid: i.prof.cid.clone(), sha256: i.prof.sha, version: i.prof.version }).collect())
        }
        Intent::Details { .. } => writes::set_pointer(ctx, Pointer::Details, blob(0)?),
        Intent::Places { .. } => writes::set_pointer(ctx, Pointer::Places, blob(0)?),
        Intent::SuperAdd(t) => writes::super_admin(ctx, SuperOp::Add, *t),
        Intent::SuperRemove(t) => writes::super_admin(ctx, SuperOp::Remove, *t),
        Intent::SuperTransfer(t) => writes::super_admin(ctx, SuperOp::Transfer, *t),
        Intent::Publish { .. } => return Err("a profile is no chain write".into()),
    })
}

fn send(env: &Env, e: &mut Entry, ks: &mut KeyState, commit: &dyn Fn(&Entry)) -> Result<(), String> {
    let now = env.now_ms;
    if *ks != KeyState::Ready {
        *ks = env.import();
    }
    if let KeyState::Failed(why) = ks {
        e.reject(why.clone(), now);
        return Ok(());
    }
    let me = env.me();
    let r = reads(env, e)?;
    let cid = e.blobs.first().and_then(|b| b.cid.clone());
    // The chain shows the effect already (an earlier run, another device): nothing to send.
    let mut left = e.intent.clone();
    if left.settle(&r, &me, cid.as_deref()) {
        e.to(St::Done, now);
        return Ok(());
    }
    e.intent = left;
    if !matches!(e.intent, Intent::CreateSite { .. }) && r.site.is_none() {
        return Err("the site is not on the chain yet".into());
    }
    // Another admin approved meanwhile: the keyring that we made is stale. Make it again.
    if let (Intent::Approve { roster_len, .. }, Some(s)) = (&e.intent, &r.site) {
        if *roster_len != s.roster_len {
            e.blobs.clear();
            e.to(St::Preparing, now);
            return Ok(());
        }
    }
    let ctx = Ctx { program: env.program, site_id: e.site, signer: me, claimed: claimed(&r, &me), now_ms: clock_ms(env.rpc)? };
    let tx = match build(&ctx, e, &r) {
        Ok(t) => t,
        Err(why) => {
            e.reject(why, now);
            return Ok(());
        }
    };
    // Save "Sending" first: a crash after the send must never lead to a second send.
    e.to(St::Sending, now);
    commit(e);
    match env.writer.send(&tx) {
        Ok(hash) => {
            if let (Some(kind), Some(tx)) = (e.intent.kind(), hex::decode(&hash).ok().and_then(|b| Key::try_from(b).ok())) {
                (env.broadcast)(Wire::ChainPending { tx, kind });
            }
            e.tx = Some(hash);
            e.to(St::Sent, now);
        }
        Err(why) => e.reject(why, now),
    }
    Ok(())
}

// ---- Sent and Landed: wait, then read back ----

/// Does the chain show the effect of this entry?
fn effect_on_chain(env: &Env, e: &Entry) -> Result<bool, String> {
    let r = reads(env, e)?;
    let cid = e.blobs.first().and_then(|b| b.cid.clone());
    Ok(e.intent.clone().settle(&r, &env.me(), cid.as_deref()))
}

fn watch(env: &Env, e: &mut Entry) -> Result<(), String> {
    let now = env.now_ms;
    let seen = effect_on_chain(env, e);
    if let Ok(true) = seen {
        e.to(St::Done, now);
        return Ok(());
    }
    if let Some(h) = e.tx.clone() {
        match env.writer.poll(&h) {
            Poll::Included => {
                e.to(St::Landed, now);
                return Ok(());
            }
            Poll::Rejected(why) => {
                e.reject(why, now);
                return Ok(());
            }
            Poll::Pending => {}
        }
    }
    if now.saturating_sub(e.since_ms) > SEND_WINDOW_MS {
        e.reject("timeout", now);
        return Ok(());
    }
    seen.map(|_| ())
}

fn confirm(env: &Env, e: &mut Entry) -> Result<(), String> {
    let now = env.now_ms;
    if effect_on_chain(env, e)? {
        e.to(St::Done, now);
    } else if now.saturating_sub(e.since_ms) > LAND_WINDOW_MS {
        e.reject("the chain took the transaction, but the program did not apply it", now);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests;
