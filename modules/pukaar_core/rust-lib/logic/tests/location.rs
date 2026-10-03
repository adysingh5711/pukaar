//! Editing and removing locations. Codes are identities: an edit changes the name and group,
//! never the code. Only a never-used place can be removed, after a 30-day undo window, and
//! "removed" means hidden: the signed events stay in everyone's log.
mod common;
mod util;
use common::*;
use pukaar_logic::event::{Body, Event, Location};
use pukaar_logic::node::Node;
use pukaar_logic::reducer::REMOVAL_COOLDOWN;
use serde_json::Value;

const T: u64 = 1_000;

fn report_event(n: &mut Node, location: &str) -> Event {
    n.publish(
        Body::Report {
            category: "water".into(),
            location: location.into(),
            landmark: String::new(),
            text: "tap leaking".into(),
        },
        10,
    )
    .unwrap()
}

fn info(n: &Node, now: u64) -> Value {
    serde_json::from_str(&n.site_info_json(now)).unwrap()
}

fn location(n: &Node, now: u64, code: &str) -> Option<Value> {
    info(n, now)["locations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["code"] == code)
        .cloned()
}

fn refused_quietly(n: &mut Node, f: impl FnOnce(&mut Node) -> Result<Event, String>) -> String {
    let events = n.store.events.len();
    let err = f(n).unwrap_err();
    assert_eq!(n.store.events.len(), events, "nothing was published");
    err
}

// ---------- add ----------

#[test]
fn a_new_location_is_added_and_listed() {
    let mut s = Site::new();
    s.admin
        .add_location(" W-04 ", " Tap east ", " Taps ", T)
        .unwrap();
    let l = location(&s.admin, T, "W-04").unwrap();
    assert_eq!(
        (l["label"].as_str(), l["group"].as_str()),
        (Some("Tap east"), Some("Taps"))
    );
}

#[test]
fn a_code_that_was_ever_used_cannot_be_added_again() {
    let mut s = Site::new();
    s.admin.add_location("W-04", "Tap east", "Taps", T).unwrap();
    s.admin.retire_location("W-03", "dry", T).unwrap();
    s.admin.remove_location("B-07", "typo", T).unwrap();
    s.admin.add_location("W-05", "Tap", "Taps", T).unwrap();
    s.admin.remove_location("W-05", "typo", T).unwrap();
    // removed for good (past the undo window): the code is still taken
    let late = T + REMOVAL_COOLDOWN;
    for code in ["W-04", "W-03", "B-07", "W-05"] {
        let e = refused_quietly(&mut s.admin, |n| n.add_location(code, "Tap", "Taps", late));
        assert_eq!(e, format!("{code} is already used"));
    }
}

#[test]
fn an_added_location_needs_a_real_code_name_and_group() {
    let mut s = Site::new();
    let mut add = |code: &str, label: &str, group: &str| {
        refused_quietly(&mut s.admin, |n| n.add_location(code, label, group, T))
    };
    assert_eq!(add("other", "Tap", "Taps"), "other is already used");
    assert_eq!(
        add(" ", "Tap", "Taps"),
        "a location needs a code, a name and a group"
    );
    assert_eq!(
        add("W-09", " ", "Taps"),
        "a location needs a code, a name and a group"
    );
    assert_eq!(
        add("W-09", "Tap", ""),
        "a location needs a code, a name and a group"
    );
    assert!(add("W-09", &"x".repeat(501), "Taps").starts_with("too long"));
}

#[test]
fn only_the_admin_adds_a_location() {
    let mut s = Site::new();
    let e = refused_quietly(&mut s.steward, |n| n.add_location("W-09", "Tap", "Taps", T));
    assert_eq!(e, "only admin edits the site map");
}

fn loc(code: &str) -> Location {
    Location {
        code: code.into(),
        label: "Tap".into(),
        group: "Taps".into(),
    }
}

#[test]
fn a_violating_add_event_is_rejected_whole_and_visibly() {
    let mut s = Site::new();
    let before = s.admin.state().locations.len();
    for (locations, why) in [
        (vec![loc("W-09"), loc("W-03")], "W-03 is already used"),
        (vec![loc("W-09"), loc("W-09")], "W-09 is already used"),
        (vec![loc("W-09"), loc("other")], "other is already used"),
        (
            vec![Location {
                label: String::new(),
                ..loc("W-09")
            }],
            "a location needs a code, a name and a group",
        ),
    ] {
        let e = s
            .admin
            .publish(Body::LocationsAdd { locations }, T)
            .unwrap();
        let st = s.admin.state();
        assert_eq!(st.rejected[&e.id], why);
        assert_eq!(
            st.locations.len(),
            before,
            "nothing was added, not even the valid code"
        );
    }
}

// ---------- edit ----------

#[test]
fn an_admin_renames_and_regroups_a_location() {
    let mut s = Site::new();
    s.admin
        .edit_location("W-03", "Tap, north side of the dining hall", "Taps", T)
        .unwrap();
    let l = location(&s.admin, T, "W-03").unwrap();
    assert_eq!(l["label"], "Tap, north side of the dining hall");
    assert_eq!(l["group"], "Taps");
    assert_eq!(l["renamed_from"], "Tap by dining hall");
}

#[test]
fn old_issues_show_the_current_label_and_the_old_one() {
    let mut s = Site::new();
    report_event(&mut s.asha, "W-03");
    s.sync();
    s.admin
        .edit_location("W-03", "Tap, north side", "Water points", T)
        .unwrap();
    let issues: Value = serde_json::from_str(&s.admin.issues_json(T)).unwrap();
    assert_eq!(issues[0]["location_label"], "Tap, north side");
    assert_eq!(issues[0]["location_renamed_from"], "Tap by dining hall");
}

#[test]
fn only_the_admin_edits_a_location() {
    let mut s = Site::new();
    let err = refused_quietly(&mut s.steward, |n| {
        n.edit_location("W-03", "Mine now", "Water points", T)
    });
    assert_eq!(err, "only admin edits locations");
    // a forged edit that skips the pre-check is kept but inert
    let e = s
        .steward
        .publish(
            Body::LocationEdit {
                code: "W-03".into(),
                label: "Mine now".into(),
                group: "Water points".into(),
            },
            T,
        )
        .unwrap();
    let st = s.steward.state();
    assert_eq!(st.rejected[&e.id], "only admin edits locations");
    assert_eq!(st.locations[0].label, "Tap by dining hall");
}

#[test]
fn an_edit_needs_a_known_code_a_name_and_a_group() {
    let mut s = Site::new();
    let mut edit = |code: &str, label: &str, group: &str| {
        refused_quietly(&mut s.admin, |n| n.edit_location(code, label, group, T))
    };
    assert_eq!(edit("X-99", "Tap", "Taps"), "unknown location X-99");
    assert_eq!(edit("other", "Tap", "Taps"), "unknown location other");
    assert_eq!(
        edit("W-03", " ", "Taps"),
        "a location needs a name and a group"
    );
    assert_eq!(
        edit("W-03", "Tap", ""),
        "a location needs a name and a group"
    );
    assert!(edit("W-03", &"x".repeat(501), "Taps").starts_with("too long"));
}

#[test]
fn a_location_pending_removal_cannot_be_edited() {
    let mut s = Site::new();
    s.admin.remove_location("B-07", "typo", T).unwrap();
    let err = refused_quietly(&mut s.admin, |n| {
        n.edit_location("B-07", "Bin", "Bins", T + 1)
    });
    assert_eq!(err, "location B-07 is pending removal");
}

// ---------- remove ----------

#[test]
fn a_never_used_location_enters_pending_removal() {
    let mut s = Site::new();
    s.admin
        .remove_location("B-07", "added by mistake", T)
        .unwrap();
    let l = location(&s.admin, T, "B-07").unwrap();
    assert_eq!(l["state"], "pending_removal");
    assert_eq!(l["removes_at"], T + REMOVAL_COOLDOWN);
    assert_eq!(l["ever_used"], false);
    assert_eq!(location(&s.admin, T, "W-03").unwrap()["state"], "active");
}

#[test]
fn a_location_with_any_report_cannot_be_removed_even_when_resolved() {
    let mut s = Site::new();
    let r = report_event(&mut s.asha, "W-03");
    s.sync();
    s.steward
        .publish(
            Body::CloseWontfix {
                issue: r.id,
                reason: "tap removed".into(),
            },
            20,
        )
        .unwrap();
    s.sync();
    assert_eq!(location(&s.admin, T, "W-03").unwrap()["ever_used"], true);
    let err = refused_quietly(&mut s.admin, |n| n.remove_location("W-03", "gone", T));
    assert_eq!(err, "W-03 has had reports: retire it instead");
}

#[test]
fn only_the_admin_removes_and_always_with_a_reason() {
    let mut s = Site::new();
    let err = refused_quietly(&mut s.steward, |n| n.remove_location("B-07", "x", T));
    assert_eq!(err, "only admin removes locations");
    let err = refused_quietly(&mut s.admin, |n| n.remove_location("B-07", "  ", T));
    assert_eq!(err, "a remove or undo needs a reason");
    let e = s
        .steward
        .publish(
            Body::LocationRemove {
                code: "B-07".into(),
                undo: false,
                reason: "x".into(),
            },
            T,
        )
        .unwrap();
    let st = s.steward.state();
    assert_eq!(st.rejected[&e.id], "only admin removes locations");
    assert!(st.pending_removal.is_empty());
}

#[test]
fn a_report_on_a_location_pending_removal_is_rejected_and_kept() {
    let mut s = Site::new();
    s.admin.remove_location("B-07", "typo", T).unwrap();
    s.sync();
    let e = report_event(&mut s.asha, "B-07");
    let st = s.asha.state();
    assert_eq!(st.rejected[&e.id], "location B-07 is pending removal");
    assert!(st.issues.is_empty());
}

#[test]
fn undo_during_the_window_makes_it_active_again() {
    let mut s = Site::new();
    s.admin.remove_location("B-07", "typo", T).unwrap();
    s.admin
        .undo_remove_location("B-07", "still needed", T + REMOVAL_COOLDOWN - 1)
        .unwrap();
    assert_eq!(location(&s.admin, T, "B-07").unwrap()["state"], "active");
    s.sync();
    report_event(&mut s.asha, "B-07");
    assert_eq!(s.asha.state().issues.len(), 1);
}

#[test]
fn undo_after_thirty_days_is_refused() {
    let mut s = Site::new();
    s.admin.remove_location("B-07", "typo", T).unwrap();
    let late = T + REMOVAL_COOLDOWN;
    let err = refused_quietly(&mut s.admin, |n| {
        n.undo_remove_location("B-07", "oops", late)
    });
    assert_eq!(err, "B-07 is removed: the 30-day undo window has ended");
    // the reducer applies the same rule by event ts, so a forged late undo is inert everywhere
    let e = s
        .admin
        .publish(
            Body::LocationRemove {
                code: "B-07".into(),
                undo: true,
                reason: "oops".into(),
            },
            late,
        )
        .unwrap();
    let st = s.admin.state();
    assert_eq!(
        st.rejected[&e.id],
        "B-07 is removed: the 30-day undo window has ended"
    );
    assert!(st.pending_removal.contains_key("B-07"));
}

#[test]
fn nonsense_removes_and_undos_are_refused() {
    let mut s = Site::new();
    let err = refused_quietly(&mut s.admin, |n| n.undo_remove_location("B-07", "x", T));
    assert_eq!(err, "B-07 is not pending removal");
    let err = refused_quietly(&mut s.admin, |n| n.remove_location("X-99", "x", T));
    assert_eq!(err, "unknown location X-99");
    s.admin.remove_location("B-07", "typo", T).unwrap();
    let err = refused_quietly(&mut s.admin, |n| n.remove_location("B-07", "again", T));
    assert_eq!(err, "location B-07 is pending removal");
    let err = refused_quietly(&mut s.admin, |n| n.retire_location("B-07", "x", T));
    assert_eq!(err, "location B-07 is pending removal");
    let err = refused_quietly(&mut s.admin, |n| n.restore_location("B-07", "x", T));
    assert_eq!(err, "B-07 is not retired");
}

#[test]
fn removed_is_derived_from_now_at_the_thirty_day_boundary() {
    let mut s = Site::new();
    s.admin
        .remove_location("B-07", "added by mistake", T)
        .unwrap();
    let edge = T + REMOVAL_COOLDOWN;
    assert_eq!(
        location(&s.admin, edge - 1, "B-07").unwrap()["state"],
        "pending_removal"
    );
    assert!(
        location(&s.admin, edge, "B-07").is_none(),
        "hidden from every list"
    );
    let removed = info(&s.admin, edge)["removed_locations"].clone();
    assert_eq!(removed.as_array().unwrap().len(), 1);
    let r = &removed[0];
    assert_eq!(r["code"], "B-07");
    assert_eq!(r["label"], "Bin at path junction");
    assert_eq!(r["reason"], "added by mistake");
    assert_eq!(r["by_name"], "Site admin");
    assert_eq!(r["since"], T);
    assert_eq!(r["removed_at"], edge);
    assert_eq!(
        info(&s.admin, edge - 1)["removed_locations"],
        Value::Array(vec![])
    );
}

#[test]
fn a_retired_never_used_location_can_be_removed_and_undo_makes_it_active() {
    let mut s = Site::new();
    s.admin
        .retire_location("B-07", "bin taken away", T)
        .unwrap();
    assert_eq!(location(&s.admin, T, "B-07").unwrap()["state"], "retired");
    s.admin
        .remove_location("B-07", "never used", T + 1)
        .unwrap();
    let l = location(&s.admin, T + 1, "B-07").unwrap();
    assert_eq!(l["state"], "pending_removal");
    assert_eq!(
        l["retired"], false,
        "never both retired and pending removal"
    );
    s.admin
        .undo_remove_location("B-07", "keep it", T + 2)
        .unwrap();
    assert_eq!(
        location(&s.admin, T + 2, "B-07").unwrap()["state"],
        "active"
    );
}

#[test]
fn a_report_racing_a_remove_converges_on_every_replica() {
    let mut s = Site::new();
    // neither has seen the other: the lamport order decides which one the rules accept
    let report = report_event(&mut s.asha, "B-07");
    let remove = s
        .admin
        .publish(
            Body::LocationRemove {
                code: "B-07".into(),
                undo: false,
                reason: "typo".into(),
            },
            10,
        )
        .unwrap();
    s.sync();
    let states: Vec<_> = s.nodes().into_iter().map(|n| n.state()).collect();
    assert!(states.windows(2).all(|w| w[0] == w[1]), "replicas diverged");
    let st = &states[0];
    let report_ok = st.issues.contains_key(&report.id);
    let remove_ok = st.pending_removal.contains_key("B-07");
    assert!(
        report_ok != remove_ok,
        "report_ok={report_ok} remove_ok={remove_ok}"
    );
    assert!(st
        .rejected
        .contains_key(if report_ok { &remove.id } else { &report.id }));
}
