mod common;
mod util;
use common::*;
use pukaar_logic::event::{new_key, Body, Id, Role};
use pukaar_logic::node::Node;
use pukaar_logic::reducer::Status;

fn report(n: &mut Node) -> Id {
    n.publish(
        Body::Report {
            category: "water".into(),
            location: "W-03".into(),
            landmark: String::new(),
            text: "tap leaking".into(),
        },
        10,
    )
    .id
}

fn status(n: &Node, issue: Id) -> Status {
    n.state().issues[&issue].status
}

#[test]
fn claim_reopen_claim_confirm() {
    let mut s = Site::new();
    let i = report(&mut s.asha);
    s.sync();
    s.steward.publish(
        Body::Acknowledge {
            issue: i,
            eta_h: 4,
            note: "".into(),
        },
        11,
    );
    s.steward.publish(
        Body::ClaimResolved {
            issue: i,
            note: "washer replaced".into(),
        },
        12,
    );
    s.sync();
    assert_eq!(status(&s.asha, i), Status::AwaitingConfirmation);
    s.asha.publish(
        Body::Reopen {
            issue: i,
            reason: "still dripping".into(),
        },
        13,
    );
    s.sync();
    assert_eq!(status(&s.steward, i), Status::Open);
    assert_eq!(s.steward.state().issues[&i].reopen_count, 1);
    s.steward.publish(
        Body::ClaimResolved {
            issue: i,
            note: "new tap".into(),
        },
        14,
    );
    s.sync();
    s.asha.publish(
        Body::Confirm {
            issue: i,
            note: "works".into(),
        },
        15,
    );
    s.sync();
    for n in s.nodes() {
        assert_eq!(status(n, i), Status::ConfirmedResolved);
    }
}

#[test]
fn forbidden_transitions_are_inert_and_visible() {
    let mut s = Site::new();
    let i = report(&mut s.asha);
    s.sync();
    let e1 = s.ravi.publish(
        Body::Acknowledge {
            issue: i,
            eta_h: 1,
            note: "".into(),
        },
        11,
    ); // resident can't
    let e2 = s.asha.publish(
        Body::Confirm {
            issue: i,
            note: "".into(),
        },
        12,
    ); // nothing claimed yet
    let e3 = s.steward.publish(
        Body::CloseWontfix {
            issue: i,
            reason: " ".into(),
        },
        13,
    ); // no reason
    s.sync();
    let st = s.admin.state();
    assert_eq!(st.issues[&i].status, Status::Open);
    for e in [e1, e2, e3] {
        assert!(st.rejected.contains_key(&e.id), "rejection recorded");
        assert!(st.issues[&i].timeline.contains(&e.id), "shown in timeline");
    }
}

#[test]
fn fixer_cannot_confirm_own_fix() {
    let mut s = Site::new();
    let i = s
        .steward
        .publish(
            Body::Report {
                category: "waste".into(),
                location: "B-07".into(),
                landmark: String::new(),
                text: "bin full, for kitchen".into(),
            },
            10,
        )
        .id;
    s.steward.publish(
        Body::ClaimResolved {
            issue: i,
            note: "emptied".into(),
        },
        11,
    );
    let own = s.steward.publish(
        Body::Confirm {
            issue: i,
            note: "".into(),
        },
        12,
    );
    s.sync();
    assert_eq!(status(&s.admin, i), Status::AwaitingConfirmation);
    assert!(s.admin.state().rejected[&own.id].contains("fixer"));
    // two residents, neither the claimant, can close it
    s.ravi.publish(
        Body::Confirm {
            issue: i,
            note: "".into(),
        },
        13,
    );
    s.sync();
    assert_eq!(status(&s.admin, i), Status::AwaitingConfirmation);
    s.meera.publish(
        Body::Confirm {
            issue: i,
            note: "".into(),
        },
        14,
    );
    s.sync();
    assert_eq!(status(&s.admin, i), Status::ConfirmedResolved);
}

#[test]
fn two_residents_reopen_a_wontfix() {
    let mut s = Site::new();
    let i = report(&mut s.asha);
    s.sync();
    s.steward.publish(
        Body::CloseWontfix {
            issue: i,
            reason: "not ours".into(),
        },
        11,
    );
    s.sync();
    s.ravi.publish(
        Body::Reopen {
            issue: i,
            reason: "it is".into(),
        },
        12,
    );
    s.sync();
    assert_eq!(status(&s.admin, i), Status::ClosedWontfix);
    s.meera.publish(
        Body::Reopen {
            issue: i,
            reason: "agreed".into(),
        },
        13,
    );
    s.sync();
    assert_eq!(status(&s.admin, i), Status::Open);
}

#[test]
fn membership_and_caps() {
    let mut s = Site::new();
    let mut stranger = Node::join(new_key(), s.admin.store.site);
    stranger.publish(
        Body::Profile {
            display_name: Some("Kiran".into()),
        },
        1,
    );
    let r = report(&mut stranger);
    for e in stranger.store.events.values() {
        s.admin.receive(&e.bytes).unwrap();
    }
    let st = s.admin.state();
    assert!(st.pending_members.contains(&stranger.me()));
    assert!(st.rejected[&r].contains("not a member"));

    let ids: Vec<Id> = (0..4).map(|_| report(&mut s.asha)).collect();
    let st = s.asha.state();
    assert!(
        st.rejected[&ids[3]].contains("cap"),
        "max_open_per_author = 3"
    );
}

#[test]
fn anchored_checkpoint_shows_in_timeline() {
    let mut s = Site::new();
    let i = report(&mut s.asha);
    s.sync();
    let cp: serde_json::Value = serde_json::from_str(&s.ravi.checkpoint_json()).unwrap();
    assert!(cp["spel"]
        .as_str()
        .unwrap()
        .starts_with("spel anchor --site-id "));
    assert!(
        s.ravi
            .record_anchor(&cp["heads"].to_string(), " ", 30)
            .is_err(),
        "needs a LEZ reference"
    );
    s.ravi
        .record_anchor(&cp["heads"].to_string(), "pda:Public/abc", 30)
        .unwrap();
    s.sync();
    let tl: serde_json::Value =
        serde_json::from_str(&s.steward.timeline_json(&hex::encode(i))).unwrap();
    assert_eq!(tl["events"][0]["anchored_tx"], "pda:Public/abc");
    let later = s.steward.publish(
        Body::Comment {
            issue: i,
            text: "after the anchor".into(),
        },
        40,
    );
    let tl: serde_json::Value =
        serde_json::from_str(&s.steward.timeline_json(&hex::encode(i))).unwrap();
    assert!(tl["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["id"] == hex::encode(later.id) && e["anchored_tx"].is_null()));
}

#[test]
fn site_info_lists_members_and_map() {
    let s = Site::new();
    let info: serde_json::Value = serde_json::from_str(&s.admin.site_info_json()).unwrap();
    assert_eq!(info["members"].as_array().unwrap().len(), 5);
    assert_eq!(info["locations"][0]["code"], "W-03");
    let me: serde_json::Value = serde_json::from_str(&s.asha.identity_json()).unwrap();
    assert_eq!(me["role"], "Resident");
    assert_eq!(me["fingerprint"].as_str().unwrap().len(), 6);
}

#[test]
fn updates_track_what_happens_next() {
    let mut s = Site::new();
    let i = report(&mut s.asha);
    s.sync();
    s.steward.publish(
        Body::Acknowledge {
            issue: i,
            eta_h: 4,
            note: "seen".into(),
        },
        11,
    );
    let blank = s.steward.publish(
        Body::Update {
            issue: i,
            note: " ".into(),
            next_step: "".into(),
            eta_h: 0,
        },
        12,
    );
    s.steward.publish(
        Body::Update {
            issue: i,
            note: "washer ordered from Phagi".into(),
            next_step: "fit washer".into(),
            eta_h: 24,
        },
        13,
    );
    s.sync();
    let st = s.asha.state();
    assert!(st.rejected[&blank.id].contains("needs a note"));
    let iss = &st.issues[&i];
    assert_eq!(iss.status, Status::InProgress);
    let p = iss.progress.as_ref().unwrap();
    assert_eq!(
        (p.next_step.as_str(), p.eta_h, p.by),
        ("fit washer", 24, s.steward.me())
    );
    let list: serde_json::Value = serde_json::from_str(&s.asha.issues_json()).unwrap();
    assert_eq!(list[0]["stage"], "In progress");
    assert_eq!(list[0]["progress"]["by_name"], "Facilities");
    assert_eq!(list[0]["progress"]["due_ts"], 13 + 24 * 3600);
    // a fix claim must say what was done
    let silent = s.steward.publish(
        Body::ClaimResolved {
            issue: i,
            note: "".into(),
        },
        14,
    );
    s.sync();
    assert!(s.asha.state().rejected[&silent.id].contains("what was done"));
}

#[test]
fn staff_are_named_and_residents_choose() {
    let mut s = Site::new();
    let mut newbie = Node::join(new_key(), s.admin.store.site);
    newbie.publish(Body::Profile { display_name: None }, 1);
    for e in newbie.store.events.values() {
        s.admin.receive(&e.bytes).unwrap();
    }
    let nameless = s.admin.publish(
        Body::RoleGrant {
            subject: newbie.me(),
            role: Role::Steward,
            name: None,
        },
        2,
    );
    let outed = s.admin.publish(
        Body::RoleGrant {
            subject: newbie.me(),
            role: Role::Resident,
            name: Some("Kiran".into()),
        },
        3,
    );
    let st = s.admin.state();
    assert!(st.rejected[&nameless.id].contains("staff must be named"));
    assert!(st.rejected[&outed.id].contains("residents choose"));
    // a steward can't rename (or un-name) themselves
    s.sync();
    let hide = s.steward.publish(Body::Profile { display_name: None }, 4);
    s.sync();
    let st = s.admin.state();
    assert!(st.rejected[&hide.id].contains("set by the admin"));
    assert_eq!(st.names[&s.steward.me()], "Facilities");
    assert_eq!(st.names[&s.admin.me()], "Site admin");
    assert!(
        !st.names.contains_key(&s.ravi.me()),
        "residents stay pseudonymous by default"
    );
}

#[test]
fn unlisted_place_needs_a_landmark() {
    let mut s = Site::new();
    let bare = s.asha.publish(
        Body::Report {
            category: "water".into(),
            location: "other".into(),
            landmark: " ".into(),
            text: "leak".into(),
        },
        1,
    );
    let ok = s.asha.publish(
        Body::Report {
            category: "water".into(),
            location: "other".into(),
            landmark: "pipe behind tent 4".into(),
            text: "leak".into(),
        },
        2,
    );
    let st = s.asha.state();
    assert!(st.rejected[&bare.id].contains("landmark"));
    assert_eq!(st.issues[&ok.id].landmark, "pipe behind tent 4");
    let info: serde_json::Value = serde_json::from_str(&s.asha.site_info_json()).unwrap();
    assert_eq!(info["locations"][0]["group"], "Water points");
}

#[test]
fn a_pseudonymous_joiner_still_shows_up_for_approval() {
    let mut s = Site::new();
    let joiner = Node::join_announced(new_key(), s.admin.store.site, 5);
    for e in joiner.store.events.values() {
        s.admin.receive(&e.bytes).unwrap();
    }
    let st = s.admin.state();
    assert!(st.pending_members.contains(&joiner.me()));
    assert!(!st.names.contains_key(&joiner.me()), "no name was set");
}

#[test]
fn finished_cards_carry_no_pending_next_step() {
    let mut s = Site::new();
    let (done, dup, other) = (
        report(&mut s.asha),
        report(&mut s.asha),
        report(&mut s.asha),
    );
    s.sync();
    s.steward.publish(
        Body::ClaimResolved {
            issue: done,
            note: "washer".into(),
        },
        11,
    );
    s.sync();
    s.asha.publish(
        Body::Confirm {
            issue: done,
            note: "works".into(),
        },
        12,
    );
    s.steward.publish(
        Body::MarkDuplicate {
            issue: dup,
            of: other,
        },
        13,
    );
    s.sync();
    let list: serde_json::Value = serde_json::from_str(&s.asha.issues_json()).unwrap();
    for (id, who) in [(done, "works"), (dup, "duplicate")] {
        let p = list
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["id"] == hex::encode(id))
            .unwrap()["progress"]
            .clone();
        assert_eq!(p["next_step"], "", "no pending step: {p}");
        assert!(p["note"].as_str().unwrap().contains(who), "{p}");
    }
}
