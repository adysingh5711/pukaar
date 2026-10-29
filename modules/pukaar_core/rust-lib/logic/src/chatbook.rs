//! Private 1:1 chat between members of one site (see chat-module.md). The messages themselves live in
//! `chat_module`; this file holds what Pukaar adds around it, in `chat-<site>.json`:
//!
//! - the chat address of each member. A member signs its address and announces it on the site channel
//!   (`Wire::ChatAddr`); the receiver accepts it only from an active member. The module gives a new address
//!   at each start, so the book keeps the newest address of a member and the older ones: an old conversation
//!   still shows the right name, and only a conversation with the newest address can be written into.
//! - which person each conversation belongs to (`links`).
//! - the queue: a message that is not sent yet (the other address has not arrived, or the module is down)
//!   waits here, per person, and goes out when the core can send it.
//! - groups (the module's `group` conversations): which ones are ours, the members still to be added (their address
//!   has not arrived), the unsent messages and the read mark of each group, keyed by its conversation id.
//! - recovery after a restart (chat-module.md, "Recovery scenarios"): the last roster of each group, so that a member
//!   who restarted still knows who was in it, and the link from a group that was started again to its new group.
//!   A member who comes online asks the others for their address (`Wire::ChatAsk`).

use crate::event::Key;
use crate::meet::Signed;
use crate::persist::SiteFile;
use crate::sync::Wire;
use ed25519_dalek::SigningKey;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const ADDR_DOMAIN: &[u8] = b"pukaar/chataddr/v1\0";
/// A chat address is about 64 characters; the cap keeps a signed announce small.
pub const MAX_ADDR: usize = 256;
/// Addresses of one member that we keep (one per start of their chat module).
const KEEP_ADDRS: usize = 8;
/// Unsent messages to one person, so a person who never comes online cannot fill the disk.
pub const MAX_QUEUED: usize = 50;
pub const MAX_TEXT: usize = 4000;
/// How long a new conversation waits before the first message goes into it (see `ChatBook::opened`). The other
/// module joined within about a second in the field; the background step runs every 5 s.
const SETTLE_MS: u64 = 8_000;
const PREVIEW_CHARS: usize = 160;
pub const MAX_GROUP_NAME: usize = 60;
const ASK_DOMAIN: &[u8] = b"pukaar/chatask/v1\0";
/// We ask the others for their address at most this often, and answer an ask at most this often, so asks cannot flood.
const ASK_EVERY_MS: u64 = 60_000;
const ANSWER_EVERY_MS: u64 = 30_000;
/// The screen says "asking" for this long after an ask.
const ASKING_MS: u64 = 20_000;
/// The description of a group that was started again: this prefix, then the id of the old group.
pub const AFTER: &str = "pukaar:after:";
/// A chain of groups started again is followed this far back.
const MAX_CHAIN: usize = 8;
/// A move that we started and that did not finish is tried again after this long.
const MOVE_RETRY_MS: u64 = 120_000;
/// When the member who moves a group has not done it after this long, the screen offers Start again to everyone.
pub const FALLBACK_MS: u64 = 180_000;
/// A re-add (`readd`) has this long to commit before the group moves to a new one. The steward commits after about 60 s.
const ROLL_AFTER_MS: u64 = 150_000;

/// The body of an ask for the chat addresses of the others.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AskBody {
    pub from: Key,
    pub ts: u64,
}

pub type AskMsg = Signed<AskBody>;

/// The body of an announce: `ts` is the clock of the sender and the newest wins.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AddrBody {
    pub from: Key,
    pub ts: u64,
    pub addr: String,
}

pub type AddrMsg = Signed<AddrBody>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct Held {
    who: Key,
    ts: u64,
    addr: String,
}

/// A conversation of the chat module, and the person and the address that it was opened with.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct Link {
    convo: String,
    who: Key,
    addr: String,
}

/// How far we have read a thread (a person's `Key`, or a group's conversation id): the `last_ms` of its newest
/// conversation at that time. A file from before groups wrote `who`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct Read<K> {
    #[serde(alias = "who")]
    id: K,
    ms: u64,
}

/// Move the mark of `id` forward to `ms`, never back. False when nothing changed.
fn advance<K: PartialEq + Clone>(marks: &mut Vec<Read<K>>, id: &K, ms: u64) -> bool {
    match marks.iter_mut().find(|r| r.id == *id) {
        Some(r) if r.ms >= ms => return false,
        Some(r) => r.ms = ms,
        None => marks.push(Read { id: id.clone(), ms }),
    }
    true
}

/// A thread is unread when a live conversation has a message after the mark. A history-only one cannot get a new message.
fn unread<K: PartialEq>(marks: &[Read<K>], id: &K, convos: &[&Convo]) -> bool {
    let read = marks.iter().find(|r| r.id == *id).map_or(0, |r| r.ms);
    convos.iter().any(|c| !c.history_only && c.last_ms > read)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Queued {
    pub to: Key,
    pub text: String,
    pub ts_ms: u64,
}

/// A member that is to be added to a group once their chat address is known.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct Invite {
    convo: String,
    who: Key,
}

/// A message to a group that is not sent yet.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GroupQueued {
    pub convo: String,
    pub text: String,
    pub ts_ms: u64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ChatBook {
    addrs: Vec<Held>,
    links: Vec<Link>,
    queue: Vec<Queued>,
    /// Absent in a file from before the unread dot.
    #[serde(default)]
    read: Vec<Read<Key>>,
    /// The groups of this site (conversation ids): made here, or joined with a member that we know. Absent in a file
    /// from before groups.
    #[serde(default)]
    groups: Vec<String>,
    #[serde(default)]
    invites: Vec<Invite>,
    #[serde(default)]
    group_queue: Vec<GroupQueued>,
    #[serde(default)]
    group_read: Vec<Read<String>>,
    /// The roster of each group the last time it was live for us. After our restart the module has no roster for it,
    /// so this says who was in it. Absent in a file from before recovery.
    #[serde(default)]
    last_roster: BTreeMap<String, Vec<Member>>,
    /// A group that was started again: `(old id, new id)`. Absent in a file from before recovery.
    #[serde(default)]
    next: Vec<(String, String)>,
    /// A move that we started: the old group and the time (ms). It stops a second round, or a start after a crash, from
    /// making a second new group for `MOVE_RETRY_MS`. Absent in a file from before recovery.
    #[serde(default)]
    moves: Vec<(String, u64)>,
    /// When we last asked the others for their address, and when we last answered an ask (ms, 0 = never).
    #[serde(skip)]
    asked: u64,
    #[serde(skip)]
    answered: u64,
    /// Since when each group has waited for its move (ms), for the fallback on the screen.
    #[serde(skip)]
    due: BTreeMap<String, u64>,
    #[serde(skip)]
    pub site: Key,
    #[serde(skip)]
    dir: PathBuf,
    /// Our own current address and the time of its announce: set at each start of the chat module.
    #[serde(skip)]
    mine: Option<(u64, String)>,
    /// Conversations that this start of the core opened, with the time. The other side needs a moment to join one;
    /// a message sent before that goes to a topic that nobody listens to yet and is lost.
    #[serde(skip)]
    opened: Vec<(String, u64)>,
    /// Per group, from its roster the last time we looked: members in, invites not committed, and since when the
    /// roster has been the same.
    #[serde(skip)]
    rosters: BTreeMap<String, (usize, usize, u64)>,
}

impl SiteFile for ChatBook {
    const PREFIX: &'static str = "chat";

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

/// The module sends times as JSON numbers that may arrive as floats.
fn ms<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    f64::deserialize(d).map(|f| f as u64)
}

/// One element of the module's `list_conversations`.
#[derive(Deserialize, Clone, Debug)]
pub struct Convo {
    #[serde(rename = "convo_id")]
    pub id: String,
    #[serde(default, rename = "last_activity_ms", deserialize_with = "ms")]
    pub last_ms: u64,
    #[serde(default)]
    pub preview: Option<String>,
    /// Messages in the conversation, also those that the module no longer holds in memory.
    #[serde(default, deserialize_with = "ms")]
    pub message_count: u64,
    #[serde(default)]
    pub history_only: bool,
    #[serde(default)]
    pub kind: String,
    /// A group's shared name.
    #[serde(default)]
    pub name: Option<String>,
    /// A group's shared description. A group that was started again holds `AFTER` and the old id here.
    #[serde(default)]
    pub description: Option<String>,
}

/// One element of the module's `list_group_members`: an account address, and whether the group has not
/// committed it yet (it cannot read the group until then).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub address: String,
    #[serde(default)]
    pub pending: bool,
}

/// One element of the module's `get_messages`.
#[derive(Deserialize, Clone, Debug)]
pub struct Msg {
    pub from_self: bool,
    pub content: String,
    #[serde(default, rename = "timestamp_ms", deserialize_with = "ms")]
    pub timestamp_ms: u64,
    #[serde(default)]
    pub sender: Option<String>,
}

/// The list that a module call answered: a JSON array, or inside `{"value": …}`.
pub fn list_of<T: DeserializeOwned>(v: Value) -> Result<Vec<T>, String> {
    let v = if v.is_array() { v } else { v["value"].clone() };
    serde_json::from_value(v).map_err(|e| e.to_string())
}

fn row(from_self: bool, content: &str, ts_ms: u64, old: bool, queued: bool) -> Value {
    json!({ "from_self": from_self, "content": content, "ts_ms": ts_ms, "old": old, "queued": queued })
}

/// `rows` oldest first (stable: the order inside a conversation stays), then the unsent messages `(time, text)`.
fn ordered<'a>(
    mut rows: Vec<(u64, Value)>,
    unsent: impl Iterator<Item = (u64, &'a str)>,
) -> Vec<Value> {
    rows.sort_by_key(|r| r.0);
    rows.into_iter()
        .map(|r| r.1)
        .chain(unsent.map(|(ts, text)| row(true, text, ts, false, true)))
        .collect()
}

/// The messages of a thread, oldest first: each conversation's messages (`old` for a conversation that
/// is history only), then the unsent ones, marked `queued`.
#[must_use]
pub fn thread_messages(parts: Vec<(bool, Vec<Msg>)>, queued: &[&Queued]) -> Vec<Value> {
    let rows = parts
        .into_iter()
        .flat_map(|(old, msgs)| {
            msgs.into_iter().map(move |m| {
                (
                    m.timestamp_ms,
                    row(m.from_self, &m.content, m.timestamp_ms, old, false),
                )
            })
        })
        .collect();
    ordered(rows, queued.iter().map(|q| (q.ts_ms, q.text.as_str())))
}

/// A message text that can be kept: trimmed, 1 to `MAX_TEXT` characters.
fn clean(text: &str) -> Result<&str, String> {
    let text = text.trim();
    if text.is_empty() || text.len() > MAX_TEXT {
        return Err(format!("A message needs 1 to {MAX_TEXT} characters."));
    }
    Ok(text)
}

fn preview_of(text: &str) -> String {
    text.chars().take(PREVIEW_CHARS).collect()
}

/// Drop the first `n` items that `is` picks.
fn drop_first<T>(items: &mut Vec<T>, mut n: usize, is: impl Fn(&T) -> bool) {
    items.retain(|i| {
        !is(i) || n == 0 || {
            n -= 1;
            false
        }
    });
}

/// The items once each, in order of first appearance.
fn distinct<T: PartialEq + Clone>(items: impl Iterator<Item = T>) -> Vec<T> {
    let mut out: Vec<T> = vec![];
    for i in items {
        if !out.contains(&i) {
            out.push(i);
        }
    }
    out
}

impl ChatBook {
    fn newest(&self, who: &Key) -> Option<&Held> {
        self.addrs
            .iter()
            .filter(|h| h.who == *who)
            .max_by_key(|h| h.ts)
    }

    /// The newest chat address of `who`, or `None` when none has arrived.
    #[must_use]
    pub fn latest(&self, who: &Key) -> Option<&str> {
        self.newest(who).map(|h| h.addr.as_str())
    }

    /// The member that announced `addr`, an old address too.
    #[must_use]
    pub fn who_of(&self, addr: &str) -> Option<Key> {
        self.addrs.iter().find(|h| h.addr == addr).map(|h| h.who)
    }

    /// Our signed announce of `addr`, when it is not the one that we announced last (a start of the chat module).
    pub fn announce(
        &mut self,
        key: &SigningKey,
        site: &Key,
        addr: &str,
        now: u64,
    ) -> Option<Vec<u8>> {
        if self.mine.as_ref().is_some_and(|(_, a)| a == addr) {
            return None;
        }
        self.mine = Some((now, addr.to_owned()));
        self.resend(key, site)
    }

    /// Our last announce again. Delivery gives no acknowledgement and a peer may have been offline.
    #[must_use]
    pub fn resend(&self, key: &SigningKey, site: &Key) -> Option<Vec<u8>> {
        let (ts, addr) = self.mine.as_ref()?;
        let body = AddrBody {
            from: key.verifying_key().to_bytes(),
            ts: *ts,
            addr: addr.clone(),
        };
        Some(Wire::ChatAddr(Signed::new(key, ADDR_DOMAIN, site, body)).encode())
    }

    /// A `Wire::ChatAddr` from Delivery. Only an active member, with a good signature, counts. Gives the
    /// member when their newest address changed (then their queue can go out).
    pub fn on_addr(&mut self, site: &Key, m: &AddrMsg, members: &[Key]) -> Option<Key> {
        let AddrBody { from, ts, addr } = &m.body;
        let ok = !addr.is_empty()
            && addr.len() <= MAX_ADDR
            && members.contains(from)
            && m.check(ADDR_DOMAIN, site, from);
        if !ok || self.addrs.iter().any(|h| h.who == *from && h.addr == *addr) {
            return None;
        }
        let newest = self.newest(from).is_none_or(|h| h.ts < *ts);
        self.addrs.push(Held {
            who: *from,
            ts: *ts,
            addr: addr.clone(),
        });
        let held = self
            .addrs
            .iter()
            .enumerate()
            .filter(|(_, h)| h.who == *from);
        if held.clone().count() > KEEP_ADDRS {
            if let Some(oldest) = held.min_by_key(|(_, h)| h.ts).map(|(i, _)| i) {
                self.addrs.remove(oldest);
            }
        }
        self.save();
        newest.then_some(*from)
    }

    /// A signed ask to the other members to announce their chat address again. A member that was offline missed
    /// their announces. `None` when we asked less than `ASK_EVERY_MS` ago.
    pub fn ask(&mut self, key: &SigningKey, site: &Key, now: u64) -> Option<Vec<u8>> {
        if self.asked > 0 && now.saturating_sub(self.asked) < ASK_EVERY_MS {
            return None;
        }
        self.asked = now;
        let body = AskBody {
            from: key.verifying_key().to_bytes(),
            ts: now,
        };
        Some(Wire::ChatAsk(Signed::new(key, ASK_DOMAIN, site, body)).encode())
    }

    /// True for a short time after we asked: the screen says that we wait for the answers.
    #[must_use]
    pub fn asking(&self, now: u64) -> bool {
        self.asked > 0 && now.saturating_sub(self.asked) < ASKING_MS
    }

    /// A `Wire::ChatAsk` from Delivery: our announce again. Only an ask from another active member with a good
    /// signature counts, and we answer at most once per `ANSWER_EVERY_MS`, whoever asks.
    pub fn on_ask(&mut self, key: &SigningKey, site: &Key, m: &AskMsg, members: &[Key], now: u64) -> Option<Vec<u8>> {
        let from = m.body.from;
        let ok = from != key.verifying_key().to_bytes() && members.contains(&from) && m.check(ASK_DOMAIN, site, &from);
        if !ok || (self.answered > 0 && now.saturating_sub(self.answered) < ANSWER_EVERY_MS) {
            return None;
        }
        let out = self.resend(key, site)?;
        self.answered = now;
        Some(out)
    }

    pub fn link(&mut self, convo: &str, who: Key, addr: &str) {
        if self.who_of_convo(convo).is_none() {
            self.links.push(Link {
                convo: convo.to_owned(),
                who,
                addr: addr.to_owned(),
            });
            self.save();
        }
    }

    /// Note that we opened `convo` just now.
    pub fn opened(&mut self, convo: &str, now_ms: u64) {
        self.opened.push((convo.to_owned(), now_ms));
    }

    /// True when `convo` is old enough to send into: the other side has had `SETTLE_MS` to join it.
    #[must_use]
    pub fn settled(&self, convo: &str, now_ms: u64) -> bool {
        self.opened
            .iter()
            .all(|(c, at)| c != convo || now_ms.saturating_sub(*at) >= SETTLE_MS)
    }

    /// Link a conversation that someone opened with us, by the address of its sender. False when no member announced that address.
    pub fn link_sender(&mut self, convo: &str, sender: &str) -> bool {
        let who = self.who_of(sender);
        if let Some(who) = who {
            self.link(convo, who, sender);
        }
        who.is_some()
    }

    #[must_use]
    pub fn who_of_convo(&self, convo: &str) -> Option<Key> {
        self.links.iter().find(|l| l.convo == convo).map(|l| l.who)
    }

    /// The conversations with `who` that were opened with `addr`: the only ones that can reach their newest address.
    pub fn convos_for<'a>(
        &'a self,
        who: &'a Key,
        addr: &'a str,
    ) -> impl Iterator<Item = &'a str> + 'a {
        self.links
            .iter()
            .filter(move |l| l.who == *who && l.addr == addr)
            .map(|l| l.convo.as_str())
    }

    /// Keep a message for `to` until it can be sent.
    pub fn enqueue(&mut self, to: Key, text: &str, now_ms: u64) -> Result<(), String> {
        let text = clean(text)?;
        if self.queued(&to).count() >= MAX_QUEUED {
            return Err("There are too many unsent messages to this person.".into());
        }
        self.queue.push(Queued {
            to,
            text: text.to_owned(),
            ts_ms: now_ms,
        });
        self.save();
        Ok(())
    }

    /// The unsent messages to `who`, oldest first.
    pub fn queued<'a>(&'a self, who: &'a Key) -> impl Iterator<Item = &'a Queued> + 'a {
        self.queue.iter().filter(move |q| q.to == *who)
    }

    /// The people that have unsent messages.
    #[must_use]
    pub fn queue_targets(&self) -> Vec<Key> {
        distinct(self.queue.iter().map(|q| q.to))
    }

    /// The thread with `who` is read up to `ms` (never back).
    pub fn mark_read(&mut self, who: &Key, ms: u64) {
        if advance(&mut self.read, who, ms) {
            self.save();
        }
    }

    /// The thread with `who` is read up to the newest message of its conversations.
    pub fn mark_thread_read(&mut self, who: &Key, convos: &[Convo]) {
        let newest = convos
            .iter()
            .filter(|c| self.who_of_convo(&c.id) == Some(*who))
            .map(|c| c.last_ms)
            .max();
        if let Some(ms) = newest {
            self.mark_read(who, ms);
        }
    }

    /// The first `n` unsent messages to `who` were sent.
    pub fn unqueue(&mut self, who: &Key, n: usize) {
        drop_first(&mut self.queue, n, |q| q.to == *who);
        self.save();
    }

    /// The chat list, newest first: one row per active member that we have a message with, or an unsent one, and one
    /// per group of the site (a group is listed as soon as it exists). A linked conversation without any message
    /// (it was only opened) is not listed.
    #[must_use]
    pub fn threads_json(
        &self,
        convos: &[Convo],
        names: &BTreeMap<Key, String>,
        members: &[Key],
        me: &Key,
    ) -> Value {
        let mut people: Vec<Key> = self
            .links
            .iter()
            .map(|l| l.who)
            .chain(self.queue.iter().map(|q| q.to))
            .filter(|w| w != me && members.contains(w))
            .collect();
        people.sort_unstable();
        people.dedup();
        let mut rows: Vec<(u64, Value)> = people
            .iter()
            .filter_map(|who| {
                let mine: Vec<&Convo> = convos
                    .iter()
                    .filter(|c| self.who_of_convo(&c.id) == Some(*who))
                    .collect();
                let talked = mine.iter().any(|c| c.last_ms > 0 || c.message_count > 0);
                if !talked && self.queued(who).next().is_none() {
                    return None;
                }
                let newest = mine.iter().max_by_key(|c| c.last_ms);
                let (preview, last_ms) = match (self.queued(who).last(), newest) {
                    (Some(q), _) => (format!("Sending: {}", q.text), q.ts_ms),
                    (None, Some(c)) => (c.preview.clone().unwrap_or_default(), c.last_ms),
                    (None, None) => (String::new(), 0),
                };
                let row = json!({
                    "key": hex::encode(who),
                    "name": names.get(who),
                    "preview": preview_of(&preview),
                    "last_ms": last_ms,
                    "queued": self.queued(who).count(),
                    "history_only": !mine.is_empty() && mine.iter().all(|c| c.history_only),
                    "has_address": self.latest(who).is_some(),
                    "unread": unread(&self.read, who, &mine),
                });
                Some((last_ms, row))
            })
            .chain(self.group_rows(convos))
            .collect();
        rows.sort_by_key(|r| std::cmp::Reverse(r.0));
        rows.into_iter().map(|r| r.1).collect()
    }

    // ---- groups ----

    /// Our address in the chat module, from the last announce.
    #[must_use]
    pub fn my_addr(&self) -> Option<&str> {
        self.mine.as_ref().map(|(_, a)| a.as_str())
    }

    #[must_use]
    pub fn is_group(&self, convo: &str) -> bool {
        self.groups.iter().any(|g| g == convo)
    }

    /// `convo` is a group that we made: it is ours, and `who` are to be added when their address is known.
    pub fn start_group(&mut self, convo: &str, who: &[Key]) {
        if !self.is_group(convo) {
            self.groups.push(convo.to_owned());
        }
        for w in who {
            self.want(convo, *w);
        }
        self.save();
    }

    /// Keep `who` to be added to `convo`, once.
    pub fn want(&mut self, convo: &str, who: Key) {
        if !self
            .invites
            .iter()
            .any(|i| i.convo == convo && i.who == who)
        {
            self.invites.push(Invite {
                convo: convo.to_owned(),
                who,
            });
            self.save();
        }
    }

    /// The members still to be added to `convo`.
    pub fn waiting<'a>(&'a self, convo: &'a str) -> impl Iterator<Item = &'a Key> + 'a {
        self.invites
            .iter()
            .filter(move |i| i.convo == convo)
            .map(|i| &i.who)
    }

    /// `who` was handed to the module to be added to `convo`.
    pub fn invited(&mut self, convo: &str, who: &Key) {
        self.invites.retain(|i| i.convo != convo || i.who != *who);
        self.save();
    }

    /// The members to add now: `(group, member, their newest address)`. An invite to a member who is no longer
    /// active, or to a group that is not live any more (from before a restart), is forgotten. An invite whose
    /// address has not arrived stays.
    pub fn due_invites(&mut self, members: &[Key], live: &[Convo]) -> Vec<(String, Key, String)> {
        let kept = self.invites.len();
        self.invites.retain(|i| {
            members.contains(&i.who) && live.iter().any(|c| c.id == i.convo && !c.history_only)
        });
        if self.invites.len() != kept {
            self.save();
        }
        self.invites
            .iter()
            .filter_map(|i| Some((i.convo.clone(), i.who, self.latest(&i.who)?.to_owned())))
            .collect()
    }

    /// We looked at the roster of `convo`. A group that we did not know is ours when a member of the site that we
    /// have an address for is in it. Gives true when a message can go in now: the roster has not changed for
    /// `SETTLE_MS` (a member that was just added needs that time to join, and a message sent before is lost to
    /// them), and somebody else is in the group or nobody is being added.
    pub fn group_seen(
        &mut self,
        convo: &str,
        roster: &[Member],
        members: &[Key],
        me: &Key,
        now: u64,
    ) -> bool {
        if !self.is_group(convo) {
            let ours = roster
                .iter()
                .filter_map(|m| self.who_of(&m.address))
                .any(|w| w != *me && members.contains(&w));
            if !ours {
                return false;
            }
            self.groups.push(convo.to_owned());
            self.save();
        }
        self.keep_roster(convo, roster);
        let pending = roster.iter().filter(|m| m.pending).count();
        let sig = (roster.len() - pending, pending);
        let since = match self.rosters.get(convo) {
            Some(&(inn, out, at)) if (inn, out) == sig => at,
            _ => now,
        };
        self.rosters.insert(convo.to_owned(), (sig.0, sig.1, since));
        let alone = sig.0 <= 1 && (pending > 0 || self.waiting(convo).next().is_some());
        !alone && now.saturating_sub(since) >= SETTLE_MS
    }

    /// A member of `convo` that restarted is in its roster with an old chat address, and their newest address is not.
    /// Keep them to be added again. This is the first, cheap try: the add is a vote, and the old address never votes
    /// again, so it commits only while enough live members vote. In a group of two it never commits and is not tried.
    /// When it does not commit, the group moves to a new one (`moving`). Only the live member of the group with the
    /// lowest key adds, so that an add is not proposed twice. A member that restarted is not live and cannot add
    /// anybody. A member that is still pending is not asked again. A group that was started again is left alone.
    pub fn readd(&mut self, convo: &str, roster: &[Member], members: &[Key], me: &Key) {
        if !self.is_group(convo) || self.next_of(convo).is_some() {
            return;
        }
        self.keep_roster(convo, roster);
        let seats = roster.iter().filter(|m| !m.pending).count();
        if seats <= 2 || self.adder(roster, members, me) != Some(*me) {
            return;
        }
        let inn: Vec<Key> = roster
            .iter()
            .filter_map(|m| self.who_of(&m.address))
            .filter(|w| members.contains(w))
            .collect();
        for w in inn {
            if self.latest(&w).is_some_and(|l| !roster.iter().any(|m| m.address == l)) {
                self.want(convo, w);
            }
        }
    }

    /// The member whose task is to add the others back: the live member of `roster` with the lowest key. Live means
    /// committed, active in the site, and in the group with their newest address (for us: our current one).
    fn adder(&self, roster: &[Member], members: &[Key], me: &Key) -> Option<Key> {
        roster
            .iter()
            .filter(|m| !m.pending)
            .filter_map(|m| {
                if self.my_addr() == Some(m.address.as_str()) {
                    return Some(*me);
                }
                let w = self.who_of(&m.address)?;
                (self.latest(&w) == Some(m.address.as_str())).then_some(w)
            })
            .filter(|w| members.contains(w))
            .min()
    }

    /// Keep the roster of a live group (see `last_roster`). An empty roster is a failed read and is not kept.
    fn keep_roster(&mut self, convo: &str, roster: &[Member]) {
        if !roster.is_empty() && self.last_roster.get(convo).is_none_or(|r| r != roster) {
            self.last_roster.insert(convo.to_owned(), roster.to_vec());
            self.save();
        }
    }

    /// The last roster that we saw of `convo`, empty when we never saw it live.
    #[must_use]
    pub fn last_roster(&self, convo: &str) -> Vec<Member> {
        self.last_roster.get(convo).cloned().unwrap_or_default()
    }

    /// Whether `convo` must move to a new group, and who moves it, from `roster` (the module's while the group is live
    /// for us, else `last_roster`). A move is due when a member restarted: an old address of theirs is in the roster
    /// and their newest address is not committed. That is also us when the group is read-only for us. Gives
    /// `(mover, when)`:
    /// - the mover is the live member with the lowest key. When nobody is live, it is the lowest key among us and the
    ///   members that announced a new address. Each member works this out alone, so there is one mover by our view.
    /// - when: at once in a group of two or with nobody live (a re-add cannot commit there), else `ROLL_AFTER_MS` after
    ///   we first saw it, so that the re-add of `readd` can commit first.
    ///
    /// `None` when nothing is to do, the group already moved, or we do not know its roster (a file from before recovery).
    pub fn moving(&mut self, convo: &str, roster: &[Member], members: &[Key], me: &Key, now: u64) -> Option<(Key, u64)> {
        let committed = |a: &str| roster.iter().any(|m| !m.pending && m.address == a);
        let mine = self.my_addr().map(str::to_owned);
        let moved: Vec<Key> = roster
            .iter()
            .filter_map(|m| Some((m, self.who_of(&m.address)?)))
            .filter(|(_, w)| w != me && members.contains(w))
            .filter(|(m, w)| self.latest(w).is_some_and(|l| l != m.address && !committed(l)))
            .map(|(_, w)| w)
            .collect();
        let quiet = moved.is_empty() && mine.as_deref().is_some_and(committed);
        if !self.is_group(convo) || self.next_of(convo).is_some() || roster.is_empty() || mine.is_none() || quiet {
            self.due.remove(convo);
            return None;
        }
        let since = *self.due.entry(convo.to_owned()).or_insert(now);
        let live = self.adder(roster, members, me);
        let seats = roster.iter().filter(|m| !m.pending).count();
        let wait = if live.is_some() && seats > 2 { ROLL_AFTER_MS } else { 0 };
        let by = live.or_else(|| moved.into_iter().chain([*me]).min()).unwrap_or(*me);
        Some((by, since + wait))
    }

    /// We start the move of `old`: false when we started one less than `MOVE_RETRY_MS` ago (a second round, or a crash
    /// after the new group was made). The record is kept in the file and goes when the move is linked (`succeed`).
    pub fn begin_move(&mut self, old: &str, now: u64) -> bool {
        if self.moves.iter().any(|(o, at)| o == old && now.saturating_sub(*at) < MOVE_RETRY_MS) {
            return false;
        }
        self.moves.retain(|(o, _)| o != old);
        self.moves.push((old.to_owned(), now));
        self.save();
        true
    }

    /// The members of an old group, to put in the group that starts it again: the active members (not us) in its last
    /// roster, those still to be added, and the senders of its messages (the roster of a file from before recovery is
    /// not known).
    #[must_use]
    pub fn old_members(&self, convo: &str, senders: &[String], members: &[Key], me: &Key) -> Vec<Key> {
        let roster = self.last_roster(convo);
        let by_addr = roster
            .iter()
            .map(|m| m.address.as_str())
            .chain(senders.iter().map(String::as_str))
            .filter_map(|a| self.who_of(a));
        distinct(by_addr.chain(self.waiting(convo).copied()))
            .into_iter()
            .filter(|w| w != me && members.contains(w))
            .collect()
    }

    /// The group that `old` was started again as, if any.
    #[must_use]
    pub fn next_of(&self, old: &str) -> Option<&str> {
        self.next.iter().find(|(o, _)| o == old).map(|(_, n)| n.as_str())
    }

    /// The groups that `convo` started again, oldest first.
    #[must_use]
    pub fn before(&self, convo: &str) -> Vec<String> {
        let mut out: Vec<String> = vec![];
        let mut at = convo.to_owned();
        while out.len() < MAX_CHAIN {
            match self.next.iter().find(|(o, n)| *n == at && !out.contains(o) && *o != convo) {
                Some((o, _)) => {
                    at = o.clone();
                    out.insert(0, o.clone());
                }
                None => break,
            }
        }
        out
    }

    /// The group `old` goes on as `new`: `new` is ours, and what waits to be sent to `old` (or to the group it was
    /// linked to before) goes to `new`. The old conversation stays in the module as history; nothing is deleted.
    pub fn succeed(&mut self, old: &str, new: &str) {
        if old == new || self.next_of(old) == Some(new) {
            return;
        }
        let was = self.next_of(old).map(str::to_owned);
        self.next.retain(|(o, _)| o != old);
        self.next.push((old.to_owned(), new.to_owned()));
        if !self.is_group(new) {
            self.groups.push(new.to_owned());
        }
        for q in self.group_queue.iter_mut().filter(|q| q.convo == old || was.as_ref() == Some(&q.convo)) {
            q.convo = new.to_owned();
        }
        self.invites.retain(|i| i.convo != old);
        self.moves.retain(|(o, _)| o != old);
        self.save();
    }

    /// A member moved one of our groups and added us to the new group: its description names the old one. Link the
    /// two. This is also the check against a second new group: the first live one that we see wins, and a link to a
    /// new group that is not live for us (we restarted, or it is gone) gives way to a live one. A description that
    /// names a group that is not ours is ignored.
    pub fn follow(&mut self, convos: &[Convo]) {
        let live = |id: &str| convos.iter().any(|c| c.id == id && !c.history_only);
        for c in convos.iter().filter(|c| c.kind == "group" && !c.history_only) {
            if let Some(old) = c.description.as_deref().and_then(|d| d.strip_prefix(AFTER)) {
                if self.is_group(old) && !self.next_of(old).is_some_and(live) {
                    self.succeed(old, &c.id);
                }
            }
        }
    }

    /// The group that `convo` moved to in the end (itself when it did not move).
    #[must_use]
    pub fn newest_group(&self, convo: &str) -> String {
        let mut at = convo.to_owned();
        for _ in 0..MAX_CHAIN {
            match self.next_of(&at) {
                Some(n) => at = n.to_owned(),
                None => break,
            }
        }
        at
    }

    /// The groups of the chat list: one row each, also when nothing was said in it.
    fn group_rows<'a>(&'a self, convos: &'a [Convo]) -> impl Iterator<Item = (u64, Value)> + 'a {
        self.groups.iter().filter_map(move |id| {
            let c = convos.iter().find(|c| c.id == *id)?;
            let (preview, last_ms) = match self.queued_group(id).last() {
                Some(q) => (format!("Sending: {}", q.text), q.ts_ms),
                None => (c.preview.clone().unwrap_or_default(), c.last_ms),
            };
            let (members, pending, _) = self.rosters.get(id).copied().unwrap_or_default();
            let row = json!({
                "key": id,
                "group": true,
                "name": c.name.as_deref().unwrap_or("Group"),
                "members": members,
                "pending": pending,
                "preview": preview_of(&preview),
                "last_ms": last_ms,
                "queued": self.queued_group(id).count(),
                "history_only": c.history_only,
                "has_address": true,
                "unread": unread(&self.group_read, id, &[c]),
                "next": self.next_of(id),
            });
            Some((last_ms, row))
        })
    }

    /// The members of `convo` for the screen: those in the group, those the group has not committed, and those
    /// that wait for their chat address. `key` is the member's key, or the address when we do not know whose it is.
    #[must_use]
    pub fn roster_json(
        &self,
        convo: &str,
        roster: &[Member],
        names: &BTreeMap<Key, String>,
    ) -> Vec<Value> {
        let mine = self.my_addr();
        let known = |who: Option<Key>, key: &str, pending: bool, me: bool| json!({ "key": who.map_or_else(|| key.to_owned(), hex::encode), "name": who.and_then(|w| names.get(&w)), "me": me, "pending": pending });
        let inn = roster.iter().map(|m| {
            known(
                self.who_of(&m.address),
                &m.address,
                m.pending,
                mine == Some(m.address.as_str()),
            )
        });
        let waiting = self
            .waiting(convo)
            .map(|w| known(Some(*w), "", true, false));
        inn.chain(waiting).collect()
    }

    /// Keep a message for the group `convo` until it can be sent.
    pub fn enqueue_group(&mut self, convo: &str, text: &str, now_ms: u64) -> Result<(), String> {
        let text = clean(text)?;
        let convo = &self.newest_group(convo); // a group that moved takes its messages in the new group
        if self.queued_group(convo).count() >= MAX_QUEUED {
            return Err("There are too many unsent messages to this group.".into());
        }
        self.group_queue.push(GroupQueued {
            convo: convo.to_owned(),
            text: text.to_owned(),
            ts_ms: now_ms,
        });
        self.save();
        Ok(())
    }

    /// The unsent messages to the group `convo`, oldest first.
    pub fn queued_group<'a>(
        &'a self,
        convo: &'a str,
    ) -> impl Iterator<Item = &'a GroupQueued> + 'a {
        self.group_queue.iter().filter(move |q| q.convo == convo)
    }

    /// The groups that have unsent messages.
    #[must_use]
    pub fn group_targets(&self) -> Vec<String> {
        distinct(self.group_queue.iter().map(|q| q.convo.clone()))
    }

    /// The first `n` unsent messages to `convo` were sent.
    pub fn unqueue_group(&mut self, convo: &str, n: usize) {
        drop_first(&mut self.group_queue, n, |q| q.convo == convo);
        self.save();
    }

    /// The group `convo` is read up to `ms` (never back).
    pub fn mark_group_read_at(&mut self, convo: &str, ms: u64) {
        if advance(&mut self.group_read, &convo.to_owned(), ms) {
            self.save();
        }
    }

    /// The group `convo` is read up to its newest message.
    pub fn mark_group_read(&mut self, convo: &str, convos: &[Convo]) {
        if let Some(c) = convos.iter().find(|c| c.id == convo) {
            self.mark_group_read_at(convo, c.last_ms);
        }
    }

    /// The messages of the group `convo`, oldest first, then the unsent ones. Each message that came in names its
    /// sender (`sender_name`, and `sender_key`: the member's key, or the address when no member announced it).
    #[must_use]
    pub fn group_thread(
        &self,
        msgs: Vec<Msg>,
        queued: &[&GroupQueued],
        names: &BTreeMap<Key, String>,
        old: bool,
    ) -> Vec<Value> {
        let rows = msgs
            .into_iter()
            .map(|m| {
                let mut r = row(m.from_self, &m.content, m.timestamp_ms, old, false);
                if let Some(addr) = m.sender.as_deref().filter(|_| !m.from_self) {
                    let who = self.who_of(addr);
                    r["sender_key"] = json!(who.map_or_else(|| addr.to_owned(), hex::encode));
                    r["sender_name"] = json!(who.and_then(|w| names.get(&w)));
                }
                (m.timestamp_ms, r)
            })
            .collect();
        ordered(rows, queued.iter().map(|q| (q.ts_ms, q.text.as_str())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    const SITE: Key = [7; 32];
    const NOW: u64 = 1_000_000;

    fn person(seed: u8) -> (SigningKey, Key) {
        let k = SigningKey::from_bytes(&[seed; 32]);
        let pk = k.verifying_key().to_bytes();
        (k, pk)
    }

    fn book() -> ChatBook {
        ChatBook {
            site: SITE,
            ..ChatBook::default()
        }
    }

    fn addr_msg(book: &mut ChatBook, key: &SigningKey, addr: &str, now: u64) -> Option<AddrMsg> {
        let wire = book.announce(key, &SITE, addr, now)?;
        match Wire::decode(&wire) {
            Some(Wire::ChatAddr(m)) => Some(m),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn a_new_conversation_takes_no_message_until_the_other_side_has_had_time_to_join() {
        let mut b = book();
        b.opened("new", NOW);
        assert!(!b.settled("new", NOW + SETTLE_MS - 1));
        assert!(b.settled("new", NOW + SETTLE_MS));
        assert!(b.settled("other", NOW)); // a conversation that this start did not open is ready
    }

    #[test]
    fn an_announced_address_is_kept_and_the_newest_wins_with_the_old_ones_held() {
        let ((ka, a), (_, me)) = (person(1), person(2));
        let (mut sender, mut mine) = (book(), book());
        let members = [a, me];
        let first = addr_msg(&mut sender, &ka, "addr-1", NOW).unwrap();
        assert_eq!(
            mine.on_addr(&SITE, &first, &members),
            Some(a),
            "news: the newest address changed"
        );
        assert_eq!(
            mine.on_addr(&SITE, &first, &members),
            None,
            "a repeat is not news"
        );
        assert_eq!(mine.latest(&a), Some("addr-1"));
        let second = addr_msg(&mut sender, &ka, "addr-2", NOW + 10).unwrap();
        assert_eq!(mine.on_addr(&SITE, &second, &members), Some(a));
        assert_eq!(mine.latest(&a), Some("addr-2"));
        assert_eq!(
            mine.who_of("addr-1"),
            Some(a),
            "an old chat still shows the right name"
        );
        assert_eq!(
            mine.on_addr(&SITE, &first, &members),
            None,
            "an old announce that arrives late is held but not news"
        );
        assert_eq!(mine.latest(&a), Some("addr-2"));
    }

    #[test]
    fn an_announce_from_a_non_member_or_with_a_bad_signature_is_ignored() {
        let ((ka, a), (_, me)) = (person(1), person(2));
        let (mut sender, mut mine) = (book(), book());
        let m = addr_msg(&mut sender, &ka, "addr-1", NOW).unwrap();
        assert_eq!(
            mine.on_addr(&SITE, &m, &[me]),
            None,
            "not a member of the site"
        );
        let mut forged = m.clone();
        forged.body.addr = "evil".into();
        assert_eq!(
            mine.on_addr(&SITE, &forged, &[a, me]),
            None,
            "the signature covers the address"
        );
        assert_eq!(
            mine.on_addr(&[8; 32], &m, &[a, me]),
            None,
            "the signature covers the site"
        );
        assert_eq!(mine.latest(&a), None);
    }

    #[test]
    fn we_announce_again_only_when_the_address_changed_and_repeat_the_same_message() {
        let (k, _) = person(1);
        let mut b = book();
        let first = b.announce(&k, &SITE, "addr-1", NOW).unwrap();
        assert_eq!(b.announce(&k, &SITE, "addr-1", NOW + 5), None);
        assert_eq!(
            b.resend(&k, &SITE),
            Some(first),
            "the repeat is the same signed message"
        );
        assert!(
            b.announce(&k, &SITE, "addr-2", NOW + 9).is_some(),
            "a new start gives a new address"
        );
    }

    #[test]
    fn the_queue_holds_messages_in_order_per_person_and_drops_what_was_sent() {
        let (_, a) = person(1);
        let (_, c) = person(3);
        let mut b = book();
        b.enqueue(a, " hello ", NOW).unwrap();
        b.enqueue(c, "other", NOW + 1).unwrap();
        b.enqueue(a, "second", NOW + 2).unwrap();
        assert!(b.enqueue(a, "   ", NOW).is_err(), "no empty message");
        assert!(
            b.enqueue(a, &"x".repeat(MAX_TEXT + 1), NOW).is_err(),
            "no huge message"
        );
        assert_eq!(
            b.queued(&a).map(|q| q.text.as_str()).collect::<Vec<_>>(),
            ["hello", "second"]
        );
        assert_eq!(b.queue_targets(), vec![a, c]);
        b.unqueue(&a, 1);
        assert_eq!(
            b.queued(&a).map(|q| q.text.as_str()).collect::<Vec<_>>(),
            ["second"]
        );
        assert_eq!(b.queued(&c).count(), 1);
        for i in 0..MAX_QUEUED {
            let _ = b.enqueue(c, "more", NOW + i as u64);
        }
        assert!(
            b.enqueue(c, "one too many", NOW).is_err(),
            "one person cannot fill the disk"
        );
    }

    #[test]
    fn a_conversation_belongs_to_a_person_by_its_sender_and_the_file_keeps_everything() {
        let dir = std::env::temp_dir().join(format!("pukaar-chat-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let ((ka, a), (_, me)) = (person(1), person(2));
        let mut sender = book();
        let m = addr_msg(&mut sender, &ka, "addr-1", NOW).unwrap();
        let mut mine = ChatBook::load(&dir, SITE);
        assert!(
            !mine.link_sender("c1", "addr-1"),
            "an unknown sender is not linked"
        );
        mine.on_addr(&SITE, &m, &[a, me]);
        assert!(mine.link_sender("c1", "addr-1"));
        mine.link("c2", a, "addr-0");
        mine.enqueue(a, "later", NOW).unwrap();
        let again = ChatBook::load(&dir, SITE);
        assert_eq!(again.who_of_convo("c1"), Some(a));
        assert_eq!(
            again.convos_for(&a, "addr-1").collect::<Vec<_>>(),
            ["c1"],
            "only a conversation with their newest address can be sent into"
        );
        assert_eq!(again.latest(&a), Some("addr-1"));
        assert_eq!(again.queued(&a).count(), 1, "the queue survives a restart");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_module_lists_read_with_float_times_and_either_wrapping() {
        let convo = json!({ "convo_id": "c1", "message_count": 2, "last_activity_ms": 1781234567890.0, "kind": "direct", "history_only": false });
        let a: Vec<Convo> = list_of(json!([convo.clone()])).unwrap();
        let b: Vec<Convo> = list_of(json!({ "success": true, "value": [convo] })).unwrap();
        assert_eq!(
            (a[0].last_ms, a[0].preview.clone()),
            (1_781_234_567_890, None)
        );
        assert_eq!(b.len(), 1);
        assert!(list_of::<Msg>(json!("junk")).is_err());
    }

    #[test]
    fn a_thread_joins_the_old_and_new_conversations_and_ends_with_the_unsent_messages() {
        let (_, a) = person(1);
        let q = |t: &str, ts| Queued {
            to: a,
            text: t.into(),
            ts_ms: ts,
        };
        let old = (
            true,
            vec![Msg {
                from_self: true,
                content: "hi".into(),
                timestamp_ms: 10,
                sender: None,
            }],
        );
        let new = (
            false,
            vec![
                Msg {
                    from_self: false,
                    content: "yo".into(),
                    timestamp_ms: 30,
                    sender: None,
                },
                Msg {
                    from_self: true,
                    content: "hey".into(),
                    timestamp_ms: 20,
                    sender: None,
                },
            ],
        );
        let rows = thread_messages(vec![new, old], &[&q("later", 40)]);
        let text: Vec<_> = rows
            .iter()
            .map(|r| r["content"].as_str().unwrap())
            .collect();
        assert_eq!(text, ["hi", "hey", "yo", "later"]);
        assert_eq!(rows[0]["old"], true);
        assert_eq!(rows[2]["old"], false);
        assert_eq!(rows[3]["queued"], true);
    }

    #[test]
    fn a_linked_conversation_without_a_message_is_not_in_the_chat_list_until_one_is_queued() {
        let ((_, a), (_, me)) = (person(1), person(2));
        let mut mine = book();
        mine.link("c1", a, "addr-a");
        let empty = [Convo {
            id: "c1".into(),
            last_ms: 0,
            preview: None,
            message_count: 0,
            history_only: false,
            kind: "direct".into(),
            name: None,
            description: None,
        }];
        let list = |b: &ChatBook| b.threads_json(&empty, &BTreeMap::new(), &[a, me], &me);
        assert_eq!(list(&mine).as_array().unwrap().len(), 0);
        mine.enqueue(a, "hi", NOW).unwrap();
        assert_eq!(list(&mine).as_array().unwrap().len(), 1);
    }

    #[test]
    fn the_chat_list_shows_active_members_newest_first_and_an_unsent_message_as_the_preview() {
        let (((ka, a), (_, b)), (_, me)) = ((person(1), person(3)), person(2));
        let mut mine = book();
        mine.link("c1", a, "addr-a");
        mine.link("c2", b, "addr-b");
        mine.enqueue(b, "wait", NOW + 9).unwrap();
        let convo = |id: &str, last_ms, history_only| Convo {
            id: id.into(),
            last_ms,
            preview: Some("hi".into()),
            message_count: 1,
            history_only,
            kind: "direct".into(),
            name: None,
            description: None,
        };
        let convos = [
            convo("c1", NOW, true),
            convo("c2", NOW - 5, false),
            convo("other-app", NOW + 99, false),
        ];
        let names = BTreeMap::from([(a, "Asha".to_string())]);
        let rows = mine.threads_json(&convos, &names, &[a, b, me], &me);
        let keys: Vec<_> = rows
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["key"].as_str().unwrap().to_owned())
            .collect();
        assert_eq!(
            keys,
            [hex::encode(b), hex::encode(a)],
            "the unsent message is the newest activity; a chat of another app is not shown"
        );
        assert_eq!(rows[0]["preview"], "Sending: wait");
        assert_eq!(
            (rows[1]["name"].as_str(), rows[1]["history_only"].clone()),
            (Some("Asha"), json!(true))
        );
        let removed = mine.threads_json(&convos, &names, &[b, me], &me);
        assert_eq!(
            removed.as_array().unwrap().len(),
            1,
            "a removed member has no chat in the list"
        );
        let _ = ka;
    }

    #[test]
    fn a_thread_is_unread_until_its_newest_live_message_is_marked_read_and_that_survives_a_restart()
    {
        let dir = std::env::temp_dir().join(format!("pukaar-chat-read-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let ((_, a), (_, me)) = (person(1), person(2));
        let mut mine = ChatBook::load(&dir, SITE);
        mine.link("c1", a, "addr-a");
        mine.link("old", a, "addr-0");
        let convo = |id: &str, last_ms, history_only| Convo {
            id: id.into(),
            last_ms,
            preview: None,
            message_count: 0,
            history_only,
            kind: "direct".into(),
            name: None,
            description: None,
        };
        let unread = |b: &ChatBook, convos: &[Convo]| {
            b.threads_json(convos, &BTreeMap::new(), &[a, me], &me)[0]["unread"].clone()
        };
        let live = [convo("c1", NOW, false)];
        assert_eq!(unread(&mine, &live), json!(true), "never read");
        mine.mark_thread_read(&a, &live);
        assert_eq!(unread(&mine, &live), json!(false));
        assert_eq!(
            unread(&mine, &[convo("c1", NOW + 1, false)]),
            json!(true),
            "a newer message"
        );
        assert_eq!(
            unread(&ChatBook::load(&dir, SITE), &live),
            json!(false),
            "the mark is kept in the file"
        );
        let after_restart = [convo("c1", NOW + 5, true), convo("old", NOW + 9, true)];
        assert_eq!(
            unread(&mine, &after_restart),
            json!(false),
            "a history-only conversation is never unread"
        );
        mine.mark_read(&a, 5);
        assert_eq!(
            unread(&mine, &live),
            json!(false),
            "the mark never goes back"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn group_convo(id: &str, name: &str, last_ms: u64, history_only: bool) -> Convo {
        Convo {
            id: id.into(),
            last_ms,
            preview: Some("hello all".into()),
            message_count: u64::from(last_ms > 0),
            history_only,
            kind: "group".into(),
            name: Some(name.into()),
            description: None,
        }
    }

    fn roster(addrs: &[(&str, bool)]) -> Vec<Member> {
        addrs
            .iter()
            .map(|(a, pending)| Member {
                address: (*a).into(),
                pending: *pending,
            })
            .collect()
    }

    /// A book that knows the chat address of `who` (as announced on the site channel).
    fn knows(b: &mut ChatBook, seed: u8, addr: &str, members: &[Key]) -> Key {
        let (k, who) = person(seed);
        let m = addr_msg(&mut book(), &k, addr, NOW).unwrap();
        b.on_addr(&SITE, &m, members);
        who
    }

    #[test]
    fn a_group_is_listed_at_once_with_its_name_and_member_count_and_only_when_it_is_ours() {
        let ((_, a), (_, b), (_, me)) = (person(1), person(3), person(2));
        let mut mine = book();
        mine.start_group("g1", &[a, b]);
        let convos = [
            group_convo("g1", "Book club", 0, false),
            group_convo("not-ours", "Other app", NOW, false),
        ];
        let list = |bk: &ChatBook| bk.threads_json(&convos, &BTreeMap::new(), &[a, b, me], &me);
        let rows = list(&mine);
        let rows = rows.as_array().unwrap();
        assert_eq!(
            rows.len(),
            1,
            "a new group shows without any message; a group of another app does not"
        );
        assert_eq!(
            (
                rows[0]["key"].as_str(),
                rows[0]["group"].clone(),
                rows[0]["name"].as_str()
            ),
            (Some("g1"), json!(true), Some("Book club"))
        );
        let ours = roster(&[("me", false), ("addr-a", false), ("addr-b", true)]);
        mine.group_seen("g1", &ours, &[a, b, me], &me, NOW);
        let rows = list(&mine);
        assert_eq!(
            (rows[0]["members"].clone(), rows[0]["pending"].clone()),
            (json!(2), json!(1))
        );
        assert!(mine.is_group("g1") && !mine.is_group("not-ours"));
    }

    #[test]
    fn a_group_that_someone_made_is_ours_once_a_member_we_know_is_in_it() {
        let ((_, a), (_, me)) = (person(1), person(2));
        let mut mine = book();
        knows(&mut mine, 1, "addr-a", &[a, me]);
        let with = |who: &str| roster(&[(who, false), ("me", false)]);
        assert!(!mine.group_seen("g", &with("stranger"), &[a, me], &me, NOW));
        assert!(!mine.is_group("g"), "nobody of the site is in it");
        mine.group_seen("g", &with("addr-a"), &[me], &me, NOW);
        assert!(
            !mine.is_group("g"),
            "a member who left the site does not count"
        );
        mine.group_seen("g", &with("addr-a"), &[a, me], &me, NOW);
        assert!(mine.is_group("g"));
    }

    #[test]
    fn a_member_is_added_once_and_goes_in_only_when_their_address_is_known_and_the_group_is_live() {
        let ((_, me), (_, c)) = (person(2), person(5));
        let mut mine = book();
        let a = knows(&mut mine, 1, "addr-a", &[person(1).1, me, c]);
        let all = [a, c, me];
        mine.start_group("g1", &[a, c]);
        mine.want("g1", a); // asked twice
        let live = [group_convo("g1", "Club", 0, false)];
        let due = mine.due_invites(&all, &live);
        assert_eq!(
            due,
            vec![("g1".to_owned(), a, "addr-a".to_owned())],
            "c has no address yet, so c waits"
        );
        assert_eq!(mine.waiting("g1").count(), 2);
        mine.invited("g1", &a);
        assert!(mine.due_invites(&all, &live).is_empty());
        assert_eq!(mine.waiting("g1").collect::<Vec<_>>(), [&c]);
        assert!(
            mine.due_invites(&[a, me], &live).is_empty(),
            "a member who left the site is dropped"
        );
        assert_eq!(mine.waiting("g1").count(), 0);
        mine.want("g1", a);
        assert!(
            mine.due_invites(&all, &[group_convo("g1", "Club", 0, true)])
                .is_empty(),
            "a group from before a restart cannot take members"
        );
        assert_eq!(mine.waiting("g1").count(), 0);
    }

    #[test]
    fn a_member_who_restarted_is_added_again_by_the_lowest_member_only() {
        let lowest = [1u8, 2, 5].into_iter().min_by_key(|s| person(*s).1).unwrap();
        let others: Vec<u8> = [1u8, 2, 5].into_iter().filter(|s| *s != lowest).collect();
        let ((sk, me), (_, a), (_, c)) = (person(lowest), person(others[0]), person(others[1]));
        let all = [a, c, me];
        let mut mine = book();
        mine.announce(&sk, &SITE, "me", NOW);
        knows(&mut mine, others[0], "addr-a", &all);
        knows(&mut mine, others[1], "addr-c", &all);
        mine.start_group("g1", &[]);
        let ours = roster(&[("me", false), ("addr-a", false), ("addr-c", false)]);
        mine.readd("g1", &ours, &all, &me);
        assert_eq!(mine.waiting("g1").count(), 0, "nobody restarted");
        let (kc, _) = person(others[1]);
        let m = addr_msg(&mut book(), &kc, "addr-c2", NOW + 1).unwrap();
        mine.on_addr(&SITE, &m, &all);
        mine.readd("g1", &ours, &all, &me);
        assert_eq!(mine.waiting("g1").collect::<Vec<_>>(), [&c], "c has a new address that the group does not have");
        mine.readd("g1", &ours, &all, &a);
        assert_eq!(mine.waiting("g1").count(), 1, "a member that is not the lowest adds nobody (and a is not the lowest)");
        let now_in = roster(&[("me", false), ("addr-a", false), ("addr-c2", true)]);
        mine.invited("g1", &c);
        mine.readd("g1", &now_in, &all, &me);
        assert_eq!(mine.waiting("g1").count(), 0, "a pending add is not asked again");
    }

    #[test]
    fn nothing_is_sent_into_a_group_until_its_members_had_time_to_join_and_what_waits_is_kept() {
        let (_, me) = person(2);
        let mut mine = book();
        mine.start_group("g1", &[person(1).1]);
        let alone = roster(&[("me", false)]);
        assert!(
            !mine.group_seen("g1", &alone, &[me], &me, NOW),
            "only me, and an invite is waiting"
        );
        let joined = roster(&[("me", false), ("addr-a", false)]);
        assert!(
            !mine.group_seen("g1", &joined, &[me], &me, NOW + 1),
            "the roster just changed"
        );
        assert!(
            !mine.group_seen("g1", &joined, &[me], &me, NOW + SETTLE_MS),
            "a new member was added {} ms ago",
            SETTLE_MS - 1
        );
        assert!(mine.group_seen("g1", &joined, &[me], &me, NOW + 1 + SETTLE_MS));
        mine.enqueue_group("g1", " hi ", NOW).unwrap();
        assert!(mine.enqueue_group("g1", "  ", NOW).is_err());
        assert_eq!(
            mine.queued_group("g1")
                .map(|q| q.text.as_str())
                .collect::<Vec<_>>(),
            ["hi"]
        );
        assert_eq!(mine.group_targets(), ["g1"]);
        mine.unqueue_group("g1", 1);
        assert!(mine.group_targets().is_empty());
    }

    #[test]
    fn a_group_message_shows_who_sent_it_by_the_address_book_and_a_short_key_when_unknown() {
        let (_, me) = person(2);
        let mut mine = book();
        let a = knows(&mut mine, 1, "addr-asha-0123456789", &[person(1).1, me]);
        let names = BTreeMap::from([(a, "Asha".to_string())]);
        let msg = |from_self, text: &str, ts, sender: Option<&str>| Msg {
            from_self,
            content: text.into(),
            timestamp_ms: ts,
            sender: sender.map(Into::into),
        };
        let unsent = GroupQueued {
            convo: "g1".into(),
            text: "later".into(),
            ts_ms: 40,
        };
        let rows = mine.group_thread(
            vec![
                msg(false, "from a stranger", 30, Some("addr-stranger-99")),
                msg(true, "mine", 20, None),
                msg(false, "from Asha", 10, Some("addr-asha-0123456789")),
            ],
            &[&unsent],
            &names,
            false,
        );
        let text: Vec<_> = rows
            .iter()
            .map(|r| r["content"].as_str().unwrap())
            .collect();
        assert_eq!(text, ["from Asha", "mine", "from a stranger", "later"]);
        assert_eq!(
            (
                rows[0]["sender_name"].as_str(),
                rows[0]["sender_key"].as_str()
            ),
            (Some("Asha"), Some(hex::encode(a).as_str()))
        );
        assert_eq!(
            (
                rows[2]["sender_name"].clone(),
                rows[2]["sender_key"].as_str()
            ),
            (Value::Null, Some("addr-stranger-99"))
        );
        assert!(
            rows[1]["sender_key"].is_null(),
            "our own message has no sender"
        );
        assert_eq!(rows[3]["queued"], true);
    }

    #[test]
    fn the_members_of_a_group_show_names_us_and_who_still_waits_and_unread_is_kept_per_group() {
        let dir = std::env::temp_dir().join(format!("pukaar-chat-group-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (_, me) = person(2);
        let mut mine = ChatBook::load(&dir, SITE);
        let a = knows(&mut mine, 1, "addr-a", &[person(1).1, me]);
        mine.announce(&person(2).0, &SITE, "addr-me", NOW);
        let c = person(5).1;
        mine.start_group("g1", &[c]);
        let names = BTreeMap::from([(a, "Asha".to_string())]);
        let members = mine.roster_json(
            "g1",
            &roster(&[("addr-me", false), ("addr-a", true), ("addr-x", false)]),
            &names,
        );
        let view: Vec<_> = members
            .iter()
            .map(|m| {
                (
                    m["name"].as_str().unwrap_or(""),
                    m["me"].as_bool().unwrap(),
                    m["pending"].as_bool().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            view,
            [
                ("", true, false),
                ("Asha", false, true),
                ("", false, false),
                ("", false, true)
            ],
            "the last one waits for an address"
        );
        let convos = |last_ms, old| [group_convo("g1", "Club", last_ms, old)];
        let unread = |b: &ChatBook, c: &[Convo]| {
            b.threads_json(c, &BTreeMap::new(), &[a, c_key(), me], &me)[0]["unread"].clone()
        };
        fn c_key() -> Key {
            person(5).1
        }
        assert_eq!(
            unread(&mine, &convos(NOW, false)),
            json!(true),
            "never read"
        );
        mine.mark_group_read("g1", &convos(NOW, false));
        assert_eq!(unread(&mine, &convos(NOW, false)), json!(false));
        assert_eq!(
            unread(&mine, &convos(NOW + 1, false)),
            json!(true),
            "a newer message"
        );
        assert_eq!(
            unread(&ChatBook::load(&dir, SITE), &convos(NOW, false)),
            json!(false),
            "the mark is kept in the file, with the group"
        );
        assert_eq!(
            unread(&mine, &convos(NOW + 9, true)),
            json!(false),
            "a group from before a restart is never unread"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn ask_msg(wire: &[u8]) -> AskMsg {
        match Wire::decode(wire) {
            Some(Wire::ChatAsk(m)) => m,
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn an_ask_is_answered_with_our_announce_by_members_only_and_neither_side_can_flood() {
        let ((ka, a), (kb, b), (_, x)) = (person(1), person(3), person(9));
        let (mut asker, mut other) = (book(), book());
        let ask = asker.ask(&ka, &SITE, NOW).unwrap();
        assert!(asker.asking(NOW + 1));
        assert!(!asker.asking(NOW + ASKING_MS));
        assert_eq!(asker.ask(&ka, &SITE, NOW + 1), None, "one ask per minute");
        assert!(asker.ask(&ka, &SITE, NOW + ASK_EVERY_MS).is_some());
        let m = ask_msg(&ask);
        assert_eq!(other.on_ask(&kb, &SITE, &m, &[a, b], NOW), None, "no address to give yet");
        let mine = other.announce(&kb, &SITE, "addr-b", NOW).unwrap();
        assert_eq!(other.on_ask(&kb, &SITE, &m, &[b, x], NOW), None, "the asker is not a member");
        let mut forged = m.clone();
        forged.body.ts += 1;
        assert_eq!(other.on_ask(&kb, &SITE, &forged, &[a, b], NOW), None, "the signature covers the body");
        assert_eq!(other.on_ask(&kb, &[8; 32], &m, &[a, b], NOW), None, "the signature covers the site");
        assert_eq!(other.on_ask(&ka, &SITE, &m, &[a, b], NOW), None, "our own ask is not answered");
        assert_eq!(other.on_ask(&kb, &SITE, &m, &[a, b], NOW), Some(mine.clone()), "the answer is our announce");
        assert_eq!(other.on_ask(&kb, &SITE, &m, &[a, b], NOW + 1), None, "one answer per 30 seconds");
        assert_eq!(other.on_ask(&kb, &SITE, &m, &[a, b], NOW + ANSWER_EVERY_MS), Some(mine));
    }

    #[test]
    fn a_restarted_member_with_the_lowest_key_does_not_stop_the_lowest_live_member_from_adding_them() {
        let mut seeds = [1u8, 3, 5];
        seeds.sort_by_key(|s| person(*s).1);
        let [low, mid, high] = seeds;
        let ((_, l), (km, m), (_, h)) = (person(low), person(mid), person(high));
        let all = [l, m, h];
        let mut mine = book();
        mine.announce(&km, &SITE, "addr-m", NOW);
        knows(&mut mine, low, "addr-l", &all);
        knows(&mut mine, high, "addr-h", &all);
        mine.start_group("g1", &[]);
        let ours = roster(&[("addr-l", false), ("addr-m", false), ("addr-h", false)]);
        mine.readd("g1", &ours, &all, &m);
        assert_eq!(mine.waiting("g1").count(), 0, "l is live and has the lowest key, so l adds");
        let again = addr_msg(&mut book(), &person(low).0, "addr-l2", NOW + 1).unwrap();
        mine.on_addr(&SITE, &again, &all);
        mine.readd("g1", &ours, &all, &m);
        assert_eq!(mine.waiting("g1").collect::<Vec<_>>(), [&l], "l restarted, so m is the lowest live member");
        let mut theirs = book();
        theirs.announce(&person(high).0, &SITE, "addr-h", NOW);
        knows(&mut theirs, low, "addr-l2", &all);
        knows(&mut theirs, mid, "addr-m", &all);
        theirs.start_group("g1", &[]);
        theirs.readd("g1", &ours, &all, &h);
        assert_eq!(theirs.waiting("g1").count(), 0, "h leaves it to m");
    }

    #[test]
    fn a_member_who_restarts_before_their_add_commits_in_a_group_of_two_moves_the_group_at_once() {
        let ((k, me), (ka, a)) = (person(1), person(3));
        let all = [a, me];
        let mut mine = book();
        mine.announce(&k, &SITE, "addr-me", NOW);
        knows(&mut mine, 3, "addr-a", &all);
        mine.start_group("g1", &[]);
        let pending_old = roster(&[("addr-me", false), ("addr-a", true)]);
        assert_eq!(mine.moving("g1", &pending_old, &all, &me, NOW), None, "nobody restarted");
        let m = addr_msg(&mut book(), &ka, "addr-a2", NOW + 1).unwrap();
        mine.on_addr(&SITE, &m, &all);
        mine.readd("g1", &pending_old, &all, &me);
        assert_eq!(mine.waiting("g1").count(), 0, "a re-add into a group of two never commits, so it is not tried");
        assert_eq!(mine.moving("g1", &pending_old, &all, &me, NOW + 2), Some((me, NOW + 2)), "FC-15: we move it at once");
    }

    /// Three members `l < m < h` in `g1`, from the view of `who` (its seed), which knows the first addresses of all.
    fn three(who: usize) -> (ChatBook, [Key; 3], [u8; 3], Vec<Member>) {
        let mut seeds = [1u8, 3, 5];
        seeds.sort_by_key(|s| person(*s).1);
        let keys = seeds.map(|s| person(s).1);
        let mut b = book();
        let addrs = ["addr-l", "addr-m", "addr-h"];
        for (i, s) in seeds.iter().enumerate() {
            if i == who {
                b.announce(&person(*s).0, &SITE, addrs[i], NOW);
            } else {
                knows(&mut b, *s, addrs[i], &keys);
            }
        }
        b.start_group("g1", &[]);
        let r = roster(&[("addr-l", false), ("addr-m", false), ("addr-h", false)]);
        (b, keys, seeds, r)
    }

    #[test]
    fn the_lowest_live_member_moves_the_group_when_the_re_add_does_not_commit_in_time() {
        let (mut mine, [l, m, h], seeds, r) = three(1);
        let all = [l, m, h];
        assert_eq!(mine.moving("g1", &r, &all, &m, NOW), None, "nobody restarted");
        knows(&mut mine, seeds[0], "addr-l2", &all);
        mine.readd("g1", &r, &all, &m);
        assert_eq!(mine.waiting("g1").collect::<Vec<_>>(), [&l], "the cheap try first");
        assert_eq!(mine.moving("g1", &r, &all, &m, NOW + 5), Some((m, NOW + 5 + ROLL_AFTER_MS)), "m is the lowest live member");
        let pending = roster(&[("addr-l", false), ("addr-m", false), ("addr-h", false), ("addr-l2", true)]);
        assert_eq!(mine.moving("g1", &pending, &all, &m, NOW + 9), Some((m, NOW + 5 + ROLL_AFTER_MS)), "a pending re-add does not stop the clock");
        let done = roster(&[("addr-l", false), ("addr-m", false), ("addr-h", false), ("addr-l2", false)]);
        assert_eq!(mine.moving("g1", &done, &all, &m, NOW + 10), None, "the re-add committed");
        let (mut theirs, ..) = three(2);
        knows(&mut theirs, seeds[0], "addr-l2", &all);
        assert_eq!(theirs.moving("g1", &r, &all, &h, NOW).map(|w| w.0), Some(m), "h agrees that m moves it");
        assert_eq!(theirs.moving("g1", &r, &[m, h], &h, NOW), None, "a member removed from the site does not count");
    }

    #[test]
    fn after_our_restart_a_live_member_moves_the_group_and_when_all_restarted_the_lowest_with_a_new_address_does() {
        let (mut mine, [l, m, h], seeds, r) = three(2);
        let all = [l, m, h];
        mine.readd("g1", &r, &all, &h); // keeps the roster
        mine.announce(&person(seeds[2]).0, &SITE, "addr-h2", NOW + 1); // our restart: the group is read-only for us
        let old = mine.last_roster("g1");
        assert_eq!(mine.moving("g1", &old, &all, &h, NOW + 2), Some((l, NOW + 2 + ROLL_AFTER_MS)), "l can add us back first");
        knows(&mut mine, seeds[0], "addr-l2", &all);
        assert_eq!(mine.moving("g1", &old, &all, &h, NOW + 3).map(|w| w.0), Some(m), "l restarted too");
        knows(&mut mine, seeds[1], "addr-m2", &all);
        assert_eq!(mine.moving("g1", &old, &all, &h, NOW + 4), Some((l, NOW + 2)), "nobody is live: the lowest with a new address, at once");
        assert_eq!(mine.moving("g1", &old, &[m, h], &h, NOW + 4).map(|w| w.0), Some(m), "l left the site");
        assert_eq!(mine.moving("unknown", &[], &all, &h, NOW), None, "a roster that we never saw: only the button");
        let senders = ["addr-m".to_owned(), "stranger".to_owned()];
        let mut back = mine.old_members("g1", &senders, &all, &h);
        back.sort_unstable();
        assert_eq!(back, vec![l, m], "the roster and the senders, once each, not us");
        assert_eq!(mine.old_members("g1", &[], &[m, h], &h), vec![m], "a member who left the site is not put back");
        assert_eq!(mine.old_members("never-seen", &senders, &all, &h), vec![m], "a file from before recovery: the senders");
    }

    #[test]
    fn a_new_group_is_made_once_and_its_fresh_invites_do_not_move_it_again() {
        let (mut mine, [l, m, h], seeds, r) = three(1);
        let all = [l, m, h];
        knows(&mut mine, seeds[0], "addr-l2", &all);
        assert!(mine.begin_move("g1", NOW));
        assert!(!mine.begin_move("g1", NOW + 1), "a second round makes no second group");
        assert!(mine.begin_move("g1", NOW + MOVE_RETRY_MS), "a move that did not finish is tried again");
        mine.start_group("g2", &[l, h]);
        mine.succeed("g1", "g2");
        assert!(mine.begin_move("g1", NOW + MOVE_RETRY_MS + 1), "the record went with the link");
        assert_eq!(mine.moving("g1", &r, &all, &m, NOW), None, "a group that moved is not moved again");
        let fresh = roster(&[("addr-m", false), ("addr-l2", true), ("addr-h", true)]);
        assert_eq!(mine.moving("g2", &fresh, &all, &m, NOW), None, "pending invites with the newest addresses are no restart");
    }

    #[test]
    fn a_group_that_moved_takes_the_queue_hides_behind_the_new_one_and_other_members_follow_one_live_group() {
        let dir = std::env::temp_dir().join(format!("pukaar-chat-again-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let ((_, a), (_, me)) = (person(1), person(2));
        let mut mine = ChatBook::load(&dir, SITE);
        mine.start_group("g1", &[a]);
        mine.enqueue_group("g1", "still here?", NOW).unwrap();
        mine.succeed("g1", "g2");
        assert_eq!(mine.next_of("g1"), Some("g2"));
        assert!(mine.is_group("g2"));
        assert_eq!(mine.group_targets(), ["g2"], "the message waits for the new group");
        mine.enqueue_group("g1", "written during the move", NOW + 1).unwrap();
        assert_eq!(mine.queued_group("g2").count(), 2, "a message to the old group goes to the new one");
        assert_eq!(mine.waiting("g1").count(), 0);
        mine.succeed("g2", "g4");
        assert_eq!(mine.before("g4"), ["g1", "g2"]);
        assert!(mine.before("g1").is_empty());
        assert_eq!(mine.newest_group("g1"), "g4");
        assert_eq!(mine.group_targets(), ["g4"]);
        let again = ChatBook::load(&dir, SITE);
        assert_eq!(again.next_of("g2"), Some("g4"), "the link is kept in the file");
        let mut new = group_convo("g4", "Club", NOW, false);
        new.description = None;
        let rows = mine.threads_json(&[group_convo("g1", "Club", NOW, true), new], &BTreeMap::new(), &[a, me], &me);
        let row = |id: &str| rows.as_array().unwrap().iter().find(|r| r["key"] == id).unwrap().clone();
        assert_eq!(row("g1")["next"], "g2", "the screen moves from the old group to the new one");
        assert!(row("g4")["next"].is_null());

        let mut theirs = book();
        theirs.start_group("old", &[]);
        let after = |id: &str, old: &str, history_only| {
            let mut c = group_convo(id, "Club", 0, history_only);
            c.description = Some(format!("{AFTER}{old}"));
            c
        };
        theirs.follow(&[after("dead", "old", true), after("evil", "someone-else", false)]);
        assert_eq!(theirs.next_of("old"), None, "a new group that is not live, or that names a group not ours, is not followed");
        assert!(!theirs.is_group("evil"));
        theirs.follow(&[after("new", "old", false), after("twin", "old", false)]);
        assert_eq!(theirs.next_of("old"), Some("new"), "the first live one wins");
        theirs.follow(&[after("new", "old", true), after("twin", "old", false)]);
        assert_eq!(theirs.next_of("old"), Some("twin"), "a dead link gives way to a live group");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_from_before_recovery_still_loads() {
        let b: ChatBook = serde_json::from_str(r#"{"addrs":[],"links":[],"queue":[],"groups":["g1"]}"#).unwrap();
        assert!(b.is_group("g1") && b.next_of("g1").is_none() && b.last_roster("g1").is_empty());
    }
}
