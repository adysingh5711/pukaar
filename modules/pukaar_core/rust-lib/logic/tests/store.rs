mod util;
use pukaar_logic::event::{new_key, sign, Body, Unsigned, VERSION, ZERO};
use pukaar_logic::store::{Accept, Store};
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
    assert_eq!(
        st.insert(twin.clone()),
        Accept::Fork,
        "same (author, seq), different bytes"
    );
    // a seq-1 event built on the twin conflicts with the held seq 0
    let mut u1 = a[1].u.clone();
    u1.prev = twin.id;
    assert_eq!(st.insert(sign(&k, u1)), Accept::Fork);
    assert_eq!(st.forks.len(), 2);
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
