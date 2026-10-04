mod common;
mod util;
use common::*;
use pukaar_logic::identity::{export_identity, import_identity, import_into};
use pukaar_logic::persist::{load, save};
use std::path::PathBuf;

const PW: &str = "correct horse battery";

fn tmp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("pukaar-id-{tag}-{}", std::process::id()))
}

fn blob(s: &Site) -> String {
    export_identity(&s.asha.key, &s.asha.store.site, PW).unwrap()
}

#[test]
fn export_then_import_restores_key_and_site() {
    let s = Site::new();
    let b = blob(&s);
    assert!(b.starts_with("pukaar-id-1:"));
    let (key, site) = import_identity(&b, PW).unwrap();
    assert_eq!(key.to_bytes(), s.asha.key.to_bytes());
    assert_eq!(site, s.asha.store.site);
}

/// Made by v0.2.0 (key = [7; 32], site = [9; 32], password PW). Every backup a user holds must
/// keep importing: changing the KDF or the layout means a new version prefix, not this one.
const V1_BACKUP: &str = "pukaar-id-1:b1332844aff36cca3fb8e05bc8c48c776c5ff90fa5b5f4e06b4e51982b6c69be37a11d42b9c72290c1e041db800009f6f2851a61a6afedce695238e2533402efc8ef35f279e9275caba9968119fe1175f97802fd9863e7b2a7bc80004cda98886390744a016b32c202a5f29453c82df9884b4acb43dcb36f";

#[test]
fn an_existing_v1_backup_still_imports() {
    let (key, site) = import_identity(V1_BACKUP, PW).unwrap();
    assert_eq!(key.to_bytes(), [7; 32]);
    assert_eq!(site, [9; 32]);
}

/// The UI gives a core call 20 s in all; the import's own work (Argon2id, ~15 ms here) must stay
/// a small slice of it. Generous for slow CI, still far under the budget.
#[test]
fn import_fits_the_ui_call_budget() {
    let t = std::time::Instant::now();
    import_identity(V1_BACKUP, PW).unwrap();
    let took = t.elapsed();
    assert!(
        took < std::time::Duration::from_secs(2),
        "import took {took:?}"
    );
}

#[test]
fn the_blob_does_not_contain_the_key() {
    let s = Site::new();
    assert!(!blob(&s).contains(&hex::encode(s.asha.key.to_bytes())));
}

#[test]
fn two_exports_differ() {
    let s = Site::new();
    assert_ne!(blob(&s), blob(&s), "fresh salt and nonce each time");
}

#[test]
fn wrong_password_fails() {
    let s = Site::new();
    let e = import_identity(&blob(&s), "not the password").unwrap_err();
    assert!(e.contains("wrong password"), "{e}");
}

#[test]
fn tampered_blob_fails() {
    let s = Site::new();
    let mut b = blob(&s).into_bytes();
    let i = b.len() - 3;
    b[i] = if b[i] == b'0' { b'1' } else { b'0' };
    let e = import_identity(&String::from_utf8(b).unwrap(), PW).unwrap_err();
    assert!(e.contains("wrong password or damaged"), "{e}");
}

#[test]
fn wrong_version_and_junk_are_refused() {
    let s = Site::new();
    let v2 = blob(&s).replacen("pukaar-id-1:", "pukaar-id-2:", 1);
    assert!(import_identity(&v2, PW).unwrap_err().contains("version"));
    for junk in [
        "",
        "hello",
        "pukaar-id-1:",
        "pukaar-id-1:zz",
        "pukaar-id-1:abcd",
    ] {
        assert!(import_identity(junk, PW).is_err(), "{junk:?}");
    }
}

#[test]
fn short_password_is_rejected() {
    let s = Site::new();
    let e = export_identity(&s.asha.key, &s.asha.store.site, "short").unwrap_err();
    assert!(e.contains("at least 8"), "{e}");
}

#[test]
fn pasted_whitespace_is_tolerated() {
    let s = Site::new();
    let b = format!("  {}\n", blob(&s));
    assert!(import_identity(&b, PW).is_ok());
}

#[test]
fn import_into_a_fresh_device_gives_an_empty_node_with_the_same_identity() {
    let s = Site::new();
    let dir = tmp("fresh");
    let n = import_into(&dir, &blob(&s), PW, 0).unwrap();
    assert_eq!(n.me(), s.asha.me());
    assert_eq!(n.store.site, s.asha.store.site);
    assert!(n.store.events.is_empty(), "history arrives through sync");
    let back = load(&dir).unwrap().unwrap();
    assert_eq!(back.me(), s.asha.me());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn import_is_refused_when_a_site_exists() {
    let s = Site::new();
    let dir = tmp("taken");
    save(&s.ravi, &dir).unwrap();
    let e = import_into(&dir, &blob(&s), PW, 0).err().unwrap();
    assert_eq!(e, "this device already has an identity");
    assert_eq!(load(&dir).unwrap().unwrap().me(), s.ravi.me(), "untouched");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_failed_import_writes_nothing() {
    let s = Site::new();
    let dir = tmp("nothing");
    assert!(import_into(&dir, &blob(&s), "wrong password", 0).is_err());
    assert!(load(&dir).unwrap().is_none());
    assert!(!dir.join("key").exists());
}

// ---- a restored identity must not fork its own chain before its history is back ----

use pukaar_logic::event::{Body, Key};
use pukaar_logic::node::{Node, HEADS_TO_SETTLE, SKIP_SYNC_AFTER};
use pukaar_logic::store::Accept;

const SYNCING: &str = "still syncing your history, try again shortly";

fn note(n: &mut Node) -> Result<pukaar_logic::event::Event, String> {
    n.publish(
        Body::Profile {
            display_name: Some("asha".into()),
        },
        200,
    )
}

/// Asha has two events out there (seq 0 and 1); then she restores on a new device.
fn restored(tag: &str) -> (Site, Node, PathBuf) {
    let mut s = Site::new();
    note(&mut s.asha).unwrap();
    note(&mut s.asha).unwrap();
    s.sync();
    let dir = tmp(tag);
    let n = import_into(&dir, &blob(&s), PW, 100).unwrap();
    (s, n, dir)
}

fn heads(n: &mut Node, from: &[(Key, u64)], times: u32) {
    for _ in 0..times {
        n.observe_heads(from);
    }
}

#[test]
fn a_restored_node_refuses_to_publish_until_its_history_is_back() {
    let (s, mut n, dir) = restored("guard");
    assert_eq!(note(&mut n).unwrap_err(), SYNCING);
    assert!(n.store.events.is_empty(), "nothing inserted");
    let me: serde_json::Value = serde_json::from_str(&n.identity_json()).unwrap();
    assert_eq!(me["syncing_own_history"], true);
    // peers keep naming us at seq 1, but we hold none of our events yet
    heads(&mut n, &s.admin.store.head_seqs(), HEADS_TO_SETTLE + 5);
    assert_eq!(note(&mut n).unwrap_err(), SYNCING);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn once_caught_up_the_next_event_continues_the_old_chain() {
    let (mut s, mut n, dir) = restored("caught-up");
    heads(&mut n, &s.admin.store.head_seqs(), HEADS_TO_SETTLE);
    copy_all(&s.admin, &mut n);
    let me: serde_json::Value = serde_json::from_str(&n.identity_json()).unwrap();
    assert_eq!(me["syncing_own_history"], false);
    let e = note(&mut n).unwrap();
    assert_eq!(e.u.seq, 2);
    assert_eq!(s.admin.receive(&e.bytes).unwrap(), Accept::New, "no fork");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_peer_naming_a_later_seq_keeps_us_waiting() {
    let (mut s, mut n, dir) = restored("later");
    copy_all(&s.admin, &mut n); // seq 0 and 1 back
    note(&mut s.asha).unwrap(); // the old device also signed seq 2, which only it has
    heads(&mut n, &s.asha.store.head_seqs(), HEADS_TO_SETTLE);
    assert_eq!(
        note(&mut n).unwrap_err(),
        SYNCING,
        "seq 2 is still out there"
    );
    copy_all(&s.asha, &mut n);
    assert_eq!(note(&mut n).unwrap().u.seq, 3);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn too_few_heads_messages_keep_us_waiting() {
    let (s, mut n, dir) = restored("few");
    copy_all(&s.admin, &mut n);
    heads(&mut n, &s.admin.store.head_seqs(), HEADS_TO_SETTLE - 1);
    assert_eq!(note(&mut n).unwrap_err(), SYNCING);
    heads(&mut n, &s.admin.store.head_seqs(), 1);
    assert!(note(&mut n).is_ok());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn the_user_can_skip_the_wait_only_after_a_timeout() {
    let (_s, mut n, dir) = restored("skip");
    let early = n.skip_history_sync(100 + SKIP_SYNC_AFTER - 1).unwrap_err();
    assert!(early.contains("minute"), "{early}");
    assert_eq!(note(&mut n).unwrap_err(), SYNCING);
    n.skip_history_sync(100 + SKIP_SYNC_AFTER).unwrap();
    assert!(note(&mut n).is_ok());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn the_syncing_flag_survives_a_restart_and_clears_on_disk() {
    let (s, mut n, dir) = restored("restart");
    let back = load(&dir).unwrap().unwrap();
    assert_eq!(back.restore.as_ref().map(|r| r.since), Some(100));
    heads(&mut n, &s.admin.store.head_seqs(), HEADS_TO_SETTLE);
    copy_all(&s.admin, &mut n);
    save(&n, &dir).unwrap();
    assert!(load(&dir).unwrap().unwrap().restore.is_none());
    std::fs::remove_dir_all(dir).unwrap();
}
