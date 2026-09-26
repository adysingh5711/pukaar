mod util;
use pukaar_logic::event::new_key;
use pukaar_logic::store::Store;
use pukaar_logic::sync::{
    heads_msg, should_answer, to_resend, to_resend_own, Wire, ANSWERERS, MAX_RESEND,
};
use util::*;

#[test]
fn a_behind_store_catches_up_from_heads() {
    let g = genesis_event(&new_key());
    let (mut ahead, mut behind) = (Store::new(g.id), Store::new(g.id));
    ahead.insert(g.clone());
    behind.insert(g.clone());
    for e in chain(&new_key(), g.id, 5) {
        ahead.insert(e);
    }
    // behind broadcasts heads; ahead answers with what's missing, all over the wire format
    let Some(Wire::Heads(theirs)) = Wire::decode(&heads_msg(&behind).encode()) else {
        panic!()
    };
    for b in to_resend(&ahead, &theirs) {
        let Some(Wire::Event(bytes)) = Wire::decode(&Wire::Event(b).encode()) else {
            panic!()
        };
        behind.insert(pukaar_logic::event::decode(&bytes).unwrap());
    }
    assert_eq!(behind.heads(), ahead.heads());
    assert!(to_resend(
        &ahead,
        &match heads_msg(&behind) {
            Wire::Heads(h) => h,
            _ => unreachable!(),
        }
    )
    .is_empty());
}

#[test]
fn resend_is_capped() {
    let g = genesis_event(&new_key());
    let mut ahead = Store::new(g.id);
    ahead.insert(g);
    for e in chain(&new_key(), ahead.site, MAX_RESEND as u64 + 30) {
        ahead.insert(e);
    }
    assert_eq!(to_resend(&ahead, &[]).len(), MAX_RESEND);
    assert!(
        Wire::decode(&[0xff, 0xff, 0xff]).is_none(),
        "junk bytes are ignored"
    );
}

#[test]
fn only_a_few_peers_answer_each_heads_message() {
    let members: Vec<[u8; 32]> = (0..30)
        .map(|_| new_key().verifying_key().to_bytes())
        .collect();
    for minute in 0..20u64 {
        let msg = format!("heads-{minute}");
        let answering = members
            .iter()
            .filter(|m| should_answer(m, &members, msg.as_bytes(), minute))
            .count();
        assert_eq!(answering, ANSWERERS, "exactly {ANSWERERS} of 30 answer");
    }
    // a different minute picks a different set (so offline answerers don't stall a newcomer)
    let pick = |minute: u64| {
        members
            .iter()
            .filter(|m| should_answer(m, &members, b"h", minute))
            .cloned()
            .collect::<Vec<_>>()
    };
    assert!((1..10).any(|m| pick(m) != pick(0)));
    // small sites: everyone answers
    assert!(members[..2]
        .iter()
        .all(|m| should_answer(m, &members[..2], b"h", 0)));
}

#[test]
fn an_author_always_resends_their_own_events() {
    let g = genesis_event(&new_key());
    let mut mine = Store::new(g.id);
    mine.insert(g.clone());
    let me = new_key();
    let other = new_key();
    for e in chain(&me, g.id, 2)
        .into_iter()
        .chain(chain(&other, g.id, 2))
    {
        mine.insert(e);
    }
    // a peer that has only the genesis: we send just our own two events, not the other author's
    let Wire::Heads(theirs) = heads_msg(&{
        let mut s = Store::new(g.id);
        s.insert(g);
        s
    }) else {
        unreachable!()
    };
    let own = to_resend_own(&mine, &theirs, &me.verifying_key().to_bytes());
    assert_eq!(own.len(), 2);
    assert_eq!(
        to_resend(&mine, &theirs).len(),
        4,
        "both authors when elected (the peer already has genesis)"
    );
}
