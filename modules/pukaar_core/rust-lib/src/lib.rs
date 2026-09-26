//! Logos glue for Pukaar. The trait below IS the module's API (the builder
//! derives the .lidl from it). All rules live in ./logic (pukaar_logic, a
//! standalone plain-`cargo test`-able crate nested under this one so
//! logos-module-builder's `cp -r <codegen.rust.crate>` staging sees it — see
//! rust-lib/Cargo.toml); this file only moves bytes between that crate, the
//! disk and Delivery.

use pukaar_logic::event::{Body, Event, Location, Role, SigningKey};
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
    fn set_profile(&mut self, display_name: String) -> String;
    /// `name` is required for steward/admin and must be empty for residents.
    fn grant_role(&mut self, subject_hex: String, role: String, name: String) -> String;
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

struct Shared {
    dir: PathBuf,
    node: Option<Node>,
    /// Wire messages waiting to go out. Only drained inside module method calls,
    /// so Delivery is never called from our own threads.
    outbox: Vec<Vec<u8>>,
    last_heads: Option<Instant>,
    last_answer: Option<Instant>,
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

fn open_channel(site_hex: &str, me_hex: &str) {
    let topic = format!("/pukaar/1/site-{site_hex}/proto");
    if let Err(e) = modules()
        .delivery_module
        .channel_create(site_hex, &topic, me_hex)
    {
        eprintln!("pukaar: channelCreate failed: {e}");
    }
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
                if let Err(e) = persist::save(node, dir) {
                    eprintln!("pukaar: save failed: {e}");
                }
            }
        }
        Some(Wire::Heads(theirs)) if last_answer.is_none_or(|t| t.elapsed() >= ANSWER_EVERY) => {
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
        Some(Wire::Heads(_)) => {} // answered one recently
        None => {}                 // junk on the topic
    }
}

/// Send what's queued, plus our heads once a minute. The UI polls every 2 s, so this runs often.
fn flush() {
    let (site, out) = {
        let mut g = SHARED.lock().unwrap();
        let Some(Shared {
            node: Some(node),
            outbox,
            last_heads,
            ..
        }) = g.as_mut()
        else {
            return;
        };
        if last_heads.is_none_or(|t| t.elapsed() >= HEADS_EVERY) {
            *last_heads = Some(Instant::now());
            outbox.push(heads_msg(&node.store).encode());
        }
        (hex::encode(node.store.site), std::mem::take(outbox))
    };
    for msg in out {
        if let Err(e) = modules().delivery_module.channel_send(&site, &msg) {
            eprintln!("pukaar: channelSend failed: {e}");
        }
    }
}

fn read<T>(f: impl FnOnce(&Node) -> T) -> Result<T, String> {
    let g = SHARED.lock().unwrap();
    let sh = g.as_ref().ok_or("module not ready")?;
    let node = sh.node.as_ref().ok_or("no site yet: create or join one")?;
    Ok(f(node))
}

/// Sign an event, save, queue it for sending. Rule violations are still published
/// (inert and visible, by design); the caller gets `rejected: <why>`.
fn publish_with(f: impl FnOnce(&mut Node) -> Result<Event, String>) -> String {
    let r = (|| {
        let mut g = SHARED.lock().unwrap();
        let sh = g.as_mut().ok_or("module not ready")?;
        let node = sh.node.as_mut().ok_or("no site yet: create or join one")?;
        let e = f(node)?;
        if let Err(e) = persist::save(node, &sh.dir) {
            eprintln!("pukaar: save failed: {e}");
        }
        sh.outbox.push(Wire::Event(e.bytes.clone()).encode());
        Ok::<_, String>(match node.state().rejected.get(&e.id) {
            Some(why) => format!("rejected: {why}"),
            None => hex::encode(e.id),
        })
    })();
    flush();
    r.unwrap_or_else(err)
}

fn publish(body: Body) -> String {
    publish_with(|n| Ok(n.publish(body, now())))
}

/// Create or join: install a fresh Node for this instance.
fn start_site(make: impl FnOnce(SigningKey) -> Node) -> String {
    let (site, me) = {
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
        let node = make(key);
        if let Err(e) = persist::save(&node, &sh.dir) {
            return err(e);
        }
        let ids = (hex::encode(node.store.site), hex::encode(node.me()));
        // send what we just signed (genesis, or the joiner's announce) straight away
        sh.outbox.extend(
            node.store
                .events
                .values()
                .map(|e| Wire::Event(e.bytes.clone()).encode()),
        );
        sh.node = Some(node);
        ids
    };
    open_channel(&site, &me);
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
            Some(site) => start_site(|key| Node::join_announced(key, site, now())),
            None => err("site id must be 64 hex characters"),
        }
    }

    fn my_identity(&mut self) -> String {
        read(|n| n.identity_json()).unwrap_or_else(|_| "{\"site\":null}".into())
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
        flush();
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
        let ids = node
            .as_ref()
            .map(|n| (hex::encode(n.store.site), hex::encode(n.me())));
        *SHARED.lock().unwrap() = Some(Shared {
            dir,
            node,
            outbox: vec![],
            last_heads: None,
            last_answer: None,
        });

        // delivery_module is one node per Logos Core, shared by every module:
        // "already created" is normal. PUKAAR_DELIVERY_CFG overrides it (LAN entry-node, L2).
        let cfg = std::env::var("PUKAAR_DELIVERY_CFG")
            .unwrap_or_else(|_| r#"{"mode":"Edge","preset":"logos.test"}"#.to_string());
        if let Err(e) = modules().delivery_module.create_node(&cfg) {
            eprintln!("pukaar: createNode: {e} (fine if another module created the node)");
        }
        if let Err(e) = modules().delivery_module.start() {
            eprintln!("pukaar: start: {e}");
        }
        if let Some((site, me)) = ids {
            open_channel(&site, &me);
        }
        match modules().delivery_module.on_channel_message_received() {
            Ok(sub) => {
                std::thread::spawn(move || {
                    for ev in sub {
                        if let Some(m) =
                            delivery_module::DeliveryModuleClient::decode_channel_message_received(
                                &ev,
                            )
                        {
                            on_wire(&m.payload);
                        }
                    }
                });
            }
            Err(e) => eprintln!("pukaar: subscribe to channelMessageReceived failed: {e}"),
        }
    }
}

#[no_mangle]
pub extern "Rust" fn logos_module_install() {
    install::<Pukaar>();
}
