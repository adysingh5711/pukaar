//! Retiring a location: never a delete, only an admin, only once nothing there is still open.
mod common;
mod util;
use common::*;
use pukaar_logic::event::{Body, Event, Id};
use pukaar_logic::node::Node;
use pukaar_logic::reducer::Status;

fn report_at(n: &mut Node, location: &str) -> Id {
    report_event(n, location).id
}

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

fn act(n: &mut Node, issue: Id, body: fn(Id) -> Body) {
    n.publish(body(issue), 20).unwrap();
}

fn claim(issue: Id) -> Body {
    Body::ClaimResolved {
        issue,
        note: "washer replaced".into(),
    }
}

fn confirm(issue: Id) -> Body {
    Body::Confirm {
        issue,
        note: "dry now".into(),
    }
}

fn wontfix(issue: Id) -> Body {
    Body::CloseWontfix {
        issue,
        reason: "tap removed".into(),
    }
}

fn retire_body(code: &str, retired: bool) -> Body {
    Body::LocationRetire {
        code: code.into(),
        retired,
        reason: "no longer used".into(),
    }
}

fn is_retired(n: &Node, code: &str) -> bool {
    n.state().retired.contains_key(code)
}

fn location_json(n: &Node, code: &str) -> serde_json::Value {
    let info: serde_json::Value = serde_json::from_str(&n.site_info_json()).unwrap();
    info["locations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["code"] == code)
        .unwrap()
        .clone()
}

#[test]
fn an_admin_retires_a_location_with_no_open_issues() {
    let mut s = Site::new();
    s.admin.retire_location("B-07", "bin removed", 30).unwrap();
    assert!(is_retired(&s.admin, "B-07"));
    assert!(!is_retired(&s.admin, "W-03"));
    let l = location_json(&s.admin, "B-07");
    assert_eq!(l["retired"], true);
    assert_eq!(l["retired_reason"], "bin removed");
    assert_eq!(l["open_issues"], 0);
    assert_eq!(location_json(&s.admin, "W-03")["retired"], false);
}

#[test]
fn open_issues_block_a_retire_until_they_are_done() {
    let mut s = Site::new();
    let i = report_at(&mut s.asha, "W-03");
    report_at(&mut s.ravi, "W-03");
    s.sync();
    let events = s.admin.store.events.len();
    let err = s.admin.retire_location("W-03", "closed", 30).unwrap_err();
    assert_eq!(err, "W-03 has 2 open issues");
    assert_eq!(s.admin.store.events.len(), events, "nothing was published");
    assert_eq!(location_json(&s.admin, "W-03")["open_issues"], 2);

    // a claimed fix still waits for confirmation, so it still blocks
    act(&mut s.steward, i, claim);
    s.sync();
    assert!(s.admin.retire_location("W-03", "closed", 30).is_err());
    // resolved and won't-fix are terminal, so they stop blocking
    act(&mut s.asha, i, confirm);
    let j = *s
        .steward
        .state()
        .issues
        .values()
        .find(|x| x.status == Status::Open)
        .map(|x| &x.id)
        .unwrap();
    act(&mut s.steward, j, wontfix);
    s.sync();
    assert_eq!(location_json(&s.admin, "W-03")["open_issues"], 0);
    s.admin.retire_location("W-03", "closed", 40).unwrap();
    assert!(is_retired(&s.admin, "W-03"));
}

#[test]
fn only_the_admin_retires() {
    let mut s = Site::new();
    for n in [&mut s.steward, &mut s.asha] {
        let events = n.store.events.len();
        assert_eq!(
            n.retire_location("B-07", "x", 30).unwrap_err(),
            "only admin retires locations"
        );
        assert_eq!(n.store.events.len(), events);
    }
    // a forged event that skips the pre-check is kept but inert
    let e = s.steward.publish(retire_body("B-07", true), 30).unwrap();
    let st = s.steward.state();
    assert_eq!(st.rejected[&e.id], "only admin retires locations");
    assert!(st.retired.is_empty());
}

#[test]
fn a_report_on_a_retired_location_is_rejected_and_kept() {
    let mut s = Site::new();
    s.admin.retire_location("B-07", "bin removed", 30).unwrap();
    s.sync();
    let e = report_event(&mut s.asha, "B-07");
    let st = s.asha.state();
    assert_eq!(st.rejected[&e.id], "location B-07 is retired");
    assert!(st.issues.is_empty());
    // other places stay reportable
    report_at(&mut s.asha, "W-03");
    assert_eq!(s.asha.state().issues.len(), 1);
}

#[test]
fn restoring_makes_the_location_reportable_again() {
    let mut s = Site::new();
    s.admin.retire_location("B-07", "bin removed", 30).unwrap();
    s.admin.restore_location("B-07", "bin is back", 31).unwrap();
    assert!(!is_retired(&s.admin, "B-07"));
    s.sync();
    report_at(&mut s.asha, "B-07");
    assert_eq!(s.asha.state().issues.len(), 1);
}

#[test]
fn nonsense_changes_are_refused_without_publishing() {
    let mut s = Site::new();
    s.admin.retire_location("B-07", "bin removed", 30).unwrap();
    let events = s.admin.store.events.len();
    let refused = |n: &mut Node, retire: bool, code: &str, reason: &str| {
        if retire {
            n.retire_location(code, reason, 40)
        } else {
            n.restore_location(code, reason, 40)
        }
        .unwrap_err()
    };
    assert_eq!(
        refused(&mut s.admin, true, "B-07", "again"),
        "B-07 is already retired"
    );
    assert_eq!(
        refused(&mut s.admin, false, "W-03", "oops"),
        "W-03 is not retired"
    );
    assert_eq!(
        refused(&mut s.admin, true, "X-99", "x"),
        "unknown location X-99"
    );
    assert_eq!(
        refused(&mut s.admin, true, "other", "x"),
        "unknown location other"
    );
    assert_eq!(
        refused(&mut s.admin, true, "W-03", "  "),
        "a retire or restore needs a reason"
    );
    assert_eq!(s.admin.store.events.len(), events);
}

#[test]
fn a_report_racing_a_retire_converges_on_every_replica() {
    let mut s = Site::new();
    // neither has seen the other: both extend the same history
    let report = report_event(&mut s.asha, "B-07");
    let retire = s.admin.publish(retire_body("B-07", true), 10).unwrap();
    s.sync();
    let states: Vec<_> = s.nodes().into_iter().map(|n| n.state()).collect();
    assert!(states.windows(2).all(|w| w[0] == w[1]), "replicas diverged");
    let st = &states[0];
    // exactly one wins: the pair can never leave an open issue on a retired location
    let report_ok = st.issues.contains_key(&report.id);
    let retire_ok = st.retired.contains_key("B-07");
    assert!(
        report_ok != retire_ok,
        "report_ok={report_ok} retire_ok={retire_ok}"
    );
    assert!(st
        .rejected
        .contains_key(if report_ok { &retire.id } else { &report.id }));
}

#[test]
fn a_reopen_cannot_bring_an_issue_back_onto_a_retired_location() {
    let mut s = Site::new();
    let i = report_at(&mut s.asha, "B-07");
    s.sync();
    act(&mut s.steward, i, wontfix);
    s.sync();
    s.admin.retire_location("B-07", "bin removed", 30).unwrap();
    s.sync();
    let e = s
        .asha
        .publish(
            Body::Reopen {
                issue: i,
                reason: "still smells".into(),
            },
            50,
        )
        .unwrap();
    let st = s.asha.state();
    assert_eq!(st.rejected[&e.id], "location B-07 is retired");
    assert_eq!(st.issues[&i].status, Status::ClosedWontfix);
}

#[test]
fn old_issues_stay_readable_at_a_retired_location() {
    let mut s = Site::new();
    let i = report_at(&mut s.asha, "B-07");
    s.sync();
    act(&mut s.steward, i, wontfix);
    s.sync();
    s.admin.retire_location("B-07", "bin removed", 30).unwrap();
    let issues: serde_json::Value = serde_json::from_str(&s.admin.issues_json(100)).unwrap();
    assert_eq!(issues[0]["location_label"], "Bin at path junction");
    assert_eq!(issues[0]["location_retired"], true);
}
