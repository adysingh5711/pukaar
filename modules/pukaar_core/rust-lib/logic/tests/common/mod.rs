#![allow(dead_code)]
use pukaar_logic::event::{new_key, Body, Key, SigningKey};
use pukaar_logic::node::Node;
use pukaar_logic::reducer::Rank;
use std::collections::BTreeMap;

/// The key that holds `rank` (the test site has one of the first two).
fn roles_key(roles: &BTreeMap<Key, Rank>, rank: Rank) -> Key {
    *roles.iter().find(|(_, r)| **r == rank).unwrap().0
}

/// A node of the test site: its key, the chain roles of everyone, and the site setup.
pub fn node(key: SigningKey, roles: &BTreeMap<Key, Rank>) -> Node {
    equip(Node::join(key, crate::util::SITE), roles)
}

/// Give a node the chain roles and the site setup (a loaded or restored node has none).
pub fn equip(mut n: Node, roles: &BTreeMap<Key, Rank>) -> Node {
    let roles = roles.clone();
    n.roles = Box::new(move |k, _, _| roles.get(k).copied());
    n.config = crate::util::config();
    n.locations = crate::util::locations();
    n
}

/// The same identity on another device: same key, a copy of `n`'s replica and the same roles.
pub fn second_device(n: &Node, roles: &BTreeMap<Key, Rank>) -> Node {
    let mut d = node(n.key.clone(), roles);
    copy_all(n, &mut d);
    d
}

/// Everyone shares one replica through `copy_all`, like a perfectly connected LAN.
pub struct Site {
    pub admin: Node,
    pub steward: Node,
    pub asha: Node,
    pub ravi: Node,
    pub meera: Node,
    /// The chain roles: admin is the super admin, then a steward and three residents.
    pub roles: BTreeMap<Key, Rank>,
}

pub fn copy_all(from: &Node, to: &mut Node) {
    for e in from.store.held() {
        to.receive(&e.bytes).unwrap();
    }
}

impl Site {
    pub fn new() -> Site {
        let keys: Vec<SigningKey> = (0..5).map(|_| new_key()).collect();
        let ranks = [
            Rank::SuperAdmin,
            Rank::Steward,
            Rank::Resident,
            Rank::Resident,
            Rank::Resident,
        ];
        let roles: BTreeMap<Key, Rank> = keys
            .iter()
            .zip(ranks)
            .map(|(k, r)| (k.verifying_key().to_bytes(), r))
            .collect();
        let mut nodes: Vec<Node> = keys.into_iter().map(|k| node(k, &roles)).collect();
        for n in &mut nodes {
            n.names.insert(roles_key(&roles, Rank::SuperAdmin), "Site admin".into());
            n.names.insert(roles_key(&roles, Rank::Steward), "Facilities".into());
        }
        let mut it = nodes.into_iter();
        let mut next = || it.next().unwrap();
        Site {
            admin: next(),
            steward: next(),
            asha: next(),
            ravi: next(),
            meera: next(),
            roles,
        }
    }

    pub fn nodes(&mut self) -> [&mut Node; 5] {
        [
            &mut self.admin,
            &mut self.steward,
            &mut self.asha,
            &mut self.ravi,
            &mut self.meera,
        ]
    }

    /// Gossip until everyone holds everything.
    pub fn sync(&mut self) {
        for _ in 0..2 {
            let all: Vec<Vec<u8>> = self
                .nodes()
                .iter()
                .flat_map(|n| n.store.held().map(|e| e.bytes.clone()).collect::<Vec<_>>())
                .collect();
            for n in self.nodes() {
                for b in &all {
                    n.receive(b).unwrap();
                }
            }
        }
    }
}

/// The site's five nodes plus `extra` (say, a second device).
pub fn with<'a>(s: &'a mut Site, extra: &'a mut Node) -> [&'a mut Node; 6] {
    let [a, b, c, d, e] = s.nodes();
    [a, b, c, d, e, extra]
}

/// One round of full gossip: every node receives every event any of them holds,
/// canonical and fork evidence alike.
pub fn gossip(nodes: &mut [&mut Node]) {
    let all: Vec<Vec<u8>> = nodes
        .iter()
        .flat_map(|n| n.store.held().map(|e| e.bytes.clone()))
        .collect();
    for n in nodes.iter_mut() {
        for b in &all {
            n.receive(b).unwrap();
        }
    }
}

/// A node's canonical chain ids (the events its reducer applies), in key order.
pub fn canonical(n: &Node) -> Vec<pukaar_logic::event::Id> {
    n.store.events.values().map(|e| e.id).collect()
}

/// A body that needs no state, for tests about delivery and forks.
pub fn comment(text: &str) -> Body {
    Body::Comment {
        issue: pukaar_logic::event::ZERO,
        text: text.into(),
    }
}
