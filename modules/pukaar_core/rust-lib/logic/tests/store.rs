mod util;
use ed25519_dalek::SigningKey;
use pukaar_logic::event::{new_key, sign, Body, Event, Id, Key, Unsigned, VERSION, ZERO};
use pukaar_logic::store::{Accept, Store, MAX_ALTS};
use util::*;

#[test]
fn out_of_order_events_wait_then_apply() {
    let admin = new_key();
    let g = genesis_event(&admin);
    let mut st = Store::new(g.id);
    assert_eq!(st.insert(g.clone()), Accept::New);
    let k = new_key();
    let evs = chain(&k, g.id, 3);
    assert_eq!(st.insert(evs[2].clone()), Accept::Pending);
    assert_eq!(st.insert(evs[1].clone()), Accept::Pending);
    assert_eq!(st.insert(evs[0].clone()), Accept::New);
    assert!(st.pending.is_empty());
    assert_eq!(st.heads()[&k.verifying_key().to_bytes()], (2, evs[2].id));
    assert_eq!(st.insert(evs[0].clone()), Accept::Duplicate);
}

#[test]
fn wrong_site_and_fake_genesis_are_rejected() {
    let g = genesis_event(&new_key());
    let mut st = Store::new(g.id);
    let other = genesis_event(&new_key());
    assert!(
        matches!(st.insert(other.clone()), Accept::Rejected(_)),
        "a different genesis"
    );
    let stray = chain(&new_key(), other.id, 1).remove(0);
    assert!(
        matches!(st.insert(stray), Accept::Rejected(_)),
        "event for another site"
    );
}

#[test]
fn equivocation_is_caught() {
    let g = genesis_event(&new_key());
    let mut st = Store::new(g.id);
    st.insert(g.clone());
    let k = new_key();
    let a = chain(&k, g.id, 2);
    let mut u = a[0].u.clone();
    u.body = Body::Profile {
        display_name: Some("someone else".into()),
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

fn store_with(g: &Event, evs: &[&Event]) -> Store {
    let mut st = Store::new(g.id);
    st.insert(g.clone());
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
    let g = genesis_event(&new_key());
    let k = new_key();
    let (low, high) = twins(&k, g.id);
    let me = k.verifying_key().to_bytes();
    for order in [[&low, &high], [&high, &low]] {
        let st = store_with(&g, &order);
        assert_eq!(st.events[&(me, 0)].id, low.id);
        assert_eq!(st.fork_count(), 1, "the loser is kept as evidence");
        assert_eq!(st.alts[&(me, 0)][0].id, high.id);
        assert_eq!(st.forked_authors(), vec![me]);
    }
}

#[test]
fn a_displaced_branch_and_its_descendants_become_evidence() {
    let g = genesis_event(&new_key());
    let k = new_key();
    let me = k.verifying_key().to_bytes();
    let (low, high) = twins(&k, g.id);
    let high_kids = branch(&k, g.id, Some(&high), 2, "h");
    let mut st = store_with(&g, &[&high, &high_kids[0], &high_kids[1]]);
    assert_eq!(st.heads()[&me], (2, high_kids[1].id));
    assert_eq!(st.insert(low.clone()), Accept::New, "lower id displaces");
    assert_eq!(st.heads()[&me], (0, low.id));
    assert_eq!(st.fork_count(), 3, "the old seq 0 and both its children");
    assert!(st
        .ordered()
        .iter()
        .all(|e| e.u.author != me || e.id == low.id));
    // the winner's own children extend it; the loser's stay out
    let low_kids = branch(&k, g.id, Some(&low), 3, "l");
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
    let other = store_with(&g, &all);
    assert_eq!(other.heads(), st.heads());
    assert_eq!(other.fork_count(), 3);
    assert!(other.pending.is_empty());
}

#[test]
fn the_alternatives_cap_keeps_the_lowest_ids_in_any_order() {
    let g = genesis_event(&new_key());
    let k = new_key();
    let me = k.verifying_key().to_bytes();
    let mut evs: Vec<Event> = (0..7)
        .map(|i| branch(&k, g.id, None, 1, &format!("alt{i}-")).remove(0))
        .collect();
    let mut lowest: Vec<Id> = evs.iter().map(|e| e.id).collect();
    lowest.sort();
    lowest.truncate(MAX_ALTS);
    for _ in 0..2 {
        let st = store_with(&g, &evs.iter().collect::<Vec<_>>());
        assert_eq!(held_ids(&st, me, 0), lowest);
        assert_eq!(st.events[&(me, 0)].id, lowest[0]);
        assert_eq!(st.fork_count(), MAX_ALTS - 1);
        evs.reverse();
    }
}

#[test]
fn a_seq0_twin_of_the_genesis_never_displaces_it() {
    let admin = new_key();
    let g = genesis_event(&admin);
    let me = admin.verifying_key().to_bytes();
    // many tries, so some twins have a lower id than the genesis
    let twins: Vec<Event> = (0..8)
        .map(|i| branch(&admin, g.id, None, 1, &format!("t{i}-")).remove(0))
        .collect();
    let mut st = Store::new(g.id);
    for e in &twins {
        st.insert(e.clone());
    }
    st.insert(g.clone());
    assert_eq!(st.events[&(me, 0)].id, g.id);
    assert!(
        held_ids(&st, me, 0).contains(&g.id),
        "the cap keeps the genesis"
    );
}

#[test]
fn lamport_must_increase_along_a_chain() {
    let g = genesis_event(&new_key());
    let mut st = Store::new(g.id);
    let k = new_key();
    let first = chain(&k, g.id, 1).remove(0);
    st.insert(first.clone());
    let u = Unsigned {
        v: VERSION,
        site: g.id,
        author: k.verifying_key().to_bytes(),
        seq: 1,
        prev: first.id,
        lamport: first.u.lamport,
        ts: 0,
        body: Body::Profile { display_name: None },
    };
    assert!(matches!(st.insert(sign(&k, u)), Accept::Rejected(_)));
    let _ = ZERO;
}
