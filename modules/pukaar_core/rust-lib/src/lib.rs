//! Logos glue for Pukaar. The trait below IS the module's API (the builder
//! derives the .lidl from it). All rules live in ./logic (pukaar_logic, a
//! standalone plain-`cargo test`-able crate nested under this one so
//! logos-module-builder's `cp -r <codegen.rust.crate>` staging sees it — see
//! rust-lib/Cargo.toml); this file only moves bytes between that crate, the
//! disk and Delivery.

use pukaar_logic::event::{Body, Event, Role, SigningKey};
use pukaar_logic::identity;
use pukaar_logic::node::{action_body, genesis_from_json, parse_id, Node};
use pukaar_logic::persist;
use pukaar_logic::store::Accept;
use pukaar_logic::sync::{
    heads_msg, lost_node, send_queue, should_answer, to_resend, to_resend_own, SendError, Wire,
    DELIVERY_CFG,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
// Mutex comes from the generated scaffold below (same module scope): a second
// `use std::sync::Mutex;` here would be a duplicate import (confirmed at Step 8).
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub trait PukaarCoreModule: Send + 'static {
    fn site_create(&mut self, genesis_json: String) -> String;
    fn site_join(&mut self, site_hex: String) -> String;
    /// Drop this device's copy of the site (no event is published); the key stays, so
    /// rejoining is the same person. `ok` or `error: …`.
    fn leave_site(&mut self) -> String;
    fn my_identity(&mut self) -> String;
    /// A password-sealed backup of this identity: one `pukaar-id-1:…` line, or `error: …`.
    fn export_identity(&mut self, password: String) -> String;
    /// Restore a backup on a fresh device (no site yet): `ok` or `error: …`. Publishing is
    /// refused until our own history is back (`my_identity().syncing_own_history`).
    fn import_identity(&mut self, blob: String, password: String) -> String;
    /// Stop waiting for a restored identity's history (allowed 10 min after the restore).
    fn skip_history_sync(&mut self) -> String;
    fn set_profile(&mut self, display_name: String) -> String;
    /// `name` is required for steward/admin and optional for residents. Only a super admin
    /// grants "admin" or changes an admin's role; nobody changes a super admin's.
    fn grant_role(&mut self, subject_hex: String, role: String, name: String) -> String;
    /// Admin only (the rules reject anyone else's); `reason` is required. Revoking an admin is
    /// a super admin's call; a super admin can't be revoked.
    fn revoke_role(&mut self, subject_hex: String, reason: String) -> String;
    /// Super admin only: a second super admin (a named member; at most two), made an admin too.
    fn add_super_admin(&mut self, subject_hex: String) -> String;
    /// Super admin only, on the other super admin, who stays an admin.
    fn remove_super_admin(&mut self, subject_hex: String) -> String;
    /// Super admin only: hand our place to `to_hex` (made an admin too); we stay an admin.
    fn transfer_super_admin(&mut self, to_hex: String) -> String;
    /// Super admin only, on an admin who isn't one: they keep admin powers until `deadline`
    /// (unix seconds, now or later), then become a steward (`outcome` "steward") or leave
    /// ("remove"). `reason` is required. Replaces their earlier notice.
    fn admin_notice(
        &mut self,
        subject_hex: String,
        outcome: String,
        deadline: i64,
        reason: String,
    ) -> String;
    /// Super admin only, before the notice is sealed.
    fn cancel_notice(&mut self, subject_hex: String) -> String;
    /// Super admin only: the notice ends now and is sealed at once.
    fn end_notice_now(&mut self, subject_hex: String) -> String;
    fn add_location(&mut self, code: String, label: String, group: String) -> String;
    /// Admin only. Never a delete: refused with `error: W-01 has 2 open issues` while any
    /// issue there is still open. Retired locations take no new reports. `reason` is required.
    fn retire_location(&mut self, code: String, reason: String) -> String;
    /// Admin only; makes a retired location reportable again. `reason` is required.
    fn restore_location(&mut self, code: String, reason: String) -> String;
    /// Admin only. New name and group; the code never changes. Both are required.
    fn edit_location(&mut self, code: String, label: String, group: String) -> String;
    /// Admin only, and only for a location no report ever named (`error: W-01 has had reports:
    /// retire it instead`). It stays pending removal for 30 days, then is hidden for good
    /// (the signed events stay). `reason` is required.
    fn remove_location(&mut self, code: String, reason: String) -> String;
    /// Admin only; brings back a location pending removal, within the 30 days. `reason` is required.
    fn undo_remove_location(&mut self, code: String, reason: String) -> String;
    /// `location` is a site-list code, or "other" with a `landmark` note.
    fn report(
        &mut self,
        category: String,
        location: String,
        landmark: String,
        text: String,
    ) -> String;
    /// action = acknowledge|update|claim_resolved|confirm|reopen|close_wontfix|mark_duplicate|comment
    fn act(
        &mut self,
        issue_hex: String,
        action: String,
        note: String,
        next_step: String,
        eta_h: i64,
    ) -> String;
    fn list_issues(&mut self) -> String;
    fn issue_timeline(&mut self, issue_hex: String) -> String;
    fn site_info(&mut self) -> String;
    fn checkpoint_now(&mut self) -> String;
    fn record_anchor(&mut self, heads_json: String, lez_ref: String) -> String;
    fn on_context_ready(&mut self, _ctx: &RustModuleContext) {}
}

// The builder writes the scaffold here: install, RustModuleContext, modules(), emitters.
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/provider_gen.rs"
));

const HEADS_EVERY: Duration = Duration::from_secs(60);
const ANSWER_EVERY: Duration = Duration::from_secs(10);
/// Received events reach the disk at most this often (see `Shared::save_due`).
const SAVE_EVERY: Duration = Duration::from_secs(2);
/// Bound on each Delivery call. A healthy one answers in 10-50 ms, but the protocol default is
/// 20 s, which is ALSO the UI's whole budget for a call to us: this module runs one call at a
/// time, so while Delivery was unreachable every poll's flush() blocked 20 s and any call queued
/// behind it (an import, a report) timed out. Seen in Basecamp: "no listener at
/// local:logos_delivery_module_…" every 20 s, then `import_identity timed out after 20000ms`.
const DELIVERY_TIMEOUT: Duration = Duration::from_secs(5);
/// After a Delivery failure, flush() leaves Delivery alone this long, so a dead Delivery costs
/// one DELIVERY_TIMEOUT stall per window instead of one per call.
const RETRY_AFTER: Duration = Duration::from_secs(10);

/// Delivery bring-up, advanced one step per flush(). Not done in on_context_ready: the host
/// authorises our calls to delivery_module only after that returns, so a createNode/start made
/// there fails "unauthorized", the node never starts, and nothing ever crosses (seen headless).
/// Only moves forward, so a nodeStarted that lands mid-flush is never overwritten; the one way back
/// is to Down, when Delivery's host restarted without its node (see `advance`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Link {
    #[default]
    Down,
    /// Node created and our two subscriptions live: never redone, so a failed start retries
    /// only start (redoing this step leaked a listener thread per 2 s poll).
    Listening,
    /// start() dispatched; nodeStarted moves it on.
    Starting,
    Started,
    /// Channel open on our site: the outbox goes out.
    Open,
}

struct Shared {
    dir: PathBuf,
    node: Option<Node>,
    /// Wire messages waiting to go out. Only drained inside module method calls,
    /// so Delivery is never called from our own threads. What Delivery couldn't take stays here.
    outbox: Vec<Vec<u8>>,
    last_heads: Option<Instant>,
    last_answer: Option<Instant>,
    link: Link,
    /// Last Delivery failure, shown as `"delivery": "error: …"` until a later step succeeds.
    link_error: Option<String>,
    /// Delivery took a message but then couldn't get it onto the network (channelMessageError),
    /// until one gets through (channelMessageSent). Our calls all succeed meanwhile, so without
    /// this the UI said "online" while nothing left the device (seen live: every send failed
    /// "The node does not have a usable RLN membership").
    send_error: Option<String>,
    /// Set by a failed step: no Delivery call before then (see RETRY_AFTER).
    retry_at: Option<Instant>,
    /// Events arrived (on_wire) since the last save.
    dirty: bool,
    last_save: Instant,
}

impl Shared {
    /// Write the replica now. A failed write leaves `dirty` set, so a later poll retries it.
    fn persist(&mut self) {
        self.dirty = !self.node.as_ref().is_none_or(|n| save(n, &self.dir));
        self.last_save = Instant::now();
    }

    /// Save what on_wire marked dirty, at most every SAVE_EVERY (or `force`d, at unload).
    // ponytail: one whole-file rewrite per 2 s window instead of per received event, so a
    // crash loses at most ~2 s of RECEIVED events, which peers re-send (anti-entropy); our own
    // events are saved the moment they are signed (`write`). Append-only log if this ever matters.
    fn save_due(&mut self, force: bool) {
        if self.dirty && (force || self.last_save.elapsed() >= SAVE_EVERY) {
            self.persist();
        }
    }

    /// Seal every notice whose deadline has passed (`Node::auto_seal` signs each one once),
    /// save, and queue the seals to go out.
    fn auto_seal(&mut self) {
        let Some(node) = self.node.as_mut() else {
            return;
        };
        let seals = node.auto_seal(now());
        if !seals.is_empty() {
            self.outbox
                .extend(seals.into_iter().map(|e| Wire::Event(e.bytes).encode()));
            self.persist();
        }
    }
}

static SHARED: Mutex<Option<Shared>> = Mutex::new(None);

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn err(e: impl std::fmt::Display) -> String {
    format!("error: {e}")
}

const NO_SITE: &str = "no site yet: create or join one";

/// Save the replica; a failure is logged, never fatal (the events are still in memory).
/// Returns whether it reached the disk.
fn save(node: &Node, dir: &Path) -> bool {
    let r = persist::save(node, dir);
    if let Err(e) = &r {
        eprintln!("pukaar: save failed: {e}");
    }
    r.is_ok()
}

/// Delivery methods answer `{"success", "value", "error"}`, so a refusal ("context not
/// initialized") arrives as `Ok`: fold it into the error it is.
fn delivered(
    method: &str,
    r: Result<serde_json::Value, logos_rust_sdk::LogosError>,
) -> Result<(), String> {
    let v = r.map_err(|e| format!("{method}: {e}"))?;
    match v.get("success").unwrap_or(&v) {
        serde_json::Value::Bool(false) => Err(format!(
            "{method}: {}",
            v.get("error")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("refused")
        )),
        _ => Ok(()),
    }
}

/// `delivered` for a step that is idempotent: Delivery is a shared, long-lived module, so another
/// module, an earlier load of ours, or a leave_site whose channelClose failed may already have done
/// it, and Delivery then refuses with "already initialized" / "already exists". The state we wanted
/// holds, so that refusal is success.
fn delivered_once(
    method: &str,
    r: Result<serde_json::Value, logos_rust_sdk::LogosError>,
) -> Result<(), String> {
    delivered(method, r).or_else(|e| e.contains("already ").then_some(()).ok_or(e))
}

/// Record the outcome of a bring-up step. An error keeps the link where it is, so the first flush
/// after RETRY_AFTER retries that step, unless Delivery lost its node: its host restarted, and
/// only a new createNode, start and channelCreate bring it back (before, every later send was
/// refused "Context not initialized" and nothing went out until Basecamp restarted).
fn advance(r: Result<Link, String>) {
    if let Some(sh) = SHARED.lock().unwrap().as_mut() {
        match r {
            Ok(l) => {
                sh.link = sh.link.max(l);
                sh.link_error = None;
                sh.retry_at = None;
            }
            Err(e) => {
                if lost_node(&e) {
                    sh.link = Link::Down;
                }
                sh.link_error = Some(e);
                sh.retry_at = Some(Instant::now() + RETRY_AFTER);
            }
        }
    }
}

fn delivery_status() -> String {
    match SHARED.lock().unwrap().as_ref() {
        None => "module not ready".into(),
        Some(Shared {
            link_error: Some(e),
            ..
        })
        | Some(Shared {
            send_error: Some(e),
            ..
        }) => err(e),
        Some(sh) => format!("{:?}", sh.link),
    }
}

/// Run `handle` on every `decode`d event from `sub`, on a thread of its own.
fn listen<T: 'static>(
    sub: Result<logos_rust_sdk::EventSubscription, logos_rust_sdk::LogosError>,
    decode: fn(&logos_rust_sdk::EventData) -> Option<T>,
    handle: fn(T),
) -> Result<(), String> {
    let sub = sub.map_err(|e| format!("subscribe: {e}"))?;
    std::thread::spawn(move || sub.filter_map(|ev| decode(&ev)).for_each(handle));
    Ok(())
}

/// Set once our event listeners exist: the SDK re-arms them when Delivery's host restarts, so a
/// bring-up after that only redoes createNode (a second set would handle every message twice).
static LISTENING: AtomicBool = AtomicBool::new(false);

/// Note a channel send's final outcome (see `Shared::send_error`).
fn sent(error: Option<String>) {
    if let Some(sh) = SHARED.lock().unwrap().as_mut() {
        sh.send_error = error.map(|e| format!("messages aren't reaching the network ({e})"));
    }
}

/// createNode (another module may own it), then subscribe.
// ponytail: if a later subscribe fails after an earlier one worked, the retry adds one duplicate
// listener (harmless: advance, on_wire and sent are idempotent); split the step if that matters.
fn create_and_listen() -> Result<(), String> {
    use delivery_module::DeliveryModuleClient as D;
    // PUKAAR_DELIVERY_CFG overrides the network (LAN entry-node, L2).
    let cfg = std::env::var("PUKAAR_DELIVERY_CFG").unwrap_or_else(|_| DELIVERY_CFG.to_string());
    delivered_once(
        "createNode",
        modules()
            .delivery_module
            .create_node_with_timeout(&cfg, DELIVERY_TIMEOUT),
    )?;
    if LISTENING.load(Ordering::Relaxed) {
        return Ok(());
    }
    listen(
        modules().delivery_module.on_node_started(),
        D::decode_node_started,
        |ev| {
            // A failed start (e.g. the node was already running) still lets us try the channel
            // (after RETRY_AFTER).
            advance(Ok(Link::Started));
            if !ev.success {
                advance(Err(format!("nodeStarted: {}", ev.message)));
            }
        },
    )?;
    listen(
        modules().delivery_module.on_channel_message_received(),
        D::decode_channel_message_received,
        |m| on_wire(&m.channel_id, &m.payload),
    )?;
    listen(
        modules().delivery_module.on_channel_message_error(),
        D::decode_channel_message_error,
        |m| sent(Some(m.error)),
    )?;
    listen(
        modules().delivery_module.on_channel_message_sent(),
        D::decode_channel_message_sent,
        |_| sent(None),
    )?;
    LISTENING.store(true, Ordering::Relaxed);
    Ok(())
}

fn open_channel(site_hex: &str, me_hex: &str) -> Result<(), String> {
    let topic = format!("/pukaar/1/site-{site_hex}/proto");
    // An existing channel (our earlier load, or a failed channelClose) already carries our site.
    delivered_once(
        "channelCreate",
        modules().delivery_module.channel_create_with_timeout(
            site_hex,
            &topic,
            me_hex,
            DELIVERY_TIMEOUT,
        ),
    )
}

/// Called from the Delivery event thread for every channel message; only our site's count
/// (a channel left behind by leave_site must not feed the new site, e.g. a restore's Heads).
fn on_wire(channel: &str, bytes: &[u8]) {
    let mut g = SHARED.lock().unwrap();
    let Some(Shared {
        node: Some(node),
        outbox,
        last_answer,
        dirty,
        ..
    }) = g.as_mut()
    else {
        return;
    };
    if channel != hex::encode(node.store.site) {
        return;
    }
    match Wire::decode(bytes) {
        Some(Wire::Event(b)) => {
            // fork evidence is held and saved too, and a winning twin changes the board
            *dirty |= matches!(
                node.receive(&b),
                Ok(Accept::New | Accept::Fork | Accept::Rejected(_))
            );
        }
        Some(Wire::Heads(theirs)) => {
            *dirty |= node.observe_heads(&theirs); // true: a restore's wait just ended
            if last_answer.is_some_and(|t| t.elapsed() < ANSWER_EVERY) {
                return; // answered one recently
            }
            // Everyone re-sends their OWN missing events; only the kiosk (admin) and 3 elected
            // peers re-send everyone's, so a newcomer gets 3-4 copies instead of 30.
            *last_answer = Some(Instant::now());
            let st = node.state();
            let members: Vec<_> = st.roles.keys().copied().collect();
            let me = node.me();
            let batch = if st.admin == Some(me) || should_answer(&me, &members, bytes, now() / 60) {
                to_resend(&node.store, &theirs)
            } else {
                to_resend_own(&node.store, &theirs, &me)
            };
            outbox.extend(batch.into_iter().map(|b| Wire::Event(b).encode()));
        }
        None => {} // junk on the topic
    }
}

/// Send what's queued, plus our heads once a minute. The outbox waits until the channel is open.
fn send_queued(site: &str) -> Result<(), String> {
    let out = {
        let mut g = SHARED.lock().unwrap();
        let Some(Shared {
            node: Some(node),
            outbox,
            last_heads,
            ..
        }) = g.as_mut()
        else {
            return Ok(());
        };
        if last_heads.is_none_or(|t| t.elapsed() >= HEADS_EVERY) {
            *last_heads = Some(Instant::now());
            outbox.push(heads_msg(&node.store).encode());
        }
        std::mem::take(outbox)
    };
    // Before, a send batch was dropped whole when Delivery didn't answer, so a join made while
    // Delivery was down waited for a peer's Heads to come back. Now it goes out on reconnect.
    // ponytail: while Delivery stays down one stale Heads per minute piles up here (tens of bytes
    // each, harmless when sent); drop queued Heads before adding a new one if outages run to days.
    let (unsent, err) = send_queue(out, |msg| {
        let r = modules()
            .delivery_module
            .channel_send_with_timeout(site, msg, DELIVERY_TIMEOUT);
        let answered = r.is_ok();
        delivered("channelSend", r).map_err(|e| SendError::new(answered, e))
    });
    if !unsent.is_empty() {
        // in front of anything on_wire queued meanwhile, in order
        if let Some(sh) = SHARED.lock().unwrap().as_mut() {
            sh.outbox.splice(..0, unsent);
        }
    }
    err.map_or(Ok(()), Err)
}

/// One Delivery step: bring the node up, open our site's channel, then send. The UI polls
/// my_identity every 2 s, so this runs often.
fn flush() {
    let (link, ids) = {
        let mut g = SHARED.lock().unwrap();
        let Some(sh) = g.as_mut() else {
            return;
        };
        sh.auto_seal(); // the poll is also the clock for notice deadlines
        sh.save_due(false); // the poll is the clock for debounced saves
        if sh.retry_at.is_some_and(|t| Instant::now() < t) {
            return; // Delivery just failed: don't stall this call on it again (the outbox waits)
        }
        let ids = sh
            .node
            .as_ref()
            .map(|n| (hex::encode(n.store.site), hex::encode(n.me())));
        (sh.link, ids)
    };
    advance(match (link, ids) {
        (Link::Down, _) => create_and_listen().map(|()| Link::Listening),
        (Link::Listening, _) => delivered(
            "start",
            modules()
                .delivery_module
                .start_with_timeout(DELIVERY_TIMEOUT),
        )
        .map(|()| Link::Starting),
        (Link::Started, Some((site, me))) => open_channel(&site, &me).map(|()| Link::Open),
        (Link::Open, Some((site, _))) => send_queued(&site).map(|()| Link::Open),
        _ => return, // waiting for nodeStarted, or no site yet
    });
}

fn read<T>(f: impl FnOnce(&Node) -> T) -> Result<T, String> {
    let g = SHARED.lock().unwrap();
    let sh = g.as_ref().ok_or("module not ready")?;
    let node = sh.node.as_ref().ok_or(NO_SITE)?;
    Ok(f(node))
}

/// Change our node, then save it. `f` may queue wire messages on the outbox.
fn write<T>(
    f: impl FnOnce(&mut Node, &mut Vec<Vec<u8>>) -> Result<T, String>,
) -> Result<T, String> {
    let mut g = SHARED.lock().unwrap();
    let sh = g.as_mut().ok_or("module not ready")?;
    let node = sh.node.as_mut().ok_or(NO_SITE)?;
    let r = f(node, &mut sh.outbox)?;
    sh.persist(); // our own events are never debounced
    Ok(r)
}

/// Sign an event, save, queue it for sending. Rule violations are still published
/// (inert and visible, by design); the caller gets `rejected: <why>`.
fn publish_with(f: impl FnOnce(&mut Node) -> Result<Event, String>) -> String {
    let r = write(|node, outbox| {
        let e = f(node)?;
        outbox.push(Wire::Event(e.bytes.clone()).encode());
        Ok(match node.state().rejected.get(&e.id) {
            Some(why) => format!("rejected: {why}"),
            None => hex::encode(e.id),
        })
    });
    flush();
    r.unwrap_or_else(err)
}

fn publish(body: Body) -> String {
    publish_with(|n| n.publish(body, now()))
}

/// Create or join: install a fresh Node for this instance.
fn start_site(make: impl FnOnce(SigningKey, &Path) -> Result<Node, String>) -> String {
    let site = {
        let mut g = SHARED.lock().unwrap();
        let Some(sh) = g.as_mut() else {
            return err("module not ready");
        };
        if let Some(n) = &sh.node {
            return err(format!("already in site {}", hex::encode(n.store.site)));
        }
        let key = match persist::load_or_create_key(&sh.dir) {
            Ok(k) => k,
            Err(e) => return err(e),
        };
        let node = match make(key, &sh.dir) {
            Ok(n) => n,
            Err(e) => return err(e),
        };
        if let Err(e) = persist::save(&node, &sh.dir) {
            return err(e);
        }
        let site = hex::encode(node.store.site);
        // send what we just signed (genesis, or the joiner's announce) straight away
        sh.outbox.extend(
            node.store
                .events
                .values()
                .map(|e| Wire::Event(e.bytes.clone()).encode()),
        );
        sh.node = Some(node);
        site
    };
    flush();
    site
}

#[derive(Default)]
struct Pukaar;

/// Unload: flush what the debounce is still holding.
impl logos_rust_sdk::AboutToUnload for Pukaar {
    fn about_to_unload(&self) -> logos_rust_sdk::Shutdown {
        if let Some(sh) = SHARED.lock().unwrap().as_mut() {
            sh.save_due(true);
        }
        logos_rust_sdk::Shutdown::Synchronous
    }
}

impl PukaarCoreModule for Pukaar {
    fn site_create(&mut self, genesis_json: String) -> String {
        match genesis_from_json(&genesis_json) {
            Ok(g) => start_site(|key, _| Node::create_site(key, g, now())),
            Err(e) => err(e),
        }
    }

    fn site_join(&mut self, site_hex: String) -> String {
        match parse_id(site_hex.trim()) {
            // announce with an empty profile, so a pseudonymous joiner still appears as pending
            Some(site) => start_site(|key, dir| Ok(persist::join_site(dir, key, site, now()))),
            None => err("site id must be 64 hex characters"),
        }
    }

    fn leave_site(&mut self) -> String {
        let r = (|| {
            let mut g = SHARED.lock().unwrap();
            let sh = g.as_mut().ok_or("module not ready")?;
            let site = sh.node.as_ref().ok_or(NO_SITE)?.store.site;
            persist::leave(&sh.dir, &site).map_err(|e| e.to_string())?;
            sh.node = None;
            sh.outbox.clear();
            sh.last_heads = None;
            let was_open = sh.link == Link::Open;
            sh.link = sh.link.min(Link::Started); // the next site opens its own channel
            Ok::<_, String>(was_open.then(|| hex::encode(site)))
        })();
        match r {
            Ok(open) => {
                // best effort: on_wire ignores other channels anyway
                if let Some(site) = open {
                    let _ = modules()
                        .delivery_module
                        .channel_close_with_timeout(&site, DELIVERY_TIMEOUT);
                }
                "ok".into()
            }
            Err(e) => err(e),
        }
    }

    fn my_identity(&mut self) -> String {
        flush();
        let mut v = read(|n| n.identity_json_at(now()))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({ "site": null }));
        v["delivery"] = delivery_status().into();
        v.to_string()
    }

    fn export_identity(&mut self, password: String) -> String {
        read(|n| identity::export_identity(&n.key, &n.store.site, &password))
            .and_then(|r| r)
            .unwrap_or_else(err)
    }

    fn import_identity(&mut self, blob: String, password: String) -> String {
        let r = (|| {
            let mut g = SHARED.lock().unwrap();
            let sh = g.as_mut().ok_or("module not ready")?;
            sh.node = Some(identity::import_into(&sh.dir, &blob, &password, now())?);
            Ok::<_, String>("ok".to_string())
        })();
        flush(); // opens the channel; our heads go out and peers re-send our history
        r.unwrap_or_else(err)
    }

    fn skip_history_sync(&mut self) -> String {
        write(|n, _| n.skip_history_sync(now()))
            .map(|()| "ok".to_string())
            .unwrap_or_else(err)
    }

    fn set_profile(&mut self, display_name: String) -> String {
        let name = Some(display_name.trim().to_string()).filter(|n| !n.is_empty());
        publish(Body::Profile { display_name: name })
    }

    fn grant_role(&mut self, subject_hex: String, role: String, name: String) -> String {
        let role = match role.as_str() {
            "resident" => Role::Resident,
            "steward" => Role::Steward,
            "admin" => Role::Admin,
            other => return err(format!("unknown role {other}")),
        };
        match parse_id(subject_hex.trim()) {
            Some(subject) => {
                let name = Some(name.trim().to_string()).filter(|n| !n.is_empty());
                publish(Body::RoleGrant {
                    subject,
                    role,
                    name,
                })
            }
            None => err("bad key"),
        }
    }

    fn revoke_role(&mut self, subject_hex: String, reason: String) -> String {
        publish_with(|n| n.revoke_role(&subject_hex, &reason, now()))
    }

    fn add_super_admin(&mut self, subject_hex: String) -> String {
        publish_with(|n| n.add_super_admin(&subject_hex, now()))
    }

    fn remove_super_admin(&mut self, subject_hex: String) -> String {
        publish_with(|n| n.remove_super_admin(&subject_hex, now()))
    }

    fn transfer_super_admin(&mut self, to_hex: String) -> String {
        publish_with(|n| n.transfer_super_admin(&to_hex, now()))
    }

    fn admin_notice(
        &mut self,
        subject_hex: String,
        outcome: String,
        deadline: i64,
        reason: String,
    ) -> String {
        let Ok(deadline) = u64::try_from(deadline) else {
            return err("bad deadline");
        };
        publish_with(|n| n.admin_notice(&subject_hex, &outcome, deadline, &reason, now()))
    }

    fn cancel_notice(&mut self, subject_hex: String) -> String {
        publish_with(|n| n.cancel_notice(&subject_hex, now()))
    }

    fn end_notice_now(&mut self, subject_hex: String) -> String {
        publish_with(|n| n.end_notice_now(&subject_hex, now()))
    }

    fn add_location(&mut self, code: String, label: String, group: String) -> String {
        publish_with(|n| n.add_location(&code, &label, &group, now()))
    }

    fn retire_location(&mut self, code: String, reason: String) -> String {
        publish_with(|n| n.retire_location(&code, &reason, now()))
    }

    fn restore_location(&mut self, code: String, reason: String) -> String {
        publish_with(|n| n.restore_location(&code, &reason, now()))
    }

    fn edit_location(&mut self, code: String, label: String, group: String) -> String {
        publish_with(|n| n.edit_location(&code, &label, &group, now()))
    }

    fn remove_location(&mut self, code: String, reason: String) -> String {
        publish_with(|n| n.remove_location(&code, &reason, now()))
    }

    fn undo_remove_location(&mut self, code: String, reason: String) -> String {
        publish_with(|n| n.undo_remove_location(&code, &reason, now()))
    }

    fn report(
        &mut self,
        category: String,
        location: String,
        landmark: String,
        text: String,
    ) -> String {
        publish(Body::Report {
            category,
            location,
            landmark: landmark.trim().into(),
            text: text.trim().into(),
        })
    }

    fn act(
        &mut self,
        issue_hex: String,
        action: String,
        note: String,
        next_step: String,
        eta_h: i64,
    ) -> String {
        let Some(issue) = parse_id(&issue_hex) else {
            return err("bad issue id");
        };
        match action_body(
            issue,
            &action,
            note.trim(),
            next_step.trim(),
            eta_h.clamp(0, 24 * 30) as u32,
        ) {
            Ok(body) => publish(body),
            Err(e) => err(e),
        }
    }

    fn list_issues(&mut self) -> String {
        read(|n| n.issues_json(now())).unwrap_or_else(err)
    }

    fn issue_timeline(&mut self, issue_hex: String) -> String {
        read(|n| n.timeline_json(&issue_hex, now())).unwrap_or_else(err)
    }

    fn site_info(&mut self) -> String {
        read(|n| n.site_info_json(now())).unwrap_or_else(err)
    }

    fn checkpoint_now(&mut self) -> String {
        read(|n| n.checkpoint_json()).unwrap_or_else(err)
    }

    fn record_anchor(&mut self, heads_json: String, lez_ref: String) -> String {
        publish_with(|n| n.record_anchor(&heads_json, &lez_ref, now()))
    }

    fn on_context_ready(&mut self, ctx: &RustModuleContext) {
        let dir = PathBuf::from(&ctx.instance_persistence_path);
        let node = persist::load(&dir).unwrap_or_else(|e| {
            eprintln!("pukaar: could not load replica: {e}");
            None
        });
        // No Delivery calls here (see Link): flush() brings the node up on the first poll.
        *SHARED.lock().unwrap() = Some(Shared {
            dir,
            node,
            outbox: vec![],
            last_heads: None,
            last_answer: None,
            link: Link::Down,
            link_error: None,
            send_error: None,
            retry_at: None,
            dirty: false,
            last_save: Instant::now(),
        });
    }
}

#[no_mangle]
pub extern "Rust" fn logos_module_install() {
    logos_install!(Pukaar);
}
