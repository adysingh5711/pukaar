//! Anti-entropy: every node broadcasts its heads every 60 s; whoever holds newer
//! events re-sends them. This is the catch-up path; Delivery's storeQuery is best-effort only.

use crate::event::{sha256, Key};
use crate::store::Store;
use serde::{Deserialize, Serialize};

// ponytail: fixed batch; a ~300-event site catches up in 2 pulls. Page by author if logs grow past ~10k.
pub const MAX_RESEND: usize = 200;
/// How many peers answer each Heads message (besides the admin's kiosk, which always does).
pub const ANSWERERS: usize = 3;

/// What goes on the site channel. `Heads` is unsigned: lying about heads only
/// changes what gets re-sent, and re-sends are capped.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Wire {
    Event(Vec<u8>),
    Heads(Vec<(Key, u64)>),
}

impl Wire {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("postcard encode")
    }
    #[must_use]
    pub fn decode(b: &[u8]) -> Option<Wire> {
        postcard::from_bytes(b).ok()
    }
}

#[must_use]
pub fn heads_msg(store: &Store) -> Wire {
    Wire::Heads(store.head_seqs())
}

/// Should this node answer a Heads message? Every receiver ranks the members the same way,
/// by sha256(member || message || minute), and only the top `ANSWERERS` answer, so a
/// newcomer on check-in day gets 3 copies instead of 30. The minute is in the hash, so if
/// those 3 are offline, a different 3 are picked a minute later.
#[must_use]
pub fn should_answer(me: &Key, members: &[Key], heads_msg: &[u8], minute: u64) -> bool {
    let score = |k: &Key| sha256(&[k.as_slice(), heads_msg, &minute.to_be_bytes()].concat());
    let mine = score(me);
    members
        .iter()
        .filter(|k| *k != me && score(k) < mine)
        .count()
        < ANSWERERS
}

/// Events we hold that a peer with `theirs` heads is missing, oldest first, capped.
#[must_use]
pub fn to_resend(store: &Store, theirs: &[(Key, u64)]) -> Vec<Vec<u8>> {
    resend(store, theirs, None)
}

/// Only our own missing events. Every node always does this, elected or not: we're the
/// one sure source of our own log (a report filed offline exists nowhere else yet).
#[must_use]
pub fn to_resend_own(store: &Store, theirs: &[(Key, u64)], me: &Key) -> Vec<Vec<u8>> {
    resend(store, theirs, Some(me))
}

fn resend(store: &Store, theirs: &[(Key, u64)], only: Option<&Key>) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    for ((author, seq), e) in &store.events {
        if only.is_some_and(|me| me != author) {
            continue;
        }
        let behind = match theirs.iter().find(|(a, _)| a == author) {
            Some((_, their_seq)) => seq > their_seq,
            None => true,
        };
        if behind {
            out.push(e.bytes.clone());
            if out.len() == MAX_RESEND {
                break;
            }
        }
    }
    out
}
