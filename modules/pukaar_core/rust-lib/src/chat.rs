//! Private chat, 1:1 and in groups (chat-module.md). `pukaar_core` calls `chat_module` 0.3.0 through the raw SDK,
//! the same way `storage.rs` calls Storage. The module needs `delivery_module` and a key service
//! (`devnet.chat-kc.logos.co`): `init` publishes the new installation to it, so `init` can take a
//! few seconds and fails when the key service is down.

use crate::storage::call;
use crate::{chats, now_ms};
use logos_rust_sdk::{LogosModuleSDK, PluginProxy};
use pukaar_logic::chatbook::{list_of, thread_messages, Convo, Member, Msg, AFTER, FALLBACK_MS, MAX_GROUP_NAME};
use pukaar_logic::event::Key;
use pukaar_logic::node::Node;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

const MODULE: &str = "chat_module";
/// Pukaar's Delivery preset. The first module that creates the shared node decides its settings
/// (chat defaults to `logos.test`), so chat must ask for the same one.
const PRESET: &str = "logos.dev";
/// `init` does one HTTPS round trip to the key service.
const CALL: Duration = Duration::from_secs(20);
/// The background step (start, announce, send what waits) runs at most this often.
const EVERY: Duration = Duration::from_secs(5);
/// A new conversation is not written into at once: the other module joins it a moment after the invite, and a
/// message sent before that is lost (nobody listens to its topic yet). `send_message` only hands to Delivery.
const OPENING: &str = "Opening the chat with them.";
const NO_ADDR: &str = "Their chat address has not arrived yet.";
/// A group takes a message once the members that were added have joined (see `ChatBook::group_seen`).
const OPENING_GROUP: &str = "Waiting for the other members to join the group.";
const OLD_GROUP: &str = "Rejoining: you restarted, so this group moves to a new chat. Your message waits and goes to the new chat.";
const MOVING: &str = "A member restarted, so this group moves to a new chat. Your message waits and goes to the new chat.";
const NOBODY: &str = "Nobody else of this group is known. Make a new group instead.";
const NOT_GROUP: &str = "That is not a group of this site.";
/// Shown when chat, not Pukaar, created the shared Delivery node (see `delivery_adopted`).
const OTHER_MODE: &str = "Chat started the shared Delivery node first, so it runs in the chat's setup and not in Pukaar's. Messages should work. If the site stops syncing, restart Basecamp.";

fn proxy() -> PluginProxy {
    LogosModuleSDK::new().plugin(MODULE)
}

/// Start the chat module and read its address. `init` answers at once while the Delivery node
/// comes up (`delivery` is `online` later), and a second `init` while running is a no-op.
fn start() -> Result<Value, String> {
    let p = proxy();
    call(&p, "init", json!([{ "delivery_preset": PRESET }]), CALL)?;
    let address = call(&p, "get_address", json!([]), CALL)?;
    let address = address
        .as_str()
        .filter(|a| !a.is_empty())
        .ok_or("chat_module has no address yet")?;
    let status = call(&p, "status", json!([]), CALL)?;
    let adopted = status["delivery_adopted"].as_bool().unwrap_or(false);
    Ok(json!({
        "address": address,
        "delivery": status["delivery_state"],
        "detail": status["detail"],
        "adopted": adopted,
        "preset": PRESET,
        "note": (!adopted).then_some(OTHER_MODE),
    }))
}

fn conversations(p: &PluginProxy) -> Result<Vec<Convo>, String> {
    list_of(call(p, "list_conversations", json!([]), CALL)?)
        .map_err(|e| format!("list_conversations: {e}"))
}

fn messages_of(p: &PluginProxy, convo: &str) -> Result<Vec<Msg>, String> {
    list_of(call(p, "get_messages", json!([convo]), CALL)?)
        .map_err(|e| format!("get_messages: {e}"))
}

fn roster(p: &PluginProxy, convo: &str) -> Result<Vec<Member>, String> {
    list_of(call(p, "list_group_members", json!([convo]), CALL)?)
        .map_err(|e| format!("list_group_members: {e}"))
}

/// A thread of the chat list is a person (its key) or a group (its conversation id).
enum Chat {
    Person(Key),
    Group(String),
}

fn chat_of(id: &str) -> Result<Chat, String> {
    if chats(|_, book, _| Ok(book.is_group(id)))? {
        Ok(Chat::Group(id.to_owned()))
    } else {
        crate::key_arg(id).map(Chat::Person)
    }
}

/// Chat is for active members of the site, both ends.
fn active(node: &Node, who: &Key) -> Result<(), String> {
    let members = node.approved_keys();
    if members.contains(&node.me()) && members.contains(who) {
        Ok(())
    } else {
        Err("Chat is for active members of this site.".into())
    }
}

/// What the last `ensure` found: the status, or why chat is not running.
static STATUS: Mutex<Option<Result<Value, String>>> = Mutex::new(None);

fn status_json() -> Value {
    match STATUS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
    {
        Some(Ok(v)) => json!({ "ok": true, "status": v }),
        Some(Err(e)) => json!({ "ok": false, "error": e }),
        None => json!({ "ok": false, "starting": true }),
    }
}

/// Start the module, then announce its address to the site when it is new (a new one at each start; the
/// announce repeats from `send_queued`). With a new address we also ask the others for theirs: they may have
/// restarted while we were away, and Delivery does not keep their announces for us.
pub fn ensure() -> Result<Value, String> {
    let r = chats(|node, _, _| active(node, &node.me()))
        .and_then(|()| start())
        .and_then(|st| {
            let addr = st["address"].as_str().unwrap_or_default().to_owned();
            chats(|node, book, outbox| {
                let (site, now) = (node.store.site, now_ms());
                if let Some(a) = book.announce(&node.key, &site, &addr, now) {
                    outbox.push(a);
                    outbox.extend(book.ask(&node.key, &site, now));
                }
                Ok(())
            })?;
            Ok(st)
        });
    *STATUS.lock().unwrap_or_else(PoisonError::into_inner) = Some(r.clone());
    r
}

/// Hand `texts` to the module for `convo`, in order. Gives how many went out, and why the rest did not.
fn send_all(p: &PluginProxy, convo: &str, texts: &[String]) -> (usize, Option<String>) {
    for (sent, text) in texts.iter().enumerate() {
        if let Err(e) = call(p, "send_message", json!([convo, text]), CALL) {
            return (sent, Some(e));
        }
    }
    (texts.len(), None)
}

/// Only one send runs at a time, so a message is never sent twice (the background step and `send` both drain).
static DRAIN: Mutex<()> = Mutex::new(());

/// Send the unsent messages to `who`, in order. A conversation that was opened with their newest address is
/// used, else a new one is opened and given time to be joined. What was not sent stays queued. Gives how many went out.
fn drain(who: &Key) -> Result<usize, String> {
    let _one = DRAIN.lock().unwrap_or_else(PoisonError::into_inner);
    let (addr, texts, known) = chats(|node, book, _| {
        active(node, who)?;
        let addr = book.latest(who).ok_or(NO_ADDR)?.to_owned();
        let known: Vec<String> = book.convos_for(who, &addr).map(str::to_owned).collect();
        Ok((
            addr,
            book.queued(who).map(|q| q.text.clone()).collect::<Vec<_>>(),
            known,
        ))
    })?;
    if texts.is_empty() {
        return Ok(0);
    }
    let p = proxy();
    let live = conversations(&p)?;
    let usable = known
        .into_iter()
        .find(|id| live.iter().any(|c| c.id == *id && !c.history_only));
    let convo = match usable {
        Some(id) => id,
        None => {
            let id = call(&p, "create_conversation", json!([addr]), CALL)?
                .as_str()
                .ok_or("chat_module gave no conversation id")?
                .to_owned();
            chats(|_, book, _| {
                book.link(&id, *who, &addr);
                book.opened(&id, now_ms());
                Ok(())
            })?;
            id
        }
    };
    if !chats(|_, book, _| Ok(book.settled(&convo, now_ms())))? {
        return Err(OPENING.into()); // stays queued; the next round sends it
    }
    let (sent, failed) = send_all(&p, &convo, &texts);
    chats(|_, book, _| {
        book.unqueue(who, sent);
        if sent > 0 {
            book.mark_read(who, now_ms()); // our own message is not news to us
        }
        Ok(())
    })?;
    failed.map_or(Ok(sent), Err)
}

/// What a send answers: `sent` is false, with the reason, for a message that only waits.
fn reply(sent: Result<usize, String>) -> Value {
    match sent {
        Ok(_) => json!({ "ok": true, "sent": true }),
        Err(why) => json!({ "ok": true, "sent": false, "why": why }),
    }
}

/// Keep `text` for a person or a group and send it when it can go. An error means the message was refused; a message
/// that only waits answers `sent: false` with the reason.
pub fn send(id: &str, text: &str) -> Result<Value, String> {
    match chat_of(id)? {
        Chat::Person(who) => {
            chats(|node, book, _| {
                active(node, &who)?;
                if who == node.me() {
                    return Err("You cannot message yourself.".into());
                }
                book.enqueue(who, text, now_ms())
            })?;
            Ok(reply(drain(&who)))
        }
        Chat::Group(convo) => {
            chats(|node, book, _| {
                active(node, &node.me())?;
                book.enqueue_group(&convo, text, now_ms())
            })?;
            let convo = chats(|_, book, _| Ok(book.newest_group(&convo)))?;
            Ok(reply(drain_group(&convo)))
        }
    }
}

/// The chat list: `{"threads", "known", "chat"}` (see `ChatBook::threads_json`; the groups of the site are in it). `known` lists the members whose
/// chat address has arrived. A conversation that someone opened with us is linked to the member by the sender
/// of its first message.
// ponytail: the first message of each conversation without a link is read on every call (a chat of another app
// stays unlinked); keep a set of ids that were checked if that ever shows.
pub fn threads() -> Result<Value, String> {
    let p = proxy();
    let (convos, problem) = conversations(&p).map_or_else(|e| (vec![], Some(e)), |c| (c, None));
    let open: Vec<&str> = chats(|_, book, _| {
        Ok(convos
            .iter()
            .filter(|c| c.kind == "direct" && book.who_of_convo(&c.id).is_none())
            .map(|c| c.id.as_str())
            .collect())
    })?;
    // ponytail: the roster of each live group is read on every call (a group of another app is never ours); keep a set
    // of the ones that were checked if that ever shows.
    let rosters: Vec<(&str, Vec<Member>)> = convos
        .iter()
        .filter(|c| c.kind == "group" && !c.history_only)
        .filter_map(|c| Some((c.id.as_str(), roster(&p, &c.id).ok()?)))
        .collect();
    let senders: Vec<(&str, String)> = open
        .into_iter()
        .filter_map(|id| {
            Some((
                id,
                messages_of(&p, id)
                    .ok()?
                    .into_iter()
                    .find_map(|m| m.sender)?,
            ))
        })
        .collect();
    chats(|node, book, _| {
        for (id, sender) in &senders {
            book.link_sender(id, sender);
        }
        book.follow(&convos);
        let (members, me) = (node.approved_keys(), node.me());
        for (id, members_in) in &rosters {
            book.group_seen(id, members_in, &members, &me, now_ms());
        }
        let threads = book.threads_json(&convos, &node.names, &members, &me);
        Ok(json!({
            "unread": threads.as_array().is_some_and(|rows| rows.iter().any(|t| t["unread"] == true)),
            "threads": threads,
            "known": members.iter().filter(|m| book.latest(m).is_some()).map(hex::encode).collect::<Vec<_>>(),
            "chat": status_json(),
            "problem": problem,
            "asking": book.asking(now_ms()),
        }))
    })
}

/// The thread (a person or a group) is read: its dot goes out. Reads the chat module for the time of its newest message.
pub fn mark_read(id: &str) -> Result<Value, String> {
    let chat = chat_of(id)?;
    let convos = conversations(&proxy())?;
    chats(|_, book, _| {
        match &chat {
            Chat::Person(who) => book.mark_thread_read(who, &convos),
            Chat::Group(convo) => book.mark_group_read(convo, &convos),
        }
        Ok(json!({ "ok": true }))
    })
}

/// The thread with a person: every conversation with them, old ones first, then what is not sent yet. Or the messages
/// of a group, each with its sender, after those of the groups that it started again. An open chat also asks the
/// others for their chat address (at most once a minute), with ours: the one who opens a chat refreshes the addresses.
pub fn messages(id: &str) -> Result<Value, String> {
    chats(|node, book, outbox| {
        let site = node.store.site;
        if let Some(ask) = book.ask(&node.key, &site, now_ms()) {
            outbox.extend(book.resend(&node.key, &site));
            outbox.push(ask);
        }
        Ok(())
    })?;
    let p = proxy();
    let convos = conversations(&p).unwrap_or_default();
    let rows = match chat_of(id)? {
        Chat::Person(who) => {
            let mine: Vec<(String, bool)> = chats(|_, book, _| {
                Ok(convos
                    .iter()
                    .filter(|c| book.who_of_convo(&c.id) == Some(who))
                    .map(|c| (c.id.clone(), c.history_only))
                    .collect())
            })?;
            let parts = mine
                .into_iter()
                .map(|(id, old)| Ok((old, messages_of(&p, &id)?)))
                .collect::<Result<Vec<_>, String>>()?;
            chats(|_, book, _| {
                Ok(thread_messages(
                    parts,
                    &book.queued(&who).collect::<Vec<_>>(),
                ))
            })?
        }
        Chat::Group(convo) => {
            let old = convos.iter().any(|c| c.id == convo && c.history_only);
            let before = chats(|_, book, _| Ok(book.before(&convo)))?;
            let earlier: Vec<Vec<Msg>> = before.iter().map(|b| messages_of(&p, b).unwrap_or_default()).collect();
            let msgs = messages_of(&p, &convo)?;
            chats(|node, book, _| {
                let mut rows: Vec<Value> = earlier
                    .into_iter()
                    .flat_map(|m| book.group_thread(m, &[], &node.names, true))
                    .collect();
                rows.extend(book.group_thread(
                    msgs,
                    &book.queued_group(&convo).collect::<Vec<_>>(),
                    &node.names,
                    old,
                ));
                Ok(rows)
            })?
        }
    };
    Ok(json!({ "messages": rows }))
}

/// The members to put in a group: active members other than us, once each. At least one.
fn pickable(node: &Node, who: &[Key]) -> Result<Vec<Key>, String> {
    active(node, &node.me())?;
    let mut out: Vec<Key> = Vec::with_capacity(who.len());
    for w in who.iter().filter(|w| **w != node.me()) {
        active(node, w)?;
        if !out.contains(w) {
            out.push(*w);
        }
    }
    if out.is_empty() {
        return Err("Pick at least one other member.".into());
    }
    Ok(out)
}

/// Make a group named `name`. The members are added as soon as their chat address is known: the ones whose address
/// has not arrived stay on the list and are added by the background step. `{"ok":true,"convo"}`; the group is in the
/// chat list at once.
pub fn group_create(name: &str, who: &[Key]) -> Result<Value, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_GROUP_NAME {
        return Err(format!(
            "A group name needs 1 to {MAX_GROUP_NAME} characters."
        ));
    }
    let who = chats(|node, _, _| pickable(node, who))?;
    let convo = call(
        &proxy(),
        "create_group_conversation",
        json!([name, ""]),
        CALL,
    )?
    .as_str()
    .ok_or("chat_module gave no conversation id")?
    .to_owned();
    chats(|_, book, _| {
        book.start_group(&convo, &who);
        Ok(())
    })?;
    let _ = invite_due(); // what cannot go now stays on the list
    Ok(json!({ "ok": true, "convo": convo }))
}

/// Add members to a group of the site. Any member of the group can; the ones that are in already are skipped.
pub fn group_add(convo: &str, who: &[Key]) -> Result<Value, String> {
    let who = chats(|node, book, _| {
        if !book.is_group(convo) {
            return Err(NOT_GROUP.into());
        }
        pickable(node, who)
    })?;
    let have = roster(&proxy(), convo)?;
    chats(|_, book, _| {
        let fresh: Vec<Key> = who
            .into_iter()
            .filter(|w| !have.iter().any(|m| book.who_of(&m.address) == Some(*w)))
            .collect();
        for w in fresh {
            book.want(convo, w);
        }
        Ok(())
    })?;
    let _ = invite_due();
    Ok(json!({ "ok": true }))
}

/// A group for its header: `{"name","history_only","members":[{"key","name","me","pending"}]}`. A member that waits for
/// their chat address is in the list as pending.
pub fn group_info(convo: &str) -> Result<Value, String> {
    let p = proxy();
    let c = conversations(&p)?.into_iter().find(|c| c.id == convo);
    let members = roster(&p, convo).unwrap_or_default();
    chats(|node, book, _| {
        if !book.is_group(convo) {
            return Err(NOT_GROUP.into());
        }
        // a group that is read-only for us has no roster in the module: show the last one that we saw
        let members = if members.is_empty() { book.last_roster(convo) } else { members };
        let (me, now) = (node.me(), now_ms());
        let mover = book.moving(convo, &members, &node.approved_keys(), &me, now);
        Ok(json!({
            "ok": true,
            "name": c.as_ref().and_then(|c| c.name.as_deref()),
            "history_only": c.as_ref().is_some_and(|c| c.history_only),
            "members": book.roster_json(convo, &members, &node.names),
            "mover": mover.map(|(w, _)| hex::encode(w)),
            "mover_name": mover.and_then(|(w, _)| node.names.get(&w)),
            "mover_me": mover.is_some_and(|(w, _)| w == me),
            "fallback": mover.is_some_and(|(_, at)| now >= at + FALLBACK_MS),
            "next": book.next_of(convo),
        }))
    })
}

/// Start a group again by hand: the fallback when the member who moves it (`ChatBook::moving`) has not done it in
/// time, or when we never saw its roster. A new group that exists already is opened, not made twice. `{"ok":true,"convo"}`.
pub fn group_again(convo: &str) -> Result<Value, String> {
    let p = proxy();
    let live = conversations(&p)?;
    let c = live
        .iter()
        .find(|c| c.id == convo && c.kind == "group")
        .ok_or(NOT_GROUP)?;
    let done = chats(|node, book, _| {
        if !book.is_group(convo) {
            return Err(NOT_GROUP.into());
        }
        active(node, &node.me())?;
        book.follow(&live);
        if book.next_of(convo).is_some() {
            return Ok(Some(book.newest_group(convo)));
        }
        if !book.begin_move(convo, now_ms()) {
            return Err("The group is moving to a new chat now. Wait a moment.".into());
        }
        Ok(None)
    })?;
    let next = match done {
        Some(next) => next,
        None => move_group(&p, c)?,
    };
    Ok(json!({ "ok": true, "convo": next }))
}

/// Move the group `c` to a new group with the same name and the same active members, and its description naming the
/// old group. A member whose address is not known waits on the invite list of the new group. The old conversation
/// stays in the module as history: the thread shows it above the new one. Gives the new group's id.
fn move_group(p: &PluginProxy, c: &Convo) -> Result<String, String> {
    let senders: Vec<String> = messages_of(p, &c.id)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|m| m.sender)
        .collect();
    let who = chats(|node, book, _| Ok(book.old_members(&c.id, &senders, &node.approved_keys(), &node.me())))?;
    if who.is_empty() {
        return Err(NOBODY.into());
    }
    let name = c.name.as_deref().unwrap_or("Group");
    let next = call(p, "create_group_conversation", json!([name, format!("{AFTER}{}", c.id)]), CALL)?
        .as_str()
        .ok_or("chat_module gave no conversation id")?
        .to_owned();
    chats(|_, book, _| {
        book.start_group(&next, &who);
        book.succeed(&c.id, &next);
        Ok(())
    })?;
    let _ = invite_due(); // what cannot go now stays on the list
    Ok(next)
}

/// Only one round of invites runs at a time, so a member is never added twice.
static INVITE: Mutex<()> = Mutex::new(());

/// Add the members that wait for a group, those whose address is known. A failed call leaves the invite for the next round.
fn invite_due() -> Result<usize, String> {
    let _one = INVITE.lock().unwrap_or_else(PoisonError::into_inner);
    let p = proxy();
    let live = conversations(&p)?;
    let due = chats(|node, book, _| Ok(book.due_invites(&node.approved_keys(), &live)))?;
    let (mut added, mut failed) = (0, None);
    for (convo, who, addr) in due {
        match call(&p, "add_group_member", json!([convo, addr]), CALL) {
            Ok(_) => {
                chats(|_, book, _| {
                    book.invited(&convo, &who);
                    Ok(())
                })?;
                added += 1;
            }
            Err(e) => failed = Some(e),
        }
    }
    failed.map_or(Ok(added), Err)
}

/// Recovery after a restart (chat-module.md, "Recovery scenarios"), once for each round: link the groups that a
/// member moved, try the cheap re-add (`ChatBook::readd`), and move a group to a new one when we are its mover and its
/// time has come (`ChatBook::moving`). The roster of each live group of ours is read once.
fn recover() {
    let p = proxy();
    let Ok(live) = conversations(&p) else { return };
    let _ = chats(|_, book, _| {
        book.follow(&live);
        Ok(())
    });
    for c in live.iter().filter(|c| c.kind == "group") {
        if !chats(|_, book, _| Ok(book.is_group(&c.id))).unwrap_or(false) {
            continue;
        }
        let members_in = if c.history_only {
            None
        } else {
            let Ok(r) = roster(&p, &c.id) else { continue };
            Some(r)
        };
        let now = now_ms();
        let go = chats(|node, book, _| {
            let (members, me) = (node.approved_keys(), node.me());
            let r = match &members_in {
                Some(r) => {
                    book.readd(&c.id, r, &members, &me);
                    r.clone()
                }
                None => book.last_roster(&c.id),
            };
            let due = book.moving(&c.id, &r, &members, &me, now).is_some_and(|(by, at)| by == me && now >= at);
            Ok(due && book.begin_move(&c.id, now))
        })
        .unwrap_or(false);
        if go {
            let _ = move_group(&p, c); // a failure is tried again after `MOVE_RETRY_MS`
        }
    }
}

/// Send the unsent messages to the group `convo`, in order, once the members that were added have joined. What was not
/// sent stays queued. Gives how many went out.
fn drain_group(convo: &str) -> Result<usize, String> {
    let _one = DRAIN.lock().unwrap_or_else(PoisonError::into_inner);
    let texts: Vec<String> =
        chats(|_, book, _| Ok(book.queued_group(convo).map(|q| q.text.clone()).collect()))?;
    if texts.is_empty() {
        return Ok(0);
    }
    let p = proxy();
    if !conversations(&p)?
        .iter()
        .any(|c| c.id == convo && !c.history_only)
    {
        return Err(OLD_GROUP.into());
    }
    let members = roster(&p, convo)?;
    // while the group waits to move, a message would not reach the member who restarted: it waits for the new group
    if chats(|node, book, _| Ok(book.moving(convo, &members, &node.approved_keys(), &node.me(), now_ms()).is_some()))? {
        return Err(MOVING.into());
    }
    if !chats(|node, book, _| {
        Ok(book.group_seen(convo, &members, &node.approved_keys(), &node.me(), now_ms()))
    })? {
        return Err(OPENING_GROUP.into());
    }
    let (sent, failed) = send_all(&p, convo, &texts);
    chats(|_, book, _| {
        book.unqueue_group(convo, sent);
        if sent > 0 {
            book.mark_group_read_at(convo, now_ms()); // our own message is not news to us
        }
        Ok(())
    })?;
    failed.map_or(Ok(sent), Err)
}

/// The background step: start chat and announce, add the members that wait for a group, then send what waits. Called from `flush`, at most once per
/// `EVERY`, on a thread of its own (a module call can take seconds and a call from the UI never waits for it).
pub fn tick() {
    static NEXT: Mutex<Option<Instant>> = Mutex::new(None);
    static BUSY: AtomicBool = AtomicBool::new(false);
    {
        let mut next = NEXT.lock().unwrap_or_else(PoisonError::into_inner);
        if next.is_some_and(|t| Instant::now() < t) || BUSY.swap(true, Ordering::AcqRel) {
            return;
        }
        *next = Some(Instant::now() + EVERY);
    }
    std::thread::spawn(|| {
        if ensure().is_ok() {
            for who in chats(|_, book, _| Ok(book.queue_targets())).unwrap_or_default() {
                let _ = drain(&who); // a failure leaves the message queued for the next round
            }
            recover();
            let _ = invite_due();
            for convo in chats(|_, book, _| Ok(book.group_targets())).unwrap_or_default() {
                let _ = drain_group(&convo);
            }
        }
        BUSY.store(false, Ordering::Release);
    });
}
