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
    /// Still staff's to act on: acknowledge, update, claim a fix or close.
    #[must_use]
    pub fn awaits_staff(self) -> bool {
        matches!(
            self,
            Status::Open | Status::Acknowledged | Status::InProgress
        )
    }

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
    /// First applied acknowledge and fix claim: History's SLA scars (a reopen keeps them).
    pub acked_ts: Option<u64>,
    pub claimed_ts: Option<u64>,
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

/// (who recorded it, heads it covers, LEZ tx)
pub type CheckpointRecord = (Key, Vec<(Key, u64)>, String);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub admin: Option<Key>,
    pub config: Config,
    pub locations: Vec<Location>,
    /// code -> why it was retired. Retired locations stay in `locations`.
    pub retired: BTreeMap<String, String>,
    /// code -> its removal. Stays in `locations`; never also in `retired`.
    pub pending_removal: BTreeMap<String, Removal>,
    /// code -> the name it had before its latest rename.
    pub renamed_from: BTreeMap<String, String>,
    pub roles: BTreeMap<Key, Role>,
    pub names: BTreeMap<Key, String>,
    pub pending_members: BTreeSet<Key>,
    pub issues: BTreeMap<Id, Issue>,
    pub rejected: BTreeMap<Id, String>,
    pub checkpoints: Vec<CheckpointRecord>,
}

/// How long a removed location can be brought back, in seconds of event time.
// ponytail: judged by the events' `ts` (the author's clock), never a local clock, so every
// replica agrees; a skewed admin clock can shift the window. Anchor-backed time if that matters.
pub const REMOVAL_COOLDOWN: u64 = 30 * 24 * 3600;
const DAY: u64 = 24 * 3600;

/// A location on its way out: who started it, when (event ts) and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Removal {
    pub by: Key,
    pub since: u64,
    pub reason: String,
}

impl Removal {
    #[must_use]
    pub fn removes_at(&self) -> u64 {
        self.since.saturating_add(REMOVAL_COOLDOWN)
    }

    /// "Removed" is a view, not a state: pending and the window is over at `now`.
    #[must_use]
    pub fn is_done(&self, now: u64) -> bool {
        now >= self.removes_at()
    }
}

impl State {
    /// Issues at `code` that still need someone to act: they block retiring it.
    #[must_use]
    pub fn open_issues_at(&self, code: &str) -> usize {
        self.issues
            .values()
            .filter(|i| i.location == code && !i.status.is_terminal())
            .count()
    }

    /// Has any accepted report ever named `code`? Issues are never dropped from state, so
    /// this covers every status, terminal ones included.
    #[must_use]
    pub fn ever_used(&self, code: &str) -> bool {
        self.issues.values().any(|i| i.location == code)
    }

    /// The start of every location change: an admin, and a listed code (never `other`).
    fn admin_on_location(&self, by: &Key, code: &str, verb: &str) -> Result<(), String> {
        require(
            self.roles.get(by) == Some(&Role::Admin),
            format!("only admin {verb} locations"),
        )?;
        require(
            code != OTHER_LOCATION && self.locations.iter().any(|l| l.code == code),
            format!("unknown location {code}"),
        )
    }

    fn not_pending_removal(&self, code: &str) -> Result<(), String> {
        require(
            !self.pending_removal.contains_key(code),
            format!("location {code} is pending removal"),
        )
    }

    /// THE retire/restore rule. The reducer enforces it on every replica and `Node` runs the
    /// same check before signing, so a refusal never reaches the network.
    pub fn check_location_change(
        &self,
        by: &Key,
        code: &str,
        retire: bool,
        reason: &str,
    ) -> Result<(), String> {
        self.admin_on_location(by, code, "retires")?;
        require(
            !reason.trim().is_empty(),
            "a retire or restore needs a reason",
        )?;
        let retired = self.retired.contains_key(code);
        if !retire {
            return require(retired, format!("{code} is not retired"));
        }
        require(!retired, format!("{code} is already retired"))?;
        self.not_pending_removal(code)?;
        let n = self.open_issues_at(code);
        require(
            n == 0,
            format!("{code} has {n} open issue{}", if n == 1 { "" } else { "s" }),
        )
    }

    /// THE add rule: every code is new (retired, pending and removed ones are still in
    /// `locations`), not `other`, and unique within the event; code, name and group are set.
    fn check_locations_add(&self, by: &Key, locations: &[Location]) -> Result<(), String> {
        require(
            self.roles.get(by) == Some(&Role::Admin),
            "only admin edits the site map",
        )?;
        for (i, l) in locations.iter().enumerate() {
            require(
                [&l.code, &l.label, &l.group]
                    .iter()
                    .all(|f| !f.trim().is_empty()),
                "a location needs a code, a name and a group",
            )?;
            let taken = l.code == OTHER_LOCATION
                || self.locations.iter().any(|x| x.code == l.code)
                || locations[..i].iter().any(|x| x.code == l.code);
            require(!taken, format!("{} is already used", l.code))?;
        }
        Ok(())
    }

    /// THE rule for every location-changing body (`ts` is the event's): the reducer applies it
    /// on every replica and `Node` checks it before signing. Other bodies pass.
    pub fn check_location_body(&self, by: &Key, body: &Body, ts: u64) -> Result<(), String> {
        match body {
            Body::LocationsAdd { locations } => self.check_locations_add(by, locations),
            Body::LocationRetire {
                code,
                retired,
                reason,
            } => self.check_location_change(by, code, *retired, reason),
            Body::LocationEdit { code, label, group } => {
                self.admin_on_location(by, code, "edits")?;
                self.not_pending_removal(code)?;
                require(
                    !label.trim().is_empty() && !group.trim().is_empty(),
                    "a location needs a name and a group",
                )
            }
            Body::LocationRemove { code, undo, reason } => {
                self.admin_on_location(by, code, "removes")?;
                require(!reason.trim().is_empty(), "a remove or undo needs a reason")?;
                if !*undo {
                    self.not_pending_removal(code)?;
                    return require(
                        !self.ever_used(code),
                        format!("{code} has had reports: retire it instead"),
                    );
                }
                let r = self
                    .pending_removal
                    .get(code)
                    .ok_or_else(|| format!("{code} is not pending removal"))?;
                require(
                    ts < r.removes_at(),
                    format!(
                        "{code} is removed: the {}-day undo window has ended",
                        REMOVAL_COOLDOWN / DAY
                    ),
                )
            }
            _ => Ok(()),
        }
    }

    /// Apply a location change `check_location_body` accepted.
    fn apply_location_body(&mut self, by: Key, body: &Body, ts: u64) {
        match body {
            Body::LocationRetire {
                code,
                retired: true,
                reason,
            } => {
                self.retired.insert(code.clone(), reason.clone());
            }
            Body::LocationRetire { code, .. }
            | Body::LocationRemove {
                code, undo: true, ..
            } => {
                self.retired.remove(code);
                self.pending_removal.remove(code);
            }
            Body::LocationRemove { code, reason, .. } => {
                // never both: a retired place that leaves the list stops being "retired"
                self.retired.remove(code);
                self.pending_removal.insert(
                    code.clone(),
                    Removal {
                        by,
                        since: ts,
                        reason: reason.clone(),
                    },
                );
            }
            Body::LocationsAdd { locations } => self.locations.extend_from_slice(locations),
            Body::LocationEdit { code, label, group } => {
                if let Some(l) = self.locations.iter_mut().find(|l| &l.code == code) {
                    if &l.label != label {
                        let old = std::mem::replace(&mut l.label, label.clone());
                        self.renamed_from.insert(code.clone(), old);
                    }
                    l.group.clone_from(group);
                }
            }
            _ => {}
        }
    }
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
fn progress_by(by: Key, ts: u64, note: &str, next_step: &str, eta_h: u32) -> Option<Progress> {
    Some(Progress {
        by,
        ts,
        note: note.trim_end_matches([' ', ':']).to_string(),
        next_step: next_step.to_string(),
        eta_h,
    })
}

pub(crate) fn require(cond: bool, msg: impl Into<String>) -> Result<(), String> {
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
        b @ (LocationsAdd { .. }
        | LocationRetire { .. }
        | LocationEdit { .. }
        | LocationRemove { .. }) => {
            s.check_location_body(&a, b, e.u.ts)?;
            s.apply_location_body(a, b, e.u.ts);
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
                require(
                    !s.retired.contains_key(location),
                    format!("location {location} is retired"),
                )?;
                s.not_pending_removal(location)?;
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
                    acked_ts: None,
                    claimed_ts: None,
                    timeline: vec![e.id],
                },
            );
            Ok(())
        }
        Checkpoint { heads, lez_tx } => {
            s.checkpoints.push((a, heads.clone(), lez_tx.clone()));
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
            i.progress = progress_by(a, e.u.ts, "Marked duplicate", "", 0);
            Ok(())
        }
        body => {
            let reporter_ok = |i: &Issue| i.reporter == a;
            let resident = role == Some(Role::Resident);
            let iid = issue_of(body).expect("remaining bodies reference an issue");
            let i = s.issues.get_mut(&iid).ok_or("unknown issue")?;
            let st = i.status;
            let progress = |note: &str, next_step: &str, eta_h: u32| {
                progress_by(a, e.u.ts, note, next_step, eta_h)
            };
            match body {
                Comment { .. } => {}
                Acknowledge { note, eta_h, .. } => {
                    require(staff, "only stewards acknowledge")?;
                    require(st == Open, not_permitted(st))?;
                    i.status = Acknowledged;
                    i.progress = progress(note, "", *eta_h);
                    i.acked_ts.get_or_insert(e.u.ts);
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
                    require(st.awaits_staff(), not_permitted(st))?;
                    i.status = InProgress;
                    i.progress = progress(note, next_step, *eta_h);
                }
                ClaimResolved { note, .. } => {
                    require(staff, "only stewards claim resolution")?;
                    require(
                        !note.trim().is_empty(),
                        "a fix claim needs a note: what was done",
                    )?;
                    require(st.awaits_staff(), not_permitted(st))?;
                    i.progress = progress(note, "reporter or two residents confirm", 0);
                    i.status = AwaitingConfirmation;
                    i.claimant = Some(a);
                    i.claimed_ts.get_or_insert(e.u.ts);
                    i.confirms.clear();
                    i.reopen_votes.clear();
                }
                CloseWontfix { reason, .. } => {
                    require(staff, "only stewards close")?;
                    require(!reason.trim().is_empty(), "won't-fix needs a reason")?;
                    require(st.awaits_staff(), not_permitted(st))?;
                    i.status = ClosedWontfix;
                    i.progress = progress(reason, "", 0);
                }
                Confirm { note, .. } => {
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
                    if i.status == ConfirmedResolved {
                        // the claim's "next: confirm" step is done; show who closed it
                        i.progress = progress(&format!("Confirmed: {note}"), "", 0);
                    }
                }
                Reopen { reason, .. } => {
                    require(!reason.trim().is_empty(), "reopen needs a reason")?;
                    require(
                        !s.retired.contains_key(&i.location),
                        format!("location {} is retired", i.location),
                    )?;
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
                        i.progress = progress(&format!("Reopened: {reason}"), "", 0);
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
