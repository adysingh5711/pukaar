//! Deterministic reducer: a pure function of the accepted event set.
//! Invalid events are kept but inert, with the reason recorded (nothing is dropped silently).

use crate::event::{Body, Event, Id, Key, Location, NoticeOutcome, Role, OTHER_LOCATION};
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
    /// key -> its latest revoke, while it has no role again since (a grant clears it).
    pub revoked: BTreeMap<Key, Revocation>,
    pub issues: BTreeMap<Id, Issue>,
    pub rejected: BTreeMap<Id, String>,
    pub checkpoints: Vec<CheckpointRecord>,
    /// Every applied location change, in applied order: the admin's change log.
    pub place_log: Vec<PlaceChange>,
    /// At most two, never none; the genesis author is the first. Always admins.
    pub super_admins: BTreeSet<Key>,
    /// subject -> their latest notice, sealed or not, until a role change clears it.
    pub notices: BTreeMap<Key, Notice>,
    /// subject -> every notice sealed on them, oldest first: their demotion history.
    pub demotions: BTreeMap<Key, Vec<Notice>>,
}

/// An admin's notice period: their powers until `deadline` (event time), then `outcome`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    /// The `AdminNotice` event's id.
    pub id: Id,
    pub by: Key,
    pub ts: u64,
    pub outcome: NoticeOutcome,
    pub deadline: u64,
    pub reason: String,
    /// The winning seal, once sealed.
    pub sealed: Option<Seal>,
}

/// The seal that made a notice final.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seal {
    /// The subject's next seq as the sealer saw it (see `Body::NoticeSeal`).
    pub seen_seq: u64,
    pub by: Key,
    pub ts: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaceChangeKind {
    Added,
    Edited,
    Renamed,
    Retired,
    Restored,
    RemovalStarted,
    RemovalUndone,
    /// Not an event: shown once a removal's cooldown is over.
    Removed,
}

/// One change to a place: who, when (event ts), what and why; a rename keeps (from, to).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceChange {
    pub ts: u64,
    pub by: Key,
    pub code: String,
    pub kind: PlaceChangeKind,
    pub reason: String,
    pub rename: Option<(String, String)>,
}

/// How long a removed location can be brought back, in seconds of event time.
// ponytail: judged by the events' `ts` (the author's clock), never a local clock, so every
// replica agrees; a skewed admin clock can shift the window. Anchor-backed time if that matters.
pub const REMOVAL_COOLDOWN: u64 = 30 * 24 * 3600;
const DAY: u64 = 24 * 3600;
const MAX_SUPER_ADMINS: usize = 2;
const ONLY_SUPER: &str = "only a super admin does this";
/// Passes over the log, at most: each re-runs it knowing every seal the last one applied.
// ponytail: a backdated act can change which seal wins only through the sealer's membership,
// so two passes settle every honest log; one that keeps flipping stops at the cap (still the
// same on every replica: it's a function of the event set).
const MAX_PASSES: usize = 4;

/// A member's role taken away: by which admin, when (event ts) and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revocation {
    pub by: Key,
    pub ts: u64,
    pub reason: String,
}

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
    fn admin_on_location(&self, admin: bool, code: &str, verb: &str) -> Result<(), String> {
        require(admin, format!("only admin {verb} locations"))?;
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
        admin: bool,
        code: &str,
        retire: bool,
        reason: &str,
    ) -> Result<(), String> {
        self.admin_on_location(admin, code, "retires")?;
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
    fn check_locations_add(&self, admin: bool, locations: &[Location]) -> Result<(), String> {
        require(admin, "only admin edits the site map")?;
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
        self.location_rule(self.role_at(by, ts) == Some(Role::Admin), body, ts)
    }

    fn location_rule(&self, admin: bool, body: &Body, ts: u64) -> Result<(), String> {
        match body {
            Body::LocationsAdd { locations } => self.check_locations_add(admin, locations),
            Body::LocationRetire {
                code,
                retired,
                reason,
            } => self.check_location_change(admin, code, *retired, reason),
            Body::LocationEdit { code, label, group } => {
                self.admin_on_location(admin, code, "edits")?;
                self.not_pending_removal(code)?;
                require(
                    !label.trim().is_empty() && !group.trim().is_empty(),
                    "a location needs a name and a group",
                )
            }
            Body::LocationRemove { code, undo, reason } => {
                self.admin_on_location(admin, code, "removes")?;
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

    /// `k`'s role, where `ended` says whether their unsealed notice is over.
    fn judged_role(&self, k: &Key, ended: impl FnOnce(&Notice) -> bool) -> Option<Role> {
        match self.notices.get(k) {
            Some(n) if n.sealed.is_none() && ended(n) => n.outcome.role(),
            _ => self.roles.get(k).copied(),
        }
    }

    /// `k`'s role for an act at `ts`: a notice whose deadline has passed counts before its seal.
    #[must_use]
    pub fn role_at(&self, k: &Key, ts: u64) -> Option<Role> {
        self.judged_role(k, |n| ts >= n.deadline)
    }

    /// notice id -> `seen_seq`, for every seal applied.
    fn seals(&self) -> BTreeMap<Id, u64> {
        self.demotions
            .values()
            .flatten()
            .filter_map(|n| Some((n.id, n.sealed?.seen_seq)))
            .collect()
    }

    /// A super admin's own role is nobody's to change, and the admin role (giving it, or
    /// changing an admin's) is the super admins' alone.
    fn check_role_change(&self, sup: bool, subject: &Key, to_admin: bool) -> Result<(), String> {
        require(
            sup || !(to_admin || self.roles.get(subject) == Some(&Role::Admin)),
            "only a super admin grants or changes the admin role",
        )
    }

    fn check_new_super(&self, k: &Key) -> Result<(), String> {
        require(
            self.roles.contains_key(k) && self.names.contains_key(k),
            "a super admin must be a named member",
        )?;
        require(!self.super_admins.contains(k), "already a super admin")
    }

    fn make_super(&mut self, k: Key) {
        self.super_admins.insert(k);
        self.roles.insert(k, Role::Admin);
        self.notices.remove(&k);
    }

    fn open_notice(&mut self, k: &Key) -> Result<&mut Notice, String> {
        self.notices
            .get_mut(k)
            .filter(|n| n.sealed.is_none())
            .ok_or_else(|| "no open notice for this member".into())
    }

    /// Make `subject`'s open notice final: its outcome for good, and a line in their history.
    fn seal(&mut self, subject: Key, seal: Seal) {
        let Ok(n) = self.open_notice(&subject) else {
            return;
        };
        n.sealed = Some(seal);
        let n = n.clone();
        match n.outcome.role() {
            Some(r) => {
                self.roles.insert(subject, r);
            }
            None => {
                self.roles.remove(&subject);
                let r = Revocation {
                    by: n.by,
                    ts: n.deadline,
                    reason: n.reason.clone(),
                };
                self.revoked.insert(subject, r);
            }
        }
        self.demotions.entry(subject).or_default().push(n);
    }

    /// Apply a location change `check_location_body` accepted, and log it.
    fn apply_location_body(&mut self, by: Key, body: &Body, ts: u64) {
        use PlaceChangeKind as K;
        let change = |code: &str, kind, reason: &str| PlaceChange {
            ts,
            by,
            code: code.to_owned(),
            kind,
            reason: reason.to_owned(),
            rename: None,
        };
        match body {
            Body::LocationRetire {
                code,
                retired: true,
                reason,
            } => {
                self.retired.insert(code.clone(), reason.clone());
                self.place_log.push(change(code, K::Retired, reason));
            }
            Body::LocationRetire { code, reason, .. }
            | Body::LocationRemove {
                code,
                undo: true,
                reason,
            } => {
                self.retired.remove(code);
                self.pending_removal.remove(code);
                let kind = if matches!(body, Body::LocationRetire { .. }) {
                    K::Restored
                } else {
                    K::RemovalUndone
                };
                self.place_log.push(change(code, kind, reason));
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
                self.place_log.push(change(code, K::RemovalStarted, reason));
            }
            Body::LocationsAdd { locations } => {
                self.locations.extend_from_slice(locations);
                self.place_log
                    .extend(locations.iter().map(|l| change(&l.code, K::Added, "")));
            }
            Body::LocationEdit { code, label, group } => {
                let mut c = change(code, K::Edited, "");
                if let Some(l) = self.locations.iter_mut().find(|l| &l.code == code) {
                    if &l.label != label {
                        let old = std::mem::replace(&mut l.label, label.clone());
                        self.renamed_from.insert(code.clone(), old.clone());
                        c.kind = K::Renamed;
                        c.rename = Some((old, label.clone()));
                    }
                    l.group.clone_from(group);
                }
                self.place_log.push(c);
            }
            _ => {}
        }
    }
}

impl Seal {
    fn new(seen_seq: u64, e: &Event) -> Self {
        Self {
            seen_seq,
            by: e.u.author,
            ts: e.u.ts,
        }
    }
}

/// The state after `ordered`. A sealed notice reaches back: the subject's events from the
/// seal's `seen_seq` on are judged with its outcome even when they order before the seal (a
/// low lamport and an old clock), so the log is re-run knowing the seals until they settle.
#[must_use]
pub fn reduce<'a>(ordered: impl IntoIterator<Item = &'a Event>) -> State {
    let events: Vec<&Event> = ordered.into_iter().collect();
    let mut known = BTreeMap::new();
    let mut s = reduce_once(&events, &known);
    for _ in 1..MAX_PASSES {
        let seals = s.seals();
        if seals == known {
            break;
        }
        known = seals;
        s = reduce_once(&events, &known);
    }
    s
}

fn reduce_once(events: &[&Event], seals: &BTreeMap<Id, u64>) -> State {
    let mut s = State::default();
    for e in events {
        if let Err(why) = apply(&mut s, e, seals) {
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

/// `seals`: notice id -> `seen_seq` of every seal known so far (see `reduce`).
fn apply(s: &mut State, e: &Event, seals: &BTreeMap<Id, u64>) -> Result<(), String> {
    use Body::*;
    use Status::*;
    let a = e.u.author;
    let role = s.judged_role(&a, |n| {
        e.u.ts >= n.deadline || seals.get(&n.id).is_some_and(|&seen| e.u.seq >= seen)
    });
    let sup = s.super_admins.contains(&a);
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
            s.make_super(a);
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
            if !s.roles.contains_key(&a) {
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
            require(
                !s.super_admins.contains(subject),
                "a super admin's role cannot be changed",
            )?;
            s.check_role_change(sup, subject, *role == Role::Admin)?;
            // A resident may be named here too (the admin hears them at the kiosk); their own
            // later Profile still replaces or clears it. Unnamed, a resident keeps their own.
            match name.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
                Some(n) => {
                    s.names.insert(*subject, n.to_string());
                }
                None if *role != Role::Resident => return Err("staff must be named".into()),
                None => {}
            }
            s.roles.insert(*subject, *role);
            s.pending_members.remove(subject);
            s.revoked.remove(subject);
            s.notices.remove(subject);
            Ok(())
        }
        RoleRevoke { subject, reason } => {
            require(admin, "only admin revokes roles")?;
            require(!s.super_admins.contains(subject), "admin cannot be revoked")?;
            s.check_role_change(sup, subject, false)?;
            s.roles.remove(subject);
            s.notices.remove(subject);
            s.revoked.insert(
                *subject,
                Revocation {
                    by: a,
                    ts: e.u.ts,
                    reason: reason.clone(),
                },
            );
            Ok(())
        }
        b @ (LocationsAdd { .. }
        | LocationRetire { .. }
        | LocationEdit { .. }
        | LocationRemove { .. }) => {
            s.location_rule(admin, b, e.u.ts)?;
            s.apply_location_body(a, b, e.u.ts);
            Ok(())
        }
        SuperAdminAdd { subject } => {
            require(sup, ONLY_SUPER)?;
            s.check_new_super(subject)?;
            require(
                s.super_admins.len() < MAX_SUPER_ADMINS,
                "there are already two super admins",
            )?;
            s.make_super(*subject);
            Ok(())
        }
        SuperAdminRemove { subject } => {
            require(sup, ONLY_SUPER)?;
            require(*subject != a, "a super admin steps down by transfer")?;
            require(s.super_admins.remove(subject), "not a super admin")
        }
        SuperAdminTransfer { to } => {
            require(sup, ONLY_SUPER)?;
            s.check_new_super(to)?;
            s.super_admins.remove(&a);
            s.make_super(*to);
            Ok(())
        }
        AdminNotice {
            subject,
            outcome,
            deadline,
            reason,
        } => {
            require(sup, ONLY_SUPER)?;
            require(
                s.roles.get(subject) == Some(&Role::Admin) && !s.super_admins.contains(subject),
                "a notice is for an admin who is not a super admin",
            )?;
            require(!reason.trim().is_empty(), "a notice needs a reason")?;
            // A deadline in the past means "now".
            let deadline = (*deadline).max(e.u.ts);
            let n = Notice {
                id: e.id,
                by: a,
                ts: e.u.ts,
                outcome: *outcome,
                deadline,
                reason: reason.clone(),
                sealed: None,
            };
            s.notices.insert(*subject, n);
            Ok(())
        }
        AdminNoticeCancel { subject } => {
            require(sup, ONLY_SUPER)?;
            s.open_notice(subject)?;
            s.notices.remove(subject);
            Ok(())
        }
        AdminNoticeEndNow { subject, seen_seq } => {
            require(sup, ONLY_SUPER)?;
            s.open_notice(subject)?.deadline = e.u.ts;
            s.seal(*subject, Seal::new(*seen_seq, e));
            Ok(())
        }
        NoticeSeal { subject, seen_seq } => {
            require(staff, "only staff can seal a notice")?;
            require(*subject != a, "a member cannot seal their own notice")?;
            let n = s.notices.get(subject).ok_or("no notice for this member")?;
            if n.sealed.is_some() {
                return Ok(()); // the first seal won; a later one changes nothing
            }
            // ponytail: the sealer's clock (event ts), like REMOVAL_COOLDOWN: a fast clock seals
            // early. Anchor-backed time if that matters.
            require(e.u.ts >= n.deadline, "the notice has not ended yet")?;
            s.seal(*subject, Seal::new(*seen_seq, e));
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
