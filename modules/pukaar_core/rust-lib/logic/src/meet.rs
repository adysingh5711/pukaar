//! Meetings between members of one site: one organizer and one to `MAX_INVITEES` invitees (see 1-1-meet.md).
//!
//! A meeting with one invitee is the 1:1 meeting of 1-1-meet.md. With more, the organizer sends one
//! sealed proposal to each invitee, with the whole list. Each invitee answers on their own (accept or
//! decline, signed) and tells the organizer and the other invitees, so everybody sees each answer.
//! The rule, in `settle`: the meeting is confirmed once every invitee answered and at least one accepted;
//! it is declined when all declined; the organizer can cancel at any time. An invitee who accepted can
//! leave: only their own part ends.
//!
//! Each member keeps the meetings in a local file (`meetings-<site>.json`): it is the source of truth.
//! A proposal, an accept, a decline and a cancel are signed by the sender and travel over Delivery
//! inside a sealed box for the other person (`Wire::Meet`), so the site channel does not show who
//! meets whom. The free/busy summary (`Wire::Busy`) is signed and open to all members: times only,
//! never a title. A member who shares nothing sends an empty summary with `shared: false`.
//! Delivery gives no acknowledgement, so `resend` repeats the last message of each open meeting.
//! Times are UTC milliseconds. The backup is the history, encrypted with a key from the Pukaar key.

use crate::event::{sha256, Key};
use crate::lezkey::{open, seal, x25519_public};
use crate::persist::SiteFile;
use crate::profile::{decrypt, encrypt};
use crate::sync::Wire;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MEET_DOMAIN: &[u8] = b"pukaar/meet/v1\0";
const BUSY_DOMAIN: &[u8] = b"pukaar/busy/v1\0";
const BACKUP_INFO: &[u8] = b"pukaar/meet-backup/v1";
const BACKUP_SCHEMA: &str = "pukaar.meetings/1";
/// A sealed message is a signed message (about 250 bytes, more with a description or a list of invitees) plus 48 bytes.
/// A proposal to 8 invitees with the longest description is about 1100 bytes sealed.
pub const MAX_MEET_BYTES: usize = 1280;
/// The busy ranges that one summary holds: the next ones in time.
pub const MAX_RANGES: usize = 64;
/// The longest meeting that a received proposal may have (the invitee side stays as it was).
const MAX_LEN_MS: u64 = 24 * 3_600_000;
/// The length a member can choose for a new meeting, in minutes.
pub const MIN_LEN_MIN: u64 = 5;
pub const MAX_LEN_MIN: u64 = 8 * 60;
/// The longest reminder lead time, in minutes. 0 means no reminder.
pub const MAX_LEAD_MIN: u64 = 24 * 60;
/// The longest description of a meeting. The byte cap keeps a sealed proposal under `MAX_MEET_BYTES` for any script.
pub const MAX_DESC_CHARS: usize = 500;
const MAX_DESC_BYTES: usize = 640;
/// The most people one meeting invites (not counting the organizer).
pub const MAX_INVITEES: usize = 8;
/// Meetings with one person that we keep, so one member cannot fill the disk.
const MAX_PER_PEER: usize = 200;
/// Slots start on this step (UTC).
pub const STEP_MS: u64 = 30 * 60_000;
pub const DEFAULT_LEAD_MIN: u64 = 10;

/// A start and an end, in ms.
pub type Range = (u64, u64);

/// A meeting length in ms, or the refusal in plain words. Takes ms so a start and end pair can use it too.
pub fn check_len_ms(len: u64) -> Result<u64, String> {
    if (MIN_LEN_MIN * 60_000..=MAX_LEN_MIN * 60_000).contains(&len) {
        Ok(len)
    } else {
        Err(format!("A meeting must last {MIN_LEN_MIN} to {MAX_LEN_MIN} minutes."))
    }
}

/// A reminder lead time in minutes (0 = none), or the refusal in plain words.
pub fn check_lead_min(min: u64) -> Result<u64, String> {
    if min <= MAX_LEAD_MIN { Ok(min) } else { Err(format!("The reminder time must be 0 to {MAX_LEAD_MIN} minutes.")) }
}

/// A meeting description (plain text, may be empty), trimmed, or the refusal in plain words.
pub fn check_desc(desc: &str) -> Result<&str, String> {
    let d = desc.trim();
    if d.chars().count() > MAX_DESC_CHARS || d.len() > MAX_DESC_BYTES {
        Err(format!("The description can have up to {MAX_DESC_CHARS} characters."))
    } else if d.chars().any(char::is_control) {
        Err("The description cannot have control characters or line breaks.".into())
    } else {
        Ok(d)
    }
}

fn hit(a: Range, b: Range) -> bool {
    a.0 < b.1 && b.0 < a.1
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Proposed,
    Confirmed,
    Declined,
    Cancelled,
}

/// What one person did to a meeting.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Did {
    Proposed,
    Accepted,
    Declined,
    Cancelled,
}

/// Where one invitee stands: what their last step says.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Answer {
    Pending,
    Accepted,
    Declined,
    /// Accepted first, then left.
    Left,
}

impl From<Did> for Answer {
    fn from(did: Did) -> Self {
        match did {
            Did::Proposed => Answer::Pending,
            Did::Accepted => Answer::Accepted,
            Did::Declined => Answer::Declined,
            Did::Cancelled => Answer::Left,
        }
    }
}

/// The rule for a meeting that the organizer has not cancelled, from the answers of its invitees:
/// somebody pending keeps it proposed; else one accept confirms it; else it is cancelled if somebody left,
/// declined if all declined.
#[must_use]
pub fn settle(answers: impl IntoIterator<Item = Answer>) -> Status {
    let mut seen = [false; 4];
    answers.into_iter().for_each(|a| seen[a as usize] = true);
    let [pending, accepted, _, left] = seen;
    if pending {
        Status::Proposed
    } else if accepted {
        Status::Confirmed
    } else if left {
        Status::Cancelled
    } else {
        Status::Declined
    }
}

/// One entry of the history of a meeting: who did it, and when this device saw it (UTC ms).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub who: Key,
    pub did: Did,
    pub at_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Meeting {
    pub id: Key,
    pub requester: Key,
    /// The first invitee: the only one of a 1:1 meeting.
    pub invitee: Key,
    /// The other invitees of a group meeting, after `invitee`. Empty for rows from before groups.
    #[serde(default)]
    pub also: Vec<Key>,
    pub start_ms: u64,
    pub end_ms: u64,
    pub status: Status,
    /// Who cancelled it: the one who repeats the cancel until the meeting is over.
    #[serde(default)]
    pub cancelled_by: Option<Key>,
    /// The invitee's signed accept message (postcard of `Msg`): proof that the meeting happened.
    #[serde(default)]
    pub accept: Option<Vec<u8>>,
    /// The signed accepts after the first one, of a group meeting.
    #[serde(default)]
    pub later_accepts: Vec<Vec<u8>>,
    /// What the requester wrote about the meeting (plain text, may be empty).
    #[serde(default)]
    pub desc: String,
    /// Who proposed, accepted, declined or cancelled, and when. Rows from before it existed have none: see `steps`.
    #[serde(default)]
    pub log: Vec<Step>,
}

impl Meeting {
    fn range(&self) -> Range {
        (self.start_ms, self.end_ms)
    }

    /// The invitees, in order.
    pub fn invited(&self) -> impl Iterator<Item = &Key> {
        std::iter::once(&self.invitee).chain(&self.also)
    }

    fn has(&self, who: &Key) -> bool {
        self.requester == *who || self.invited().any(|i| i == who)
    }

    /// The person across from `me` in a 1:1 meeting: the organizer, or the first invitee.
    fn other(&self, me: &Key) -> Key {
        if self.requester == *me { self.invitee } else { self.requester }
    }

    /// Everybody in the meeting but `me`: who our messages about it go to.
    fn peers(&self, me: &Key) -> Vec<Key> {
        std::iter::once(&self.requester).chain(self.invited()).filter(|k| *k != me).copied().collect()
    }

    /// Where `who` (an invitee) stands.
    fn answer(&self, who: &Key) -> Answer {
        self.steps().iter().rev().find(|s| s.who == *who).map_or(Answer::Pending, |s| s.did.into())
    }

    /// Not declined, cancelled or over: it still counts.
    fn live(&self) -> bool {
        matches!(self.status, Status::Proposed | Status::Confirmed)
    }

    /// Does the meeting make `me` busy? The organizer while it is live; an invitee who accepted while it is live.
    fn holds(&self, me: &Key) -> bool {
        self.live() && (self.requester == *me || self.answer(me) == Answer::Accepted)
    }

    /// Add a step. A row from before the log gets its history first, so it is not lost.
    fn log_step(&mut self, step: Step) {
        if self.log.is_empty() {
            self.log = self.steps().into_owned();
        }
        self.log.push(step);
    }

    /// The message that `me` repeats for this meeting, and who gets it. The organizer repeats the proposal to
    /// those who have not answered, or the cancel to everybody; an invitee repeats their last answer to all.
    fn last_act(&self, me: &Key) -> Option<(Kind, Vec<Key>)> {
        let id = self.id;
        if self.requester == *me {
            let all: Vec<Key> = self.invited().copied().collect();
            return match self.status {
                Status::Cancelled if self.cancelled_by == Some(*me) => Some((Kind::Cancel { id }, all)),
                Status::Proposed | Status::Confirmed => {
                    let waiting: Vec<Key> = all.iter().filter(|i| self.answer(i) == Answer::Pending).copied().collect();
                    (!waiting.is_empty()).then(|| (Kind::propose(id, self.start_ms, self.end_ms, &self.desc, &all), waiting))
                }
                _ => None,
            };
        }
        let kind = match self.answer(me) {
            Answer::Accepted if self.live() => Kind::Accept { id },
            Answer::Declined => Kind::Decline { id },
            Answer::Left => Kind::Cancel { id },
            _ => return None,
        };
        Some((kind, self.peers(me)))
    }

    /// The history of the meeting. A row saved before the log existed gets its actors from the status and
    /// its fields, with the time 0 (unknown).
    fn steps(&self) -> Cow<'_, [Step]> {
        if !self.log.is_empty() {
            return Cow::Borrowed(&self.log);
        }
        let step = |who, did| Step { who, did, at_ms: 0 };
        let mut v = vec![step(self.requester, Did::Proposed)];
        if self.accept.is_some() {
            v.push(step(self.invitee, Did::Accepted));
        }
        match (self.status, self.cancelled_by) {
            (Status::Declined, _) => v.push(step(self.invitee, Did::Declined)),
            (Status::Cancelled, Some(who)) => v.push(step(who, Did::Cancelled)),
            _ => {}
        }
        Cow::Owned(v)
    }

    /// The status the member sees: a proposal that nobody accepted before its end is "unaccepted".
    fn shown(&self, now: u64) -> &'static str {
        match self.status {
            Status::Proposed if self.end_ms <= now => "unaccepted",
            Status::Proposed => "proposed",
            Status::Confirmed => "confirmed",
            Status::Declined => "declined",
            Status::Cancelled => "cancelled",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Propose { id: Key, start_ms: u64, end_ms: u64 },
    Accept { id: Key },
    Decline { id: Key },
    Cancel { id: Key },
    /// A proposal with a description. Added after the others, so older messages still read the same.
    ProposeNote { id: Key, start_ms: u64, end_ms: u64, desc: String },
    /// A proposal to 2 or more invitees, with all of them. Added after the others too.
    ProposeGroup { id: Key, start_ms: u64, end_ms: u64, desc: String, invitees: Vec<Key> },
}

/// A proposal of any form. `guests` is empty for the two older forms: the one invitee is the receiver.
struct Prop<'a> {
    id: &'a Key,
    start_ms: u64,
    end_ms: u64,
    desc: &'a str,
    guests: &'a [Key],
}

impl Prop<'_> {
    /// A group proposal has 2 to `MAX_INVITEES` different invitees, not the sender, and the receiver is one.
    fn guests_ok(&self, from: &Key, to: &Key) -> bool {
        self.guests.is_empty() || ((2..=MAX_INVITEES).contains(&self.guests.len()) && self.guests.contains(to) && distinct(from, self.guests))
    }
}

/// `list` has each person once, and not `me`.
fn distinct(me: &Key, list: &[Key]) -> bool {
    !list.contains(me) && list.iter().enumerate().all(|(i, k)| !list[..i].contains(k))
}

impl Kind {
    /// A proposal to `to`: the old plain one for one person without a description, and the note one for one
    /// person with a description, so an older app can still read them.
    fn propose(id: Key, start_ms: u64, end_ms: u64, desc: &str, to: &[Key]) -> Kind {
        let desc = desc.to_owned();
        match to {
            [_, _, ..] => Kind::ProposeGroup { id, start_ms, end_ms, desc, invitees: to.to_vec() },
            _ if desc.is_empty() => Kind::Propose { id, start_ms, end_ms },
            _ => Kind::ProposeNote { id, start_ms, end_ms, desc },
        }
    }

    /// The parts of a proposal of any form.
    fn proposal(&self) -> Option<Prop<'_>> {
        let prop = |id, start_ms, end_ms, desc, guests| Some(Prop { id, start_ms, end_ms, desc, guests });
        match self {
            Kind::Propose { id, start_ms, end_ms } => prop(id, *start_ms, *end_ms, "", &[]),
            Kind::ProposeNote { id, start_ms, end_ms, desc } => prop(id, *start_ms, *end_ms, desc, &[]),
            Kind::ProposeGroup { id, start_ms, end_ms, desc, invitees } => prop(id, *start_ms, *end_ms, desc, invitees),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Body {
    pub from: Key,
    pub to: Key,
    pub kind: Kind,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BusyBody {
    pub from: Key,
    /// Newest wins. The sender's clock.
    pub ts: u64,
    pub shared: bool,
    pub ranges: Vec<Range>,
}

/// A body and the ed25519 signature (64 bytes) of its sender over domain, site and body.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Signed<T> {
    pub body: T,
    pub sig: Vec<u8>,
}

pub type Msg = Signed<Body>;

impl<T: Serialize> Signed<T> {
    fn message(domain: &[u8], site: &Key, body: &T) -> Vec<u8> {
        [domain, &site[..], &postcard::to_allocvec(body).expect("postcard encode")].concat()
    }

    pub(crate) fn new(key: &SigningKey, domain: &[u8], site: &Key, body: T) -> Self {
        let sig = key.sign(&Self::message(domain, site, &body)).to_bytes().to_vec();
        Signed { body, sig }
    }

    pub(crate) fn check(&self, domain: &[u8], site: &Key, from: &Key) -> bool {
        let (Ok(vk), Ok(sig)) = (VerifyingKey::from_bytes(from), <[u8; 64]>::try_from(&self.sig[..])) else { return false };
        vk.verify_strict(&Self::message(domain, site, &self.body), &ed25519_dalek::Signature::from_bytes(&sig)).is_ok()
    }
}

/// What a member can do with a meeting (`Meetings::answer`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Accept,
    Decline,
    Cancel,
}

/// The newest summary of one member.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Peer {
    who: Key,
    ts: u64,
    shared: bool,
    ranges: Vec<Range>,
}

/// What a time range runs into: our own meetings, and the busy ranges of the other person.
pub struct Clashes<'a> {
    pub mine: Vec<&'a Meeting>,
    pub theirs: Vec<Range>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Slot {
    pub range: Range,
    pub mine: bool,
    pub theirs: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Meetings {
    pub list: Vec<Meeting>,
    peers: Vec<Peer>,
    /// Share our free/busy summary with the site. Default on: it holds times only.
    pub share: bool,
    pub lead_min: u64,
    busy_ts: u64,
    #[serde(default)]
    pub backup_cid: Option<String>,
    #[serde(default)]
    pub backup_at_ms: u64,
    #[serde(skip)]
    pub site: Key,
    #[serde(skip)]
    dir: PathBuf,
    /// A Storage job that runs now ("backing up" or "restoring"), and the last error of one.
    #[serde(skip)]
    pub job: Option<&'static str>,
    #[serde(skip)]
    pub job_error: Option<String>,
}

impl SiteFile for Meetings {
    const PREFIX: &'static str = "meetings";

    fn bind(&mut self, dir: PathBuf, site: Key) {
        (self.dir, self.site) = (dir, site);
    }

    fn site(&self) -> Key {
        self.site
    }

    fn folder(&self) -> &Path {
        &self.dir
    }
}

impl Default for Meetings {
    fn default() -> Self {
        Meetings {
            list: vec![],
            peers: vec![],
            share: true,
            lead_min: DEFAULT_LEAD_MIN,
            busy_ts: 0,
            backup_cid: None,
            backup_at_ms: 0,
            site: [0; 32],
            dir: PathBuf::new(),
            job: None,
            job_error: None,
        }
    }
}

fn id_hex(id: &Key) -> String {
    hex::encode(id)
}

impl Meetings {
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn find(&self, id: &Key) -> Option<&Meeting> {
        self.list.iter().find(|m| m.id == *id)
    }

    /// The meetings that make `me` busy: our own that are live, and those that we accepted (`Meeting::holds`).
    fn busy_meetings(&self, me: &Key) -> impl Iterator<Item = &Meeting> + '_ {
        let me = *me;
        self.list.iter().filter(move |m| m.holds(&me))
    }

    /// Our busy ranges that are not over yet, the next `MAX_RANGES`.
    fn my_ranges(&self, me: &Key, now: u64) -> Vec<Range> {
        let mut r: Vec<Range> = self.busy_meetings(me).filter(|m| m.end_ms > now).map(Meeting::range).collect();
        r.sort_unstable();
        r.truncate(MAX_RANGES);
        r
    }

    /// The busy ranges of `who`, or `None` when they share nothing (or we have not heard).
    #[must_use]
    pub fn peer_ranges(&self, who: &Key) -> Option<&[Range]> {
        self.peers.iter().find(|p| p.who == *who && p.shared).map(|p| &p.ranges[..])
    }

    /// What `r` runs into. `skip` leaves one meeting out (a proposal that we judge against the rest).
    #[must_use]
    pub fn clashes(&self, me: &Key, other: Option<&Key>, r: Range, skip: Option<&Key>) -> Clashes<'_> {
        let skip = skip.copied();
        Clashes {
            mine: self.busy_meetings(me).filter(|m| Some(m.id) != skip && hit(m.range(), r)).collect(),
            theirs: other.and_then(|o| self.peer_ranges(o)).map_or_else(Vec::new, |p| p.iter().copied().filter(|b| hit(*b, r)).collect()),
        }
    }

    /// `n` slots of `len_ms`, one every `STEP_MS` from the first step at or after `from_ms`.
    #[must_use]
    pub fn slots(&self, me: &Key, other: &Key, from_ms: u64, len_ms: u64, n: u64) -> Vec<Slot> {
        let first = from_ms.div_ceil(STEP_MS) * STEP_MS;
        (0..n)
            .map(|i| {
                let range = (first + i * STEP_MS, first + i * STEP_MS + len_ms);
                let c = self.clashes(me, Some(other), range, None);
                Slot { range, mine: !c.mine.is_empty(), theirs: !c.theirs.is_empty() }
            })
            .collect()
    }

    /// The one place where a message changes a meeting, ours or a peer's. `m` was checked for its
    /// signature. The reason of a refusal is plain text for the user.
    fn apply(&mut self, m: &Msg, now: u64) -> Result<(), &'static str> {
        let Body { from, to, kind } = &m.body;
        let step = |did| Step { who: *from, did, at_ms: now };
        if let Some(p) = kind.proposal() {
            let ok = p.end_ms > p.start_ms && p.end_ms - p.start_ms <= MAX_LEN_MS && from != to && check_desc(p.desc).is_ok() && p.guests_ok(from, to);
            if !ok || self.find(p.id).is_some() {
                return Err("This meeting time is not valid.");
            }
            if self.list.iter().filter(|x| x.has(from)).count() >= MAX_PER_PEER {
                return Err("There are too many meetings with this person.");
            }
            let (invitee, also) = match p.guests {
                [first, rest @ ..] => (*first, rest.to_vec()),
                [] => (*to, vec![]),
            };
            self.list.push(Meeting {
                id: *p.id,
                requester: *from,
                invitee,
                also,
                start_ms: p.start_ms,
                end_ms: p.end_ms,
                status: Status::Proposed,
                cancelled_by: None,
                accept: None,
                later_accepts: vec![],
                desc: p.desc.to_owned(),
                log: vec![step(Did::Proposed)],
            });
            return Ok(());
        }
        let (Kind::Accept { id } | Kind::Decline { id } | Kind::Cancel { id }) = kind else { unreachable!("a proposal is handled above") };
        let Some(mt) = self.list.iter_mut().find(|x| x.id == *id) else { return Err("No such meeting.") };
        if from == to || !mt.has(from) || !mt.has(to) {
            return Err("This is not your meeting.");
        }
        let organizer = mt.requester == *from;
        let answer = mt.answer(from);
        let waiting = mt.status == Status::Proposed && answer == Answer::Pending;
        let did = match (kind, organizer) {
            (Kind::Accept { .. }, false) if waiting => Did::Accepted,
            (Kind::Decline { .. }, false) if waiting => Did::Declined,
            // The organizer withdraws the meeting for everybody; an invitee who accepted leaves, and only their part ends.
            (Kind::Cancel { .. }, true) if mt.live() => Did::Cancelled,
            (Kind::Cancel { .. }, false) if mt.live() && answer == Answer::Accepted => Did::Cancelled,
            _ => return Err("This meeting cannot change that way now."),
        };
        mt.log_step(step(did));
        if did == Did::Accepted {
            let proof = postcard::to_allocvec(m).expect("postcard encode");
            match mt.accept {
                None => mt.accept = Some(proof),
                Some(_) => mt.later_accepts.push(proof),
            }
        }
        if did == Did::Cancelled {
            mt.cancelled_by = Some(*from);
        }
        mt.status = if organizer { Status::Cancelled } else { settle(mt.invited().map(|i| mt.answer(i))) };
        Ok(())
    }

    /// Run `f`; when it changed a meeting, save, and give the new summary if our busy ranges changed.
    /// `Err` is the refusal of `f`.
    fn change(&mut self, key: &SigningKey, now: u64, f: impl FnOnce(&mut Self) -> Result<(), &'static str>) -> Result<Vec<Vec<u8>>, &'static str> {
        let me = key.verifying_key().to_bytes();
        let before = self.my_ranges(&me, now);
        f(self)?;
        self.save();
        Ok(if before == self.my_ranges(&me, now) { vec![] } else { vec![self.publish_busy(key, now)] })
    }

    /// A signed message for `to`, in a sealed box that only `to` can open.
    fn sealed(key: &SigningKey, site: &Key, to: Key, kind: Kind) -> Option<(Msg, Vec<u8>)> {
        let msg = Signed::new(key, MEET_DOMAIN, site, Body { from: key.verifying_key().to_bytes(), to, kind });
        let boxed = seal(&x25519_public(&VerifyingKey::from_bytes(&to).ok()?), &postcard::to_allocvec(&msg).expect("postcard encode"));
        Some((msg, Wire::Meet(boxed).encode()))
    }

    /// Our own action: apply it, then give the wire messages to send (one sealed message for each of `to`, and a new summary).
    fn send(&mut self, key: &SigningKey, site: &Key, to: &[Key], kind: &Kind, now: u64) -> Result<Vec<Vec<u8>>, String> {
        let sealed = to.iter().map(|t| Self::sealed(key, site, *t, kind.clone())).collect::<Option<Vec<_>>>().ok_or("That person's key is not valid.")?;
        let first = &sealed.first().ok_or("Nobody is invited.")?.0;
        let mut out = self.change(key, now, |s| s.apply(first, now))?;
        out.splice(0..0, sealed.into_iter().map(|(_, wire)| wire));
        Ok(out)
    }

    /// Propose a meeting to `to` (1 to `MAX_INVITEES` people). It does not matter if the time clashes: the app only warns.
    /// Gives the id and the messages to send.
    /// `desc` is the optional description (`check_desc`).
    pub fn propose(&mut self, key: &SigningKey, site: &Key, to: &[Key], range: Range, desc: &str, now: u64) -> Result<(Key, Vec<Vec<u8>>), String> {
        if range.1 <= range.0 {
            return Err("The meeting must end after it starts.".into());
        }
        if range.0 < now {
            return Err("That start time has already passed.".into());
        }
        check_len_ms(range.1 - range.0)?;
        let desc = check_desc(desc)?;
        let me = key.verifying_key().to_bytes();
        if to.is_empty() || to.len() > MAX_INVITEES || !distinct(&me, to) {
            return Err(format!("Invite 1 to {MAX_INVITEES} other people, each once."));
        }
        let stamp = [&me[..], &to.concat(), &range.0.to_be_bytes(), &range.1.to_be_bytes(), &now.to_be_bytes()].concat();
        let id = sha256(&[MEET_DOMAIN, &stamp].concat());
        Ok((id, self.send(key, site, to, &Kind::propose(id, range.0, range.1, desc, to), now)?))
    }

    /// Accept, decline or cancel the meeting `id`: the others in it get the answer. The invitee cannot accept after the
    /// meeting ended. An invitee who cancels leaves.
    pub fn answer(&mut self, key: &SigningKey, site: &Key, id: &Key, act: Act, now: u64) -> Result<Vec<Vec<u8>>, String> {
        let m = self.find(id).ok_or("No such meeting.")?;
        if act == Act::Accept && m.end_ms <= now {
            return Err("This meeting is over, so it cannot be accepted.".into());
        }
        let to = m.peers(&key.verifying_key().to_bytes());
        let kind = match act {
            Act::Accept => Kind::Accept { id: *id },
            Act::Decline => Kind::Decline { id: *id },
            Act::Cancel => Kind::Cancel { id: *id },
        };
        self.send(key, site, &to, &kind, now)
    }

    /// A `Wire::Meet` from Delivery. Only a message for us, from a member, with a good signature counts.
    /// Gives the summary to send when our busy ranges changed.
    pub fn on_meet(&mut self, key: &SigningKey, site: &Key, sealed: &[u8], members: &[Key], now: u64) -> Vec<Vec<u8>> {
        let Some(m) = open(key, sealed).and_then(|p| postcard::from_bytes::<Msg>(&p).ok()) else { return vec![] };
        let guests = m.body.kind.proposal().is_none_or(|p| p.guests.iter().all(|g| members.contains(g)));
        let ok = m.body.to == key.verifying_key().to_bytes() && members.contains(&m.body.from) && guests && m.check(MEET_DOMAIN, site, &m.body.from);
        if !ok {
            return vec![];
        }
        self.change(key, now, |s| s.apply(&m, now)).unwrap_or_default()
    }

    /// A `Wire::Busy` from Delivery. True when it was news.
    pub fn on_busy(&mut self, site: &Key, b: &Signed<BusyBody>, members: &[Key]) -> bool {
        let BusyBody { from, ts, shared, ranges } = &b.body;
        let ok = members.contains(from) && ranges.len() <= MAX_RANGES && b.check(BUSY_DOMAIN, site, from);
        let held = self.peers.iter().position(|p| p.who == *from);
        if !ok || held.is_some_and(|i| self.peers[i].ts >= *ts) {
            return false;
        }
        let peer = Peer { who: *from, ts: *ts, shared: *shared, ranges: if *shared { ranges.clone() } else { vec![] } };
        match held {
            Some(i) => self.peers[i] = peer,
            None => self.peers.push(peer),
        }
        self.save();
        true
    }

    /// Our summary at the number `busy_ts`: the ranges while we share, else an empty one that says so.
    fn busy_wire(&self, key: &SigningKey, site: &Key, now: u64) -> Vec<u8> {
        let from = key.verifying_key().to_bytes();
        let ranges = if self.share { self.my_ranges(&from, now) } else { vec![] };
        Wire::Busy(Signed::new(key, BUSY_DOMAIN, site, BusyBody { from, ts: self.busy_ts, shared: self.share, ranges })).encode()
    }

    /// A new summary: its number is above every one that we sent.
    fn publish_busy(&mut self, key: &SigningKey, now: u64) -> Vec<u8> {
        self.busy_ts = now.max(self.busy_ts + 1);
        self.save();
        self.busy_wire(key, &self.site, now)
    }

    /// Repeat our last message of each meeting that is not over, and our summary. Delivery gives no
    /// acknowledgement, and a peer may have been offline. The receiver ignores a repeat.
    // ponytail: one message per open meeting on each round (every 5 minutes). Acknowledgements, if volume matters.
    #[must_use]
    pub fn resend(&self, key: &SigningKey, site: &Key, now: u64) -> Vec<Vec<u8>> {
        let me = key.verifying_key().to_bytes();
        let own = self.list.iter().filter(|m| m.end_ms > now).filter_map(|m| m.last_act(&me)).flat_map(|(kind, to)| {
            to.into_iter().filter_map(move |t| Self::sealed(key, site, t, kind.clone()).map(|(_, wire)| wire))
        });
        own.chain((self.busy_ts > 0).then(|| self.busy_wire(key, site, now))).collect()
    }

    /// Change what we share and how early the reminder comes (0 = no reminder). Gives the summary to send when sharing changed.
    pub fn set_prefs(&mut self, key: &SigningKey, share: bool, lead_min: u64, now: u64) -> Vec<Vec<u8>> {
        self.lead_min = lead_min.min(MAX_LEAD_MIN);
        let changed = self.share != share;
        self.share = share;
        self.save();
        if changed { vec![self.publish_busy(key, now)] } else { vec![] }
    }

    // ---- backup (feature 8) ----

    fn backup_key(key: &SigningKey) -> Key {
        let mut k = [0u8; 32];
        Hkdf::<Sha256>::new(None, &key.to_bytes()).expand(BACKUP_INFO, &mut k).expect("32 bytes fit HKDF-SHA256");
        k
    }

    /// The history, encrypted for the owner of `key`: the bytes that go to Storage.
    #[must_use]
    pub fn backup_bytes(&self, key: &SigningKey) -> Vec<u8> {
        let plain = json!({ "schema": BACKUP_SCHEMA, "site": hex::encode(self.site), "list": self.list });
        encrypt(&Self::backup_key(key), plain.to_string().as_bytes())
    }

    /// Add the meetings of a backup that we do not hold. A meeting that we hold stays as it is.
    /// Gives how many were added.
    pub fn restore(&mut self, key: &SigningKey, bytes: &[u8]) -> Result<usize, String> {
        #[derive(Deserialize)]
        struct Backup {
            schema: String,
            site: String,
            list: Vec<Meeting>,
        }
        let b: Backup = serde_json::from_slice(&decrypt(&Self::backup_key(key), bytes)?).map_err(|_| "This is not a meetings backup.")?;
        if b.schema != BACKUP_SCHEMA || b.site != hex::encode(self.site) {
            return Err("This backup is for another site.".into());
        }
        let me = key.verifying_key().to_bytes();
        let before = self.list.len();
        for m in b.list {
            let ok = m.has(&me) && m.requester != m.invitee && m.end_ms > m.start_ms && self.find(&m.id).is_none();
            if ok {
                self.list.push(m);
            }
        }
        self.save();
        Ok(self.list.len() - before)
    }

    /// The upload ended: keep the CID, or the reason.
    pub fn backup_done(&mut self, r: Result<String, String>, now: u64) {
        self.job = None;
        match r {
            Ok(cid) => (self.backup_cid, self.backup_at_ms, self.job_error) = (Some(cid), now, None),
            Err(e) => self.job_error = Some(format!("The backup was skipped. Storage could not be reached: {e}")),
        }
        self.save();
    }

    /// The download ended: add the meetings, or keep the reason.
    pub fn restore_done(&mut self, key: &SigningKey, r: Result<Vec<u8>, String>) {
        self.job = None;
        self.job_error = r.and_then(|b| self.restore(key, &b)).err();
    }

    // ---- JSON for the module ----

    fn who_json(who: &Key, names: &BTreeMap<Key, String>) -> Value {
        json!({ "with": id_hex(who), "name": names.get(who) })
    }

    /// One history entry: who (and whether that is us), what, and when this device saw it (0 = unknown).
    fn step_json(s: &Step, me: &Key, names: &BTreeMap<Key, String>) -> Value {
        let mut v = Self::who_json(&s.who, names);
        (v["you"], v["did"], v["at_ms"]) = ((s.who == *me).into(), json!(s.did), s.at_ms.into());
        v
    }

    /// One person of a meeting: key, name, and whether that is us.
    fn person_json(who: &Key, me: &Key, names: &BTreeMap<Key, String>) -> Value {
        let mut v = Self::who_json(who, names);
        v["you"] = (who == me).into();
        v
    }

    fn range_json((start_ms, end_ms): Range) -> Value {
        json!({ "start_ms": start_ms, "end_ms": end_ms })
    }

    /// The clashes, with the name of the person of each of our own meetings.
    #[must_use]
    pub fn clashes_json(&self, me: &Key, c: &Clashes, names: &BTreeMap<Key, String>) -> Value {
        let mine = c.mine.iter().map(|m| {
            let mut v = Self::who_json(&m.other(me), names);
            (v["start_ms"], v["end_ms"], v["status"]) = (m.start_ms.into(), m.end_ms.into(), m.shown(0).into());
            v
        });
        json!({ "mine": mine.collect::<Vec<_>>(), "theirs": c.theirs.iter().map(|r| Self::range_json(*r)).collect::<Vec<_>>() })
    }

    /// Every meeting by start time, with the clashes of each open one, and the settings and the backup.
    #[must_use]
    pub fn list_json(&self, me: &Key, names: &BTreeMap<Key, String>, now: u64) -> Value {
        let mut sorted: Vec<&Meeting> = self.list.iter().collect();
        sorted.sort_by_key(|m| (m.start_ms, m.id));
        let rows = sorted.iter().map(|m| {
            let open = m.status == Status::Proposed && m.end_ms > now;
            let mut v = Self::who_json(&m.other(me), names);
            v["id"] = id_hex(&m.id).into();
            v["you"] = if m.requester == *me { "requester" } else { "invitee" }.into();
            (v["start_ms"], v["end_ms"], v["status"]) = (m.start_ms.into(), m.end_ms.into(), m.shown(now).into());
            v["cancelled_by_you"] = (m.cancelled_by == Some(*me)).into();
            v["signed"] = m.accept.is_some().into();
            v["desc"] = m.desc.as_str().into();
            v["organizer"] = Self::person_json(&m.requester, me, names);
            v["invitees"] = m
                .invited()
                .map(|i| {
                    let mut p = Self::person_json(i, me, names);
                    p["state"] = json!(m.answer(i));
                    p
                })
                .collect();
            // Our own answer, for an invitee: it says whether to show Accept and Decline, or Leave.
            v["my_answer"] = if m.requester == *me { Value::Null } else { json!(m.answer(me)) };
            v["log"] = m.steps().iter().map(|s| Self::step_json(s, me, names)).collect();
            // A proposal that waits for us (or for them) shows what it runs into in our calendar.
            v["clashes"] = if open { self.clashes_json(me, &self.clashes(me, None, m.range(), Some(&m.id)), names) } else { Value::Null };
            v
        });
        json!({
            "meetings": rows.collect::<Vec<_>>(),
            "share": self.share,
            "lead_min": self.lead_min,
            "backup": { "cid": self.backup_cid, "at_ms": self.backup_at_ms, "job": self.job, "error": self.job_error },
        })
    }

    /// The free/busy view for a proposal to `other`: our ranges, theirs when they share, and the slots.
    #[must_use]
    pub fn slots_json(&self, me: &Key, other: &Key, from_ms: u64, len_ms: u64, names: &BTreeMap<Key, String>, now: u64) -> Value {
        let slots = self.slots(me, other, from_ms, len_ms, 7 * 48).into_iter().map(|s| {
            let mut v = Self::range_json(s.range);
            (v["mine"], v["theirs"]) = (s.mine.into(), s.theirs.into());
            v
        });
        let mine = self.busy_meetings(me).filter(|m| m.end_ms > now).map(|m| {
            let mut v = Self::range_json(m.range());
            v["name"] = json!(names.get(&m.other(me)));
            v
        });
        json!({
            "shared": self.peer_ranges(other).is_some(),
            "mine": mine.collect::<Vec<_>>(),
            "theirs": self.peer_ranges(other).unwrap_or_default().iter().map(|r| Self::range_json(*r)).collect::<Vec<_>>(),
            "slots": slots.collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SITE: Key = [7; 32];
    const NOW: u64 = 1_000_000_000_000;
    const HOUR: u64 = 3_600_000;

    struct Person {
        key: SigningKey,
        m: Meetings,
    }

    impl Person {
        fn new(seed: u8) -> Person {
            Person { key: SigningKey::from_bytes(&[seed; 32]), m: Meetings { site: SITE, ..Meetings::default() } }
        }

        fn pk(&self) -> Key {
            self.key.verifying_key().to_bytes()
        }

        /// Hand every wire message in `out` to `to`, as Delivery would. Returns what `to` sends back.
        fn deliver(&self, out: Vec<Vec<u8>>, to: &mut Person, members: &[Key]) -> Vec<Vec<u8>> {
            self.deliver_at(out, to, members, NOW)
        }

        /// `deliver` when `to` gets the messages at `at` (its own clock).
        fn deliver_at(&self, out: Vec<Vec<u8>>, to: &mut Person, members: &[Key], at: u64) -> Vec<Vec<u8>> {
            let mut back = vec![];
            for w in out {
                match Wire::decode(&w) {
                    Some(Wire::Meet(b)) => back.extend(to.m.on_meet(&to.key, &SITE, &b, members, at)),
                    Some(Wire::Busy(b)) => {
                        to.m.on_busy(&SITE, &b, members);
                    }
                    other => panic!("unexpected {other:?}"),
                }
            }
            back
        }
    }

    fn pair() -> (Person, Person, [Key; 2]) {
        let (a, b) = (Person::new(1), Person::new(2));
        let members = [a.pk(), b.pk()];
        (a, b, members)
    }

    fn propose(a: &mut Person, b: &Person, start: u64) -> (Key, Vec<Vec<u8>>) {
        a.m.propose(&a.key, &SITE, &[b.pk()], (start, start + HOUR), "", NOW).unwrap()
    }

    #[test]
    fn propose_accept_and_both_sides_confirm_with_a_signed_accept() {
        let (mut a, mut b, members) = pair();
        let (id, out) = propose(&mut a, &b, NOW + 2 * HOUR);
        assert_eq!(a.m.find(&id).unwrap().status, Status::Proposed);
        // our own proposal makes us busy at once; the summary goes out with it
        assert_eq!(a.m.my_ranges(&a.pk(), NOW), vec![(NOW + 2 * HOUR, NOW + 3 * HOUR)]);
        a.deliver(out, &mut b, &members);
        assert_eq!(b.m.find(&id).unwrap().status, Status::Proposed);
        assert!(b.m.my_ranges(&b.pk(), NOW).is_empty(), "a request does not make the invitee busy");
        let out = b.m.answer(&b.key, &SITE, &id, Act::Accept, NOW).unwrap();
        b.deliver(out, &mut a, &members);
        for p in [&a, &b] {
            assert_eq!(p.m.find(&id).unwrap().status, Status::Confirmed);
        }
        // the proof: a signed accept of the invitee that anyone can check
        let proof: Msg = postcard::from_bytes(a.m.find(&id).unwrap().accept.as_ref().unwrap()).unwrap();
        assert!(proof.check(MEET_DOMAIN, &SITE, &b.pk()));
        assert_eq!(proof.body.kind, Kind::Accept { id });
    }

    #[test]
    fn a_clash_is_named_and_does_not_block_the_proposal() {
        let (mut a, b, _) = pair();
        let start = NOW + 2 * HOUR;
        let (first, _) = propose(&mut a, &b, start);
        let c = a.m.clashes(&a.pk(), None, (start + HOUR / 2, start + HOUR + HOUR / 2), None);
        assert_eq!(c.mine.len(), 1);
        assert_eq!(c.mine[0].id, first);
        // a slot that begins when a meeting ends is free
        assert!(a.m.clashes(&a.pk(), None, (start + HOUR, start + 2 * HOUR), None).mine.is_empty());
        // an overlapping time is allowed
        let (second, _) = propose(&mut a, &b, start + HOUR / 2);
        assert_ne!(first, second);
    }

    #[test]
    fn slots_show_busy_of_both_and_only_when_shared() {
        let (mut a, mut b, members) = pair();
        let start = NOW.div_ceil(STEP_MS) * STEP_MS + 4 * HOUR;
        // b is busy for an hour at `start` and shares it
        let (_, out) = propose(&mut b, &a, start);
        b.deliver(out, &mut a, &members);
        let slots = a.m.slots(&a.pk(), &b.pk(), NOW, HOUR, 48);
        let at = |s: u64| slots.iter().find(|x| x.range.0 == s).unwrap();
        assert!(at(start).theirs && !at(start).mine);
        assert!(!at(start + HOUR).theirs, "a slot that begins when the other meeting ends is free");
        assert!(at(start - HOUR / 2).theirs, "half an hour of overlap counts");
        // b stops sharing: a sees no ranges, not an empty calendar
        let off = b.m.set_prefs(&b.key, false, 10, NOW + 1);
        b.deliver(off, &mut a, &members);
        assert!(a.m.peer_ranges(&b.pk()).is_none());
        assert!(a.m.slots_json(&a.pk(), &b.pk(), NOW, HOUR, &BTreeMap::new(), NOW)["shared"] == false);
    }

    #[test]
    fn a_custom_time_and_length_is_checked_and_carries_its_own_start_and_end() {
        let (mut a, mut b, members) = pair();
        let min = 60_000;
        let to = [b.pk()];
        let try_range = |a: &mut Person, r: Range| a.m.propose(&a.key, &SITE, &to, r, "", NOW).map(|_| ());
        // An odd start (not on the half hour) and a custom length of 45 minutes pass.
        let (start, end) = (NOW + 7 * min, NOW + 52 * min);
        let (id, out) = a.m.propose(&a.key, &SITE, &[b.pk()], (start, end), "", NOW).unwrap();
        a.deliver(out, &mut b, &members);
        // The invitee learns the start and end from the proposal itself.
        assert_eq!(b.m.find(&id).unwrap().range(), (start, end));
        assert!(try_range(&mut a, (NOW + HOUR, NOW + HOUR + 5 * min)).is_ok());
        assert!(try_range(&mut a, (NOW + 2 * HOUR, NOW + 10 * HOUR)).is_ok());
        assert!(try_range(&mut a, (NOW + HOUR, NOW + HOUR + 4 * min)).is_err());
        assert!(try_range(&mut a, (NOW + HOUR, NOW + 9 * HOUR + min)).is_err());
        assert!(try_range(&mut a, (NOW + HOUR, NOW + HOUR)).is_err());
        assert!(try_range(&mut a, (NOW + 2 * HOUR, NOW + HOUR)).is_err());
        assert!(try_range(&mut a, (NOW - min, NOW + HOUR)).is_err());
        assert!(check_len_ms(4 * min).is_err() && check_len_ms(481 * min).is_err() && check_len_ms(45 * min) == Ok(45 * min));
    }

    #[test]
    fn a_custom_reminder_lead_is_kept_zero_means_none_and_too_long_is_refused() {
        let mut a = Person::new(1);
        a.m.set_prefs(&a.key, true, 45, NOW);
        assert_eq!(a.m.list_json(&a.pk(), &BTreeMap::new(), NOW)["lead_min"], 45);
        a.m.set_prefs(&a.key, true, 0, NOW);
        assert_eq!(a.m.list_json(&a.pk(), &BTreeMap::new(), NOW)["lead_min"], 0);
        assert!(check_lead_min(MAX_LEAD_MIN + 1).is_err() && check_lead_min(MAX_LEAD_MIN) == Ok(MAX_LEAD_MIN));
    }

    #[test]
    fn a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side() {
        let (mut a, mut b, members) = pair();
        let (id, out) = propose(&mut a, &b, NOW + 2 * HOUR);
        a.deliver(out, &mut b, &members);
        let out = b.m.answer(&b.key, &SITE, &id, Act::Decline, NOW).unwrap();
        b.deliver(out, &mut a, &members);
        assert_eq!(a.m.find(&id).unwrap().status, Status::Declined);
        assert!(a.m.my_ranges(&a.pk(), NOW).is_empty());
        // a second meeting, confirmed, then cancelled by the invitee
        let (id, out) = propose(&mut a, &b, NOW + 5 * HOUR);
        a.deliver(out, &mut b, &members);
        let out = b.m.answer(&b.key, &SITE, &id, Act::Accept, NOW).unwrap();
        b.deliver(out, &mut a, &members);
        let out = b.m.answer(&b.key, &SITE, &id, Act::Cancel, NOW).unwrap();
        b.deliver(out, &mut a, &members);
        assert_eq!(a.m.find(&id).unwrap().status, Status::Cancelled);
        assert!(a.m.my_ranges(&a.pk(), NOW).is_empty());
        // a declined meeting cannot be accepted later
        assert!(b.m.answer(&b.key, &SITE, &id, Act::Accept, NOW).is_err());
    }

    #[test]
    fn nobody_accepts_after_the_end_and_an_old_proposal_shows_unaccepted() {
        let (mut a, mut b, members) = pair();
        let (id, out) = propose(&mut a, &b, NOW + HOUR);
        a.deliver(out, &mut b, &members);
        let late = NOW + 3 * HOUR;
        assert!(b.m.answer(&b.key, &SITE, &id, Act::Accept, late).is_err());
        assert_eq!(b.m.find(&id).unwrap().shown(late), "unaccepted");
        assert_eq!(b.m.find(&id).unwrap().shown(NOW), "proposed");
        assert!(a.m.propose(&a.key, &SITE, &[b.pk()], (NOW - 2 * HOUR, NOW - HOUR), "", NOW).is_err());
    }

    #[test]
    fn a_stranger_a_forgery_and_the_wrong_reader_change_nothing() {
        let (mut a, mut b, members) = pair();
        let mut c = Person::new(3);
        let (id, out) = propose(&mut a, &b, NOW + 2 * HOUR);
        // c is no member (not in the list): the message is dropped even with a good signature
        let (_, wire) = Meetings::sealed(&c.key, &SITE, b.pk(), Kind::Propose { id: [9; 32], start_ms: NOW + HOUR, end_ms: NOW + 2 * HOUR }).unwrap();
        c.deliver(vec![wire], &mut b, &members);
        assert!(b.m.list.is_empty());
        // only the addressee can open it
        let to_c = [a.pk(), c.pk()];
        a.deliver(out.clone(), &mut c, &to_c);
        assert!(c.m.list.is_empty());
        a.deliver(out, &mut b, &members);
        // c cannot accept for b: a message of c about this id is refused by the owner check
        let msg = Signed::new(&c.key, MEET_DOMAIN, &SITE, Body { from: c.pk(), to: a.pk(), kind: Kind::Accept { id } });
        assert!(a.m.apply(&msg, NOW).is_err());
        // a message signed with another key than `from` is dropped
        let mut forged = Signed::new(&c.key, MEET_DOMAIN, &SITE, Body { from: b.pk(), to: a.pk(), kind: Kind::Accept { id } });
        assert!(!forged.check(MEET_DOMAIN, &SITE, &b.pk()));
        forged.sig = vec![0; 3];
        assert!(!forged.check(MEET_DOMAIN, &SITE, &b.pk()));
        // a repeat of a proposal is ignored
        assert!(a.m.apply(&Signed::new(&a.key, MEET_DOMAIN, &SITE, Body { from: a.pk(), to: b.pk(), kind: Kind::Propose { id, start_ms: 1, end_ms: 2 } }), NOW).is_err());
    }

    #[test]
    fn a_newer_summary_wins_and_an_old_one_is_ignored() {
        let (mut a, mut b, members) = pair();
        let first = b.m.set_prefs(&b.key, true, 10, NOW);
        assert!(first.is_empty(), "sharing did not change");
        let (_, out) = propose(&mut b, &a, NOW + 2 * HOUR);
        let busy = |w: &[u8]| match Wire::decode(w) {
            Some(Wire::Busy(s)) => s,
            _ => panic!("not a summary"),
        };
        let newer = busy(out.last().unwrap());
        let mut older = newer.clone();
        older.body.ts -= 1;
        older.body.ranges.clear();
        older.sig = Signed::new(&b.key, BUSY_DOMAIN, &SITE, older.body.clone()).sig;
        assert!(a.m.on_busy(&SITE, &newer, &members));
        assert!(!a.m.on_busy(&SITE, &older, &members));
        assert_eq!(a.m.peer_ranges(&b.pk()).unwrap().len(), 1);
    }

    #[test]
    fn resend_repeats_the_last_message_and_a_repeat_changes_nothing() {
        let (mut a, mut b, members) = pair();
        let (id, _lost) = propose(&mut a, &b, NOW + 2 * HOUR);
        // b was offline: the first message is lost, the resend brings it
        let again = a.m.resend(&a.key, &SITE, NOW);
        assert_eq!(again.len(), 2, "the proposal and the summary");
        a.deliver(again.clone(), &mut b, &members);
        assert_eq!(b.m.find(&id).unwrap().status, Status::Proposed);
        a.deliver(again, &mut b, &members);
        assert_eq!(b.m.list.len(), 1);
        assert_eq!(a.m.resend(&a.key, &SITE, NOW + 3 * HOUR).len(), 1, "a meeting that is over is not repeated");
    }

    #[test]
    fn the_file_keeps_the_meetings_and_the_backup_restores_on_a_new_device() {
        let dir = std::env::temp_dir().join(format!("pukaar-meet-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (mut a, b, _) = pair();
        a.m = Meetings::load(&dir, SITE);
        let (id, _) = propose(&mut a, &b, NOW + 2 * HOUR);
        assert_eq!(Meetings::load(&dir, SITE).find(&id).unwrap().status, Status::Proposed, "survives a restart");
        let blob = a.m.backup_bytes(&a.key);
        assert!(!String::from_utf8_lossy(&blob).contains("proposed"), "the backup is encrypted");
        let mut fresh = Meetings::load(&dir.join("new-device"), SITE);
        assert!(fresh.restore(&b.key, &blob).is_err(), "another key cannot read it");
        assert_eq!(fresh.restore(&a.key, &blob), Ok(1));
        assert_eq!(fresh.restore(&a.key, &blob), Ok(0), "a second restore adds nothing");
        assert_eq!(fresh.find(&id).unwrap(), a.m.find(&id).unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The proposal as `to` receives it, and how the meeting looks in its list JSON.
    fn row(p: &Person, id: &Key) -> Value {
        p.m.list_json(&p.pk(), &BTreeMap::new(), NOW)["meetings"].as_array().unwrap().iter().find(|r| r["id"] == id_hex(id)).unwrap().clone()
    }

    #[test]
    fn a_description_is_checked_travels_in_the_proposal_and_shows_on_both_sides() {
        let (mut a, mut b, members) = pair();
        for bad in ["x".repeat(MAX_DESC_CHARS + 1), "two\nlines".into(), "bell\u{7}".into(), "é".repeat(MAX_DESC_BYTES / 2 + 1)] {
            assert!(check_desc(&bad).is_err(), "{bad:?}");
        }
        assert_eq!(check_desc("  Plan the visit  "), Ok("Plan the visit"));
        assert!(check_desc(&"x".repeat(MAX_DESC_CHARS)).is_ok() && check_desc("").is_ok());
        let range = (NOW + 2 * HOUR, NOW + 3 * HOUR);
        assert!(a.m.propose(&a.key, &SITE, &[b.pk()], range, "bad\ttab", NOW).is_err());
        assert!(a.m.list.is_empty(), "a refused description saves nothing");
        // the longest allowed text still fits one sealed message
        let (id, out) = a.m.propose(&a.key, &SITE, &[b.pk()], range, &"x".repeat(MAX_DESC_CHARS), NOW).unwrap();
        assert!(matches!(Wire::decode(&out[0]), Some(Wire::Meet(_))), "the wire check accepts the longest text");
        a.deliver(out, &mut b, &members);
        for p in [&a, &b] {
            assert_eq!(row(p, &id)["desc"], "x".repeat(MAX_DESC_CHARS));
        }
        // a repeat keeps the description; a proposal without one stays the old plain kind
        assert!(a.m.resend(&a.key, &SITE, NOW).iter().any(|w| matches!(Wire::decode(w), Some(Wire::Meet(_)))));
        assert_eq!(Kind::propose([1; 32], 1, 2, "", &[[8; 32]]), Kind::Propose { id: [1; 32], start_ms: 1, end_ms: 2 });
        // a received proposal with a bad description is refused too
        let bad = Signed::new(&a.key, MEET_DOMAIN, &SITE, Body { from: a.pk(), to: b.pk(), kind: Kind::propose([5; 32], 1, 2, "a\nb", &[b.pk()]) });
        assert!(b.m.apply(&bad, NOW).is_err());
    }

    #[test]
    fn an_old_message_and_an_old_row_still_read_and_show_who_did_what() {
        // postcard writes the variant number first: the old kinds keep theirs, so an older message reads the same
        let old = postcard::to_allocvec(&Kind::Propose { id: [3; 32], start_ms: 5, end_ms: 6 }).unwrap();
        assert_eq!(postcard::from_bytes::<Kind>(&old), Ok(Kind::Propose { id: [3; 32], start_ms: 5, end_ms: 6 }));
        assert_eq!(old[0], 0);
        assert_eq!(postcard::to_allocvec(&Kind::Cancel { id: [3; 32] }).unwrap()[0], 3);
        // a row from before `desc` and `log` existed loads, and its history follows from the status
        let (mut a, b, _) = pair();
        let (id, _) = propose(&mut a, &b, NOW + 2 * HOUR);
        let mut v = serde_json::to_value(&a.m.list[0]).unwrap();
        v.as_object_mut().unwrap().retain(|k, _| !["desc", "log"].contains(&k.as_str()));
        let row: Meeting = serde_json::from_value(v).unwrap();
        assert_eq!((row.desc.as_str(), row.log.len()), ("", 0));
        assert_eq!(row.steps().iter().map(|s| (s.did, s.who, s.at_ms)).collect::<Vec<_>>(), vec![(Did::Proposed, a.pk(), 0)]);
        assert_eq!(a.m.find(&id).unwrap().steps().len(), 1);
    }

    #[test]
    fn the_history_keeps_who_proposed_accepted_and_cancelled_and_when() {
        let (mut a, mut b, members) = pair();
        let (id, out) = propose(&mut a, &b, NOW + 2 * HOUR);
        a.deliver(out, &mut b, &members);
        let out = b.m.answer(&b.key, &SITE, &id, Act::Accept, NOW + 5).unwrap();
        b.deliver_at(out, &mut a, &members, NOW + 5);
        let out = b.m.answer(&b.key, &SITE, &id, Act::Cancel, NOW + 9).unwrap();
        b.deliver_at(out, &mut a, &members, NOW + 9);
        let want = vec![(a.pk(), Did::Proposed, NOW), (b.pk(), Did::Accepted, NOW + 5), (b.pk(), Did::Cancelled, NOW + 9)];
        for p in [&a, &b] {
            let got: Vec<_> = p.m.find(&id).unwrap().log.iter().map(|s| (s.who, s.did, s.at_ms)).collect();
            assert_eq!(got, want);
            let log = &row(p, &id)["log"];
            assert_eq!((log[2]["did"].as_str(), log[2]["you"].as_bool()), (Some("cancelled"), Some(p.pk() == b.pk())));
        }
        // a decline, and a repeat of the same message, add one entry
        let (id, out) = propose(&mut a, &b, NOW + 5 * HOUR);
        a.deliver(out, &mut b, &members);
        let out = b.m.answer(&b.key, &SITE, &id, Act::Decline, NOW + 3).unwrap();
        b.deliver(out.clone(), &mut a, &members);
        b.deliver(out, &mut a, &members);
        assert_eq!(a.m.find(&id).unwrap().log.iter().map(|s| (s.who, s.did)).collect::<Vec<_>>(), vec![(a.pk(), Did::Proposed), (b.pk(), Did::Declined)]);
    }

    // ---- meetings with several invitees ----

    /// An organizer (index 0) and `n` invitees, all members.
    fn crowd(n: u8) -> (Vec<Person>, Vec<Key>) {
        let all: Vec<Person> = (1..=n + 1).map(Person::new).collect();
        let members = all.iter().map(Person::pk).collect();
        (all, members)
    }

    /// Hand the messages that `from` sent to everybody else: a sealed box opens only for its addressee.
    fn spread(all: &mut [Person], from: usize, out: &[Vec<u8>], members: &[Key]) {
        for (_, to) in all.iter_mut().enumerate().filter(|(j, _)| *j != from) {
            for w in out {
                match Wire::decode(w) {
                    Some(Wire::Meet(b)) => drop(to.m.on_meet(&to.key, &SITE, &b, members, NOW)),
                    Some(Wire::Busy(b)) => drop(to.m.on_busy(&SITE, &b, members)),
                    other => panic!("unexpected {other:?}"),
                }
            }
        }
    }

    /// Person `i` does `act` on `id`, and the messages reach the others.
    fn act(all: &mut [Person], i: usize, id: &Key, act: Act, members: &[Key]) {
        let out = try_act(&mut all[i], id, act).unwrap();
        spread(all, i, &out, members);
    }

    fn try_act(p: &mut Person, id: &Key, act: Act) -> Result<Vec<Vec<u8>>, String> {
        p.m.answer(&p.key, &SITE, id, act, NOW)
    }

    fn propose_to(p: &mut Person, to: &[Key], range: Range, desc: &str) -> Result<(Key, Vec<Vec<u8>>), String> {
        p.m.propose(&p.key, &SITE, to, range, desc, NOW)
    }

    /// The organizer proposes to everybody else.
    fn propose_all(all: &mut [Person], members: &[Key], desc: &str) -> Key {
        let to = &members[1..];
        let (id, out) = propose_to(&mut all[0], to, (NOW + 2 * HOUR, NOW + 3 * HOUR), desc).unwrap();
        spread(all, 0, &out, members);
        id
    }

    fn status_of(all: &[Person], id: &Key) -> Vec<Status> {
        all.iter().map(|p| p.m.find(id).unwrap().status).collect()
    }

    #[test]
    fn the_rule_confirms_once_all_answered_and_one_accepted() {
        use Answer::{Accepted as Y, Declined as N, Left as L, Pending as P};
        for (answers, want) in [
            (vec![P], Status::Proposed),
            (vec![Y], Status::Confirmed),
            (vec![N], Status::Declined),
            (vec![L], Status::Cancelled),
            (vec![Y, P], Status::Proposed),
            (vec![Y, Y, N], Status::Confirmed),
            (vec![N, Y, L], Status::Confirmed),
            (vec![N, N, N], Status::Declined),
            (vec![N, L], Status::Cancelled),
            (vec![N, P, N], Status::Proposed),
        ] {
            assert_eq!(settle(answers.iter().copied()), want, "{answers:?}");
        }
    }

    #[test]
    fn a_group_meeting_shows_each_answer_and_confirms_by_the_rule() {
        let (mut all, members) = crowd(3);
        let id = propose_all(&mut all, &members, "Plan the visit");
        // everybody sees the whole list and the description
        for p in &all {
            let m = p.m.find(&id).unwrap();
            assert_eq!(m.requester, members[0]);
            assert_eq!(m.invited().copied().collect::<Vec<_>>(), members[1..]);
            assert_eq!((m.status, m.desc.as_str()), (Status::Proposed, "Plan the visit"));
        }
        // only the organizer is busy while nobody has answered
        assert_eq!(all[0].m.my_ranges(&members[0], NOW).len(), 1);
        assert!(all[1].m.my_ranges(&members[1], NOW).is_empty());
        act(&mut all, 1, &id, Act::Accept, &members);
        // one accepted, two still pending: not confirmed; the accepter is busy; the others see the accept
        assert_eq!(status_of(&all, &id), vec![Status::Proposed; 4]);
        assert_eq!(all[1].m.my_ranges(&members[1], NOW).len(), 1);
        assert_eq!(all[2].m.find(&id).unwrap().answer(&members[1]), Answer::Accepted);
        act(&mut all, 2, &id, Act::Decline, &members);
        assert_eq!(status_of(&all, &id), vec![Status::Proposed; 4], "one is still pending");
        act(&mut all, 3, &id, Act::Accept, &members);
        assert_eq!(status_of(&all, &id), vec![Status::Confirmed; 4]);
        // the decliner is not busy even though the meeting is confirmed
        assert!(all[2].m.my_ranges(&members[2], NOW).is_empty());
        assert_eq!(all[3].m.my_ranges(&members[3], NOW).len(), 1);
        // each accept is a signed proof of its person
        let m = all[0].m.find(&id).unwrap();
        let from: Vec<Key> = m.accept.iter().chain(&m.later_accepts).map(|b| postcard::from_bytes::<Msg>(b).unwrap()).inspect(|p| assert!(p.check(MEET_DOMAIN, &SITE, &p.body.from))).map(|p| p.body.from).collect();
        assert_eq!(from, vec![members[1], members[3]]);
        // the JSON gives each person's state
        let r = row(&all[2], &id);
        let states: Vec<_> = r["invitees"].as_array().unwrap().iter().map(|i| i["state"].as_str().unwrap()).collect();
        assert_eq!((states, r["my_answer"].as_str(), r["status"].as_str()), (vec!["accepted", "declined", "accepted"], Some("declined"), Some("confirmed")));
        // the organizer withdraws the meeting for everybody
        act(&mut all, 0, &id, Act::Cancel, &members);
        assert_eq!(status_of(&all, &id), vec![Status::Cancelled; 4]);
        assert!(all.iter().all(|p| p.m.my_ranges(&p.pk(), NOW).is_empty()));
        assert_eq!(all[2].m.find(&id).unwrap().cancelled_by, Some(members[0]));
    }

    #[test]
    fn when_every_invitee_declines_the_meeting_is_declined() {
        let (mut all, members) = crowd(2);
        let id = propose_all(&mut all, &members, "");
        act(&mut all, 1, &id, Act::Decline, &members);
        act(&mut all, 2, &id, Act::Decline, &members);
        assert_eq!(status_of(&all, &id), vec![Status::Declined; 3]);
        assert!(all[0].m.my_ranges(&members[0], NOW).is_empty());
        assert!(try_act(&mut all[1], &id, Act::Accept).is_err(), "a declined meeting cannot be accepted");
    }

    #[test]
    fn an_invitee_who_accepted_can_leave_and_only_their_part_ends() {
        let (mut all, members) = crowd(2);
        let id = propose_all(&mut all, &members, "");
        // an invitee who did not accept cannot leave (they decline), and an invitee cannot cancel for everybody
        assert!(try_act(&mut all[1], &id, Act::Cancel).is_err());
        act(&mut all, 1, &id, Act::Accept, &members);
        act(&mut all, 2, &id, Act::Accept, &members);
        act(&mut all, 1, &id, Act::Cancel, &members);
        for p in &all {
            let m = p.m.find(&id).unwrap();
            assert_eq!((m.status, m.answer(&members[1]), m.answer(&members[2])), (Status::Confirmed, Answer::Left, Answer::Accepted));
        }
        assert!(all[1].m.my_ranges(&members[1], NOW).is_empty() && !all[2].m.my_ranges(&members[2], NOW).is_empty());
        assert!(all[0].m.my_ranges(&members[0], NOW).len() == 1, "the organizer stays busy");
        let log: Vec<_> = all[2].m.find(&id).unwrap().log.iter().map(|s| (s.who, s.did)).collect();
        assert_eq!(log, vec![(members[0], Did::Proposed), (members[1], Did::Accepted), (members[2], Did::Accepted), (members[1], Did::Cancelled)]);
        // the last one leaves too: nobody is left, the meeting is cancelled
        act(&mut all, 2, &id, Act::Cancel, &members);
        assert_eq!(status_of(&all, &id), vec![Status::Cancelled; 3]);
        // a repeat of the leave changes nothing
        let again = all[1].m.resend(&all[1].key, &SITE, NOW);
        assert_eq!(again.len(), 3, "a leave to the organizer and the other invitee, and the summary");
        spread(&mut all, 1, &again, &members);
        assert_eq!(all[0].m.find(&id).unwrap().log.len(), 5);
    }

    #[test]
    fn the_organizer_resends_the_proposal_only_to_those_who_have_not_answered() {
        let (mut all, members) = crowd(2);
        let id = propose_all(&mut all, &members, "");
        act(&mut all, 1, &id, Act::Accept, &members);
        let again = all[0].m.resend(&all[0].key, &SITE, NOW);
        assert_eq!(again.len(), 2, "one proposal for the one who is pending, and the summary");
        assert_eq!(all[0].m.find(&id).unwrap().last_act(&members[0]).map(|(_, to)| to), Some(vec![members[2]]));
    }

    #[test]
    fn a_bad_group_proposal_is_refused() {
        let (mut all, members) = crowd(3);
        let (a, b, c) = (members[0], members[1], members[2]);
        let group = |from: &Person, to: Key, invitees: Vec<Key>| Signed::new(&from.key, MEET_DOMAIN, &SITE, Body { from: from.pk(), to, kind: Kind::ProposeGroup { id: [5; 32], start_ms: 1, end_ms: 2, desc: String::new(), invitees } });
        let ok = group(&all[0], b, vec![b, c]);
        assert!(all[1].m.apply(&ok, NOW).is_ok());
        for (to, list) in [(b, vec![b]), (b, vec![b, b]), (b, vec![a, b]), (members[3], vec![b, c]), (b, (0..9).map(|i| [i; 32]).chain([b]).collect())] {
            let (mut fresh, msg) = (Person::new(9), group(&all[0], to, list.clone()));
            assert!(fresh.m.apply(&msg, NOW).is_err(), "{list:?}");
        }
        // somebody who is no member of the site cannot be invited
        let stranger = [99; 32];
        let (_, wire) = Meetings::sealed(&all[0].key, &SITE, c, Kind::ProposeGroup { id: [6; 32], start_ms: NOW + HOUR, end_ms: NOW + 2 * HOUR, desc: String::new(), invitees: vec![c, stranger] }).unwrap();
        spread(&mut all, 0, &[wire], &members);
        assert!(all[2].m.find(&[6; 32]).is_none());
        // the organizer's own checks: nobody, yourself, twice, and too many
        let me = all[0].pk();
        let r = (NOW + HOUR, NOW + 2 * HOUR);
        let many: Vec<Key> = (0..=MAX_INVITEES as u8).map(|i| [50 + i; 32]).collect();
        for bad in [vec![], vec![me, b], vec![b, b], many] {
            assert!(propose_to(&mut all[0], &bad, r, "").is_err(), "{bad:?}");
        }
        // the longest description with the most people still fits one sealed message
        let eight: Vec<Key> = (50..50 + MAX_INVITEES as u8).map(|i| Person::new(i).pk()).collect();
        let (_, out) = propose_to(&mut all[0], &eight, r, &"é".repeat(MAX_DESC_BYTES / 2)).unwrap();
        assert!(out.iter().take(MAX_INVITEES).all(|w| matches!(Wire::decode(w), Some(Wire::Meet(_)))));
    }

    #[test]
    fn a_row_from_before_groups_reads_as_a_meeting_with_one_invitee() {
        let (mut a, b, _) = pair();
        let (id, _) = propose(&mut a, &b, NOW + 2 * HOUR);
        let mut v = serde_json::to_value(&a.m.list[0]).unwrap();
        v.as_object_mut().unwrap().retain(|k, _| !["also", "later_accepts"].contains(&k.as_str()));
        a.m.list[0] = serde_json::from_value(v).unwrap();
        let r = row(&a, &id);
        assert_eq!(r["invitees"].as_array().unwrap().len(), 1);
        assert_eq!((r["invitees"][0]["with"].as_str(), r["invitees"][0]["state"].as_str()), (Some(id_hex(&b.pk()).as_str()), Some("pending")));
        assert_eq!(r["with"], id_hex(&b.pk()));
        // the old wire kinds keep their numbers, and the new kind comes after them
        let new = Kind::ProposeGroup { id, start_ms: 1, end_ms: 2, desc: String::new(), invitees: vec![b.pk(), [3; 32]] };
        assert_eq!(postcard::to_allocvec(&new).unwrap()[0], 5);
        assert_eq!(postcard::from_bytes::<Kind>(&postcard::to_allocvec(&new).unwrap()), Ok(new));
        assert_eq!(Kind::propose(id, 1, 2, "", &[b.pk()]), Kind::Propose { id, start_ms: 1, end_ms: 2 });
        // an old confirmed row (no log, a signed accept) still counts b as accepted, busy, and able to leave
        let (mut c, mut d, members) = pair();
        let (id, out) = propose(&mut c, &d, NOW + 2 * HOUR);
        c.deliver(out, &mut d, &members);
        let out = d.m.answer(&d.key, &SITE, &id, Act::Accept, NOW).unwrap();
        d.deliver(out, &mut c, &members);
        for p in [&mut c, &mut d] {
            let mut v = serde_json::to_value(&p.m.list[0]).unwrap();
            v.as_object_mut().unwrap().retain(|k, _| !["also", "later_accepts", "log"].contains(&k.as_str()));
            p.m.list[0] = serde_json::from_value(v).unwrap();
        }
        assert_eq!(d.m.find(&id).unwrap().answer(&d.pk()), Answer::Accepted);
        assert_eq!(d.m.my_ranges(&d.pk(), NOW).len(), 1);
        let out = d.m.answer(&d.key, &SITE, &id, Act::Cancel, NOW).unwrap();
        d.deliver(out, &mut c, &members);
        assert_eq!((c.m.find(&id).unwrap().status, d.m.find(&id).unwrap().status), (Status::Cancelled, Status::Cancelled));
        let kinds: Vec<_> = c.m.find(&id).unwrap().log.iter().map(|s| s.did).collect();
        assert_eq!(kinds, vec![Did::Proposed, Did::Accepted, Did::Cancelled], "the old history is kept when a new step is added");
    }
}
