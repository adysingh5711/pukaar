mod common;
mod util;
use common::*;
use pukaar_logic::node::{action_body, genesis_from_json};
use pukaar_logic::persist::{load, save};

#[test]
fn save_then_load_gives_the_same_state() {
    let s = Site::new();
    let dir = std::env::temp_dir().join(format!("pukaar-test-{}", std::process::id()));
    save(&s.asha, &dir).unwrap();
    let back = load(&dir).unwrap().unwrap();
    assert_eq!(back.me(), s.asha.me());
    assert_eq!(back.state(), s.asha.state());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn json_and_action_parsing() {
    let g = r#"{"Genesis":{"name":"Dhun","admin_name":"Ops lead","categories":["water"],"locations":[{"code":"W-03","label":"Tap","group":"Water points"}],
                "sla_ack_h":12,"sla_fix_h":48,"max_open_per_author":10}}"#;
    assert!(genesis_from_json(g).is_ok());
    assert!(genesis_from_json(r#"{"Profile":{"display_name":null}}"#).is_err());
    assert!(action_body([1; 32], "reopen", "still dripping", "", 0).is_ok());
    assert!(
        action_body([1; 32], "delete", "", "", 0).is_err(),
        "there is no delete"
    );
}

#[test]
fn genesis_must_name_the_admin() {
    let g = |name: &str| {
        format!(
            r#"{{"Genesis":{{"name":"Dhun","admin_name":"{name}","categories":["water"],"locations":[],
                "sla_ack_h":12,"sla_fix_h":48,"max_open_per_author":10}}}}"#
        )
    };
    for unnamed in ["", "   ", "<your name>", " <your name> "] {
        assert_eq!(
            genesis_from_json(&g(unnamed)),
            Err("the admin must be named".into()),
            "{unnamed:?}"
        );
    }
    assert!(genesis_from_json(&g("Ops lead")).is_ok());
}
