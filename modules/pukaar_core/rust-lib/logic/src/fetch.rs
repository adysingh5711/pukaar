//! Storage fetchers: they fill the cached copy of what the chain only points to.
//!
//! The `Site` record holds the CID and the SHA-256 of `details.json`, `places.json` and the keyring.
//! A `Member` record holds the same for `profile.json`. `fetch_cycle` downloads each file through
//! the `Store` trait, checks the SHA-256 against the chain value, and fills `Fetched`. A node copies
//! `Fetched` into its own fields (`Node::apply_fetched`). Nothing here runs inside a module call:
//! the module runs `sync_once` on a thread, because Storage can take seconds and the UI gives each
//! call 20 seconds.
//!
//! When Storage cannot reach the node that holds a file, the file is asked for over Delivery
//! (`Wire::BlobReq`) and comes back in chunks (`Wire::BlobChunk`) from any node that holds it. Nobody
//! is trusted: `note_chunk` checks the whole file against the SHA-256 that the chain holds.
//!
//! The keyring is also the member list. The program has no list of members, so `fetch_cycle`
//! gives the (account, Pukaar key) pairs of the keyring to `Chain::set_keyring`, and the chain then
//! gives the truth. An event, a join request and a profile announce also name accounts (`hint`).

use crate::event::{sha256, Key, Location};
use crate::profile::{self, Announce, MAX_INLINE_PROFILE};
use crate::sync::{Wire, BLOB_CHUNK, MAX_CHUNKS, MAX_FALLBACK};
use crate::reducer::Config;
use crate::rpc::{refresh, Rpc, SharedChain};
use ed25519_dalek::SigningKey;
use pukaar_sites_rules::Member;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Where blobs live. `LogosStore` (the module) calls `storage_module`, tests use a fake. Both calls
/// may block for seconds: only a background thread may call them.
pub trait Store: Send + Sync {
    /// Put the bytes in Storage and give back their CID.
    fn upload(&self, bytes: &[u8]) -> Result<String, String>;
    fn download(&self, cid: &str) -> Result<Vec<u8>, String>;
    /// `download` for the write thread: it must not queue behind a transfer of another thread and must
    /// not wait long. The default is `download`, for a store with no such queue.
    fn try_download(&self, cid: &str) -> Result<Vec<u8>, String> {
        self.download(cid)
    }
}

pub const DETAILS_SCHEMA: &str = "pukaar.site.details/1";
pub const PLACES_SCHEMA: &str = "pukaar.site.places/1";
/// A blob over this size is refused, whatever the chain says.
const MAX_BLOB: usize = 1 << 20;
/// Announces of authors the chain does not list (yet).
const MAX_ANNOUNCES: usize = 64;
const MAX_NAME: usize = 64;
const MAX_ABOUT: usize = 500;
/// Files that one round asks peers for, and how long before the same file is asked for again.
const MAX_ASKED: usize = 8;
const REQ_EVERY: Duration = Duration::from_secs(30);
/// A node answers a file once per `REPLY_EVERY`, and sends at most `MAX_CHUNKS_PER_MIN` chunks a minute.
const REPLY_EVERY: Duration = Duration::from_secs(10);
const MAX_CHUNKS_PER_MIN: usize = 400;
/// Chain read, then fetch, again while the fetch found new accounts.
const ROUNDS: usize = 3;

/// `details.json`: the site settings the old genesis event held.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Details {
    pub schema: String,
    pub name: String,
    pub categories: Vec<String>,
    pub sla_ack_h: u32,
    pub sla_fix_h: u32,
    pub max_open_per_author: u32,
    /// The text of each removal reason, by its SHA-256 in hex. The chain keeps only the hash, so
    /// the admin who revokes puts the text here. A text that does not match its hash is ignored.
    #[serde(default)]
    pub revoke_reasons: BTreeMap<String, String>,
}

impl Details {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("json")
    }

    pub fn from_bytes(b: &[u8]) -> Result<Details, String> {
        let d: Details = serde_json::from_slice(b).map_err(|e| format!("bad details.json: {e}"))?;
        if d.schema != DETAILS_SCHEMA {
            return Err(format!("unsupported details schema {}", d.schema));
        }
        Ok(d)
    }

    pub fn config(&self) -> Config {
        Config {
            name: self.name.clone(),
            categories: self.categories.clone(),
            sla_ack_h: self.sla_ack_h,
            sla_fix_h: self.sla_fix_h,
            max_open_per_author: self.max_open_per_author,
        }
    }

    /// The reasons whose text matches the hash they are filed under.
    pub fn reasons(&self) -> BTreeMap<Key, String> {
        let ok = |(h, t): (&String, &String)| Some((hex::decode(h).ok()?.try_into().ok().filter(|k: &Key| sha256(t.as_bytes()) == *k)?, t.clone()));
        self.revoke_reasons.iter().filter_map(ok).collect()
    }
}

/// `places.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Places {
    pub schema: String,
    pub places: Vec<Location>,
    /// Hidden places: no new report names them, an admin can restore them.
    #[serde(default)]
    pub retired: Vec<Location>,
}

impl Places {
    pub fn new(places: Vec<Location>) -> Places {
        Places { schema: PLACES_SCHEMA.into(), places, retired: Vec::new() }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("json")
    }

    pub fn from_bytes(b: &[u8]) -> Result<Places, String> {
        let p: Places = serde_json::from_slice(b).map_err(|e| format!("bad places.json: {e}"))?;
        if p.schema != PLACES_SCHEMA {
            return Err(format!("unsupported places schema {}", p.schema));
        }
        Ok(p)
    }
}

/// The chunks of one file that arrived so far.
struct Partial {
    total: u16,
    got: BTreeMap<u16, Vec<u8>>,
    bytes: usize,
}

/// What the fetchers learned. `None` or empty until Storage delivered.
pub struct Fetched {
    /// Grows when `config`, `places`, `names` or `reasons` change: a node copies them on a new number.
    pub version: u64,
    pub config: Option<Config>,
    pub places: Option<Vec<Location>>,
    /// The places that an admin hid (the `retired` list of places.json).
    pub retired: Vec<Location>,
    /// The display name of each member, by Pukaar key.
    pub names: BTreeMap<Key, String>,
    /// The bio and the experience of each member, by Pukaar key.
    pub about: BTreeMap<Key, (String, String)>,
    /// The removal reasons, by SHA-256 of the text.
    pub reasons: BTreeMap<Key, String>,
    /// How many files of the chain records have not arrived (or did not match) yet.
    pub pending: usize,
    pub error: Option<String>,
    /// Our site keys, from the keyring. Empty when we are not in it.
    site_keys: BTreeMap<u64, Key>,
    announces: BTreeMap<Key, Announce>,
    /// Blobs by CID. Every use checks the SHA-256 again, so a lying pointer cannot reuse a blob.
    cache: HashMap<String, Vec<u8>>,
    /// The newest files that we uploaded: kept in `cache` before the chain points to them, so that we
    /// can answer a `BlobReq` for them.
    recent: Vec<String>,
    /// CIDs whose last download failed: they go last in the next round.
    failed: BTreeSet<String>,
    busy: bool,
    /// Files that we asked peers for: CID -> (SHA-256 from the chain, when we asked last). Only these
    /// are accepted as chunks, and only with this SHA-256.
    asked: HashMap<String, (Key, Instant)>,
    parts: HashMap<String, Partial>,
    /// Wire messages for the module to put on the site channel (`BlobReq`).
    pub out: Vec<Wire>,
    /// How long before we ask again for the same file. Tests make it short.
    pub req_every: Duration,
    /// When we answered a file last, and the chunks sent in the current minute.
    replied: HashMap<String, Instant>,
    minute: Option<(Instant, usize)>,
}

impl Default for Fetched {
    fn default() -> Self {
        Fetched {
            version: 0,
            config: None,
            places: None,
            retired: vec![],
            names: BTreeMap::new(),
            about: BTreeMap::new(),
            reasons: BTreeMap::new(),
            pending: 0,
            error: None,
            site_keys: BTreeMap::new(),
            announces: BTreeMap::new(),
            cache: HashMap::new(),
            recent: vec![],
            failed: BTreeSet::new(),
            busy: false,
            asked: HashMap::new(),
            parts: HashMap::new(),
            out: vec![],
            req_every: REQ_EVERY,
            replied: HashMap::new(),
            minute: None,
        }
    }
}

pub type SharedFetched = Arc<Mutex<Fetched>>;

impl Fetched {
    pub fn shared() -> SharedFetched {
        Arc::new(Mutex::new(Fetched::default()))
    }

    /// Keep the announce when its signature holds and its version is the highest for that author.
    /// The caller checked the site. True when it was kept.
    // ponytail: 64 slots, no eviction. A flood of signed announces from throwaway keys can fill them,
    // then a new member's announce waits for the chain pointer. Evict the oldest if that bites.
    pub fn note_announce(&mut self, a: Announce) -> bool {
        if a.verify().is_err() {
            return false;
        }
        match self.announces.get(&a.author) {
            Some(old) if old.version >= a.version => false,
            None if self.announces.len() >= MAX_ANNOUNCES => false,
            _ => {
                self.announces.insert(a.author, a);
                true
            }
        }
    }

    /// Keep an announce together with the profile file that came with it (`Wire::ProfileBlob`), so no
    /// download is needed. The file must hash to the announced SHA-256 and the announce must verify and
    /// be kept (the newest version of its author), or already be the one that we hold. True when the
    /// file was new. A repeat is ignored.
    pub fn note_blob(&mut self, a: Announce, file: Vec<u8>) -> bool {
        if file.len() > MAX_INLINE_PROFILE || sha256(&file) != a.sha256 || self.cache.contains_key(&a.cid) {
            return false;
        }
        let known = self.announces.get(&a.author).is_some_and(|k| k.cid == a.cid && k.sha256 == a.sha256);
        if !known && !self.note_announce(a.clone()) {
            return false;
        }
        self.cache.insert(a.cid, file);
        true
    }

    /// A blob that we hold already, when its SHA-256 is `sha`. It never touches the network: the
    /// write thread reads here before it asks Storage.
    pub fn cached(&self, cid: &str, sha: &[u8; 32]) -> Option<Vec<u8>> {
        self.cache.get(cid).filter(|b| b.len() <= MAX_BLOB && sha256(b) == *sha).cloned()
    }

    /// Keep a file that we just uploaded or downloaded, so that we can serve it (`answer`). The last
    /// `MAX_ASKED` stay in the cache until the chain points to them.
    pub fn put(&mut self, cid: &str, bytes: &[u8]) {
        if bytes.len() > MAX_BLOB {
            return;
        }
        self.cache.insert(cid.to_string(), bytes.to_vec());
        self.recent.retain(|c| c != cid);
        self.recent.push(cid.to_string());
        if self.recent.len() > MAX_ASKED {
            let old = self.recent.remove(0);
            self.cache.remove(&old);
        }
    }

    /// Ask the peers for a file, unless we asked lately.
    fn ask(&mut self, cid: &str, sha: &[u8; 32]) {
        if self.asked.get(cid).is_some_and(|(_, at)| at.elapsed() < self.req_every) {
            return;
        }
        self.asked.insert(cid.to_string(), (*sha, Instant::now()));
        self.parts.remove(cid); // a new attempt starts clean
        self.out.push(Wire::BlobReq { cid: cid.to_string(), sha256: *sha });
    }

    /// The chunks of a file that a peer asked for, when we hold it with that SHA-256. Nothing when we
    /// do not hold it, when it is too big, when we answered it less than `REPLY_EVERY` ago, or when the
    /// chunks of this minute are used up: a peer cannot make us send a lot.
    pub fn answer(&mut self, cid: &str, sha: &[u8; 32]) -> Vec<Wire> {
        let Some(b) = self.cached(cid, sha).filter(|b| !b.is_empty() && b.len() <= MAX_FALLBACK) else { return vec![] };
        self.replied.retain(|_, t| t.elapsed() < REPLY_EVERY);
        let n = b.len().div_ceil(BLOB_CHUNK);
        let (since, sent) = self.minute.get_or_insert((Instant::now(), 0));
        if since.elapsed() >= Duration::from_secs(60) {
            (*since, *sent) = (Instant::now(), 0);
        }
        if self.replied.contains_key(cid) || *sent + n > MAX_CHUNKS_PER_MIN {
            return vec![];
        }
        *sent += n;
        self.replied.insert(cid.to_string(), Instant::now());
        b.chunks(BLOB_CHUNK)
            .enumerate()
            .map(|(i, c)| Wire::BlobChunk { cid: cid.to_string(), sha256: *sha, index: i as u16, total: n as u16, bytes: c.to_vec() })
            .collect()
    }

    /// Keep a chunk of a file that we asked for. When the last one arrives and the whole file hashes to
    /// the SHA-256 that we asked with, it goes in the cache and the next round uses it without Storage.
    /// A chunk that we did not ask for, with another SHA-256, of a file that we hold, a repeat, an odd
    /// size or a file over `MAX_FALLBACK` is ignored. True when the file is complete.
    // ponytail: a peer that sends a bad chunk for a file that we asked for spoils that attempt (the file
    // fails the hash and the next request starts again). Count chunks per sender if that bites.
    pub fn note_chunk(&mut self, cid: &str, sha: &[u8; 32], index: u16, total: u16, bytes: &[u8]) -> bool {
        let asked = self.asked.get(cid).is_some_and(|(s, _)| s == sha);
        if !asked || self.cache.contains_key(cid) || index >= total || usize::from(total) > MAX_CHUNKS || bytes.is_empty() || bytes.len() > BLOB_CHUNK {
            return false;
        }
        // The newest `total` wins, so a spoiled start does not block the honest chunks.
        if self.parts.get(cid).is_some_and(|p| p.total != total) {
            self.parts.remove(cid);
        }
        let p = self.parts.entry(cid.to_string()).or_insert_with(|| Partial { total, got: BTreeMap::new(), bytes: 0 });
        if p.got.contains_key(&index) {
            return false;
        }
        if p.bytes + bytes.len() > MAX_FALLBACK {
            self.parts.remove(cid); // over the largest file: start again
            return false;
        }
        p.bytes += bytes.len();
        p.got.insert(index, bytes.to_vec());
        if p.got.len() < usize::from(total) {
            return false;
        }
        let file: Vec<u8> = self.parts.remove(cid).into_iter().flat_map(|p| p.got.into_values().flatten()).collect();
        if sha256(&file) != *sha {
            return false;
        }
        self.cache.insert(cid.to_string(), file);
        self.asked.remove(cid);
        true
    }

    /// The newest announce that we kept for this Pukaar key.
    pub fn announce_of(&self, author: &Key) -> Option<Announce> {
        self.announces.get(author).cloned()
    }

    /// Every kept announce: the admin batch refreshes the pointers of the members that are behind.
    pub fn announces(&self) -> Vec<Announce> {
        self.announces.values().cloned().collect()
    }

    /// Our site keys by epoch (empty when we are not in the keyring).
    pub fn site_keys(&self) -> BTreeMap<u64, Key> {
        self.site_keys.clone()
    }

    /// `site_info.storage`.
    pub fn status_json(&self) -> serde_json::Value {
        serde_json::json!({ "pending": self.pending, "error": self.error })
    }
}

/// Read the chain, then fetch, and repeat while the fetch found new accounts (the keyring gives the
/// accounts, the next chain read gives their records, then the profiles can be read).
pub fn sync_once(rpc: &dyn Rpc, store: &dyn Store, chain: &SharedChain, fx: &SharedFetched, me: &SigningKey) {
    for _ in 0..ROUNDS {
        refresh(chain, rpc);
        if !fetch_cycle(store, chain, fx, me) {
            break;
        }
    }
}

/// One round over the cached chain state. True when it learned accounts that the chain has not read
/// yet. A second round returns false at once: only one runs at a time.
pub fn fetch_cycle(store: &dyn Store, chain: &SharedChain, fx: &SharedFetched, me: &SigningKey) -> bool {
    if std::mem::replace(&mut fx.lock().unwrap().busy, true) {
        return false;
    }
    let grew = cycle(store, chain, fx, me);
    fx.lock().unwrap().busy = false;
    grew
}

fn account(hex_id: &str) -> Option<Key> {
    hex::decode(hex_id).ok()?.try_into().ok()
}

/// A display name from a profile: trimmed, capped, and no control characters.
fn clean(name: &str) -> String {
    name.chars().filter(|c| !c.is_control()).take(MAX_NAME).collect::<String>().trim().to_string()
}

/// A bio or an experience text: capped, no control characters except a new line.
fn clean_about(text: &str) -> String {
    text.chars().filter(|c| *c == '\n' || !c.is_control()).take(MAX_ABOUT).collect::<String>().trim().to_string()
}

fn about_of(b: &profile::Body) -> (String, String) {
    (clean_about(&b.bio), clean_about(&b.experience))
}

fn read_profile(bytes: &[u8], site: &str, m: &Member, version: u64, me: &SigningKey, keys: &BTreeMap<u64, Key>) -> Result<profile::Body, String> {
    let e = profile::from_bytes(bytes)?;
    if e.site_id != site || e.user != hex::encode(m.user) || e.pukaar_key != hex::encode(m.pukaar_key) {
        return Err("the file belongs to someone else".into());
    }
    if e.version != version {
        return Err("the version differs from the pointer".into());
    }
    profile::open(&e, me, keys)
}

fn cycle(store: &dyn Store, chain: &SharedChain, fx: &SharedFetched, me: &SigningKey) -> bool {
    let (Some(site), members) = chain.lock().unwrap().snapshot() else { return false };
    let site_hex = hex::encode(site.site_id);
    let announces = fx.lock().unwrap().announces.clone();
    // The newest profile of a member: the announce when its version is higher than the chain pointer.
    let pointer = |m: &Member| match announces.get(&m.pukaar_key) {
        Some(a) if a.version > m.profile_version => (a.cid.clone(), a.sha256, a.version),
        _ => (m.profile_cid.clone(), m.profile_sha256, m.profile_version),
    };
    let strangers: Vec<&Announce> = announces.values().filter(|a| !members.iter().any(|m| m.pukaar_key == a.author)).collect();
    let mut want = vec![
        (site.keyring_cid.clone(), site.keyring_sha256),
        (site.details_cid.clone(), site.details_sha256),
        (site.places_cid.clone(), site.places_sha256),
    ];
    want.extend(members.iter().map(|m| pointer(m)).map(|(c, s, _)| (c, s)));
    want.extend(strangers.iter().map(|a| (a.cid.clone(), a.sha256)));
    want.retain(|(cid, _)| !cid.is_empty());
    {
        let f = fx.lock().unwrap();
        want.sort_by_key(|(cid, _)| f.failed.contains(cid)); // stable: the failed ones go last
    }

    let mut errors: Vec<String> = Vec::new();
    // Download what is missing. A failed download ends the round, so a dead Storage costs one wait.
    // The files that are still missing are then asked for over Delivery (once per `req_every` each).
    let mut storage_failed = false;
    for (cid, _) in &want {
        if fx.lock().unwrap().cache.contains_key(cid) {
            continue;
        }
        match store.download(cid) {
            Ok(b) => {
                let mut f = fx.lock().unwrap();
                f.failed.remove(cid);
                f.cache.insert(cid.clone(), b);
            }
            Err(e) => {
                fx.lock().unwrap().failed.insert(cid.clone());
                errors.push(format!("storage: {e}"));
                storage_failed = true;
                break;
            }
        }
    }
    if storage_failed {
        let mut f = fx.lock().unwrap();
        let due: Vec<_> = want.iter().filter(|(c, _)| !f.cache.contains_key(c) && f.asked.get(c).is_none_or(|(_, at)| at.elapsed() >= f.req_every)).take(MAX_ASKED).cloned().collect();
        for (cid, sha) in due {
            f.ask(&cid, &sha);
        }
    }

    // From here on only cached bytes: no network. `get` checks the chain SHA-256 every time.
    let mut f = fx.lock().unwrap();
    let mut pending = 0;
    let mut get = |f: &mut Fetched, cid: &str, sha: &[u8; 32]| -> Option<Vec<u8>> {
        let b = f.cache.get(cid)?;
        if b.len() <= MAX_BLOB && sha256(b) == *sha {
            return Some(b.clone());
        }
        errors.push(format!("storage: {cid} does not match the SHA-256 on the chain"));
        f.cache.remove(cid);
        None
    };
    let mut grew = false;
    let files: Vec<Option<Vec<u8>>> = want.iter().map(|(cid, sha)| get(&mut f, cid, sha)).collect();
    pending += files.iter().filter(|b| b.is_none()).count();
    let by_cid: HashMap<&str, &Vec<u8>> = want.iter().zip(&files).filter_map(|((c, _), b)| Some((c.as_str(), b.as_ref()?))).collect();
    let file = |cid: &str| by_cid.get(cid).map(|b| b.as_slice());

    // The keyring: our site keys, and the accounts to read.
    let mut site_keys = std::mem::take(&mut f.site_keys);
    if let Some(bytes) = file(&site.keyring_cid) {
        let ring = profile::keyring_from_bytes(bytes).and_then(|r| {
            if r.site_id != site_hex || r.epoch != site.key_epoch {
                return Err("the keyring is for another site or epoch".to_string());
            }
            Ok(r)
        });
        match ring {
            Ok(r) => {
                match profile::entries(&r) {
                    Ok(entries) => grew |= chain.lock().unwrap().set_keyring(&entries),
                    Err(e) => errors.push(format!("keyring: {e}")),
                }
                site_keys = profile::open_keyring(&r, me).unwrap_or_else(|e| {
                    errors.push(format!("keyring: {e}"));
                    BTreeMap::new()
                });
            }
            Err(e) => errors.push(format!("keyring: {e}")),
        }
    }

    let details = file(&site.details_cid).map(Details::from_bytes);
    let places = file(&site.places_cid).map(Places::from_bytes);
    let (mut config, mut reasons) = (f.config.clone(), f.reasons.clone());
    match details {
        Some(Ok(d)) => (config, reasons) = (Some(d.config()), d.reasons()),
        Some(Err(e)) => errors.push(e),
        None => {}
    }
    let (mut place_list, mut retired) = (f.places.clone(), f.retired.clone());
    match places {
        Some(Ok(p)) => (place_list, retired) = (Some(p.places), p.retired),
        Some(Err(e)) => errors.push(e),
        None => {}
    }

    let mut names = f.names.clone(); // a name stays when its newer file has not arrived
    let mut about = f.about.clone();
    for m in &members {
        let (cid, _, version) = pointer(m);
        let Some(bytes) = file(&cid) else { continue };
        match read_profile(bytes, &site_hex, m, version, me, &site_keys) {
            Ok(b) => {
                names.insert(m.pukaar_key, clean(&b.display_name));
                about.insert(m.pukaar_key, about_of(&b));
            }
            Err(e) => errors.push(format!("profile {}: {e}", crate::node::fingerprint(&m.pukaar_key))),
        }
    }
    // An announce of an author the chain does not list: the file names the account to read.
    for a in &strangers {
        let Some(e) = file(&a.cid).and_then(|b| profile::from_bytes(b).ok()) else { continue };
        let genuine = e.site_id == site_hex && e.pukaar_key == hex::encode(a.author) && e.version == a.version && profile::verify(&e).is_ok();
        if let Some(user) = account(&e.user).filter(|_| genuine) {
            grew |= chain.lock().unwrap().hint(user, Some(&a.author));
        }
        // A joiner that waits for approval: show the name if our key can open the profile.
        if genuine {
            if let Ok(body) = profile::open(&e, me, &site_keys) {
                names.insert(a.author, clean(&body.display_name));
                about.insert(a.author, about_of(&body));
            }
        }
    }

    // Keep only what the records still point to.
    let keep: BTreeSet<&str> = want.iter().map(|(c, _)| c.as_str()).collect();
    f.recent.retain(|c| !keep.contains(c.as_str())); // the chain points to it now: `keep` rules
    let recent = f.recent.clone();
    f.cache.retain(|c, _| keep.contains(c.as_str()) || recent.contains(c));
    f.asked.retain(|c, _| keep.contains(c.as_str()));
    f.parts.retain(|c, _| keep.contains(c.as_str()));
    f.site_keys = site_keys;
    f.pending = pending;
    f.error = errors.into_iter().next();
    if (f.config.as_ref(), f.places.as_ref(), &f.retired, &f.names, &f.about, &f.reasons)
        != (config.as_ref(), place_list.as_ref(), &retired, &names, &about, &reasons)
    {
        (f.config, f.places, f.retired, f.names, f.about, f.reasons) = (config, place_list, retired, names, about, reasons);
        f.version += 1;
    }
    grew
}
