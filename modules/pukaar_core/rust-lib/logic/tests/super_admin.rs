//! Super admins (at most two; the genesis author is the first) and the admin notice period:
//! only a super admin hands out or takes away the admin role, and a demotion happens on its
//! own at the deadline, sealed by any member's node.
mod common;
mod util;
use common::*;
use pukaar_logic::event::{Body, Event, Key, Location, NoticeOutcome, Role};
use pukaar_logic::node::{Node, SEAL_GRACE};
use serde_json::Value;

const DEADLINE: u64 = 100;
/// The first moment a node signs the seal on its own.
const SEALED_AT: u64 = DEADLINE + SEAL_GRACE;

fn hexk(n: &Node) -> String {
    hex::encode(n.me())
}

fn json(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

/// Why `n`'s replica rejected `e`, if it did.
fn rejected(n: &Node, e: &Event) -> Option<String> {
    n.state().rejected.get(&e.id).cloned()
}

fn role(n: &Node, k: Key) -> Option<Role> {
    n.state().roles.get(&k).copied()
}

fn is_super(n: &Node, k: Key) -> bool {
    n.state().super_admins.contains(&k)
}

fn grant(subject: Key, role: Role, name: &str) -> Body {
    Body::RoleGrant {
        subject,
        role,
        name: Some(name.into()),
    }
}

/// An admin-only act: a new place on the map.
fn add_place(n: &mut Node, code: &str, ts: u64) -> Event {
    n.publish(
        Body::LocationsAdd {
            locations: vec![Location {
                code: code.into(),
                label: format!("Place {code}"),
                group: "Paths".into(),
            }],
        },
        ts,
    )
    .unwrap()
}

/// `Site::new()` with the steward promoted to admin "Deputy" by the genesis super admin.
fn deputy_site() -> Site {
    let mut s = Site::new();
    let d = s.steward.me();
    s.admin.publish(grant(d, Role::Admin, "Deputy"), 2).unwrap();
    s.sync();
    s
}

/// A named member, so they can become a super admin.
fn name(n: &mut Node, ts: u64) {
    n.publish(profile("Named"), ts).unwrap();
}

/// The genesis admin gives the deputy a notice (outcome, DEADLINE), synced to everyone.
fn notice(s: &mut Site, outcome: &str) -> Event {
    let d = hexk(&s.steward);
    let e = s
        .admin
        .admin_notice(&d, outcome, DEADLINE, "stepping back", 10)
        .unwrap();
    s.sync();
    e
}

// ---------- v0.2.4 logs ----------

/// Every key of `old` is in `new` with the same value (new may add keys).
fn superset(new: &Value, old: &Value, path: &str) {
    match (new, old) {
        (Value::Object(n), Value::Object(o)) => o
            .iter()
            .for_each(|(k, v)| superset(&n[k], v, &format!("{path}.{k}"))),
        (Value::Array(n), Value::Array(o)) => {
            assert_eq!(n.len(), o.len(), "{path}: length");
            n.iter()
                .zip(o)
                .enumerate()
                .for_each(|(i, (n, o))| superset(n, o, &format!("{path}[{i}]")));
        }
        _ => assert_eq!(new, old, "{path}"),
    }
}

#[test]
fn a_v0_2_4_log_replays_to_the_same_state() {
    let fx = json(include_str!("fixtures/v0_2_4_log.json"));
    let now = fx["now"].as_u64().unwrap();
    let bytes: Vec<Vec<u8>> = fx["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| hex::decode(h.as_str().unwrap()).unwrap())
        .collect();
    let replay = |seed: u8| {
        let site = pukaar_logic::node::parse_id(fx["site_info"]["site"].as_str().unwrap()).unwrap();
        let mut n = Node::join(
            pukaar_logic::event::SigningKey::from_bytes(&[seed; 32]),
            site,
        );
        bytes.iter().for_each(|b| {
            n.receive(b).unwrap();
        });
        n
    };
    let admin = replay(1);
    let s = admin.state();
    // the State as v0.2.4 printed it, then only the new fields after it
    let old = fx["state_debug"].as_str().unwrap();
    let new = format!("{s:?}");
    let head = old.strip_suffix(" }").unwrap();
    assert!(new.starts_with(head), "old fields changed:\n{new}\n{old}");
    assert!(new[head.len()..].starts_with(", super_admins: {"));
    assert_eq!(
        s.super_admins.iter().collect::<Vec<_>>(),
        vec![&s.admin.unwrap()]
    );
    let rejected: serde_json::Map<String, Value> = s
        .rejected
        .iter()
        .map(|(id, why)| (hex::encode(id), Value::from(why.as_str())))
        .collect();
    assert_eq!(Value::Object(rejected), fx["rejected"]);
    superset(
        &json(&admin.site_info_json(now)),
        &fx["site_info"],
        "site_info",
    );
    superset(&json(&admin.issues_json(now)), &fx["issues"], "issues");
    for (id, tl) in fx["timelines"].as_object().unwrap() {
        superset(&json(&admin.timeline_json(id, now)), tl, id);
    }
    for (who, seed) in [("admin", 1), ("deputy", 5), ("ravi", 4)] {
        superset(
            &json(&replay(seed).identity_json()),
            &fx["identity"][who],
            who,
        );
    }
}

// ---------- the admin role is the super admin's to give ----------

#[test]
fn role_grant_and_revoke_cannot_touch_the_genesis_super_admin() {
    let mut s = deputy_site();
    let genesis = s.admin.me();
    assert!(is_super(&s.admin, genesis));
    let demote = s
        .steward
        .publish(grant(genesis, Role::Resident, "x"), 3)
        .unwrap();
    let revoke = s.steward.revoke_role(&hexk(&s.admin), "coup", 4).unwrap();
    // not even the super admin themself
    let own = s
        .admin
        .publish(grant(genesis, Role::Steward, "me"), 5)
        .unwrap();
    s.sync();
    assert!(rejected(&s.admin, &demote).is_some());
    assert!(rejected(&s.admin, &revoke).is_some());
    assert!(rejected(&s.admin, &own).is_some());
    assert_eq!(role(&s.admin, genesis), Some(Role::Admin));
}

#[test]
fn only_a_super_admin_grants_or_changes_the_admin_role() {
    let mut s = deputy_site();
    let (deputy, asha) = (s.steward.me(), s.asha.me());
    let up = s
        .steward
        .publish(grant(asha, Role::Admin, "Asha"), 3)
        .unwrap();
    assert_eq!(
        rejected(&s.steward, &up).as_deref(),
        Some("only a super admin grants or changes the admin role")
    );
    // an admin keeps every other admin power
    let st = s
        .steward
        .publish(grant(asha, Role::Steward, "Asha"), 4)
        .unwrap();
    assert_eq!(rejected(&s.steward, &st), None);
    s.sync();
    // a second admin can't demote or remove the super admin's other admin either
    let ravi = s.ravi.me();
    s.admin
        .publish(grant(ravi, Role::Admin, "Ravi"), 5)
        .unwrap();
    s.sync();
    let down = s
        .steward
        .publish(grant(ravi, Role::Steward, "Ravi"), 6)
        .unwrap();
    let out = s.steward.revoke_role(&hexk(&s.ravi), "no", 7).unwrap();
    assert!(rejected(&s.steward, &down).is_some());
    assert!(rejected(&s.steward, &out).is_some());
    // the super admin can
    let ok = s
        .admin
        .publish(grant(deputy, Role::Steward, "Deputy"), 8)
        .unwrap();
    assert_eq!(rejected(&s.admin, &ok), None);
    assert_eq!(role(&s.admin, deputy), Some(Role::Steward));
}

// ---------- adding, removing, transferring ----------

#[test]
fn a_super_admin_adds_one_more_named_member_and_no_third() {
    let mut s = deputy_site();
    // unnamed members can't be super admins
    let unnamed = s.admin.add_super_admin(&hexk(&s.asha), 3).unwrap();
    assert!(rejected(&s.admin, &unnamed).is_some());
    // a plain admin can't add one
    let by_admin = s.steward.add_super_admin(&hexk(&s.steward), 3).unwrap();
    assert!(rejected(&s.steward, &by_admin).is_some());
    name(&mut s.ravi, 3);
    s.sync();
    let add = s.admin.add_super_admin(&hexk(&s.ravi), 4).unwrap();
    assert_eq!(rejected(&s.admin, &add), None);
    assert!(is_super(&s.admin, s.ravi.me()));
    assert_eq!(role(&s.admin, s.ravi.me()), Some(Role::Admin));
    let third = s.admin.add_super_admin(&hexk(&s.steward), 5).unwrap();
    assert_eq!(
        rejected(&s.admin, &third).as_deref(),
        Some("there are already two super admins")
    );
}

/// Each of two super admins removes the other and adds someone new, concurrently: each add
/// fits in its author's view, but only the one applied first does.
#[test]
fn concurrent_adds_over_the_limit_keep_the_first() {
    let mut s = deputy_site();
    name(&mut s.ravi, 3);
    name(&mut s.meera, 3);
    s.sync();
    s.admin.add_super_admin(&hexk(&s.steward), 4).unwrap();
    s.sync();
    s.admin.remove_super_admin(&hexk(&s.steward), 5).unwrap();
    s.steward.remove_super_admin(&hexk(&s.admin), 5).unwrap();
    let a = s.admin.add_super_admin(&hexk(&s.ravi), 6).unwrap();
    let b = s.steward.add_super_admin(&hexk(&s.meera), 6).unwrap();
    assert_eq!(
        (rejected(&s.admin, &a), rejected(&s.steward, &b)),
        (None, None)
    );
    s.sync();
    let st = s.admin.state();
    assert_eq!(st.super_admins.len(), 2);
    let outcomes = [st.rejected.get(&a.id), st.rejected.get(&b.id)];
    assert_eq!(outcomes.iter().filter(|r| r.is_some()).count(), 1);
    for n in s.nodes() {
        assert_eq!(n.state().super_admins, st.super_admins);
    }
}

#[test]
fn mutual_concurrent_removal_keeps_the_first() {
    let mut s = deputy_site();
    s.admin.add_super_admin(&hexk(&s.steward), 3).unwrap();
    s.sync();
    let a = s.admin.remove_super_admin(&hexk(&s.steward), 4).unwrap();
    let b = s.steward.remove_super_admin(&hexk(&s.admin), 4).unwrap();
    s.sync();
    let st = s.admin.state();
    assert_eq!(st.super_admins.len(), 1);
    let second = [&a, &b]
        .into_iter()
        .find_map(|e| st.rejected.get(&e.id))
        .unwrap();
    assert_eq!(second, "only a super admin does this");
    // both stay admins
    assert_eq!(role(&s.admin, s.admin.me()), Some(Role::Admin));
    assert_eq!(role(&s.admin, s.steward.me()), Some(Role::Admin));
}

#[test]
fn transfer_hands_over_and_works_with_two() {
    let mut s = deputy_site();
    name(&mut s.ravi, 3);
    s.sync();
    s.admin.transfer_super_admin(&hexk(&s.steward), 4).unwrap();
    s.sync();
    assert!(!is_super(&s.admin, s.admin.me()));
    assert!(is_super(&s.admin, s.steward.me()));
    assert_eq!(role(&s.admin, s.admin.me()), Some(Role::Admin));
    // the old genesis super admin is now a plain admin: it can't grant admin any more
    let up = s
        .admin
        .publish(grant(s.asha.me(), Role::Admin, "A"), 5)
        .unwrap();
    assert!(rejected(&s.admin, &up).is_some());
    s.steward.add_super_admin(&hexk(&s.admin), 6).unwrap();
    s.sync();
    let t = s.admin.transfer_super_admin(&hexk(&s.ravi), 7).unwrap();
    assert_eq!(rejected(&s.admin, &t), None);
    let st = s.admin.state();
    assert!(st.super_admins.contains(&s.ravi.me()) && st.super_admins.contains(&s.steward.me()));
    assert_eq!(st.super_admins.len(), 2);
    assert_eq!(st.roles[&s.ravi.me()], Role::Admin);
}

#[test]
fn there_is_always_a_super_admin() {
    let mut s = deputy_site();
    let me = hexk(&s.admin);
    let own = s.admin.remove_super_admin(&me, 3).unwrap();
    let to_self = s.admin.transfer_super_admin(&me, 4).unwrap();
    assert!(rejected(&s.admin, &own).is_some());
    assert!(rejected(&s.admin, &to_self).is_some());
    assert_eq!(s.admin.state().super_admins.len(), 1);
}

// ---------- the notice period ----------

#[test]
fn a_notice_keeps_admin_powers_until_the_deadline() {
    let mut s = deputy_site();
    notice(&mut s, "steward");
    let before = add_place(&mut s.steward, "L-1", DEADLINE - 1);
    let at = add_place(&mut s.steward, "L-2", DEADLINE);
    assert_eq!(rejected(&s.steward, &before), None);
    assert_eq!(
        rejected(&s.steward, &at).as_deref(),
        Some("only admin edits the site map")
    );
    // the UI flips at the minute, before any seal
    let info = json(&s.admin.site_info_json(DEADLINE - 1));
    let row = |info: &Value| {
        info["members"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["key"] == hexk(&s.steward))
            .unwrap()
            .clone()
    };
    assert_eq!(row(&info)["role"], "Admin");
    assert_eq!(
        row(&json(&s.admin.site_info_json(DEADLINE)))["role"],
        "Steward"
    );
    let n = &info["notices"][0];
    assert_eq!(n["subject"], hexk(&s.steward));
    assert_eq!(n["by"], hexk(&s.admin));
    assert_eq!(n["outcome"], "steward");
    assert_eq!(n["deadline"], DEADLINE);
    assert_eq!(n["reason"], "stepping back");
    assert_eq!(n["sealed"], false);
    assert_eq!(info["super_admins"][0]["key"], hexk(&s.admin));
    let me = json(&s.steward.identity_json_at(DEADLINE));
    assert_eq!(me["role"], "Steward");
    assert_eq!(me["notice"]["deadline"], DEADLINE);
}

#[test]
fn only_a_super_admin_gives_a_notice_and_only_to_a_plain_admin() {
    let mut s = deputy_site();
    let by_admin = s
        .steward
        .admin_notice(&hexk(&s.admin), "remove", DEADLINE, "x", 3)
        .unwrap();
    assert!(rejected(&s.steward, &by_admin).is_some());
    let on_resident = s
        .admin
        .admin_notice(&hexk(&s.asha), "remove", DEADLINE, "x", 3)
        .unwrap();
    assert!(rejected(&s.admin, &on_resident).is_some());
    let d = hexk(&s.steward);
    assert!(s
        .admin
        .admin_notice(&d, "remove", DEADLINE, " ", 3)
        .is_err());
    assert!(s.admin.admin_notice(&d, "fired", DEADLINE, "x", 3).is_err());
    // a new notice replaces the old one
    s.admin
        .admin_notice(&d, "remove", DEADLINE, "x", 3)
        .unwrap();
    s.admin
        .admin_notice(&d, "steward", DEADLINE + 5, "y", 4)
        .unwrap();
    let st = s.admin.state();
    assert_eq!(st.notices.len(), 1);
    assert_eq!(st.notices[&s.steward.me()].outcome, NoticeOutcome::Steward);
}

/// The deputy's node misses the seal and signs an admin act with an old clock and a low
/// lamport, so it orders before the seal: its seq gives it away.
#[test]
fn a_backdated_act_after_the_seal_is_rejected() {
    let mut s = deputy_site();
    notice(&mut s, "steward");
    let legit = add_place(&mut s.steward, "L-1", 20);
    s.sync();
    // the rest of the site moves on (lamport rises); the deputy hears none of it
    for ts in 30..34 {
        s.asha.publish(profile(&format!("Asha {ts}")), ts).unwrap();
    }
    copy_all(&s.asha, &mut s.admin);
    let seal = s.admin.auto_seal(SEALED_AT).pop().unwrap();
    let late = add_place(&mut s.steward, "L-2", 50);
    assert!(late.u.lamport < seal.u.lamport, "orders before the seal");
    s.sync();
    let d = s.steward.me();
    for n in s.nodes() {
        let st = n.state();
        assert_eq!(st.rejected.get(&legit.id), None);
        assert_eq!(
            st.rejected.get(&late.id).map(String::as_str),
            Some("only admin edits the site map")
        );
        assert!(!st.locations.iter().any(|l| l.code == "L-2"));
        assert_eq!(st.roles[&d], Role::Steward);
    }
}

#[test]
fn the_first_seal_wins_and_later_ones_change_nothing() {
    let mut s = deputy_site();
    notice(&mut s, "steward");
    let first = s.admin.auto_seal(SEALED_AT).pop().unwrap();
    s.sync();
    // a seal that arrives after the first: accepted, a no-op
    let d = s.steward.me();
    let later = s
        .admin
        .publish(
            Body::NoticeSeal {
                subject: d,
                seen_seq: 99,
            },
            SEALED_AT + 1,
        )
        .unwrap();
    s.sync();
    let st = s.admin.state();
    assert_eq!(st.rejected.get(&later.id), None);
    let Body::NoticeSeal { seen_seq, .. } = first.u.body else {
        unreachable!()
    };
    assert_eq!(st.notices[&d].sealed.map(|x| x.seen_seq), Some(seen_seq));
    let history = &st.demotions[&d];
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].deadline, DEADLINE);
    assert_eq!(history[0].by, s.admin.me());
    assert_eq!(history[0].reason, "stepping back");
    assert_eq!(st.roles[&d], Role::Steward);
    let info = json(&s.admin.site_info_json(DEADLINE + 1));
    let row = info["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == hexk(&s.steward))
        .unwrap()
        .clone();
    assert_eq!(row["super_admin"], false);
    assert_eq!(row["demotions"][0]["outcome"], "steward");
    assert_eq!(info["notices"][0]["sealed"], true);
}

#[test]
fn a_seal_before_the_deadline_is_rejected() {
    let mut s = deputy_site();
    notice(&mut s, "steward");
    let d = s.steward.me();
    let early = s
        .admin
        .publish(
            Body::NoticeSeal {
                subject: d,
                seen_seq: 9,
            },
            DEADLINE - 1,
        )
        .unwrap();
    assert!(rejected(&s.admin, &early).is_some());
    assert_eq!(role(&s.admin, d), Some(Role::Admin));
}

#[test]
fn a_cancelled_notice_changes_nothing() {
    let mut s = deputy_site();
    notice(&mut s, "remove");
    s.admin.cancel_notice(&hexk(&s.steward), 20).unwrap();
    s.sync();
    assert!(s.admin.auto_seal(SEALED_AT).is_empty());
    let after = add_place(&mut s.steward, "L-1", DEADLINE + 1);
    assert_eq!(rejected(&s.steward, &after), None);
    // nothing left to cancel once sealed
    notice(&mut s, "steward");
    s.admin.auto_seal(SEALED_AT);
    let late = s
        .admin
        .cancel_notice(&hexk(&s.steward), DEADLINE + 2)
        .unwrap();
    assert!(rejected(&s.admin, &late).is_some());
    assert_eq!(role(&s.admin, s.steward.me()), Some(Role::Steward));
}

#[test]
fn end_now_demotes_at_once() {
    let mut s = deputy_site();
    notice(&mut s, "steward");
    let end = s.admin.end_notice_now(&hexk(&s.steward), 20).unwrap();
    assert_eq!(rejected(&s.admin, &end), None);
    s.sync();
    let d = s.steward.me();
    let st = s.admin.state();
    assert_eq!(st.roles[&d], Role::Steward);
    assert_eq!(st.demotions[&d][0].deadline, 20);
    let after = add_place(&mut s.steward, "L-1", 21);
    assert!(rejected(&s.steward, &after).is_some());
    assert!(s.admin.auto_seal(SEALED_AT).is_empty());
}

#[test]
fn a_remove_outcome_revokes_the_admin() {
    let mut s = deputy_site();
    notice(&mut s, "remove");
    s.admin.auto_seal(SEALED_AT);
    s.sync();
    let d = s.steward.me();
    let st = s.admin.state();
    assert!(!st.roles.contains_key(&d));
    let r = &st.revoked[&d];
    assert_eq!((r.by, r.reason.as_str()), (s.admin.me(), "stepping back"));
    let after = s.steward.publish(profile("x"), DEADLINE + 1).unwrap();
    let report = s
        .steward
        .publish(
            Body::Report {
                category: "water".into(),
                location: "W-03".into(),
                landmark: String::new(),
                text: "x".into(),
            },
            DEADLINE + 2,
        )
        .unwrap();
    assert_eq!(rejected(&s.steward, &after), None);
    assert_eq!(
        rejected(&s.steward, &report).as_deref(),
        Some("not a member")
    );
}

#[test]
fn the_auto_seal_waits_the_grace_and_is_signed_once() {
    let mut s = deputy_site();
    notice(&mut s, "steward");
    assert!(s.admin.auto_seal(DEADLINE).is_empty());
    assert!(s.admin.auto_seal(SEALED_AT - 1).is_empty());
    assert_eq!(s.admin.auto_seal(SEALED_AT).len(), 1);
    assert!(s.admin.auto_seal(SEALED_AT + 60).is_empty());
    // a pending member's node doesn't try
    let mut stranger = Node::join_announced(pukaar_logic::event::new_key(), s.admin.store.site, 1);
    copy_all(&s.asha, &mut stranger);
    assert!(stranger.auto_seal(SEALED_AT).is_empty());
}

#[test]
fn only_staff_other_than_the_subject_auto_seal() {
    let mut s = deputy_site();
    notice(&mut s, "steward");
    assert!(s.asha.auto_seal(SEALED_AT).is_empty(), "a resident");
    assert!(s.steward.auto_seal(SEALED_AT).is_empty(), "the subject");
    // another steward may
    let r = s.ravi.me();
    s.admin
        .publish(grant(r, Role::Steward, "Ravi"), 11)
        .unwrap();
    s.sync();
    assert_eq!(s.ravi.auto_seal(SEALED_AT).len(), 1);
}

#[test]
fn a_residents_seal_is_rejected() {
    let mut s = deputy_site();
    notice(&mut s, "steward");
    let d = s.steward.me();
    let seal = |s: &mut Site, ts| {
        s.asha
            .publish(
                Body::NoticeSeal {
                    subject: d,
                    seen_seq: 9,
                },
                ts,
            )
            .unwrap()
    };
    let e = seal(&mut s, SEALED_AT);
    assert_eq!(
        rejected(&s.asha, &e).as_deref(),
        Some("only staff can seal a notice")
    );
    assert_eq!(s.asha.state().notices[&d].sealed, None);
    // and a resident cannot pre-empt the staff seal that follows
    s.sync();
    s.admin.auto_seal(SEALED_AT);
    s.sync();
    assert!(s.asha.state().notices[&d].sealed.is_some());
}

#[test]
fn the_subject_cannot_seal_their_own_notice() {
    let mut s = deputy_site();
    notice(&mut s, "steward");
    let d = s.steward.me();
    let e = s
        .steward
        .publish(
            Body::NoticeSeal {
                subject: d,
                seen_seq: 9,
            },
            SEALED_AT,
        )
        .unwrap();
    assert_eq!(
        rejected(&s.steward, &e).as_deref(),
        Some("a member cannot seal their own notice")
    );
    assert_eq!(s.steward.state().notices[&d].sealed, None);
}

#[test]
fn the_notice_json_says_who_sealed_it_and_when() {
    let mut s = deputy_site();
    notice(&mut s, "steward");
    let n = json(&s.admin.site_info_json(DEADLINE))["notices"][0].clone();
    assert_eq!(n["sealed"], false);
    assert!(n["sealed_by"].is_null() && n["sealed_by_name"].is_null() && n["sealed_ts"].is_null());
    s.admin.auto_seal(SEALED_AT);
    s.sync();
    let n = json(&s.admin.site_info_json(SEALED_AT))["notices"][0].clone();
    assert_eq!(n["sealed"], true);
    assert_eq!(n["sealed_by"], hexk(&s.admin));
    assert_eq!(n["sealed_by_name"], s.admin.state().names[&s.admin.me()]);
    assert_eq!(n["sealed_ts"], SEALED_AT);
}

#[test]
fn end_now_is_sealed_by_the_super_admin_who_ended_it() {
    let mut s = deputy_site();
    notice(&mut s, "steward");
    s.admin.end_notice_now(&hexk(&s.steward), 20).unwrap();
    s.sync();
    let n = json(&s.admin.site_info_json(21))["notices"][0].clone();
    assert_eq!(n["sealed_by"], hexk(&s.admin));
    assert_eq!(n["sealed_ts"], 20);
}

#[test]
fn a_past_notice_deadline_means_now() {
    let mut s = deputy_site();
    let e = s
        .admin
        .admin_notice(&hexk(&s.steward), "steward", 1, "stepping back", 30)
        .unwrap();
    assert_eq!(rejected(&s.admin, &e), None);
    s.sync();
    let d = s.steward.me();
    assert_eq!(s.admin.state().notices[&d].deadline, 30);
    let after = add_place(&mut s.steward, "L-1", 31);
    assert!(rejected(&s.steward, &after).is_some());
}
