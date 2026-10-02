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
    let n = import_into(&dir, &blob(&s), PW).unwrap();
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
    let e = import_into(&dir, &blob(&s), PW).err().unwrap();
    assert_eq!(e, "this device already has an identity");
    assert_eq!(load(&dir).unwrap().unwrap().me(), s.ravi.me(), "untouched");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_failed_import_writes_nothing() {
    let s = Site::new();
    let dir = tmp("nothing");
    assert!(import_into(&dir, &blob(&s), "wrong password").is_err());
    assert!(load(&dir).unwrap().is_none());
    assert!(!dir.join("key").exists());
}
