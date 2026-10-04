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

/// Why Delivery didn't take a message.
pub enum SendError {
    /// Delivery will take it once it's back (no answer, or restarted without its node): keep it.
    Retry(String),
    /// Delivery answered no: resending the same bytes won't change that.
    Refused(String),
}

impl SendError {
    /// A failed Delivery call: `answered` is false when Delivery gave no answer at all.
    #[must_use]
    pub fn new(answered: bool, e: String) -> Self {
        if !answered || lost_node(&e) {
            SendError::Retry(e)
        } else {
            SendError::Refused(e)
        }
    }
}

/// Delivery answered but holds no node: its host restarted under us ("Context not initialized"),
/// so the node, its start and our channel must be set up again before anything can go out.
#[must_use]
pub fn lost_node(e: &str) -> bool {
    e.contains("not initialized")
}

/// Hand `queue` to `send` in order. Returns what stays queued (the first message to `Retry` and
/// everything after it, in order: each further call would only stall or fail the same way) and
/// the last error. A refused message is dropped; anti-entropy re-sends a lost event.
#[must_use]
pub fn send_queue(
    mut queue: Vec<Vec<u8>>,
    mut send: impl FnMut(&[u8]) -> Result<(), SendError>,
) -> (Vec<Vec<u8>>, Option<String>) {
    let mut last = None;
    let stop = queue.iter().position(|m| match send(m) {
        Ok(()) => false,
        Err(SendError::Refused(e)) => {
            last = Some(e);
            false
        }
        Err(SendError::Retry(e)) => {
            last = Some(e);
            true
        }
    });
    queue.drain(..stop.unwrap_or(queue.len()));
    (queue, last)
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

/// Events we hold that a peer with `theirs` heads is missing (see `resend`).
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

/// Held events a peer with `theirs` heads may lack, canonical first, capped. Heads name only a
/// seq, so a peer on the losing branch of a fork can look up to date: for an author with fork
/// evidence we send everything we hold of theirs, and the peer re-picks the same branch.
// ponytail: a forked author's whole held chain goes out with every answer (deduped on arrival);
// send from the fork point once Heads carry ids (an appended Wire variant).
fn resend(store: &Store, theirs: &[(Key, u64)], only: Option<&Key>) -> Vec<Vec<u8>> {
    store
        .held()
        .filter(|e| only.is_none_or(|me| *me == e.u.author))
        .filter(|e| {
            store.is_forked(&e.u.author)
                || theirs
                    .iter()
                    .find(|(a, _)| *a == e.u.author)
                    .is_none_or(|(_, their_seq)| e.u.seq > *their_seq)
        })
        .take(MAX_RESEND)
        .map(|e| e.bytes.clone())
        .collect()
}
