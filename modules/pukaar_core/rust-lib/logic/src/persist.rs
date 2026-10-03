//! On-disk replica: `key` (32-byte seed, 0600), `site` (32 bytes), `events.bin`
//! (u32-LE length-prefixed event bytes: every held event, canonical and fork evidence, in any
//! order, since loading re-picks the canonical chains the same way), `restoring` while a restore syncs, and `left`
//! (sites this key left). Every write goes to a temp file and is
//! renamed into place, so a crash never leaves a half-written log.

use crate::event::{decode, Id};
use crate::node::{Node, Restore};
use crate::store::Store;
use ed25519_dalek::SigningKey;
use std::fs;
use std::io;
use std::path::Path;

/// Present (holding the restore time, u64 LE) while a restored identity waits for its history.
const RESTORING: &str = "restoring";
/// Ids of sites this key left (32 bytes each): our chain there lives on in other replicas.
const LEFT: &str = "left";

/// The `left` list, and whether it names `site`.
fn left_list(dir: &Path, site: &Id) -> (Vec<u8>, bool) {
    let left = fs::read(dir.join(LEFT)).unwrap_or_default();
    let named = left.chunks(32).any(|s| s == site);
    (left, named)
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

fn write_atomic(path: &Path, data: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, data)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))?;
    }
    fs::rename(tmp, path)
}

// ponytail: rewrites the whole log each save (O(n)); append-only file past ~10k events.
pub fn save(node: &Node, dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    write_atomic(&dir.join("key"), &node.key.to_bytes())?;
    write_atomic(&dir.join("site"), &node.store.site)?;
    let total: usize = node.store.held().map(|e| 4 + e.bytes.len()).sum();
    let mut buf = Vec::with_capacity(total);
    for e in node.store.held() {
        buf.extend((e.bytes.len() as u32).to_le_bytes());
        buf.extend(&e.bytes);
    }
    write_atomic(&dir.join("events.bin"), &buf)?;
    let flag = dir.join(RESTORING);
    match &node.restore {
        Some(r) => write_atomic(&flag, &r.since.to_le_bytes()),
        None => remove_if_present(&flag),
    }
}

/// Leave the site on this device: drop the replica (site, events, restore flag) but keep the
/// key, and remember the site so rejoining it waits for our history (see `join_site`).
pub fn leave(dir: &Path, site: &Id) -> io::Result<()> {
    let (mut left, named) = left_list(dir, site);
    if !named {
        left.extend(site);
        write_atomic(&dir.join(LEFT), &left)?;
    }
    // `site` goes first: without it, load() sees no replica even if a later removal fails
    ["site", "events.bin", RESTORING]
        .iter()
        .try_for_each(|f| remove_if_present(&dir.join(f)))
}

/// Join `site`: announce ourselves, unless this key left it before, in which case our chain
/// is already out there and we wait for it like a restore (re-signing seq 0 would fork it).
pub fn join_site(dir: &Path, key: SigningKey, site: Id, now: u64) -> Node {
    if left_list(dir, &site).1 {
        Node::join_restored(key, site, now)
    } else {
        Node::join_announced(key, site, now)
    }
}

/// The key alone, created on first use, so a member has an identity before joining a site.
pub fn load_or_create_key(dir: &Path) -> io::Result<SigningKey> {
    let p = dir.join("key");
    if let Ok(b) = fs::read(&p) {
        let seed: [u8; 32] = b.try_into().map_err(|_| io::Error::other("bad key file"))?;
        return Ok(SigningKey::from_bytes(&seed));
    }
    fs::create_dir_all(dir)?;
    let k = crate::event::new_key();
    write_atomic(&p, &k.to_bytes())?;
    Ok(k)
}

pub fn load(dir: &Path) -> io::Result<Option<Node>> {
    let Ok(site) = fs::read(dir.join("site")) else {
        return Ok(None);
    };
    let site: [u8; 32] = site
        .try_into()
        .map_err(|_| io::Error::other("bad site file"))?;
    let key = load_or_create_key(dir)?;
    let restore = fs::read(dir.join(RESTORING))
        .ok()
        .map(|b| Restore::new(b.try_into().map(u64::from_le_bytes).unwrap_or(0)));
    let mut node = Node {
        key,
        store: Store::new(site),
        restore,
    };
    let buf = fs::read(dir.join("events.bin")).unwrap_or_default();
    let mut rest = &buf[..];
    while rest.len() >= 4 {
        let n = u32::from_le_bytes(rest[..4].try_into().unwrap()) as usize;
        if rest.len() < 4 + n {
            break; // truncated tail: keep what's whole
        }
        if let Ok(e) = decode(&rest[4..4 + n]) {
            node.store.insert(e);
        }
        rest = &rest[4 + n..];
    }
    Ok(Some(node))
}
