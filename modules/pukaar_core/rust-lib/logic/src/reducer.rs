//! Deterministic reducer: a pure function of the accepted event set.
//! Invalid events are kept but inert, with the reason recorded (nothing is dropped silently).

use crate::event::{Body, Event, Id, Key, Location, Role, OTHER_LOCATION};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Status {
    Open,
    Acknowledged,
    InProgress,
    AwaitingConfirmation,
    ConfirmedResolved,
    ClosedWontfix,
    Duplicate,
}

impl Status {
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Status::ConfirmedResolved | Status::ClosedWontfix | Status::Duplicate
        )
    }
}

/// The latest thing staff said about an issue: what was done, what's next, by when.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    pub by: Key,
    pub ts: u64,
    pub note: String,
    pub next_step: String,
    pub eta_h: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    pub id: Id,
    pub reporter: Key,
    pub category: String,
    pub location: String,
    pub landmark: String,
    pub text: String,
    pub status: Status,
    pub progress: Option<Progress>,
    pub claimant: Option<Key>,
    pub confirms: BTreeSet<Key>,
    pub reopen_votes: BTreeSet<Key>,
    pub reopen_count: u32,
    pub duplicate_of: Option<Id>,
    pub reported_ts: u64,
    pub timeline: Vec<Id>, // every event that touched this issue, applied or rejected
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub name: String,
    pub categories: Vec<String>,
    pub sla_ack_h: u32,
    pub sla_fix_h: u32,
    pub max_open_per_author: u32,
}

/// (checkpoint event id, heads it covers, LEZ tx)
pub type CheckpointRecord = (Id, Vec<(Key, u64)>, String);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub admin: Option<Key>,
    pub config: Config,
    pub locations: Vec<Location>,
    pub roles: BTreeMap<Key, Role>,
    pub names: BTreeMap<Key, String>,
    pub pending_members: BTreeSet<Key>,
    pub issues: BTreeMap<Id, Issue>,
    pub rejected: BTreeMap<Id, String>,
    pub checkpoints: Vec<CheckpointRecord>,
}

#[must_use]
pub fn reduce<'a>(ordered: impl IntoIterator<Item = &'a Event>) -> State {
    let mut s = State::default();
    for e in ordered {
        if let Err(why) = apply(&mut s, e) {
            s.rejected.insert(e.id, why);
        }
    }
    s
}

/// One guard for every role/permission/state check below: `cond` holds or the event is
/// rejected with `msg`. Collapses what would otherwise be an `if !cond { return Err(..) }`
/// at each call site into a single line.
fn require(cond: bool, msg: impl Into<String>) -> Result<(), String> {
    if cond {
        Ok(())
    } else {
        Err(msg.into())
    }
}

fn not_permitted(st: Status) -> String {
    format!("not permitted in state {st:?}")
}

fn issue_of(b: &Body) -> Option<Id> {
    use Body::*;
    match b {
        Acknowledge { issue, .. }
        | Update { issue, .. }
        | ClaimResolved { issue, .. }
        | Confirm { issue, .. }
        | Reopen { issue, .. }
        | CloseWontfix { issue, .. }
        | MarkDuplicate { issue, .. }
        | Comment { issue, .. } => Some(*issue),
        _ => None,
    }
}

fn apply(s: &mut State, e: &Event) -> Result<(), String> {
    use Body::*;
    use Status::*;
    let a = e.u.author;
    let role = s.roles.get(&a).copied();
    let staff = matches!(role, Some(Role::Steward | Role::Admin));
    let admin = role == Some(Role::Admin);

    if let Some(iid) = issue_of(&e.u.body) {
        if let Some(i) = s.issues.get_mut(&iid) {
            i.timeline.push(e.id);
        }
    }

    match &e.u.body {
        Genesis {
            name,
            admin_name,
            categories,
            locations,
            sla_ack_h,
            sla_fix_h,
            max_open_per_author,
        } => {
            require(s.admin.is_none(), "second genesis")?;
            require(!admin_name.trim().is_empty(), "the admin must be named")?;
            s.admin = Some(a);
            s.roles.insert(a, Role::Admin);
            s.names.insert(a, admin_name.trim().to_string());
            s.config = Config {
                name: name.clone(),
                categories: categories.clone(),
                sla_ack_h: *sla_ack_h,
                sla_fix_h: *sla_fix_h,
                max_open_per_author: *max_open_per_author,
            };
            s.locations = locations.clone();
            Ok(())
        }
        Profile { .. } if matches!(role, Some(Role::Steward | Role::Admin)) => {
            Err("staff names are set by the admin when granting the role".into())
        }
        Profile { display_name } => {
            match display_name {
                Some(n) => s.names.insert(a, n.clone()),
                None => s.names.remove(&a),
            };
            if role.is_none() {
                s.pending_members.insert(a);
            }
            Ok(())
        }
        _ if role.is_none() => Err("not a member".into()),
        RoleGrant {
            subject,
            role,
            name,
        } => {
            require(admin, "only admin grants roles")?;
            let name = name.as_deref().map(str::trim).filter(|n| !n.is_empty());
            match (role, name) {
                (Role::Steward | Role::Admin, None) => return Err("staff must be named".into()),
                (Role::Resident, Some(_)) => {
                    return Err("residents choose their own name (or none)".into())
                }
                (Role::Steward | Role::Admin, Some(n)) => {
                    s.names.insert(*subject, n.to_string());
                }
                (Role::Resident, None) => {}
            }
            s.roles.insert(*subject, *role);
            s.pending_members.remove(subject);
            Ok(())
        }
        RoleRevoke { subject, .. } => {
            require(admin, "only admin revokes roles")?;
            require(Some(*subject) != s.admin, "admin cannot be revoked")?;
            s.roles.remove(subject);
            Ok(())
        }
        LocationsAdd { locations } => {
            require(admin, "only admin edits the site map")?;
            for l in locations {
                if !s.locations.iter().any(|x| x.code == l.code) {
                    s.locations.push(l.clone());
                }
            }
            Ok(())
        }
        Report {
            category,
            location,
            landmark,
            text,
        } => {
            require(
                s.config.categories.contains(category),
                format!("unknown category {category}"),
            )?;
            if location == OTHER_LOCATION {
                require(
                    !landmark.trim().is_empty(),
                    "a place not on the list needs a landmark note",
                )?;
            } else {
                require(
                    s.locations.iter().any(|l| &l.code == location),
                    format!("unknown location {location}"),
                )?;
            }
            let open = s
                .issues
                .values()
                .filter(|i| i.reporter == a && !i.status.is_terminal())
                .count();
            require(
                open < s.config.max_open_per_author as usize,
                "open-report cap reached",
            )?;
            s.issues.insert(
                e.id,
                Issue {
                    id: e.id,
                    reporter: a,
                    category: category.clone(),
                    location: location.clone(),
                    landmark: landmark.trim().to_string(),
                    text: text.clone(),
                    status: Open,
                    progress: None,
                    claimant: None,
                    confirms: BTreeSet::new(),
                    reopen_votes: BTreeSet::new(),
                    reopen_count: 0,
                    duplicate_of: None,
                    reported_ts: e.u.ts,
                    timeline: vec![e.id],
                },
            );
            Ok(())
        }
        Checkpoint { heads, lez_tx } => {
            s.checkpoints.push((e.id, heads.clone(), lez_tx.clone()));
            Ok(())
        }
        MarkDuplicate { issue, of } => {
            require(staff, "only stewards mark duplicates")?;
            require(
                issue != of && s.issues.contains_key(of),
                "duplicate target missing",
            )?;
            let i = s.issues.get_mut(issue).ok_or("unknown issue")?;
            require(!i.status.is_terminal(), not_permitted(i.status))?;
            i.status = Duplicate;
            i.duplicate_of = Some(*of);
            Ok(())
        }
        body => {
            let reporter_ok = |i: &Issue| i.reporter == a;
            let resident = role == Some(Role::Resident);
            let iid = issue_of(body).expect("remaining bodies reference an issue");
            let i = s.issues.get_mut(&iid).ok_or("unknown issue")?;
            let st = i.status;
            let progress = |note: &str, next_step: &str, eta_h: u32| {
                Some(Progress {
                    by: a,
                    ts: e.u.ts,
                    note: note.to_string(),
                    next_step: next_step.to_string(),
                    eta_h,
                })
            };
            match body {
                Comment { .. } => {}
                Acknowledge { note, eta_h, .. } => {
                    require(staff, "only stewards acknowledge")?;
                    require(st == Open, not_permitted(st))?;
                    i.status = Acknowledged;
                    i.progress = progress(note, "", *eta_h);
                }
                Update {
                    note,
                    next_step,
                    eta_h,
                    ..
                } => {
                    require(staff, "only stewards post updates")?;
                    require(
                        !note.trim().is_empty(),
                        "an update needs a note: what is happening",
                    )?;
                    require(
                        matches!(st, Open | Acknowledged | InProgress),
                        not_permitted(st),
                    )?;
                    i.status = InProgress;
                    i.progress = progress(note, next_step, *eta_h);
                }
                ClaimResolved { note, .. } => {
                    require(staff, "only stewards claim resolution")?;
                    require(
                        !note.trim().is_empty(),
                        "a fix claim needs a note: what was done",
                    )?;
                    require(
                        matches!(st, Open | Acknowledged | InProgress),
                        not_permitted(st),
                    )?;
                    i.progress = progress(note, "reporter or two residents confirm", 0);
                    i.status = AwaitingConfirmation;
                    i.claimant = Some(a);
                    i.confirms.clear();
                    i.reopen_votes.clear();
                }
                CloseWontfix { reason, .. } => {
                    require(staff, "only stewards close")?;
                    require(!reason.trim().is_empty(), "won't-fix needs a reason")?;
                    require(
                        matches!(st, Open | Acknowledged | InProgress),
                        not_permitted(st),
                    )?;
                    i.status = ClosedWontfix;
                    i.progress = progress(reason, "", 0);
                }
                Confirm { .. } => {
                    require(st == AwaitingConfirmation, not_permitted(st))?;
                    require(
                        i.claimant != Some(a),
                        "the fixer cannot confirm their own fix",
                    )?;
                    if reporter_ok(i) {
                        i.status = ConfirmedResolved;
                    } else if resident {
                        i.confirms.insert(a);
                        if i.confirms.len() >= 2 {
                            i.status = ConfirmedResolved;
                        }
                    } else {
                        return Err("only the reporter or residents confirm".into());
                    }
                }
                Reopen { reason, .. } => {
                    require(!reason.trim().is_empty(), "reopen needs a reason")?;
                    require(
                        matches!(
                            st,
                            AwaitingConfirmation | ConfirmedResolved | ClosedWontfix | Duplicate
                        ),
                        not_permitted(st),
                    )?;
                    let go = if reporter_ok(i) {
                        true
                    } else if resident {
                        i.reopen_votes.insert(a);
                        i.reopen_votes.len() >= 2
                    } else {
                        return Err("only the reporter or residents reopen".into());
                    };
                    if go {
                        i.status = Open;
                        i.claimant = None;
                        i.confirms.clear();
                        i.reopen_votes.clear();
                        i.duplicate_of = None;
                        i.reopen_count += 1;
                    }
                }
                _ => unreachable!("handled above"),
            }
            Ok(())
        }
    }
}
