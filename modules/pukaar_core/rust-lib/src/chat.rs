//! Private 1:1 chat (chat-module.md). `pukaar_core` calls `chat_module` 0.3.0 through the raw SDK,
//! the same way `storage.rs` calls Storage. The module needs `delivery_module` and a key service
//! (`devnet.chat-kc.logos.co`): `init` publishes the new installation to it, so `init` can take a
//! few seconds and fails when the key service is down.

use crate::storage::call;
use crate::{chats, now_ms};
use logos_rust_sdk::{LogosModuleSDK, PluginProxy};
use pukaar_logic::chatbook::{list_of, thread_messages, Convo, Msg};
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
const NO_ADDR: &str = "Their chat address has not arrived yet.";
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
    let address = address.as_str().filter(|a| !a.is_empty()).ok_or("chat_module has no address yet")?;
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
    list_of(call(p, "list_conversations", json!([]), CALL)?).map_err(|e| format!("list_conversations: {e}"))
}

fn messages_of(p: &PluginProxy, convo: &str) -> Result<Vec<Msg>, String> {
    list_of(call(p, "get_messages", json!([convo]), CALL)?).map_err(|e| format!("get_messages: {e}"))
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
    match STATUS.lock().unwrap_or_else(PoisonError::into_inner).as_ref() {
        Some(Ok(v)) => json!({ "ok": true, "status": v }),
        Some(Err(e)) => json!({ "ok": false, "error": e }),
        None => json!({ "ok": false, "starting": true }),
    }
}

/// Start the module, then announce its address to the site when it is new (a new one at each start; the
/// announce repeats from `send_queued`).
pub fn ensure() -> Result<Value, String> {
    let r = chats(|node, _, _| active(node, &node.me()))
        .and_then(|()| start())
        .and_then(|st| {
            let addr = st["address"].as_str().unwrap_or_default().to_owned();
            chats(|node, book, outbox| {
                outbox.extend(book.announce(&node.key, &node.store.site, &addr, now_ms()));
                Ok(())
            })?;
            Ok(st)
        });
    *STATUS.lock().unwrap_or_else(PoisonError::into_inner) = Some(r.clone());
    r
}

/// Only one send runs at a time, so a message is never sent twice (the background step and `send` both drain).
static DRAIN: Mutex<()> = Mutex::new(());

/// Send the unsent messages to `who`, in order. A conversation that was opened with their newest address is
/// used, else a new one is opened. What was not sent stays queued. Gives how many went out.
fn drain(who: &Key) -> Result<usize, String> {
    let _one = DRAIN.lock().unwrap_or_else(PoisonError::into_inner);
    let (addr, texts, known) = chats(|node, book, _| {
        active(node, who)?;
        let addr = book.latest(who).ok_or(NO_ADDR)?.to_owned();
        let known: Vec<String> = book.convos_for(who, &addr).map(str::to_owned).collect();
        Ok((addr, book.queued(who).map(|q| q.text.clone()).collect::<Vec<_>>(), known))
    })?;
    if texts.is_empty() {
        return Ok(0);
    }
    let p = proxy();
    let live = conversations(&p)?;
    let usable = known.into_iter().find(|id| live.iter().any(|c| c.id == *id && !c.history_only));
    let created = usable.is_none();
    let convo = match usable {
        Some(id) => id,
        None => call(&p, "create_conversation", json!([addr]), CALL)?.as_str().ok_or("chat_module gave no conversation id")?.to_owned(),
    };
    let (mut sent, mut failed) = (0, None);
    for text in &texts {
        match call(&p, "send_message", json!([convo, text]), CALL) {
            Ok(_) => sent += 1,
            Err(e) => {
                failed = Some(e);
                break;
            }
        }
    }
    chats(|_, book, _| {
        if created {
            book.link(&convo, *who, &addr);
        }
        book.unqueue(who, sent);
        if sent > 0 {
            book.mark_read(who, now_ms()); // our own message is not news to us
        }
        Ok(())
    })?;
    failed.map_or(Ok(sent), Err)
}

/// Keep `text` for `who` and send it when it can go. An error means the message was refused; a message that
/// only waits answers `sent: false` with the reason.
pub fn send(who: &Key, text: &str) -> Result<Value, String> {
    chats(|node, book, _| {
        active(node, who)?;
        if *who == node.me() {
            return Err("You cannot message yourself.".into());
        }
        book.enqueue(*who, text, now_ms())
    })?;
    Ok(match drain(who) {
        Ok(_) => json!({ "ok": true, "sent": true }),
        Err(why) => json!({ "ok": true, "sent": false, "why": why }),
    })
}

/// The chat list: `{"threads", "known", "chat"}` (see `ChatBook::threads_json`). `known` lists the members whose
/// chat address has arrived. A conversation that someone opened with us is linked to the member by the sender
/// of its first message.
// ponytail: the first message of each conversation without a link is read on every call (a chat of another app
// stays unlinked); keep a set of ids that were checked if that ever shows.
pub fn threads() -> Result<Value, String> {
    let p = proxy();
    let (convos, problem) = conversations(&p).map_or_else(|e| (vec![], Some(e)), |c| (c, None));
    let open: Vec<&str> = chats(|_, book, _| Ok(convos.iter().filter(|c| c.kind == "direct" && book.who_of_convo(&c.id).is_none()).map(|c| c.id.as_str()).collect()))?;
    let senders: Vec<(&str, String)> = open.into_iter().filter_map(|id| Some((id, messages_of(&p, id).ok()?.into_iter().find_map(|m| m.sender)?))).collect();
    chats(|node, book, _| {
        for (id, sender) in &senders {
            book.link_sender(id, sender);
        }
        let (members, me) = (node.approved_keys(), node.me());
        let threads = book.threads_json(&convos, &node.names, &members, &me);
        Ok(json!({
            "unread": threads.as_array().is_some_and(|rows| rows.iter().any(|t| t["unread"] == true)),
            "threads": threads,
            "known": members.iter().filter(|m| book.latest(m).is_some()).map(hex::encode).collect::<Vec<_>>(),
            "chat": status_json(),
            "problem": problem,
        }))
    })
}

/// The thread with `who` is read: its dot goes out. Reads the chat module for the time of its newest message.
pub fn mark_read(who: &Key) -> Result<Value, String> {
    let convos = conversations(&proxy())?;
    chats(|_, book, _| {
        book.mark_thread_read(who, &convos);
        Ok(json!({ "ok": true }))
    })
}

/// The thread with `who`: every conversation with them, old ones first, then what is not sent yet.
pub fn messages(who: &Key) -> Result<Value, String> {
    let p = proxy();
    let convos = conversations(&p).unwrap_or_default();
    let mine: Vec<(String, bool)> = chats(|_, book, _| Ok(convos.iter().filter(|c| book.who_of_convo(&c.id) == Some(*who)).map(|c| (c.id.clone(), c.history_only)).collect()))?;
    let parts = mine.into_iter().map(|(id, old)| Ok((old, messages_of(&p, &id)?))).collect::<Result<Vec<_>, String>>()?;
    chats(|_, book, _| Ok(json!({ "messages": thread_messages(parts, &book.queued(who).collect::<Vec<_>>()) })))
}

/// The background step: start chat and announce, then send what waits. Called from `flush`, at most once per
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
        }
        BUSY.store(false, Ordering::Release);
    });
}
