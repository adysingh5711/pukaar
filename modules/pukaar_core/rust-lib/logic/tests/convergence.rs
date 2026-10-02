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
