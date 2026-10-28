//! 1:1 meetings between two members of one site (see 1-1-meet.md).
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
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MEET_DOMAIN: &[u8] = b"pukaar/meet/v1\0";
const BUSY_DOMAIN: &[u8] = b"pukaar/busy/v1\0";
const BACKUP_INFO: &[u8] = b"pukaar/meet-backup/v1";
const BACKUP_SCHEMA: &str = "pukaar.meetings/1";
/// A sealed message is a signed message (about 250 bytes) plus 48 bytes.
pub const MAX_MEET_BYTES: usize = 1024;
/// The busy ranges that one summary holds: the next ones in time.
pub const MAX_RANGES: usize = 64;
const MAX_LEN_MS: u64 = 24 * 3_600_000;
/// Meetings with one person that we keep, so one member cannot fill the disk.
const MAX_PER_PEER: usize = 200;
/// Slots start on this step (UTC).
pub const STEP_MS: u64 = 30 * 60_000;
pub const DEFAULT_LEAD_MIN: u64 = 10;

/// A start and an end, in ms.
pub type Range = (u64, u64);

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Meeting {
    pub id: Key,
    pub requester: Key,
    pub invitee: Key,
    pub start_ms: u64,
    pub end_ms: u64,
    pub status: Status,
    /// Who cancelled it: the one who repeats the cancel until the meeting is over.
    #[serde(default)]
    pub cancelled_by: Option<Key>,
    /// The invitee's signed accept message (postcard of `Msg`): proof that the meeting happened.
    #[serde(default)]
    pub accept: Option<Vec<u8>>,
}

impl Meeting {
    fn range(&self) -> Range {
        (self.start_ms, self.end_ms)
    }

    fn other(&self, me: &Key) -> Key {
        if self.requester == *me { self.invitee } else { self.requester }
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

    /// The meetings that make `me` busy: confirmed ones, and our own proposals that wait for an answer.
    fn busy_meetings(&self, me: &Key) -> impl Iterator<Item = &Meeting> + '_ {
        let me = *me;
        self.list.iter().filter(move |m| m.status == Status::Confirmed || (m.status == Status::Proposed && m.requester == me))
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
    fn apply(&mut self, m: &Msg) -> Result<(), &'static str> {
        let Body { from, to, kind } = &m.body;
        if let Kind::Propose { id, start_ms, end_ms } = kind {
            let ok = end_ms > start_ms && end_ms - start_ms <= MAX_LEN_MS && from != to;
            if !ok || self.find(id).is_some() {
                return Err("This meeting time is not valid.");
            }
            if self.list.iter().filter(|x| x.requester == *from || x.invitee == *from).count() >= MAX_PER_PEER {
                return Err("There are too many meetings with this person.");
            }
            self.list.push(Meeting { id: *id, requester: *from, invitee: *to, start_ms: *start_ms, end_ms: *end_ms, status: Status::Proposed, cancelled_by: None, accept: None });
            return Ok(());
        }
        let (Kind::Accept { id } | Kind::Decline { id } | Kind::Cancel { id }) = kind else { unreachable!("propose is handled above") };
        let Some(mt) = self.list.iter_mut().find(|x| x.id == *id) else { return Err("No such meeting.") };
        if mt.other(from) != *to || (mt.requester != *from && mt.invitee != *from) {
            return Err("This is not your meeting.");
        }
        let invitee = mt.invitee == *from;
        match (kind, mt.status) {
            (Kind::Accept { .. }, Status::Proposed) if invitee => {
                mt.status = Status::Confirmed;
                mt.accept = Some(postcard::to_allocvec(m).expect("postcard encode"));
            }
            (Kind::Decline { .. }, Status::Proposed) if invitee => mt.status = Status::Declined,
            (Kind::Cancel { .. }, Status::Proposed) if !invitee => (mt.status, mt.cancelled_by) = (Status::Cancelled, Some(*from)),
            (Kind::Cancel { .. }, Status::Confirmed) => (mt.status, mt.cancelled_by) = (Status::Cancelled, Some(*from)),
            _ => return Err("This meeting cannot change that way now."),
        }
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

    /// Our own action: apply it, then give the wire messages to send (the sealed one, and a new summary).
    fn send(&mut self, key: &SigningKey, site: &Key, to: Key, kind: Kind, now: u64) -> Result<Vec<Vec<u8>>, String> {
        let (msg, wire) = Self::sealed(key, site, to, kind).ok_or("That person's key is not valid.")?;
        let mut out = self.change(key, now, |s| s.apply(&msg))?;
        out.insert(0, wire);
        Ok(out)
    }

    /// Propose a meeting with `to`. It does not matter if the time clashes: the app only warns.
    /// Gives the id and the messages to send.
    pub fn propose(&mut self, key: &SigningKey, site: &Key, to: Key, range: Range, now: u64) -> Result<(Key, Vec<Vec<u8>>), String> {
        if range.1 <= now {
            return Err("That time is already over.".into());
        }
        let me = key.verifying_key().to_bytes();
        let stamp = [&me[..], &to, &range.0.to_be_bytes(), &range.1.to_be_bytes(), &now.to_be_bytes()].concat();
        let id = sha256(&[MEET_DOMAIN, &stamp].concat());
        Ok((id, self.send(key, site, to, Kind::Propose { id, start_ms: range.0, end_ms: range.1 }, now)?))
    }

    /// Accept, decline or cancel the meeting `id`. The invitee cannot accept after the meeting ended.
    pub fn answer(&mut self, key: &SigningKey, site: &Key, id: &Key, act: Act, now: u64) -> Result<Vec<Vec<u8>>, String> {
        let me = key.verifying_key().to_bytes();
        let m = self.find(id).ok_or("No such meeting.")?;
        if act == Act::Accept && m.end_ms <= now {
            return Err("This meeting is over, so it cannot be accepted.".into());
        }
        let to = m.other(&me);
        let kind = match act {
            Act::Accept => Kind::Accept { id: *id },
            Act::Decline => Kind::Decline { id: *id },
            Act::Cancel => Kind::Cancel { id: *id },
        };
        self.send(key, site, to, kind, now)
    }

    /// A `Wire::Meet` from Delivery. Only a message for us, from a member, with a good signature counts.
    /// Gives the summary to send when our busy ranges changed.
    pub fn on_meet(&mut self, key: &SigningKey, site: &Key, sealed: &[u8], members: &[Key], now: u64) -> Vec<Vec<u8>> {
        let Some(m) = open(key, sealed).and_then(|p| postcard::from_bytes::<Msg>(&p).ok()) else { return vec![] };
        let ok = m.body.to == key.verifying_key().to_bytes() && members.contains(&m.body.from) && m.check(MEET_DOMAIN, site, &m.body.from);
        if !ok {
            return vec![];
        }
        self.change(key, now, |s| s.apply(&m)).unwrap_or_default()
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
        let own = self.list.iter().filter(|m| m.end_ms > now).filter_map(|m| {
            let kind = match (m.status, m.requester == me) {
                (Status::Proposed, true) => Kind::Propose { id: m.id, start_ms: m.start_ms, end_ms: m.end_ms },
                (Status::Confirmed, false) => Kind::Accept { id: m.id },
                (Status::Declined, false) => Kind::Decline { id: m.id },
                (Status::Cancelled, _) if m.cancelled_by == Some(me) => Kind::Cancel { id: m.id },
                _ => return None,
            };
            Self::sealed(key, site, m.other(&me), kind).map(|(_, wire)| wire)
        });
        own.chain((self.busy_ts > 0).then(|| self.busy_wire(key, site, now))).collect()
    }

    /// Change what we share and how early the reminder comes. Gives the summary to send when sharing changed.
    pub fn set_prefs(&mut self, key: &SigningKey, share: bool, lead_min: u64, now: u64) -> Vec<Vec<u8>> {
        self.lead_min = lead_min.clamp(1, 24 * 60);
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
            let ok = (m.requester == me || m.invitee == me) && m.requester != m.invitee && m.end_ms > m.start_ms && self.find(&m.id).is_none();
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
            let mut back = vec![];
            for w in out {
                match Wire::decode(&w) {
                    Some(Wire::Meet(b)) => back.extend(to.m.on_meet(&to.key, &SITE, &b, members, NOW)),
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
        a.m.propose(&a.key, &SITE, b.pk(), (start, start + HOUR), NOW).unwrap()
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
        assert!(a.m.propose(&a.key, &SITE, b.pk(), (NOW - 2 * HOUR, NOW - HOUR), NOW).is_err());
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
        assert!(a.m.apply(&msg).is_err());
        // a message signed with another key than `from` is dropped
        let mut forged = Signed::new(&c.key, MEET_DOMAIN, &SITE, Body { from: b.pk(), to: a.pk(), kind: Kind::Accept { id } });
        assert!(!forged.check(MEET_DOMAIN, &SITE, &b.pk()));
        forged.sig = vec![0; 3];
        assert!(!forged.check(MEET_DOMAIN, &SITE, &b.pk()));
        // a repeat of a proposal is ignored
        assert!(a.m.apply(&Signed::new(&a.key, MEET_DOMAIN, &SITE, Body { from: a.pk(), to: b.pk(), kind: Kind::Propose { id, start_ms: 1, end_ms: 2 } })).is_err());
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
}
