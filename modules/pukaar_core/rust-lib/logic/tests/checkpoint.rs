mod util;
use pukaar_logic::checkpoint::{checkpoint_now, merkle_root, root_for, verify_anchor};
use pukaar_logic::event::new_key;
use pukaar_logic::store::Store;
use util::*;

#[test]
fn odd_promotion_differs_from_duplication() {
    let (a, b, c) = ([1; 32], [2; 32], [3; 32]);
    assert_ne!(merkle_root(vec![a, b, c]), merkle_root(vec![a, b, c, c]));
    assert_eq!(merkle_root(vec![a]), a);
    assert_eq!(merkle_root(vec![]), [0; 32]);
}

#[test]
fn checkpoint_signs_and_reproduces() {
    let mut full = Store::new(SITE);
    for e in chain(&new_key(), SITE, 3) {
        full.insert(e);
    }
    let me = new_key();
    let cp = checkpoint_now(&full, &me);
    assert_eq!(cp.n_events, 3);
    assert!(verify_anchor(
        &SITE,
        &cp.heads_root,
        cp.n_events,
        &cp.signer,
        &cp.sig
    ));
    assert!(
        !verify_anchor(&SITE, &[9; 32], cp.n_events, &cp.signer, &cp.sig),
        "wrong root"
    );
    assert_eq!(root_for(&full, &cp.heads), Some(cp.heads_root));
    let partial = Store::new(SITE);
    assert_eq!(
        root_for(&partial, &cp.heads),
        None,
        "can't reproduce without the events"
    );
}
