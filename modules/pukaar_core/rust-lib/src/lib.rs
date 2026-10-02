//! Logos glue for Pukaar. The trait below IS the module's API (the builder
//! derives the .lidl from it). All rules live in ./logic (pukaar_logic, a
//! standalone plain-`cargo test`-able crate nested under this one so
//! logos-module-builder's `cp -r <codegen.rust.crate>` staging sees it — see
//! rust-lib/Cargo.toml); this file only moves bytes between that crate, the
//! disk and Delivery.

use pukaar_logic::event::{Body, Event, Location, Role, SigningKey};
use pukaar_logic::identity;
use pukaar_logic::node::{action_body, genesis_from_json, parse_id, Node};
use pukaar_logic::persist;
use pukaar_logic::store::Accept;
use pukaar_logic::sync::{heads_msg, should_answer, to_resend, to_resend_own, Wire};
use std::path::PathBuf;
// Mutex comes from the generated scaffold below (same module scope): a second
// `use std::sync::Mutex;` here would be a duplicate import (confirmed at Step 8).
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub trait PukaarCoreModule: Send + 'static {
    fn site_create(&mut self, genesis_json: String) -> String;
    fn site_join(&mut self, site_hex: String) -> String;
    fn my_identity(&mut self) -> String;
    /// A password-sealed backup of this identity: one `pukaar-id-1:…` line, or `error: …`.
    fn export_identity(&mut self, password: String) -> String;
    /// Restore a backup on a fresh device (no site yet): `ok` or `error: …`. Publishing is
    /// refused until our own history is back (`my_identity().syncing_own_history`).
    fn import_identity(&mut self, blob: String, password: String) -> String;
    /// Stop waiting for a restored identity's history (allowed 10 min after the restore).
    fn skip_history_sync(&mut self) -> String;
    fn set_profile(&mut self, display_name: String) -> String;
    /// `name` is required for steward/admin and must be empty for residents.
    fn grant_role(&mut self, subject_hex: String, role: String, name: String) -> String;
    /// Admin only (the rules reject anyone else's); `reason` is required.
    fn revoke_role(&mut self, subject_hex: String, reason: String) -> String;
    fn add_location(&mut self, code: String, label: String, group: String) -> String;
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

/// Delivery bring-up, advanced one step per flush(). Not done in on_context_ready: the host
/// authorises our calls to delivery_module only after that returns, so a createNode/start made
/// there fails "unauthorized", the node never starts, and nothing ever crosses (seen headless).
/// Only moves forward, so a nodeStarted that lands mid-flush is never overwritten.
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
    /// so Delivery is never called from our own threads.
    outbox: Vec<Vec<u8>>,
    last_heads: Option<Instant>,
    last_answer: Option<Instant>,
    link: Link,
    /// Last Delivery failure, shown as `"delivery": "error: …"` until a later step succeeds.
    link_error: Option<String>,
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
fn save(node: &Node, dir: &std::path::Path) {
    if let Err(e) = persist::save(node, dir) {
        eprintln!("pukaar: save failed: {e}");
    }
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

/// Record the outcome of a bring-up step. An error keeps the link where it is: the next flush retries.
fn advance(r: Result<Link, String>) {
    if let Some(sh) = SHARED.lock().unwrap().as_mut() {
        match r {
            Ok(l) => {
                sh.link = sh.link.max(l);
                sh.link_error = None;
            }
            Err(e) => sh.link_error = Some(e),
        }
    }
}

fn delivery_status() -> String {
    match SHARED.lock().unwrap().as_ref() {
        None => "module not ready".into(),
        Some(Shared {
            link_error: Some(e),
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

/// createNode (another module may own it: "already initialized" is fine), then subscribe.
// ponytail: if the 2nd subscribe fails after the 1st worked, the retry adds one duplicate
// nodeStarted listener (harmless: advance is idempotent); split the step if that ever matters.
fn create_and_listen() -> Result<(), String> {
    use delivery_module::DeliveryModuleClient as D;
    // PUKAAR_DELIVERY_CFG overrides the network (LAN entry-node, L2).
    let cfg = std::env::var("PUKAAR_DELIVERY_CFG")
        .unwrap_or_else(|_| r#"{"mode":"Edge","preset":"logos.test"}"#.to_string());
    match delivered("createNode", modules().delivery_module.create_node(&cfg)) {
        Err(e) if !e.contains("already initialized") => return Err(e),
        _ => {}
    }
    listen(
        modules().delivery_module.on_node_started(),
        D::decode_node_started,
        |ev| {
            // A failed start (e.g. the node was already running) still lets us try the channel.
            advance(Ok(Link::Started));
            if !ev.success {
                advance(Err(format!("nodeStarted: {}", ev.message)));
            }
        },
    )?;
    listen(
        modules().delivery_module.on_channel_message_received(),
        D::decode_channel_message_received,
        |m| on_wire(&m.payload),
    )
}

fn open_channel(site_hex: &str, me_hex: &str) -> Result<(), String> {
    let topic = format!("/pukaar/1/site-{site_hex}/proto");
    delivered(
        "channelCreate",
        modules()
            .delivery_module
            .channel_create(site_hex, &topic, me_hex),
    )
}

/// Called from the Delivery event thread for every message on our channel.
fn on_wire(bytes: &[u8]) {
    let mut g = SHARED.lock().unwrap();
    let Some(Shared {
        dir,
        node: Some(node),
        outbox,
        last_answer,
        ..
    }) = g.as_mut()
    else {
        return;
    };
    match Wire::decode(bytes) {
        Some(Wire::Event(b)) => {
            if let Ok(Accept::New) = node.receive(&b) {
                save(node, dir);
            }
        }
        Some(Wire::Heads(theirs)) => {
            if node.observe_heads(&theirs) {
                save(node, dir); // a restore's wait just ended
            }
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
    // try every message, keep the last failure; a lost one comes back through anti-entropy
    let mut r = Ok(());
    for msg in &out {
        if let Err(e) = delivered(
            "channelSend",
            modules().delivery_module.channel_send(site, msg),
        ) {
            r = Err(e);
        }
    }
    r
}

/// One Delivery step: bring the node up, open our site's channel, then send. The UI polls
/// my_identity every 2 s, so this runs often.
fn flush() {
    let (link, ids) = {
        let g = SHARED.lock().unwrap();
        let Some(sh) = g.as_ref() else {
            return;
        };
        let ids = sh
            .node
            .as_ref()
            .map(|n| (hex::encode(n.store.site), hex::encode(n.me())));
        (sh.link, ids)
    };
    advance(match (link, ids) {
        (Link::Down, _) => create_and_listen().map(|()| Link::Listening),
        (Link::Listening, _) => {
            delivered("start", modules().delivery_module.start()).map(|()| Link::Starting)
        }
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
    save(node, &sh.dir);
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
fn start_site(make: impl FnOnce(SigningKey) -> Result<Node, String>) -> String {
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
        let node = match make(key) {
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

impl PukaarCoreModule for Pukaar {
    fn site_create(&mut self, genesis_json: String) -> String {
        match genesis_from_json(&genesis_json) {
            Ok(g) => start_site(|key| Node::create_site(key, g, now())),
            Err(e) => err(e),
        }
    }

    fn site_join(&mut self, site_hex: String) -> String {
        match parse_id(site_hex.trim()) {
            // announce with an empty profile, so a pseudonymous joiner still appears as pending
            Some(site) => start_site(|key| Ok(Node::join_announced(key, site, now()))),
            None => err("site id must be 64 hex characters"),
        }
    }

    fn my_identity(&mut self) -> String {
        flush();
        let mut v = read(|n| n.identity_json())
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

    fn add_location(&mut self, code: String, label: String, group: String) -> String {
        let loc = Location {
            code: code.trim().into(),
            label: label.trim().into(),
            group: group.trim().into(),
        };
        publish(Body::LocationsAdd {
            locations: vec![loc],
        })
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
        read(|n| n.issues_json()).unwrap_or_else(err)
    }

    fn issue_timeline(&mut self, issue_hex: String) -> String {
        read(|n| n.timeline_json(&issue_hex)).unwrap_or_else(err)
    }

    fn site_info(&mut self) -> String {
        read(|n| n.site_info_json()).unwrap_or_else(err)
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
        });
    }
}

#[no_mangle]
pub extern "Rust" fn logos_module_install() {
    install::<Pukaar>();
}
