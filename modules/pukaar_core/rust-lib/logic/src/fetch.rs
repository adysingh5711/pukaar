//! Storage fetchers: they fill the cached copy of what the chain only points to.
//!
//! The `Site` record holds the CID and the SHA-256 of `details.json`, `places.json` and the keyring.
//! A `Member` record holds the same for `profile.json`. `fetch_cycle` downloads each file through
//! the `Store` trait, checks the SHA-256 against the chain value, and fills `Fetched`. A node copies
//! `Fetched` into its own fields (`Node::apply_fetched`). Nothing here runs inside a module call:
//! the module runs `sync_once` on a thread, because Storage can take seconds and the UI gives each
//! call 20 seconds.
//!
//! The keyring is also the member list. The program has no list of members, so `fetch_cycle`
//! gives the (account, Pukaar key) pairs of the keyring to `Chain::set_keyring`, and the chain then
//! gives the truth. An event, a join request and a profile announce also name accounts (`hint`).

use crate::event::{sha256, Key, Location};
use crate::profile::{self, Announce};
use crate::reducer::Config;
use crate::rpc::{refresh, Rpc, SharedChain};
use ed25519_dalek::SigningKey;
use pukaar_sites_rules::Member;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

/// Where blobs live. `LogosStore` (the module) calls `storage_module`, tests use a fake. Both calls
/// may block for seconds: only a background thread may call them.
pub trait Store: Send + Sync {
    /// Put the bytes in Storage and give back their CID.
    fn upload(&self, bytes: &[u8]) -> Result<String, String>;
    fn download(&self, cid: &str) -> Result<Vec<u8>, String>;
}

pub const DETAILS_SCHEMA: &str = "pukaar.site.details/1";
pub const PLACES_SCHEMA: &str = "pukaar.site.places/1";
/// A blob over this size is refused, whatever the chain says.
const MAX_BLOB: usize = 1 << 20;
/// Announces of authors the chain does not list (yet).
const MAX_ANNOUNCES: usize = 64;
const MAX_NAME: usize = 64;
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

/// What the fetchers learned. `None` or empty until Storage delivered.
#[derive(Default)]
pub struct Fetched {
    /// Grows when `config`, `places`, `names` or `reasons` change: a node copies them on a new number.
    pub version: u64,
    pub config: Option<Config>,
    pub places: Option<Vec<Location>>,
    /// The display name of each member, by Pukaar key.
    pub names: BTreeMap<Key, String>,
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
    /// CIDs whose last download failed: they go last in the next round.
    failed: BTreeSet<String>,
    busy: bool,
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

fn read_profile(bytes: &[u8], site: &str, m: &Member, version: u64, me: &SigningKey, keys: &BTreeMap<u64, Key>) -> Result<String, String> {
    let e = profile::from_bytes(bytes)?;
    if e.site_id != site || e.user != hex::encode(m.user) || e.pukaar_key != hex::encode(m.pukaar_key) {
        return Err("the file belongs to someone else".into());
    }
    if e.version != version {
        return Err("the version differs from the pointer".into());
    }
    Ok(clean(&profile::open(&e, me, keys)?.display_name))
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
                break;
            }
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
    let mut place_list = f.places.clone();
    match places {
        Some(Ok(p)) => place_list = Some(p.places),
        Some(Err(e)) => errors.push(e),
        None => {}
    }

    let mut names = f.names.clone(); // a name stays when its newer file has not arrived
    for m in &members {
        let (cid, _, version) = pointer(m);
        let Some(bytes) = file(&cid) else { continue };
        match read_profile(bytes, &site_hex, m, version, me, &site_keys) {
            Ok(n) => {
                names.insert(m.pukaar_key, n);
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
            }
        }
    }

    // Keep only what the records still point to.
    let keep: BTreeSet<&str> = want.iter().map(|(c, _)| c.as_str()).collect();
    f.cache.retain(|c, _| keep.contains(c.as_str()));
    f.site_keys = site_keys;
    f.pending = pending;
    f.error = errors.into_iter().next();
    if (f.config.as_ref(), f.places.as_ref(), &f.names, &f.reasons) != (config.as_ref(), place_list.as_ref(), &names, &reasons) {
        (f.config, f.places, f.names, f.reasons) = (config, place_list, names, reasons);
        f.version += 1;
    }
    grew
}
