mod common;
mod util;
use pukaar_logic::event::new_key;
use pukaar_logic::profile::{Announce, MAX_CID};
use pukaar_logic::store::{Accept, Store};
use pukaar_logic::sync::{
    heads_msg, lost_node, send_queue, should_answer, to_resend, to_resend_own, SendError, Wire,
    WriteKind, ANSWERERS, DELIVERY_CFG, MAX_RESEND, MAX_WIRE_BYTES,
};
use pukaar_sites_rules::JOIN_REQ_LEN;
use util::*;

#[test]
fn a_behind_store_catches_up_from_heads() {
    let (mut ahead, mut behind) = (Store::new(SITE), Store::new(SITE));
    for e in chain(&new_key(), SITE, 5) {
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
    let mut ahead = Store::new(SITE);
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
    let mut mine = Store::new(SITE);
    let me = new_key();
    let other = new_key();
    for e in chain(&me, SITE, 2)
        .into_iter()
        .chain(chain(&other, SITE, 2))
    {
        mine.insert(e);
    }
    // a peer that holds nothing: we send just our own two events, not the other author's
    let Wire::Heads(theirs) = heads_msg(&Store::new(SITE)) else {
        unreachable!()
    };
    let own = to_resend_own(&mine, &theirs, &me.verifying_key().to_bytes());
    assert_eq!(own.len(), 2);
    assert_eq!(
        to_resend(&mine, &theirs).len(),
        4,
        "both authors when elected"
    );
}

#[test]
fn a_message_delivery_did_not_answer_stays_queued_with_everything_after_it() {
    // Delivery unreachable at the 2nd message: it and the 3rd come back, in order; nothing resent
    let queue = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
    let mut tried = vec![];
    let (unsent, err) = send_queue(queue, |m| {
        tried.push(m.to_vec());
        match m {
            b"b" => Err(SendError::Retry("timed out".into())),
            _ => Ok(()),
        }
    });
    assert_eq!(
        tried,
        [b"a".to_vec(), b"b".to_vec()],
        "stops at the first unanswered call"
    );
    assert_eq!(unsent, [b"b".to_vec(), b"c".to_vec()]);
    assert_eq!(err.as_deref(), Some("timed out"));
    // once Delivery is back the rest goes out, and nothing is left
    let (unsent, err) = send_queue(unsent, |_| Ok(()));
    assert!(unsent.is_empty() && err.is_none());
    // a refusal (Delivery answered) drops that message and carries on
    let (unsent, err) = send_queue(vec![b"x".to_vec(), b"y".to_vec()], |m| match m {
        b"x" => Err(SendError::Refused("too big".into())),
        _ => Ok(()),
    });
    assert!(unsent.is_empty());
    assert_eq!(err.as_deref(), Some("too big"));
}

#[test]
fn a_restarted_delivery_is_a_reason_to_retry_not_to_drop() {
    let retry =
        |answered, e: &str| matches!(SendError::new(answered, e.into()), SendError::Retry(_));
    assert!(retry(false, "channelSend: timed out"), "no answer");
    // Delivery's own words when its host came back without a node (seen headless after a restart)
    assert!(retry(true, "channelSend: Context not initialized"));
    assert!(lost_node("channelSend: Context not initialized"));
    assert!(!retry(true, "channelSend: payload too large"));
    assert!(!lost_node("createNode: Context already initialized"));
}

#[test]
fn delivery_never_holds_a_message_back_for_causal_order() {
    // Delivery's SDS layer parks every message whose 2 predecessors it never saw, for good: one
    // send that never left (logos.test without RLN) wedged both Basecamp profiles. Heads
    // anti-entropy already orders and fills gaps, so SDS must carry no causal history.
    let cfg: serde_json::Value = serde_json::from_str(DELIVERY_CFG).unwrap();
    assert_eq!(cfg["preset"], "logos.dev");
    assert_eq!(cfg["channelsOverrides"]["sdsCausalHistorySize"], 0);
}

#[test]
fn a_join_published_while_delivery_was_down_still_reaches_the_admin() {
    use pukaar_logic::node::Node;
    let mut admin = Store::new(SITE);
    // the joiner signed an event, but the send failed and the message was lost for good
    let mut joiner = Node::join(new_key(), SITE);
    joiner.publish(common::comment("hello"), 1).unwrap();
    // the admin's next Heads (once a minute) makes the joiner re-send its own event
    let Wire::Heads(theirs) = heads_msg(&admin) else {
        unreachable!()
    };
    for b in to_resend_own(&joiner.store, &theirs, &joiner.me()) {
        assert_eq!(
            admin.insert(pukaar_logic::event::decode(&b).unwrap()),
            Accept::New
        );
    }
    assert_eq!(admin.heads(), joiner.store.heads());
    // a second copy (the original turning up late, or another answer) changes nothing
    let again = to_resend(&joiner.store, &[]).pop().unwrap();
    assert_eq!(
        admin.insert(pukaar_logic::event::decode(&again).unwrap()),
        Accept::Duplicate
    );
}

#[test]
fn a_peer_on_the_losing_branch_gets_the_winner_even_when_ahead() {
    let k = new_key();
    let a = branch(&k, SITE, None, 1, "a").remove(0);
    let b = branch(&k, SITE, None, 1, "b").remove(0);
    let (low, high) = if a.id < b.id { (a, b) } else { (b, a) };
    let high_kids = branch(&k, SITE, Some(&high), 2, "h");
    let (mut knows_fork, mut on_loser) = (Store::new(SITE), Store::new(SITE));
    for e in [&low, &high] {
        knows_fork.insert(e.clone());
    }
    for e in [&high, &high_kids[0], &high_kids[1]] {
        on_loser.insert(e.clone());
    }
    // the loser's heads name seq 2, past everything the other holds, yet the fork still flows
    let Wire::Heads(theirs) = heads_msg(&on_loser) else {
        unreachable!()
    };
    for b in to_resend(&knows_fork, &theirs) {
        on_loser.insert(pukaar_logic::event::decode(&b).unwrap());
    }
    assert_eq!(on_loser.heads(), knows_fork.heads());
    assert_eq!(on_loser.fork_count(), 3);
}

// ---- the format 2 wire codec ----

fn announce() -> Announce {
    Announce::sign(&new_key(), SITE, 2, "zDvZRwzmCaFRr2ZEHrYKsbNrmCP3V2tHPNVrKvDDDs2Kw3bnkDC", [5; 32])
}

#[test]
fn every_message_kind_round_trips() {
    let ev = chain(&new_key(), SITE, 1).remove(0);
    let msgs = [
        Wire::Event(ev.bytes),
        Wire::Heads(vec![([1; 32], 4), ([2; 32], 0)]),
        Wire::JoinReq(vec![9; JOIN_REQ_LEN]),
        Wire::ProfileAnnounce(announce()),
        Wire::ChainPending { tx: [3; 32], kind: WriteKind::Approve },
    ];
    for m in msgs {
        assert_eq!(Wire::decode(&m.encode()), Some(m));
    }
    // a ProfileAnnounce keeps its signature through the wire
    let Some(Wire::ProfileAnnounce(a)) = Wire::decode(&Wire::ProfileAnnounce(announce()).encode()) else {
        panic!()
    };
    a.verify().unwrap();
}

#[test]
fn a_join_request_has_exactly_290_bytes() {
    for n in [0, 1, JOIN_REQ_LEN - 1, JOIN_REQ_LEN + 1, 1000] {
        assert!(Wire::decode(&Wire::JoinReq(vec![1; n]).encode()).is_none(), "{n} bytes");
    }
    assert!(Wire::decode(&Wire::JoinReq(vec![1; JOIN_REQ_LEN]).encode()).is_some());
}

#[test]
fn size_limits_hold() {
    let big_event = Wire::Event(vec![0; pukaar_logic::event::MAX_EVENT_BYTES + 1]).encode();
    assert!(Wire::decode(&big_event).is_none(), "event over the cap");
    let ok_event = Wire::Event(vec![0; pukaar_logic::event::MAX_EVENT_BYTES]).encode();
    assert!(ok_event.len() <= MAX_WIRE_BYTES && Wire::decode(&ok_event).is_some());
    assert!(Wire::decode(&vec![0; MAX_WIRE_BYTES + 1]).is_none(), "whole message over the cap");
    let long_cid = Announce { cid: "z".repeat(MAX_CID + 1), ..announce() };
    assert!(Wire::decode(&Wire::ProfileAnnounce(long_cid).encode()).is_none());
    let short_sig = Announce { sig: vec![0; 63], ..announce() };
    assert!(Wire::decode(&Wire::ProfileAnnounce(short_sig).encode()).is_none());
    let no_cid = Announce { cid: String::new(), ..announce() };
    assert!(Wire::decode(&Wire::ProfileAnnounce(no_cid).encode()).is_none());
}

#[test]
fn garbage_is_rejected() {
    assert!(Wire::decode(&[]).is_none());
    assert!(Wire::decode(&[0xff; 40]).is_none(), "bad variant");
    assert!(Wire::decode(&[9]).is_none(), "variant past the last one");
    let mut trailing = Wire::ChainPending { tx: [3; 32], kind: WriteKind::Revoke }.encode();
    trailing.push(0);
    assert!(Wire::decode(&trailing).is_none(), "trailing byte");
    let cut = Wire::Heads(vec![([1; 32], 4)]).encode();
    assert!(Wire::decode(&cut[..cut.len() - 1]).is_none(), "truncated");
    // a format 1 Wire::Event of any kind still decodes as bytes, but the event decoder refuses it
    let old = Wire::Event(b"logos:pukaar:1\0 not an event".to_vec());
    let Some(Wire::Event(b)) = Wire::decode(&old.encode()) else { panic!() };
    assert!(pukaar_logic::event::decode(&b).is_err());
    // a tampered announce still decodes (sizes are fine) but fails its signature check
    let mut a = announce();
    a.version += 1;
    let Some(Wire::ProfileAnnounce(a)) = Wire::decode(&Wire::ProfileAnnounce(a).encode()) else { panic!() };
    assert!(a.verify().is_err());
}
