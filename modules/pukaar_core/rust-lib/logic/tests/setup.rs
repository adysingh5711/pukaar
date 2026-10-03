//! Site setup: the UI builds a genesis from the admin's draft; the core is the authority.
use pukaar_logic::event::new_key;
use pukaar_logic::node::{genesis_from_json, Node};
use serde_json::{json, Value};

fn draft() -> Value {
    json!({"Genesis": {
        "name": "Dhun",
        "admin_name": "Ops lead",
        "categories": ["water", "waste"],
        "locations": [
            {"code": "W-01", "label": "Tap, dining hall", "group": "Water points"},
            {"code": "B-01", "label": "Bin, main path", "group": "Bins"}
        ],
        "sla_ack_h": 12, "sla_fix_h": 48, "max_open_per_author": 10
    }})
}

fn check(g: &Value) -> Result<(), String> {
    genesis_from_json(&g.to_string()).map(|_| ())
}

fn with(edit: impl FnOnce(&mut Value)) -> Result<(), String> {
    let mut g = draft();
    edit(&mut g["Genesis"]);
    check(&g)
}

#[test]
fn a_complete_draft_is_accepted() {
    assert_eq!(check(&draft()), Ok(()));
}

#[test]
fn a_site_can_start_with_no_locations() {
    assert_eq!(with(|g| g["locations"] = json!([])), Ok(()));
}

#[test]
fn the_site_needs_a_name() {
    assert_eq!(
        with(|g| g["name"] = json!("  ")),
        Err("the site needs a name".into())
    );
}

#[test]
fn categories_must_be_present_named_and_unique() {
    assert_eq!(
        with(|g| g["categories"] = json!([])),
        Err("add at least one category".into())
    );
    assert_eq!(
        with(|g| g["categories"] = json!(["water", " "])),
        Err("a category can't be empty".into())
    );
    assert_eq!(
        with(|g| g["categories"] = json!(["water", "water"])),
        Err("category water is listed twice".into())
    );
}

#[test]
fn location_codes_must_be_present_unique_and_not_reserved() {
    let loc = |code: &str| json!({"code": code, "label": "Tap", "group": "Water points"});
    assert_eq!(
        with(|g| g["locations"] = json!([loc(" ")])),
        Err("every location needs a code".into())
    );
    assert_eq!(
        with(|g| g["locations"] = json!([loc("W-01"), loc("W-01")])),
        Err("location code W-01 is used twice".into())
    );
    assert_eq!(
        with(|g| g["locations"] = json!([loc("other")])),
        Err("the code other is reserved for places not on the list".into())
    );
}

#[test]
fn every_location_needs_a_name_and_a_group() {
    for (label, group) in [("", "Water points"), ("Tap", " ")] {
        assert_eq!(
            with(|g| g["locations"] = json!([{"code": "W-01", "label": label, "group": group}])),
            Err("location W-01 needs a name and a group".into())
        );
    }
}

/// A1: the genesis is one event, so a big site map hits the 4 KiB limit; the core says so.
#[test]
fn a_site_map_over_the_event_limit_is_refused_as_too_long() {
    let mut g = draft();
    g["Genesis"]["locations"] = (0..200)
        .map(|i| json!({"code": format!("W-{i:03}"), "label": "Tap behind the old kitchen tent", "group": "Water points"}))
        .collect();
    let body = genesis_from_json(&g.to_string()).unwrap();
    let err = Node::create_site(new_key(), body, 0).err().unwrap();
    assert!(err.starts_with("too long"), "{err}");
}
