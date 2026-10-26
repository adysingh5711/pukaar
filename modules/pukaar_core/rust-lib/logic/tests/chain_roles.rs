//! Format 2: the reducer judges each author with the chain role at the event's `chain_block`.
mod common;
mod util;
use common::*;
use pukaar_logic::event::{new_key, Body, Event, Id};
use pukaar_logic::node::Node;
use pukaar_logic::reducer::{Rank, Status};

fn report(n: &mut Node) -> Event {
    n.publish(
        Body::Report {
            category: "water".into(),
            location: "W-03".into(),
            landmark: String::new(),
            text: "tap leaking".into(),
        },
        10,
    )
    .unwrap()
}

fn why(n: &Node, e: &Event) -> Option<String> {
    n.state().rejected.get(&e.id).cloned()
}

/// One node whose chain role is `rank` at every block, on a site with a resident reporter.
fn as_rank(rank: Rank) -> (Node, Node, Id) {
    let (reporter, actor) = (new_key(), new_key());
    let roles = [
        (reporter.verifying_key().to_bytes(), Rank::Resident),
        (actor.verifying_key().to_bytes(), rank),
    ]
    .into();
    let (mut reporter, mut actor) = (node(reporter, &roles), node(actor, &roles));
    let issue = report(&mut reporter).id;
    copy_all(&reporter, &mut actor);
    (reporter, actor, issue)
}

#[test]
fn an_author_who_was_not_a_member_at_the_chain_block_is_rejected_and_kept() {
    let mut s = Site::new();
    // the provider knows the author as a resident only from block 10 up to block 50
    let author = s.asha.me();
    let window = move |k: &_, block: u64| (*k == author && (10..50).contains(&block)).then_some(Rank::Resident);
    s.asha.roles = Box::new(window);
    let mut at = |block: u64| {
        s.asha.chain_block = block;
        report(&mut s.asha)
    };
    let (early, inside, after) = (at(5), at(20), at(60));
    let st = s.asha.state();
    assert!(st.issues.contains_key(&inside.id), "approved at block 20");
    assert!(st.rejected[&early.id].contains("not an approved member at block 5"));
    assert!(st.rejected[&after.id].contains("not an approved member at block 60"), "revoked by block 60");
    // rejected is kept, not dropped: still held, still in the log
    assert!(s.asha.store.events.contains_key(&(author, early.u.seq)));
    assert_eq!(s.asha.store.events.len(), 3);
}

#[test]
fn the_chain_block_decides_not_the_arrival_time() {
    // an old event of a member who is revoked now still counts: it names a block where they were in
    let mut s = Site::new();
    let author = s.ravi.me();
    s.ravi.chain_block = 20;
    let old = report(&mut s.ravi);
    let revoked_at_50 = move |k: &_, block: u64| (*k == author && block < 50).then_some(Rank::Resident);
    for n in s.nodes() {
        n.roles = Box::new(revoked_at_50);
    }
    s.sync();
    s.ravi.chain_block = 70;
    let late = report(&mut s.ravi);
    s.sync();
    for n in s.nodes() {
        assert!(n.state().issues.contains_key(&old.id), "applied on every replica");
        assert!(why(n, &late).is_some(), "rejected on every replica");
    }
}

#[test]
fn only_staff_acknowledge_claim_and_close() {
    for (rank, may) in [
        (Rank::Resident, false),
        (Rank::Steward, true),
        (Rank::Admin, true),
        (Rank::SuperAdmin, true),
    ] {
        let acts = |mk: &dyn Fn(Id) -> Body| {
            let (_, mut actor, issue) = as_rank(rank);
            let e = actor.publish(mk(issue), 11).unwrap();
            why(&actor, &e).is_none()
        };
        let ack = acts(&|issue| Body::Acknowledge { issue, eta_h: 1, note: String::new() });
        let claim = acts(&|issue| Body::ClaimResolved { issue, note: "fixed".into() });
        let close = acts(&|issue| Body::CloseWontfix { issue, reason: "no".into() });
        assert_eq!((ack, claim, close), (may, may, may), "{rank:?}");
    }
}

#[test]
fn a_resident_who_is_not_the_reporter_votes_and_staff_do_not() {
    // two residents reopen a claimed fix, the reporter alone can too
    let mut s = Site::new();
    let i = report(&mut s.asha).id;
    s.sync();
    s.steward.publish(Body::ClaimResolved { issue: i, note: "fixed".into() }, 11).unwrap();
    s.sync();
    let admin_try = s.admin.publish(Body::Reopen { issue: i, reason: "no".into() }, 12).unwrap();
    s.sync();
    assert!(why(&s.admin, &admin_try).unwrap().contains("only the reporter or residents"));
    s.ravi.publish(Body::Reopen { issue: i, reason: "still".into() }, 13).unwrap();
    s.sync();
    assert_eq!(s.admin.state().issues[&i].status, Status::AwaitingConfirmation, "one vote is not enough");
    s.meera.publish(Body::Reopen { issue: i, reason: "agreed".into() }, 14).unwrap();
    s.sync();
    assert_eq!(s.admin.state().issues[&i].status, Status::Open, "two residents reopen");
    // the reporter alone
    s.steward.publish(Body::ClaimResolved { issue: i, note: "again".into() }, 15).unwrap();
    s.sync();
    s.asha.publish(Body::Reopen { issue: i, reason: "mine".into() }, 16).unwrap();
    s.sync();
    assert_eq!(s.admin.state().issues[&i].status, Status::Open);
    assert_eq!(s.admin.state().issues[&i].reopen_count, 2);
}

#[test]
fn a_checkpoint_from_a_non_member_is_rejected_too() {
    let s = Site::new();
    let mut stranger = node(new_key(), &s.roles);
    let e = stranger
        .publish(Body::Checkpoint { heads: vec![], lez_tx: "tx".into() }, 5)
        .unwrap();
    assert!(why(&stranger, &e).is_some());
    assert!(stranger.state().checkpoints.is_empty());
}
