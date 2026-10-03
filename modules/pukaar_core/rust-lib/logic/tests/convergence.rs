//! "Everyone sees the same board": any delivery order of the same events gives the same state.
mod common;
mod util;
use common::*;
use pukaar_logic::event::{Body, Id};
use pukaar_logic::node::Node;

/// Tiny deterministic RNG, so failures reproduce from the seed.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn pick(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn random_action(r: &mut Rng, issues: &[Id]) -> Body {
    let text = || "t".to_string();
    if issues.is_empty() || r.pick(5) == 0 {
        let loc = ["W-03", "B-07", "X-99", "other"][r.pick(4)];
        let landmark = if r.pick(2) == 0 {
            "by the neem tree".to_string()
        } else {
            String::new()
        };
        return Body::Report {
            category: "water".into(),
            location: loc.into(),
            landmark,
            text: text(),
        };
    }
    let issue = issues[r.pick(issues.len())];
    match r.pick(8) {
        0 => Body::Acknowledge {
            issue,
            eta_h: 2,
            note: text(),
        },
        1 => Body::Update {
            issue,
            note: text(),
            next_step: text(),
            eta_h: 3,
        },
        2 => Body::ClaimResolved {
            issue,
            note: text(),
        },
        3 => Body::Confirm {
            issue,
            note: text(),
        },
        4 => Body::Reopen {
            issue,
            reason: text(),
        },
        5 => Body::CloseWontfix {
            issue,
            reason: text(),
        },
        6 => Body::MarkDuplicate {
            issue,
            of: issues[r.pick(issues.len())],
        },
        _ => Body::Comment {
            issue,
            text: text(),
        },
    }
}

#[test]
fn any_delivery_order_gives_the_same_state() {
    for seed in 1..=60u64 {
        let mut r = Rng(seed);
        let mut s = Site::new();
        // concurrent bursts: each author acts on its own view, then everyone syncs
        for _round in 0..6 {
            for _ in 0..4 {
                let who = r.pick(5);
                let issues: Vec<Id> = s.nodes()[who].state().issues.keys().copied().collect();
                let body = random_action(&mut r, &issues);
                s.nodes()[who].publish(body, r.next() % 1000).unwrap();
            }
            if r.pick(2) == 0 {
                s.sync();
            }
        }
        s.sync();
        let all: Vec<Vec<u8>> = s
            .admin
            .store
            .events
            .values()
            .map(|e| e.bytes.clone())
            .collect();
        let expected = s.admin.state();
        for _ in 0..20 {
            let mut order = all.clone();
            for i in (1..order.len()).rev() {
                order.swap(i, r.pick(i + 1));
            }
            let mut fresh = Node::join(pukaar_logic::event::new_key(), s.admin.store.site);
            for b in &order {
                fresh.receive(b).unwrap();
            }
            assert!(
                fresh.store.pending.is_empty(),
                "seed {seed}: all gaps filled"
            );
            assert_eq!(fresh.state(), expected, "seed {seed}: state diverged");
        }
    }
}

/// A fresh replica fed `all` in a shuffled order.
fn replay(r: &mut Rng, site: Id, all: &[Vec<u8>]) -> Node {
    let mut order: Vec<&Vec<u8>> = all.iter().collect();
    for i in (1..order.len()).rev() {
        order.swap(i, r.pick(i + 1));
    }
    let mut fresh = Node::join(pukaar_logic::event::new_key(), site);
    for b in order {
        fresh.receive(b).unwrap();
    }
    fresh
}

/// One identity on two devices double-signs every round (both publish before syncing), on top
/// of random activity: every replica, and a fresh one fed any order, keeps the same branch.
#[test]
fn a_forked_author_converges_to_one_branch_in_any_delivery_order() {
    for seed in 1..=30u64 {
        let mut r = Rng(seed);
        let mut s = Site::new();
        let mut laptop = second_device(&s.asha);
        for round in 0..4u64 {
            let mut nodes = with(&mut s, &mut laptop);
            nodes[2].publish(profile("phone"), round).unwrap();
            nodes[5].publish(profile("laptop"), round).unwrap();
            for _ in 0..4 {
                let who = r.pick(6);
                let issues: Vec<Id> = nodes[who].state().issues.keys().copied().collect();
                let body = random_action(&mut r, &issues);
                nodes[who].publish(body, r.next() % 1000).unwrap();
            }
            if r.pick(2) == 0 {
                gossip(&mut nodes);
            }
        }
        let mut nodes = with(&mut s, &mut laptop);
        gossip(&mut nodes);
        let (expected, canon) = (nodes[0].state(), canonical(nodes[0]));
        let forks = nodes[0].store.fork_count();
        assert!(forks > 0, "seed {seed}: the double-signing was caught");
        for n in &nodes {
            assert_eq!(n.state(), expected, "seed {seed}: replicas diverged");
            assert_eq!(canonical(n), canon, "seed {seed}: different branches kept");
            assert_eq!(n.store.fork_count(), forks, "seed {seed}");
        }
        let all: Vec<Vec<u8>> = nodes[0].store.held().map(|e| e.bytes.clone()).collect();
        for _ in 0..10 {
            let fresh = replay(&mut r, s.admin.store.site, &all);
            assert!(fresh.store.pending.is_empty(), "seed {seed}: gaps filled");
            assert_eq!(fresh.state(), expected, "seed {seed}: state diverged");
            assert_eq!(canonical(&fresh), canon, "seed {seed}: branch diverged");
            assert_eq!(fresh.store.fork_count(), forks, "seed {seed}");
        }
    }
}

#[test]
fn after_losing_a_fork_our_next_event_builds_on_the_winner() {
    let mut s = Site::new();
    let mut laptop = second_device(&s.asha);
    // both devices sign two events offline: seq n and n+1 on each, conflicting
    let phone: Vec<_> = (0..2)
        .map(|i| s.asha.publish(profile("phone"), i).unwrap())
        .collect();
    let lap: Vec<_> = (0..2)
        .map(|i| laptop.publish(profile("laptop"), i).unwrap())
        .collect();
    assert_eq!(phone[0].u.seq, lap[0].u.seq);
    gossip(&mut with(&mut s, &mut laptop));
    let phone_wins = phone[0].id < lap[0].id;
    let head = if phone_wins { &phone[1] } else { &lap[1] };
    assert_eq!(
        s.admin.store.fork_count(),
        2,
        "the losing seq and its child"
    );
    let loser = if phone_wins { &mut laptop } else { &mut s.asha };
    let next = loser.publish(profile("again"), 9).unwrap();
    assert_eq!((next.u.seq, next.u.prev), (head.u.seq + 1, head.id));
    let mut nodes = with(&mut s, &mut laptop);
    gossip(&mut nodes);
    let expected = nodes[0].state();
    for n in &nodes {
        assert_eq!(n.store.fork_count(), 2, "no new fork");
        assert_eq!(
            n.store
                .events
                .values()
                .find(|e| e.id == next.id)
                .map(|e| e.id),
            Some(next.id)
        );
        assert_eq!(n.state(), expected);
    }
}
