//! Checkpoints: a Merkle root over every author's head, anchored on LEZ by `spel anchor`.
//! RFC 6962-style domain separation; an odd node is promoted, not duplicated
//! (duplicating allows two leaf sets with the same root).

use crate::event::{sha256, Id, Key};
use crate::store::Store;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};

pub const CP_DOMAIN: &[u8] = b"logos:pukaar:cp:1\0";

pub fn leaf(author: &Key, seq: u64, id: &Id) -> [u8; 32] {
    sha256(&[&[0u8][..], author, &seq.to_be_bytes(), id].concat())
}

pub fn merkle_root(mut level: Vec<[u8; 32]>) -> [u8; 32] {
    if level.is_empty() {
        return [0; 32];
    }
    while level.len() > 1 {
        level = level
            .chunks(2)
            .map(|p| {
                if p.len() == 2 {
                    sha256(&[&[1u8][..], &p[0], &p[1]].concat())
                } else {
                    p[0]
                }
            })
            .collect();
    }
    level[0]
}

/// Root over the given (author, seq) heads, looked up in *this* store.
/// None if this store lacks any of those events: the root isn't reproducible here.
pub fn root_for(store: &Store, heads: &[(Key, u64)]) -> Option<[u8; 32]> {
    let mut sorted = heads.to_vec();
    sorted.sort();
    let leaves: Option<Vec<[u8; 32]>> = sorted
        .iter()
        .map(|(a, s)| store.events.get(&(*a, *s)).map(|e| leaf(a, *s, &e.id)))
        .collect();
    leaves.map(merkle_root)
}

/// How many events the heads cover: seqs start at 0, so each author's head counts seq + 1.
#[must_use]
pub fn n_events(heads: &[(Key, u64)]) -> u64 {
    heads.iter().map(|(_, s)| s + 1).sum()
}

pub struct Checkpoint {
    pub heads: Vec<(Key, u64)>,
    pub heads_root: [u8; 32],
    pub n_events: u64,
    pub signer: Key,
    pub sig: [u8; 64],
}

fn cp_message(site: &Id, root: &[u8; 32], n_events: u64) -> Vec<u8> {
    [CP_DOMAIN, site, root, &n_events.to_be_bytes()].concat()
}

pub fn checkpoint_now(store: &Store, key: &SigningKey) -> Checkpoint {
    let heads = store.head_seqs();
    let heads_root = root_for(store, &heads).expect("own heads are always present");
    let n_events = n_events(&heads);
    let sig = key
        .sign(&cp_message(&store.site, &heads_root, n_events))
        .to_bytes();
    Checkpoint {
        heads,
        heads_root,
        n_events,
        signer: key.verifying_key().to_bytes(),
        sig,
    }
}

/// Clients verify the on-chain record off-chain (the program stores, it doesn't check).
pub fn verify_anchor(
    site: &Id,
    root: &[u8; 32],
    n_events: u64,
    signer: &Key,
    sig: &[u8; 64],
) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(signer) else {
        return false;
    };
    vk.verify_strict(
        &cp_message(site, root, n_events),
        &Signature::from_bytes(sig),
    )
    .is_ok()
}
