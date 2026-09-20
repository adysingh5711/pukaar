//! Per-author hash chains (the AccountLog prefix rule applied to events).
//! Accepts (author, seq) only after (author, seq-1), with `prev` matching its id.
//! Out-of-order arrivals wait in `pending`; conflicting histories become fork proofs.

use crate::event::{Body, Event, Id, Key, ZERO};
use std::collections::BTreeMap;

// ponytail: fixed pending cap; per-author quotas if one author floods gaps.
const MAX_PENDING: usize = 10_000;

#[derive(Debug, PartialEq, Eq)]
pub enum Accept {
    New,
    Duplicate,
    Pending,
    Fork,
    Rejected(&'static str),
}

pub struct Store {
    pub site: Id,
    pub events: BTreeMap<(Key, u64), Event>,
    pub pending: BTreeMap<(Key, u64), Event>,
    /// Two signed events that can't both be in one honest history.
    pub forks: Vec<(Event, Event)>,
}

impl Store {
    pub fn new(site: Id) -> Self {
        Store {
            site,
            events: BTreeMap::new(),
            pending: BTreeMap::new(),
            forks: Vec::new(),
        }
    }

    pub fn insert(&mut self, e: Event) -> Accept {
        let (author, mut seq) = (e.u.author, e.u.seq);
        let r = self.insert_one(e);
        if r == Accept::New {
            // pull the author's waiting successors in, one by one
            while let Some(next) = self.pending.remove(&(author, seq + 1)) {
                if self.insert_one(next) != Accept::New {
                    break;
                }
                seq += 1;
            }
        }
        r
    }

    fn insert_one(&mut self, e: Event) -> Accept {
        let is_genesis = matches!(e.u.body, Body::Genesis { .. });
        if is_genesis {
            if e.id != self.site || e.u.site != ZERO || e.u.seq != 0 {
                return Accept::Rejected("genesis does not match site");
            }
        } else if e.u.site != self.site {
            return Accept::Rejected("wrong site");
        }
        let k = (e.u.author, e.u.seq);
        if let Some(have) = self.events.get(&k) {
            if have.id == e.id {
                return Accept::Duplicate;
            }
            self.forks.push((have.clone(), e));
            return Accept::Fork;
        }
        if e.u.seq == 0 {
            if e.u.prev != ZERO {
                return Accept::Rejected("seq 0 must have zero prev");
            }
        } else {
            match self.events.get(&(e.u.author, e.u.seq - 1)) {
                None => {
                    if self.pending.len() < MAX_PENDING {
                        self.pending.insert(k, e);
                    }
                    return Accept::Pending;
                }
                Some(p) if p.id != e.u.prev => {
                    self.forks.push((p.clone(), e));
                    return Accept::Fork;
                }
                Some(p) if e.u.lamport <= p.u.lamport => {
                    return Accept::Rejected("lamport must increase");
                }
                Some(_) => {}
            }
        }
        self.events.insert(k, e);
        Accept::New
    }

    /// author -> (latest seq, id of that event)
    pub fn heads(&self) -> BTreeMap<Key, (u64, Id)> {
        let mut h = BTreeMap::new();
        for ((a, s), e) in &self.events {
            h.insert(*a, (*s, e.id)); // BTreeMap iterates seq ascending, last wins
        }
        h
    }

    pub fn next_seq(&self, author: &Key) -> (u64, Id) {
        match self.heads().get(author) {
            Some((s, id)) => (s + 1, *id),
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
