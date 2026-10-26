mod util;
use ed25519_dalek::SigningKey;
use pukaar_logic::event::{new_key, sign, Body, Event, Id, Key, Unsigned, VERSION, ZERO};
use pukaar_logic::store::{Accept, Store, MAX_ALTS};
use util::*;

#[test]
fn out_of_order_events_wait_then_apply() {
    let mut st = Store::new(SITE);
    let k = new_key();
    let evs = chain(&k, SITE, 3);
    assert_eq!(st.insert(evs[2].clone()), Accept::Pending);
    assert_eq!(st.insert(evs[1].clone()), Accept::Pending);
    assert_eq!(st.insert(evs[0].clone()), Accept::New);
    assert!(st.pending.is_empty());
    assert_eq!(st.heads()[&k.verifying_key().to_bytes()], (2, evs[2].id));
    assert_eq!(st.insert(evs[0].clone()), Accept::Duplicate);
}

#[test]
fn an_event_for_another_site_is_rejected() {
    let mut st = Store::new(SITE);
    let stray = chain(&new_key(), [8; 32], 1).remove(0);
    assert!(
        matches!(st.insert(stray), Accept::Rejected("wrong site")),
        "event for another site"
    );
    assert!(st.events.is_empty() && st.pending.is_empty());
}

#[test]
fn equivocation_is_caught() {
    let mut st = Store::new(SITE);
    let k = new_key();
    let a = chain(&k, SITE, 2);
    let mut u = a[0].u.clone();
    u.body = Body::Comment {
        issue: ZERO,
        text: "someone else".into(),
    };
    let twin = sign(&k, u);
    assert_eq!(st.insert(a[0].clone()), Accept::New);
    // same (author, seq), different bytes: the lower id is canonical, the other is evidence
    let twin_wins = twin.id < a[0].id;
    let expect = if twin_wins { Accept::New } else { Accept::Fork };
    assert_eq!(st.insert(twin.clone()), expect);
    assert_eq!(st.fork_count(), 1);
    // a seq-1 event built on the losing seq 0 is evidence too
    let mut u1 = a[1].u.clone();
    u1.prev = if twin_wins { a[0].id } else { twin.id };
    assert_eq!(st.insert(sign(&k, u1)), Accept::Fork);
    assert_eq!(st.fork_count(), 2);
}

/// Two signed seq-0 events by `k`, lower id first.
fn twins(k: &SigningKey, site: Id) -> (Event, Event) {
    let a = branch(k, site, None, 1, "a").remove(0);
    let b = branch(k, site, None, 1, "b").remove(0);
    if a.id < b.id {
        (a, b)
    } else {
        (b, a)
    }
}

fn store_with(evs: &[&Event]) -> Store {
    let mut st = Store::new(SITE);
    for e in evs {
        st.insert((*e).clone());
    }
    st
}

/// Every held event at (author, seq), canonical included, as sorted ids.
fn held_ids(st: &Store, k: Key, seq: u64) -> Vec<Id> {
    let mut v: Vec<Id> = st
        .held()
        .filter(|e| (e.u.author, e.u.seq) == (k, seq))
        .map(|e| e.id)
        .collect();
    v.sort();
    v
}

#[test]
fn the_lower_id_wins_whichever_arrives_first() {
    let k = new_key();
    let (low, high) = twins(&k, SITE);
    let me = k.verifying_key().to_bytes();
    for order in [[&low, &high], [&high, &low]] {
        let st = store_with(&order);
        assert_eq!(st.events[&(me, 0)].id, low.id);
        assert_eq!(st.fork_count(), 1, "the loser is kept as evidence");
        assert_eq!(st.alts[&(me, 0)][0].id, high.id);
        assert_eq!(st.forked_authors(), vec![me]);
    }
}

#[test]
fn a_displaced_branch_and_its_descendants_become_evidence() {
    let k = new_key();
    let me = k.verifying_key().to_bytes();
    let (low, high) = twins(&k, SITE);
    let high_kids = branch(&k, SITE, Some(&high), 2, "h");
    let mut st = store_with(&[&high, &high_kids[0], &high_kids[1]]);
    assert_eq!(st.heads()[&me], (2, high_kids[1].id));
    assert_eq!(st.insert(low.clone()), Accept::New, "lower id displaces");
    assert_eq!(st.heads()[&me], (0, low.id));
    assert_eq!(st.fork_count(), 3, "the old seq 0 and both its children");
    assert!(st
        .ordered()
        .iter()
        .all(|e| e.u.author != me || e.id == low.id));
    // the winner's own children extend it; the loser's stay out
    let low_kids = branch(&k, SITE, Some(&low), 3, "l");
    for e in &low_kids {
        assert_eq!(st.insert(e.clone()), Accept::New);
    }
    assert_eq!(st.heads()[&me], (3, low_kids[2].id));
    assert_eq!(st.fork_count(), 3);
    // and the same set in any order gives the same canonical chain and evidence
    let mut all = vec![&low, &high];
    all.extend(&high_kids);
    all.extend(&low_kids);
    all.reverse();
    let other = store_with(&all);
    assert_eq!(other.heads(), st.heads());
    assert_eq!(other.fork_count(), 3);
    assert!(other.pending.is_empty());
}

#[test]
fn the_alternatives_cap_keeps_the_lowest_ids_in_any_order() {
    let k = new_key();
    let me = k.verifying_key().to_bytes();
    let mut evs: Vec<Event> = (0..7)
        .map(|i| branch(&k, SITE, None, 1, &format!("alt{i}-")).remove(0))
        .collect();
    let mut lowest: Vec<Id> = evs.iter().map(|e| e.id).collect();
    lowest.sort();
    lowest.truncate(MAX_ALTS);
    for _ in 0..2 {
        let st = store_with(&evs.iter().collect::<Vec<_>>());
        assert_eq!(held_ids(&st, me, 0), lowest);
        assert_eq!(st.events[&(me, 0)].id, lowest[0]);
        assert_eq!(st.fork_count(), MAX_ALTS - 1);
        evs.reverse();
    }
}

#[test]
fn lamport_must_increase_along_a_chain() {
    let mut st = Store::new(SITE);
    let k = new_key();
    let first = chain(&k, SITE, 1).remove(0);
    st.insert(first.clone());
    let u = Unsigned {
        v: VERSION,
        site: SITE,
        author: k.verifying_key().to_bytes(),
        seq: 1,
        prev: first.id,
        lamport: first.u.lamport,
        ts: 0,
        chain_block: 0,
        body: Body::Comment {
            issue: ZERO,
            text: String::new(),
        },
    };
    assert!(matches!(st.insert(sign(&k, u)), Accept::Rejected(_)));
}
