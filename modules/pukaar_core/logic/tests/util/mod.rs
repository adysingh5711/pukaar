//! Helpers for tests that run below `Node` (event, store, checkpoint, sync).
#![allow(dead_code)]
use ed25519_dalek::SigningKey;
use pukaar_logic::event::{sign, Body, Event, Id, Location, Unsigned, VERSION, ZERO};

pub fn genesis_body() -> Body {
    Body::Genesis {
        name: "Dhun test".into(),
        admin_name: "Site admin".into(),
        categories: vec!["water".into(), "waste".into()],
        locations: vec![
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
        ],
        sla_ack_h: 12,
        sla_fix_h: 48,
        max_open_per_author: 3,
    }
}

pub fn genesis_event(admin: &SigningKey) -> Event {
    let u = Unsigned {
        v: VERSION,
        site: ZERO,
        author: admin.verifying_key().to_bytes(),
        seq: 0,
        prev: ZERO,
        lamport: 1,
        ts: 0,
        body: genesis_body(),
    };
    sign(admin, u)
}

/// `n` chained profile events by `key` on `site`.
pub fn chain(key: &SigningKey, site: Id, n: u64) -> Vec<Event> {
    let mut out: Vec<Event> = Vec::new();
    for seq in 0..n {
        let prev = out.last().map(|e| e.id).unwrap_or(ZERO);
        let u = Unsigned {
            v: VERSION,
            site,
            author: key.verifying_key().to_bytes(),
            seq,
            prev,
            lamport: seq + 2,
            ts: seq,
            body: Body::Profile {
                display_name: Some(format!("v{seq}")),
            },
        };
        out.push(sign(key, u));
    }
    out
}
