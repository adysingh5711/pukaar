//! Helpers for tests that run below `Node` (event, store, checkpoint, sync).
#![allow(dead_code)]
use ed25519_dalek::SigningKey;
use pukaar_logic::event::{sign, Body, Event, Id, Location, Unsigned, VERSION, ZERO};
use pukaar_logic::reducer::Config;

/// The site id of every test site. In format 2 it comes from the chain, not from an event.
pub const SITE: Id = [7; 32];

pub fn config() -> Config {
    Config {
        name: "Dhun test".into(),
        categories: vec!["water".into(), "waste".into()],
        sla_ack_h: 12,
        sla_fix_h: 48,
        max_open_per_author: 3,
    }
}

pub fn locations() -> Vec<Location> {
    vec![
        Location {
            code: "W-03".into(),
            label: "Tap by dining hall".into(),
            group: "Water points".into(),
        },
        Location {
            code: "B-07".into(),
            label: "Bin at path junction".into(),
            group: "Bins".into(),
        },
    ]
}

/// A comment event (a body that needs no state) with the given position in the author's chain.
pub fn event(key: &SigningKey, site: Id, seq: u64, prev: Id, lamport: u64, tag: &str) -> Event {
    sign(
        key,
        Unsigned {
            v: VERSION,
            site,
            author: key.verifying_key().to_bytes(),
            seq,
            prev,
            lamport,
            ts: seq,
            chain_block: 0,
            author_account: [0; 32],
            body: Body::Comment {
                issue: ZERO,
                text: tag.into(),
            },
        },
    )
}

/// The first event of `key` on `site`.
pub fn first_event(key: &SigningKey) -> Event {
    event(key, SITE, 0, ZERO, 1, "first")
}

/// `n` chained events by `key` on `site`.
pub fn chain(key: &SigningKey, site: Id, n: u64) -> Vec<Event> {
    branch(key, site, None, n, "v")
}

/// `n` events by `key` continuing `parent` (or starting at seq 0). Two branches with
/// different `tag`s differ in bytes, so they fork wherever they share a seq.
pub fn branch(key: &SigningKey, site: Id, parent: Option<&Event>, n: u64, tag: &str) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let (seq, prev, lamport) = match out.last().or(parent) {
            Some(p) => (p.u.seq + 1, p.id, p.u.lamport + 1),
            None => (0, ZERO, 2),
        };
        out.push(event(key, site, seq, prev, lamport, &format!("{tag}{seq}")));
    }
    out
}
