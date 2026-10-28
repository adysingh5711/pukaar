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

use crate::event::Key;
use crate::meet::Signed;
use crate::persist::SiteFile;
use crate::sync::Wire;
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Deserializer, Serialize};
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
const PREVIEW_CHARS: usize = 160;

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Queued {
    pub to: Key,
    pub text: String,
    pub ts_ms: u64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ChatBook {
    addrs: Vec<Held>,
    links: Vec<Link>,
    queue: Vec<Queued>,
    #[serde(skip)]
    pub site: Key,
    #[serde(skip)]
    dir: PathBuf,
    /// Our own current address and the time of its announce: set at each start of the chat module.
    #[serde(skip)]
    mine: Option<(u64, String)>,
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
    #[serde(default)]
    pub history_only: bool,
    #[serde(default)]
    pub kind: String,
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

/// The messages of a thread, oldest first: each conversation's messages (`old` for a conversation that
/// is history only), then the unsent ones, marked `queued`.
#[must_use]
pub fn thread_messages(parts: Vec<(bool, Vec<Msg>)>, queued: &[&Queued]) -> Vec<Value> {
    let mut rows: Vec<(u64, Value)> = parts
        .into_iter()
        .flat_map(|(old, msgs)| msgs.into_iter().map(move |m| (m.timestamp_ms, json!({ "from_self": m.from_self, "content": m.content, "ts_ms": m.timestamp_ms, "old": old, "queued": false }))))
        .collect();
    rows.sort_by_key(|r| r.0); // stable: the order inside a conversation stays
    let unsent = queued.iter().map(|q| (q.ts_ms, json!({ "from_self": true, "content": q.text, "ts_ms": q.ts_ms, "old": false, "queued": true })));
    rows.into_iter().chain(unsent).map(|r| r.1).collect()
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
        let text = text.trim();
        if text.is_empty() || text.len() > MAX_TEXT {
            return Err(format!("A message needs 1 to {MAX_TEXT} characters."));
        }
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
        let mut t: Vec<Key> = vec![];
        for q in &self.queue {
            if !t.contains(&q.to) {
                t.push(q.to);
            }
        }
        t
    }

    /// The first `n` unsent messages to `who` were sent.
    pub fn unqueue(&mut self, who: &Key, n: usize) {
        let mut left = n;
        self.queue.retain(|q| {
            q.to != *who || left == 0 || {
                left -= 1;
                false
            }
        });
        self.save();
    }

    /// The chat list, newest first: one row per active member that we have a conversation or an unsent message with.
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
            .map(|who| {
                let mine: Vec<&Convo> = convos
                    .iter()
                    .filter(|c| self.who_of_convo(&c.id) == Some(*who))
                    .collect();
                let newest = mine.iter().max_by_key(|c| c.last_ms);
                let (preview, last_ms) = match (self.queued(who).last(), newest) {
                    (Some(q), _) => (format!("Not sent yet: {}", q.text), q.ts_ms),
                    (None, Some(c)) => (c.preview.clone().unwrap_or_default(), c.last_ms),
                    (None, None) => (String::new(), 0),
                };
                let row = json!({
                    "key": hex::encode(who),
                    "name": names.get(who),
                    "preview": preview.chars().take(PREVIEW_CHARS).collect::<String>(),
                    "last_ms": last_ms,
                    "queued": self.queued(who).count(),
                    "history_only": !mine.is_empty() && mine.iter().all(|c| c.history_only),
                    "has_address": self.latest(who).is_some(),
                });
                (last_ms, row)
            })
            .collect();
        rows.sort_by_key(|r| std::cmp::Reverse(r.0));
        rows.into_iter().map(|r| r.1).collect()
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
            history_only,
            kind: "direct".into(),
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
        assert_eq!(rows[0]["preview"], "Not sent yet: wait");
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
}
