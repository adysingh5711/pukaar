mod common;
mod util;
use common::*;
use pukaar_logic::event::{new_key, Body, Id, Location, Role, MAX_TEXT};
use pukaar_logic::node::Node;
use pukaar_logic::reducer::{Status, REMOVAL_COOLDOWN};

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
    .unwrap()
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
    s.steward
        .publish(
            Body::Acknowledge {
                issue: i,
                eta_h: 4,
                note: "".into(),
            },
            11,
        )
        .unwrap();
    s.steward
        .publish(
            Body::ClaimResolved {
                issue: i,
                note: "washer replaced".into(),
            },
            12,
        )
        .unwrap();
    s.sync();
    assert_eq!(status(&s.asha, i), Status::AwaitingConfirmation);
    s.asha
        .publish(
            Body::Reopen {
                issue: i,
                reason: "still dripping".into(),
            },
            13,
        )
        .unwrap();
    s.sync();
    assert_eq!(status(&s.steward, i), Status::Open);
    assert_eq!(s.steward.state().issues[&i].reopen_count, 1);
    s.steward
        .publish(
            Body::ClaimResolved {
                issue: i,
                note: "new tap".into(),
            },
            14,
        )
        .unwrap();
    s.sync();
    s.asha
        .publish(
            Body::Confirm {
                issue: i,
                note: "works".into(),
            },
            15,
        )
        .unwrap();
    s.sync();
    for n in s.nodes() {
        assert_eq!(status(n, i), Status::ConfirmedResolved);
        // History's SLA scars: the first acknowledge and the first fix claim, the same on every
        // replica; the terminal event's time stays in progress.ts
        let v = &issues(n, 20)[0];
        assert_eq!(
            (&v["acked_ts"], &v["claimed_ts"], &v["progress"]["ts"]),
            (&11.into(), &12.into(), &15.into())
        );
    }
}

#[test]
fn an_unanswered_report_has_no_ack_or_claim_time() {
    let mut s = Site::new();
    report(&mut s.asha);
    let v = &issues(&s.asha, 20)[0];
    assert!(v["acked_ts"].is_null() && v["claimed_ts"].is_null());
}

#[test]
fn forbidden_transitions_are_inert_and_visible() {
    let mut s = Site::new();
    let i = report(&mut s.asha);
    s.sync();
    let e1 = s
        .ravi
        .publish(
            Body::Acknowledge {
                issue: i,
                eta_h: 1,
                note: "".into(),
            },
            11,
        )
        .unwrap(); // resident can't
    let e2 = s
        .asha
        .publish(
            Body::Confirm {
                issue: i,
                note: "".into(),
            },
            12,
        )
        .unwrap(); // nothing claimed yet
    let e3 = s
        .steward
        .publish(
            Body::CloseWontfix {
                issue: i,
                reason: " ".into(),
            },
            13,
        )
        .unwrap(); // no reason
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
        .unwrap()
        .id;
    s.steward
        .publish(
            Body::ClaimResolved {
                issue: i,
                note: "emptied".into(),
            },
            11,
        )
        .unwrap();
    let own = s
        .steward
        .publish(
            Body::Confirm {
                issue: i,
                note: "".into(),
            },
            12,
        )
        .unwrap();
    s.sync();
    assert_eq!(status(&s.admin, i), Status::AwaitingConfirmation);
    assert!(s.admin.state().rejected[&own.id].contains("fixer"));
    // two residents, neither the claimant, can close it
    s.ravi
        .publish(
            Body::Confirm {
                issue: i,
                note: "".into(),
            },
            13,
        )
        .unwrap();
    s.sync();
    assert_eq!(status(&s.admin, i), Status::AwaitingConfirmation);
    s.meera
        .publish(
            Body::Confirm {
                issue: i,
                note: "".into(),
            },
            14,
        )
        .unwrap();
    s.sync();
    assert_eq!(status(&s.admin, i), Status::ConfirmedResolved);
}

#[test]
fn two_residents_reopen_a_wontfix() {
    let mut s = Site::new();
    let i = report(&mut s.asha);
    s.sync();
    s.steward
        .publish(
            Body::CloseWontfix {
                issue: i,
                reason: "not ours".into(),
            },
            11,
        )
        .unwrap();
    s.sync();
    s.ravi
        .publish(
            Body::Reopen {
                issue: i,
                reason: "it is".into(),
            },
            12,
        )
        .unwrap();
    s.sync();
    assert_eq!(status(&s.admin, i), Status::ClosedWontfix);
    s.meera
        .publish(
            Body::Reopen {
                issue: i,
                reason: "agreed".into(),
            },
            13,
        )
        .unwrap();
    s.sync();
    assert_eq!(status(&s.admin, i), Status::Open);
}

#[test]
fn membership_and_caps() {
    let mut s = Site::new();
    let mut stranger = Node::join(new_key(), s.admin.store.site);
    stranger
        .publish(
            Body::Profile {
                display_name: Some("Kiran".into()),
            },
            1,
        )
        .unwrap();
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
        serde_json::from_str(&s.steward.timeline_json(&hex::encode(i), 0)).unwrap();
    assert_eq!(tl["events"][0]["anchored_tx"], "pda:Public/abc");
    // the UI says who recorded it: any member can, only the root is checked
    assert_eq!(tl["events"][0]["anchored_by"], hex::encode(s.ravi.me()));
    assert!(
        tl["events"][0]["anchored_by_name"].is_null(),
        "ravi is a pseudonym"
    );
    let later = s
        .steward
        .publish(
            Body::Comment {
                issue: i,
                text: "after the anchor".into(),
            },
            40,
        )
        .unwrap();
    let tl: serde_json::Value =
        serde_json::from_str(&s.steward.timeline_json(&hex::encode(i), 0)).unwrap();
    assert!(tl["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["id"] == hex::encode(later.id) && e["anchored_tx"].is_null()));
}

#[test]
fn site_info_reports_the_latest_anchor() {
    let mut s = Site::new();
    let info =
        |n: &Node| -> serde_json::Value { serde_json::from_str(&n.site_info_json(0)).unwrap() };
    assert!(
        info(&s.steward)["last_anchor"].is_null(),
        "nothing anchored yet"
    );
    report(&mut s.asha);
    s.sync();
    let cp: serde_json::Value = serde_json::from_str(&s.ravi.checkpoint_json()).unwrap();
    s.ravi
        .record_anchor(&cp["heads"].to_string(), "pda:Public/old", 30)
        .unwrap();
    s.ravi
        .record_anchor(&cp["heads"].to_string(), "pda:Public/abc", 50)
        .unwrap();
    s.sync();
    let last = &info(&s.steward)["last_anchor"];
    assert_eq!(last["tx"], "pda:Public/abc", "newest by ts");
    assert_eq!(last["ts"], 50);
    assert_eq!(last["by"], hex::encode(s.ravi.me()));
    assert!(last["by_name"].is_null(), "ravi is a pseudonym");
    assert_eq!(last["events_covered"], cp["n_events"]);
    assert_eq!(last["reproducible"], true);
    // the two checkpoint events themselves are newer than what they cover
    let total = info(&s.steward)["events"].as_u64().unwrap();
    assert_eq!(total - cp["n_events"].as_u64().unwrap(), 2);

    let ghost = format!("[[\"{}\", 99]]", hex::encode(s.ravi.me()));
    s.ravi
        .record_anchor(&ghost, "pda:Public/ghost", 60)
        .unwrap();
    s.sync();
    let last = &info(&s.steward)["last_anchor"];
    assert_eq!(last["tx"], "pda:Public/ghost");
    assert_eq!(
        last["reproducible"], false,
        "heads name an event nobody has"
    );
}

#[test]
fn site_info_lists_members_and_map() {
    let s = Site::new();
    let info: serde_json::Value = serde_json::from_str(&s.admin.site_info_json(0)).unwrap();
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
    s.steward
        .publish(
            Body::Acknowledge {
                issue: i,
                eta_h: 4,
                note: "seen".into(),
            },
            11,
        )
        .unwrap();
    let blank = s
        .steward
        .publish(
            Body::Update {
                issue: i,
                note: " ".into(),
                next_step: "".into(),
                eta_h: 0,
            },
            12,
        )
        .unwrap();
    s.steward
        .publish(
            Body::Update {
                issue: i,
                note: "washer ordered from Phagi".into(),
                next_step: "fit washer".into(),
                eta_h: 24,
            },
            13,
        )
        .unwrap();
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
    let list: serde_json::Value = serde_json::from_str(&s.asha.issues_json(0)).unwrap();
    assert_eq!(list[0]["stage"], "In progress");
    assert_eq!(list[0]["progress"]["by_name"], "Facilities");
    assert_eq!(list[0]["progress"]["due_ts"], 13 + 24 * 3600);
    // a fix claim must say what was done
    let silent = s
        .steward
        .publish(
            Body::ClaimResolved {
                issue: i,
                note: "".into(),
            },
            14,
        )
        .unwrap();
    s.sync();
    assert!(s.asha.state().rejected[&silent.id].contains("what was done"));
}

#[test]
fn staff_are_named_and_residents_choose() {
    let mut s = Site::new();
    let mut newbie = Node::join(new_key(), s.admin.store.site);
    newbie
        .publish(Body::Profile { display_name: None }, 1)
        .unwrap();
    for e in newbie.store.events.values() {
        s.admin.receive(&e.bytes).unwrap();
    }
    let nameless = s
        .admin
        .publish(
            Body::RoleGrant {
                subject: newbie.me(),
                role: Role::Steward,
                name: None,
            },
            2,
        )
        .unwrap();
    let st = s.admin.state();
    assert!(st.rejected[&nameless.id].contains("staff must be named"));
    // a steward can't rename (or un-name) themselves
    s.sync();
    let hide = s
        .steward
        .publish(Body::Profile { display_name: None }, 4)
        .unwrap();
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
fn a_name_the_admin_gives_a_resident_shows_on_both_sides_until_they_change_it() {
    let mut s = Site::new();
    let mut asha = Node::join_announced(new_key(), s.admin.store.site, 5);
    copy_all(&asha, &mut s.admin);
    let grant = s
        .admin
        .publish(
            Body::RoleGrant {
                subject: asha.me(),
                role: Role::Resident,
                name: Some(" Asha ".into()),
            },
            6,
        )
        .unwrap();
    copy_all(&s.admin, &mut asha);
    assert!(!s.admin.state().rejected.contains_key(&grant.id));
    let info: serde_json::Value = serde_json::from_str(&s.admin.site_info_json(0)).unwrap();
    let me = hex::encode(asha.me());
    let row = info["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == me.as_str());
    assert_eq!(row.unwrap()["name"], "Asha", "the admin's Members page");
    let mine: serde_json::Value = serde_json::from_str(&asha.identity_json()).unwrap();
    assert_eq!(mine["name"], "Asha", "the resident's own sidebar");
    // residents still choose: their own profile replaces or clears it
    asha.publish(Body::Profile { display_name: None }, 7)
        .unwrap();
    assert!(!asha.state().names.contains_key(&asha.me()));
}

#[test]
fn unlisted_place_needs_a_landmark() {
    let mut s = Site::new();
    let bare = s
        .asha
        .publish(
            Body::Report {
                category: "water".into(),
                location: "other".into(),
                landmark: " ".into(),
                text: "leak".into(),
            },
            1,
        )
        .unwrap();
    let ok = s
        .asha
        .publish(
            Body::Report {
                category: "water".into(),
                location: "other".into(),
                landmark: "pipe behind tent 4".into(),
                text: "leak".into(),
            },
            2,
        )
        .unwrap();
    let st = s.asha.state();
    assert!(st.rejected[&bare.id].contains("landmark"));
    assert_eq!(st.issues[&ok.id].landmark, "pipe behind tent 4");
    let info: serde_json::Value = serde_json::from_str(&s.asha.site_info_json(0)).unwrap();
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
    s.steward
        .publish(
            Body::ClaimResolved {
                issue: done,
                note: "washer".into(),
            },
            11,
        )
        .unwrap();
    s.sync();
    s.asha
        .publish(
            Body::Confirm {
                issue: done,
                note: "works".into(),
            },
            12,
        )
        .unwrap();
    s.steward
        .publish(
            Body::MarkDuplicate {
                issue: dup,
                of: other,
            },
            13,
        )
        .unwrap();
    s.sync();
    let list: serde_json::Value = serde_json::from_str(&s.asha.issues_json(0)).unwrap();
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

// ---- our own events obey the same limits every receiver applies ----

fn report_text(n: &mut Node, text: &str) -> Result<pukaar_logic::event::Event, String> {
    n.publish(
        Body::Report {
            category: "water".into(),
            location: "W-03".into(),
            landmark: String::new(),
            text: text.into(),
        },
        10,
    )
}

#[test]
fn an_oversize_hindi_report_is_refused_and_nothing_is_inserted() {
    let mut s = Site::new();
    let hindi = "नल टपक रहा है ".repeat(15); // 210 chars, 540 bytes
    assert!(hindi.chars().count() < MAX_TEXT && hindi.len() > MAX_TEXT);
    let before = s.asha.store.events.len();
    let e = report_text(&mut s.asha, &hindi).unwrap_err();
    assert!(e.starts_with("too long"), "{e}");
    assert_eq!(s.asha.store.events.len(), before);
    assert!(
        report_text(&mut s.asha, &"a".repeat(MAX_TEXT)).is_ok(),
        "the limit itself is fine"
    );
}

#[test]
fn an_oversize_note_is_refused() {
    let mut s = Site::new();
    let i = report(&mut s.asha);
    s.sync();
    let before = s.steward.store.events.len();
    let e = s
        .steward
        .publish(
            Body::Update {
                issue: i,
                note: "ok".into(),
                next_step: "x".repeat(MAX_TEXT + 1),
                eta_h: 1,
            },
            11,
        )
        .unwrap_err();
    assert!(e.starts_with("too long"), "{e}");
    assert_eq!(s.steward.store.events.len(), before);
}

fn many_locations(n: usize) -> Vec<Location> {
    (0..n)
        .map(|k| Location {
            code: format!("W-{k:03}"),
            label: format!("Tap number {k} behind the long row of tents"),
            group: "Water points".into(),
        })
        .collect()
}

#[test]
fn an_event_over_the_size_cap_is_refused() {
    let mut s = Site::new();
    let before = s.admin.store.events.len();
    let e = s
        .admin
        .publish(
            Body::LocationsAdd {
                locations: many_locations(100),
            },
            5,
        )
        .unwrap_err();
    assert!(e.starts_with("too long"), "{e}");
    assert_eq!(s.admin.store.events.len(), before);
}

#[test]
fn an_oversize_genesis_is_refused() {
    let mut g = genesis();
    if let Body::Genesis { locations, .. } = &mut g {
        *locations = many_locations(100);
    }
    let e = Node::create_site(new_key(), g, 0).err().unwrap();
    assert!(e.starts_with("too long"), "{e}");
}

// ---- role revoke ----

#[test]
fn the_admin_revokes_a_steward_with_a_reason() {
    let mut s = Site::new();
    let steward = hex::encode(s.steward.me());
    let before = s.admin.store.events.len();
    assert_eq!(
        s.admin.revoke_role(&steward, "  ", 5).unwrap_err(),
        "a revoke needs a reason"
    );
    assert_eq!(s.admin.store.events.len(), before, "nothing signed");
    let e = s.admin.revoke_role(&steward, "left the site", 5).unwrap();
    assert!(!s.admin.state().rejected.contains_key(&e.id));
    assert!(!s.admin.state().roles.contains_key(&s.steward.me()));
    // a non-admin's revoke is recorded but inert
    let admin = hex::encode(s.admin.me());
    let r = s.asha.revoke_role(&admin, "coup", 6).unwrap();
    assert_eq!(s.asha.state().rejected[&r.id], "only admin revokes roles");
    assert!(
        json(&s.asha.site_info_json(0))["revoked"]
            .as_array()
            .unwrap()
            .is_empty(),
        "an inert revoke records nothing"
    );
}

fn json(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap()
}

#[test]
fn a_revoke_is_kept_with_who_when_and_why_until_a_new_grant() {
    let mut s = Site::new();
    let i = report(&mut s.asha);
    s.asha
        .publish(
            Body::Comment {
                issue: i,
                text: "still leaking".into(),
            },
            11,
        )
        .unwrap();
    s.sync();
    let asha = hex::encode(s.asha.me());
    let admin = hex::encode(s.admin.me());
    s.admin.revoke_role(&asha, "left the camp", 20).unwrap();
    s.sync();

    let info = json(&s.steward.site_info_json(0));
    let r = &info["revoked"][0];
    assert_eq!(info["revoked"].as_array().unwrap().len(), 1);
    assert_eq!(r["key"], asha);
    assert_eq!(r["fingerprint"].as_str().unwrap().len(), 6);
    assert!(r["name"].is_null(), "asha is a pseudonym");
    assert_eq!(r["by"], admin);
    assert_eq!(r["by_name"], "Site admin");
    assert_eq!(r["ts"], 20);
    assert_eq!(r["reason"], "left the camp");

    let me = json(&s.asha.identity_json());
    assert!(me["role"].is_null());
    assert_eq!(me["revoked"]["by"], admin);
    assert_eq!(me["revoked"]["by_name"], "Site admin");
    assert_eq!(me["revoked"]["ts"], 20);
    assert_eq!(me["revoked"]["reason"], "left the camp");
    assert!(json(&s.ravi.identity_json())["revoked"].is_null());

    // their past report and its timeline say "former member"; everyone else's do not
    assert_eq!(issues(&s.steward, 30)[0]["former"], true);
    let tl = json(&s.steward.timeline_json(&hex::encode(i), 30));
    assert_eq!(tl["issue"]["former"], true);
    let events = tl["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|e| e["former"] == true));

    // granted again: no longer removed, no longer former
    s.admin
        .publish(
            Body::RoleGrant {
                subject: s.asha.me(),
                role: Role::Resident,
                name: None,
            },
            25,
        )
        .unwrap();
    s.sync();
    assert!(json(&s.steward.site_info_json(0))["revoked"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(json(&s.asha.identity_json())["revoked"].is_null());
    assert_eq!(issues(&s.steward, 30)[0]["former"], false);
    let tl = json(&s.steward.timeline_json(&hex::encode(i), 30));
    assert!(tl["events"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["former"] == false));
}

#[test]
fn removed_members_are_listed_newest_first_and_the_genesis_admin_stays() {
    let mut s = Site::new();
    let [asha, ravi, admin] = [s.asha.me(), s.ravi.me(), s.admin.me()].map(hex::encode);
    s.admin.revoke_role(&asha, "left", 20).unwrap();
    s.admin.revoke_role(&ravi, "moved", 30).unwrap();
    let e = s.admin.revoke_role(&admin, "oops", 40).unwrap();
    assert_eq!(s.admin.state().rejected[&e.id], "admin cannot be revoked");
    let keys: Vec<_> = json(&s.admin.site_info_json(0))["revoked"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["key"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(keys, [ravi, asha]);
    assert!(json(&s.admin.identity_json())["revoked"].is_null());
}

// ---- board order, SLA flags ----

const H: u64 = 3600;

fn report_at(n: &mut Node, ts: u64) -> Id {
    n.publish(
        Body::Report {
            category: "water".into(),
            location: "W-03".into(),
            landmark: String::new(),
            text: format!("reported at {ts}"),
        },
        ts,
    )
    .unwrap()
    .id
}

fn issues(n: &Node, now: u64) -> Vec<serde_json::Value> {
    serde_json::from_str(&n.issues_json(now)).unwrap()
}

#[test]
fn the_board_lists_oldest_reports_first() {
    let mut s = Site::new();
    report_at(&mut s.asha, 30);
    report_at(&mut s.ravi, 10);
    report_at(&mut s.meera, 20);
    s.sync();
    let ts: Vec<u64> = issues(&s.admin, 40)
        .iter()
        .map(|i| i["reported_ts"].as_u64().unwrap())
        .collect();
    assert_eq!(ts, [10, 20, 30]);
}

#[test]
fn sla_flags_follow_the_site_settings() {
    // the test site: acknowledge within 12 h, fix within 48 h
    let mut s = Site::new();
    let i = report_at(&mut s.asha, 10);
    s.sync();
    let flags = |n: &Node, now: u64| {
        let v = &issues(n, now)[0];
        (
            v["ack_overdue"].clone(),
            v["fix_overdue"].clone(),
            v["awaiting_48h"].clone(),
        )
    };
    let (t, f) = (
        serde_json::Value::Bool(true),
        serde_json::Value::Bool(false),
    );
    assert_eq!(
        flags(&s.asha, 10 + 12 * H),
        (f.clone(), f.clone(), f.clone())
    );
    assert_eq!(
        flags(&s.asha, 11 + 12 * H),
        (t.clone(), f.clone(), f.clone())
    );
    s.steward
        .publish(
            Body::Acknowledge {
                issue: i,
                eta_h: 4,
                note: "seen".into(),
            },
            20,
        )
        .unwrap();
    s.sync();
    assert_eq!(
        flags(&s.asha, 11 + 12 * H),
        (f.clone(), f.clone(), f.clone())
    );
    assert_eq!(
        flags(&s.asha, 11 + 48 * H),
        (f.clone(), t.clone(), f.clone())
    );
    let claim = 100 * H;
    s.steward
        .publish(
            Body::ClaimResolved {
                issue: i,
                note: "new washer".into(),
            },
            claim,
        )
        .unwrap();
    s.sync();
    assert_eq!(
        flags(&s.asha, claim + 48 * H),
        (f.clone(), f.clone(), f.clone())
    );
    assert_eq!(flags(&s.asha, claim + 48 * H + 1), (f.clone(), f, t));
    let info: serde_json::Value = serde_json::from_str(&s.asha.site_info_json(0)).unwrap();
    assert_eq!(
        (info["sla_ack_h"].as_u64(), info["sla_fix_h"].as_u64()),
        (Some(12), Some(48))
    );
}

/// Every applied place change, as (kind, code, ts, reason) newest first.
fn place_log(n: &Node, now: u64) -> Vec<serde_json::Value> {
    json(&n.site_info_json(now))["place_log"]
        .as_array()
        .unwrap()
        .clone()
}

/// P2 added, edited, renamed, retired, restored, removal started and undone; B-07 removal started.
fn place_history(s: &mut Site) {
    let a = &mut s.admin;
    a.add_location("P2", "Lane behind the clinic", "Paths", 100)
        .unwrap();
    a.edit_location("P2", "Lane behind the clinic", "Lanes", 110)
        .unwrap();
    a.edit_location("P2", "Lane by the clinic", "Lanes", 120)
        .unwrap();
    a.retire_location("P2", "closed for construction", 130)
        .unwrap();
    a.restore_location("P2", "open again", 140).unwrap();
    a.remove_location("P2", "added by mistake", 150).unwrap();
    a.undo_remove_location("P2", "still needed", 160).unwrap();
    a.remove_location("B-07", "bin taken away", 170).unwrap();
    s.sync();
    // a forged steward edit skips the pre-check: kept, rejected, and not a change
    let forged = s
        .steward
        .publish(
            Body::LocationEdit {
                code: "P2".into(),
                label: "Mine now".into(),
                group: "Lanes".into(),
            },
            180,
        )
        .unwrap();
    s.sync();
    assert!(s.admin.state().rejected.contains_key(&forged.id));
}

#[test]
fn the_change_log_lists_every_applied_place_change_newest_first() {
    let mut s = Site::new();
    place_history(&mut s);
    let log = place_log(&s.admin, 171);
    let rows: Vec<(&str, &str, u64, &str)> = log
        .iter()
        .map(|e| {
            (
                e["kind"].as_str().unwrap(),
                e["code"].as_str().unwrap(),
                e["ts"].as_u64().unwrap(),
                e["reason"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("removal_started", "B-07", 170, "bin taken away"),
            ("removal_undone", "P2", 160, "still needed"),
            ("removal_started", "P2", 150, "added by mistake"),
            ("restored", "P2", 140, "open again"),
            ("retired", "P2", 130, "closed for construction"),
            ("renamed", "P2", 120, ""),
            ("edited", "P2", 110, ""),
            ("added", "P2", 100, ""),
        ]
    );
    let admin = hex::encode(s.admin.me());
    for e in &log {
        assert_eq!(e["by"], admin.as_str());
        assert_eq!(e["by_name"], "Site admin");
    }
    assert_eq!(log[0]["label"], "Bin at path junction");
    assert_eq!(log[0]["ends_at"], 170 + REMOVAL_COOLDOWN);
    assert_eq!(log[1]["label"], "Lane by the clinic", "the current name");
    assert_eq!(log[5]["from"], "Lane behind the clinic");
    assert_eq!(log[5]["to"], "Lane by the clinic");
    assert!(
        log[6].get("from").is_none(),
        "a group change is not a rename"
    );
}

#[test]
fn a_removal_shows_as_removed_only_once_its_cooldown_is_over() {
    let mut s = Site::new();
    place_history(&mut s);
    let end = 170 + REMOVAL_COOLDOWN;
    assert!(place_log(&s.admin, end - 1)
        .iter()
        .all(|e| e["kind"] != "removed"));
    let log = place_log(&s.admin, end);
    assert_eq!(log.len(), 9);
    let r = &log[0];
    assert_eq!(r["kind"], "removed");
    assert_eq!(r["code"], "B-07");
    assert_eq!(r["ts"], end);
    assert_eq!(r["since"], 170);
    assert_eq!(r["reason"], "bin taken away");
    assert_eq!(r["by_name"], "Site admin");
    assert_eq!(log[1]["kind"], "removal_started");
}
