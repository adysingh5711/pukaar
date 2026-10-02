#![allow(dead_code)]
use pukaar_logic::event::{new_key, Body, Role};
use pukaar_logic::node::Node;

pub fn genesis() -> Body {
    crate::util::genesis_body()
}

/// Everyone shares one replica through `copy_all`, like a perfectly connected LAN.
pub struct Site {
    pub admin: Node,
    pub steward: Node,
    pub asha: Node,
    pub ravi: Node,
    pub meera: Node,
}

pub fn copy_all(from: &Node, to: &mut Node) {
    for e in from.store.events.values() {
        to.receive(&e.bytes).unwrap();
    }
}

impl Site {
    pub fn new() -> Site {
        let mut admin = Node::create_site(new_key(), genesis(), 0).unwrap();
        let site = admin.store.site;
        let steward = Node::join(new_key(), site);
        let asha = Node::join(new_key(), site);
        let ravi = Node::join(new_key(), site);
        let meera = Node::join(new_key(), site);
        admin
            .publish(
                Body::RoleGrant {
                    subject: steward.me(),
                    role: Role::Steward,
                    name: Some("Facilities".into()),
                },
                1,
            )
            .unwrap();
        for r in [&asha, &ravi, &meera] {
            admin
                .publish(
                    Body::RoleGrant {
                        subject: r.me(),
                        role: Role::Resident,
                        name: None,
                    },
                    1,
                )
                .unwrap();
        }
        let mut s = Site {
            admin,
            steward,
            asha,
            ravi,
            meera,
        };
        s.sync();
        s
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
                .flat_map(|n| {
                    n.store
                        .events
                        .values()
                        .map(|e| e.bytes.clone())
                        .collect::<Vec<_>>()
                })
                .collect();
            for n in self.nodes() {
                for b in &all {
                    n.receive(b).unwrap();
                }
            }
        }
    }
}
