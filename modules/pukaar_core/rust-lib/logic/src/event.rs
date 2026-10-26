//! Signed event envelope, wire format 2. Wire form, AccountLog-style:
//!   bytes   = sig(64) || payload
//!   payload = DOMAIN || postcard(Unsigned)
//!   id      = sha256(payload)
//! Verification runs over the received bytes, so no canonical re-encoding is needed.

/// Re-exported so the module glue needs no direct ed25519 dependency.
pub use ed25519_dalek::SigningKey;
use ed25519_dalek::{Signature, Signer, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const DOMAIN: &[u8] = b"logos:pukaar:2\0";
pub const VERSION: u8 = 2;
// ponytail: 4 KiB fits a checkpoint listing ~100 authors; move heads to Storage past that.
pub const MAX_EVENT_BYTES: usize = 4096;
pub const MAX_TEXT: usize = 500;
/// Report location for a place not on the list yet; needs a landmark note.
pub const OTHER_LOCATION: &str = "other";
pub const ZERO: [u8; 32] = [0; 32];

pub type Key = [u8; 32];
pub type Id = [u8; 32];

pub fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// A place on the site map. It lives in `places.json` on Storage, not in the log.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub code: String,
    pub label: String,
    /// Picker grouping, e.g. "Water points", "Bins", "Paths", "Rooms".
    pub group: String,
}

/// Format 2 log bodies: issue events and checkpoints only. Members, roles, profiles and places live
/// on the chain. The variant index is part of the wire format, so append only.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Body {
    /// `landmark`: free-text detail ("behind tent 4, by the neem tree"); required when location is `other`.
    Report {
        category: String,
        location: String,
        landmark: String,
        text: String,
    },
    Acknowledge {
        issue: Id,
        eta_h: u32,
        note: String,
    },
    /// Staff progress update: what's happening, what happens next, and by when.
    Update {
        issue: Id,
        note: String,
        next_step: String,
        eta_h: u32,
    },
    ClaimResolved {
        issue: Id,
        note: String,
    },
    Confirm {
        issue: Id,
        note: String,
    },
    Reopen {
        issue: Id,
        reason: String,
    },
    CloseWontfix {
        issue: Id,
        reason: String,
    },
    MarkDuplicate {
        issue: Id,
        of: Id,
    },
    Comment {
        issue: Id,
        text: String,
    },
    Checkpoint {
        heads: Vec<(Key, u64)>,
        lez_tx: String,
    },
}

impl Body {
    /// Every free-text field, for the length check.
    #[must_use]
    pub fn texts(&self) -> Vec<&str> {
        use Body::*;
        match self {
            Report {
                category,
                location,
                landmark,
                text,
            } => vec![category, location, landmark, text],
            Update {
                note, next_step, ..
            } => vec![note, next_step],
            Reopen { reason: t, .. }
            | CloseWontfix { reason: t, .. }
            | Acknowledge { note: t, .. }
            | ClaimResolved { note: t, .. }
            | Confirm { note: t, .. }
            | Comment { text: t, .. }
            | Checkpoint { lez_tx: t, .. } => vec![t],
            MarkDuplicate { .. } => vec![],
        }
    }
}

/// Field order is part of the wire format: `v` then `site` then `author` puts the
/// author key at a fixed offset, so the signature is checked before the body is decoded.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Unsigned {
    pub v: u8,
    pub site: Id, // the site id of the chain (`Site.site_id`)
    pub author: Key,
    pub seq: u64,
    pub prev: Id, // id of author's seq-1 event, ZERO at seq 0
    pub lamport: u64,
    pub ts: u64, // author's wall clock, informational only
    /// A recent chain block id. The reducer judges the author with the chain role at this block.
    pub chain_block: u64,
    pub body: Body,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub u: Unsigned,
    pub id: Id,
    pub bytes: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    TooLarge,
    TooShort,
    BadDomain,
    BadKey,
    BadSig,
    Malformed,
    BadVersion,
    TextTooLong,
}

pub fn new_key() -> SigningKey {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).expect("os rng");
    SigningKey::from_bytes(&seed)
}

pub fn sign(key: &SigningKey, u: Unsigned) -> Event {
    let body = postcard::to_allocvec(&u).expect("postcard encode");
    let payload = [DOMAIN, &body].concat();
    let sig = key.sign(&payload);
    let bytes = [&sig.to_bytes()[..], &payload].concat();
    Event {
        id: sha256(&payload),
        u,
        bytes,
    }
}

pub fn decode(bytes: &[u8]) -> Result<Event, DecodeError> {
    use DecodeError::*;
    if bytes.len() > MAX_EVENT_BYTES {
        return Err(TooLarge);
    }
    // sig + domain + v(1) + site(32) + author(32)
    if bytes.len() < 64 + DOMAIN.len() + 65 {
        return Err(TooShort);
    }
    let (sig, payload) = bytes.split_at(64);
    let rest = payload.strip_prefix(DOMAIN).ok_or(BadDomain)?;
    let author: Key = rest[33..65].try_into().expect("length checked");
    let vk = VerifyingKey::from_bytes(&author).map_err(|_| BadKey)?;
    let sig = Signature::from_slice(sig).map_err(|_| BadSig)?;
    vk.verify_strict(payload, &sig).map_err(|_| BadSig)?;
    let (u, tail): (Unsigned, &[u8]) = postcard::take_from_bytes(rest).map_err(|_| Malformed)?;
    if !tail.is_empty() {
        return Err(Malformed);
    }
    if u.v != VERSION {
        return Err(BadVersion);
    }
    let over = u.body.texts().iter().any(|t| t.len() > MAX_TEXT);
    if over {
        return Err(TextTooLong);
    }
    Ok(Event {
        id: sha256(payload),
        u,
        bytes: bytes.to_vec(),
    })
}
