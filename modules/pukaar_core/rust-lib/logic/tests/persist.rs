mod common;
mod util;
use common::*;
use pukaar_logic::node::Node;
use pukaar_logic::node::action_body;
use pukaar_logic::persist::{join_site, leave, load, load_or_create_key, save};

#[test]
fn save_then_load_gives_the_same_state() {
    let s = Site::new();
    let dir = std::env::temp_dir().join(format!("pukaar-test-{}", std::process::id()));
    save(&s.asha, &dir).unwrap();
    let back = equip(load(&dir).unwrap().unwrap(), &s.roles);
    assert_eq!(back.me(), s.asha.me());
    assert_eq!(back.state(), s.asha.state());
    assert_eq!(back.store.site, util::SITE, "the site id is the one the caller gave");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn action_parsing() {
    assert!(action_body([1; 32], "reopen", "still dripping", "", 0).is_ok());
    assert!(
        action_body([1; 32], "delete", "", "", 0).is_err(),
        "there is no delete"
    );
}

#[test]
fn leaving_drops_the_replica_but_keeps_the_key() {
    let s = Site::new();
    let site = s.asha.store.site;
    let dir = std::env::temp_dir().join(format!("pukaar-leave-{}", std::process::id()));
    save(&s.asha, &dir).unwrap();
    leave(&dir, &site).unwrap();
    assert!(load(&dir).unwrap().is_none(), "no site, no events");
    let key = load_or_create_key(&dir).unwrap();
    assert_eq!(key.to_bytes(), s.asha.key.to_bytes(), "same person");
    // our chain in that site lives on: rejoining waits for it instead of re-signing seq 0
    let back = join_site(&dir, key.clone(), site, 7);
    assert!(back.restore.is_some() && back.store.events.is_empty());
    let other = join_site(&dir, key, [9; 32], 7);
    assert!(other.restore.is_none());
    assert!(other.store.events.is_empty(), "joining signs nothing");
    std::fs::remove_dir_all(dir).unwrap();
}

/// Canonical ids, every held id (sorted), and the fork count: what a restart must reproduce.
fn view(
    n: &Node,
) -> (
    Vec<pukaar_logic::event::Id>,
    Vec<pukaar_logic::event::Id>,
    usize,
) {
    let mut held: Vec<_> = n.store.held().map(|e| e.id).collect();
    held.sort();
    (canonical(n), held, n.store.fork_count())
}

#[test]
fn fork_evidence_survives_a_restart_whatever_the_file_order() {
    let mut s = Site::new();
    let mut laptop = second_device(&s.asha, &s.roles);
    s.asha.publish(comment("phone"), 5).unwrap();
    laptop.publish(comment("laptop"), 5).unwrap();
    copy_all(&s.asha, &mut s.admin);
    copy_all(&laptop, &mut s.admin);
    assert_eq!(s.admin.store.fork_count(), 1);
    let dir = std::env::temp_dir().join(format!("pukaar-forks-{}", std::process::id()));
    save(&s.admin, &dir).unwrap();
    let back = equip(load(&dir).unwrap().unwrap(), &s.roles);
    assert_eq!(view(&back), view(&s.admin));
    assert_eq!(back.state(), s.admin.state());
    // same file format (u32-LE length-prefixed events), records reversed: same view
    let path = dir.join("events.bin");
    let buf = std::fs::read(&path).unwrap();
    let mut records = Vec::new();
    let mut rest = &buf[..];
    while !rest.is_empty() {
        let n = 4 + u32::from_le_bytes(rest[..4].try_into().unwrap()) as usize;
        records.push(&rest[..n]);
        rest = &rest[n..];
    }
    records.reverse();
    std::fs::write(&path, records.concat()).unwrap();
    let reversed = equip(load(&dir).unwrap().unwrap(), &s.roles);
    assert_eq!(view(&reversed), view(&s.admin));
    assert_eq!(reversed.state(), s.admin.state());
    std::fs::remove_dir_all(dir).unwrap();
}
