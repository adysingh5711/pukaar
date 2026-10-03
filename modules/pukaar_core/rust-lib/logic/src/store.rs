//! Per-author hash chains (the AccountLog prefix rule applied to events), with forks resolved
//! the same way on every replica.
//!
//! Every validly signed event for this site is *held*. Per author, the **canonical chain** is
//! picked by a fixed rule that ignores arrival order: walk seq 0, 1, 2, …; at each seq take, among
//! the held candidates whose `prev` is the canonical event one seq lower (seq 0: any, `prev` is
//! ZERO by construction) and whose lamport is higher than it, the one with the **lowest id**
//! (the site's genesis always wins its slot). The chain ends at the first seq with no such
//! candidate. Only canonical events are applied, named in heads and covered by checkpoints.
//! Every other held event (a double-signed twin, an event built on one, an event breaking the
//! chain rules) is **fork evidence**: kept, persisted, re-sent, counted, never applied. So two
//! devices running one identity can fork it, and every replica still keeps the same branch.
//! Out-of-order arrivals (nothing held yet at seq-1) wait in `pending`.

use crate::event::{Body, Event, Id, Key, ZERO};
use std::collections::BTreeMap;

// ponytail: fixed pending cap; per-author quotas if one author floods gaps.
const MAX_PENDING: usize = 10_000;
/// Candidates held per (author, seq), canonical included. Honest double-device use makes 2.
// ponytail: only an author can sign at their own seq, so a flood here only hurts that author's
// chain; the lowest ids are kept, so every replica keeps the same ones.
pub const MAX_ALTS: usize = 4;

#[derive(Debug, PartialEq, Eq)]
pub enum Accept {
    /// Now canonical (it may have displaced a fork sibling with a higher id).
    New,
    Duplicate,
    Pending,
    /// Held as fork evidence, not applied.
    Fork,
    /// Not held (wrong site, bad genesis, bad seq 0), or, for "lamport must increase", held as
    /// evidence but never canonical: an honest client never signs it.
    Rejected(&'static str),
}

pub struct Store {
    pub site: Id,
    /// The canonical chains: what the reducer applies, heads name and checkpoints cover.
    pub events: BTreeMap<(Key, u64), Event>,
    /// Held candidates that aren't canonical: fork evidence, sorted by `rank`.
    pub alts: BTreeMap<(Key, u64), Vec<Event>>,
    /// Waiting for an event at seq-1, sorted by `rank`.
    pub pending: BTreeMap<(Key, u64), Vec<Event>>,
}

/// Lowest first: the site's genesis, then by id.
fn rank(site: &Id, e: &Event) -> (bool, Id) {
    (e.id != *site, e.id)
}

/// Insert `e` in rank order, keeping only the `MAX_ALTS` best. False if `e` didn't make the cut.
fn hold(site: &Id, v: &mut Vec<Event>, e: Event) -> bool {
    match v.binary_search_by_key(&rank(site, &e), |x| rank(site, x)) {
        Err(i) if i < MAX_ALTS => {
            v.insert(i, e);
            v.truncate(MAX_ALTS);
            true
        }
        _ => false,
    }
}

fn has_id(v: Option<&Vec<Event>>, id: &Id) -> bool {
    v.is_some_and(|v| v.iter().any(|e| e.id == *id))
}

impl Store {
    pub fn new(site: Id) -> Self {
        Store {
            site,
            events: BTreeMap::new(),
            alts: BTreeMap::new(),
            pending: BTreeMap::new(),
        }
    }

    /// Checks that need nothing but the event itself.
    fn check(&self, e: &Event) -> Result<(), &'static str> {
        if matches!(e.u.body, Body::Genesis { .. }) {
            if e.id != self.site || e.u.site != ZERO || e.u.seq != 0 {
                return Err("genesis does not match site");
            }
        } else if e.u.site != self.site {
            return Err("wrong site");
        }
        if e.u.seq == 0 && e.u.prev != ZERO {
            return Err("seq 0 must have zero prev");
        }
        Ok(())
    }

    /// The held event (canonical or not) at `k` with this id.
    fn find(&self, k: &(Key, u64), id: &Id) -> Option<&Event> {
        let canon = self.events.get(k).filter(|e| e.id == *id);
        canon.or_else(|| self.alts.get(k)?.iter().find(|e| e.id == *id))
    }

    fn holds_any(&self, k: &(Key, u64)) -> bool {
        self.events.contains_key(k) || self.alts.contains_key(k)
    }

    pub fn insert(&mut self, e: Event) -> Accept {
        if let Err(why) = self.check(&e) {
            return Accept::Rejected(why);
        }
        let (author, seq, id) = (e.u.author, e.u.seq, e.id);
        let k = (author, seq);
        if self.find(&k, &id).is_some() || has_id(self.pending.get(&k), &id) {
            return Accept::Duplicate;
        }
        if seq > 0 && !self.holds_any(&(author, seq - 1)) {
            if self.pending.len() < MAX_PENDING || self.pending.contains_key(&k) {
                hold(&self.site, self.pending.entry(k).or_default(), e);
            }
            return Accept::Pending;
        }
        let lamport_ok = seq == 0
            || self
                .find(&(author, seq - 1), &e.u.prev)
                .is_none_or(|p| e.u.lamport > p.u.lamport);
        self.hold_at(k, e);
        // seq was empty until now if anything waited on it: pull the waiting successors in
        let mut top = seq;
        while let Some(waiting) = self.pending.remove(&(author, top + 1)) {
            top += 1;
            waiting
                .into_iter()
                .for_each(|w| self.hold_at((author, top), w));
        }
        self.recanon(author, seq, top);
        if self.events.get(&k).is_some_and(|c| c.id == id) {
            Accept::New
        } else if !lamport_ok {
            Accept::Rejected("lamport must increase")
        } else {
            Accept::Fork
        }
    }

    /// Move the canonical event at `k`, if any, down among the candidates. Its id, if moved.
    fn demote(&mut self, k: (Key, u64)) -> Option<Id> {
        let e = self.events.remove(&k)?;
        let id = e.id;
        hold(&self.site, self.alts.entry(k).or_default(), e);
        Some(id)
    }

    /// Hold `e` among the candidates at `k`. The canonical one moves down with them (the cap
    /// counts it); `recanon` puts the winner back.
    fn hold_at(&mut self, k: (Key, u64), e: Event) {
        self.demote(k);
        hold(&self.site, self.alts.entry(k).or_default(), e);
    }

    /// Re-pick `author`'s canonical chain from seq `from` (see the module doc). Candidates
    /// changed at seqs `from..=until`; past that, an unchanged pick means an unchanged rest.
    // ponytail: a displacement walks the rest of the author's chain, O(chain); fine below ~10k
    // events per author.
    fn recanon(&mut self, author: Key, from: u64, until: u64) {
        let mut prev = match from.checked_sub(1) {
            None => None,
            Some(p) => match self.events.get(&(author, p)) {
                Some(e) => Some((e.id, e.u.lamport)),
                None => return, // the chain ends lower down: nothing from `from` can be canonical
            },
        };
        let mut seq = from;
        loop {
            let k = (author, seq);
            let old = self.demote(k);
            let Some(v) = self.alts.get_mut(&k) else {
                break;
            };
            let fits = |e: &Event| prev.is_none_or(|(id, l)| e.u.prev == id && e.u.lamport > l);
            let won = v.iter().position(fits).map(|i| v.remove(i));
            if v.is_empty() {
                self.alts.remove(&k);
            }
            let Some(w) = won else {
                break;
            };
            prev = Some((w.id, w.u.lamport));
            let unchanged = old == Some(w.id);
            self.events.insert(k, w);
            if unchanged && seq >= until {
                return;
            }
            seq += 1;
        }
        // the chain now ends below `seq`: what was canonical above it is evidence
        let tail: Vec<(Key, u64)> = self
            .events
            .range((author, seq)..=(author, u64::MAX))
            .map(|(k, _)| *k)
            .collect();
        for k in tail {
            self.demote(k);
        }
    }

    /// Every held event, canonical first (oldest first per author), then the fork evidence.
    pub fn held(&self) -> impl Iterator<Item = &Event> {
        self.events.values().chain(self.alts.values().flatten())
    }

    /// How many held events are fork evidence.
    #[must_use]
    pub fn fork_count(&self) -> usize {
        self.alts.values().map(Vec::len).sum()
    }

    /// Authors with fork evidence, each once.
    #[must_use]
    pub fn forked_authors(&self) -> Vec<Key> {
        let mut v: Vec<Key> = self.alts.keys().map(|(a, _)| *a).collect();
        v.dedup(); // keys are sorted by author
        v
    }

    #[must_use]
    pub fn is_forked(&self, author: &Key) -> bool {
        self.alts
            .range((*author, 0)..=(*author, u64::MAX))
            .next()
            .is_some()
    }

    /// author -> (latest seq, id of that event)
    pub fn heads(&self) -> BTreeMap<Key, (u64, Id)> {
        let mut h = BTreeMap::new();
        for ((a, s), e) in &self.events {
            h.insert(*a, (*s, e.id)); // BTreeMap iterates seq ascending, last wins
        }
        h
    }

    /// `heads()` without the ids: what a checkpoint or a gossip message names.
    pub fn head_seqs(&self) -> Vec<(Key, u64)> {
        self.heads().into_iter().map(|(a, (s, _))| (a, s)).collect()
    }

    /// One author's latest (seq, id), found directly instead of building every author's head.
    pub fn next_seq(&self, author: &Key) -> (u64, Id) {
        match self
            .events
            .range((*author, 0)..=(*author, u64::MAX))
            .next_back()
        {
            Some((_, e)) => (e.u.seq + 1, e.id),
            None => (0, ZERO),
        }
    }

    pub fn max_lamport(&self) -> u64 {
        self.events.values().map(|e| e.u.lamport).max().unwrap_or(0)
    }

    /// Deterministic total order every client agrees on.
    pub fn ordered(&self) -> Vec<&Event> {
        let mut v: Vec<&Event> = self.events.values().collect();
        v.sort_by_key(|e| (e.u.lamport, e.u.author, e.id));
        v
    }
}
