//! Deterministic reducer: a pure function of the accepted event set and the chain roles.
//! Invalid events are kept but inert, with the reason recorded (nothing is dropped silently).
//! Each author is judged with their chain role at the event's `chain_block` (O10). An author who
//! was not an approved member then has the event rejected, not dropped.

use crate::event::{Body, Event, Id, Key, Location, OTHER_LOCATION};
pub use pukaar_sites_rules::Rank;
use std::collections::{BTreeMap, BTreeSet};

/// The chain role of `who` at `chain_block` (`u64::MAX` means the newest block). `None` means not an
/// approved member then. The second argument is the LEZ account that an event names: when it is
/// `Some`, the `Member` record there must hold the Pukaar key `who`. The chain roster of `chain.rs`
/// backs it.
pub type RoleFn = Box<dyn Fn(&Key, Option<&Key>, u64) -> Option<Rank> + Send>;

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

/// A recorded anchor: who recorded it, the heads it covers, the claimed LEZ tx, and when
/// (the recorder's clock, `Unsigned::ts`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointRecord {
    pub by: Key,
    pub heads: Vec<(Key, u64)>,
    pub lez_tx: String,
    pub ts: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub config: Config,
    pub locations: Vec<Location>,
    pub issues: BTreeMap<Id, Issue>,
    pub rejected: BTreeMap<Id, String>,
    pub checkpoints: Vec<CheckpointRecord>,
}

impl State {
    /// Issues at `code` that still need someone to act.
    #[must_use]
    pub fn open_issues_at(&self, code: &str) -> usize {
        self.issues
            .values()
            .filter(|i| i.location == code && !i.status.is_terminal())
            .count()
    }
}

/// The state after `ordered`. `config` and `locations` come from `details.json` and
/// `places.json`, `roles` from the chain.
#[must_use]
pub fn reduce<'a>(
    ordered: impl IntoIterator<Item = &'a Event>,
    roles: &dyn Fn(&Key, Option<&Key>, u64) -> Option<Rank>,
    config: &Config,
    locations: &[Location],
) -> State {
    let mut s = State {
        config: config.clone(),
        locations: locations.to_vec(),
        ..State::default()
    };
    for e in ordered {
        let role = roles(&e.u.author, Some(&e.u.author_account), e.u.chain_block);
        if let Err(why) = apply(&mut s, e, role) {
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

pub(crate) fn issue_of(b: &Body) -> Option<Id> {
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

fn apply(s: &mut State, e: &Event, role: Option<Rank>) -> Result<(), String> {
    use Body::*;
    use Status::*;
    let a = e.u.author;
    let staff = role.is_some_and(|r| r >= Rank::Steward);

    if let Some(iid) = issue_of(&e.u.body) {
        if let Some(i) = s.issues.get_mut(&iid) {
            i.timeline.push(e.id);
        }
    }
    require(
        role.is_some(),
        format!("not an approved member at block {}", e.u.chain_block),
    )?;

    match &e.u.body {
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
                    acked_ts: None,
                    claimed_ts: None,
                    timeline: vec![e.id],
                },
            );
            Ok(())
        }
        Checkpoint { heads, lez_tx } => {
            s.checkpoints.push(CheckpointRecord {
                by: a,
                heads: heads.clone(),
                lez_tx: lez_tx.clone(),
                ts: e.u.ts,
            });
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
            let resident = role == Some(Rank::Resident);
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
