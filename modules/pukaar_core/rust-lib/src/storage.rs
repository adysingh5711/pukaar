//! The real `Store`: `pukaar_core` calls `storage_module` through the raw SDK (the "core path" that
//! P0 chose). The `Store` trait and everything that uses it live in `logic/src/fetch.rs`, so the
//! tests run against a fake. This file is the only code that talks to `storage_module`.
//!
//! Rules:
//! - No module call waits for Storage. Only the sync thread of `lib.rs` calls `upload` and
//!   `download`. They can take seconds, and the UI gives each call 20 seconds.
//! - The first call starts the Storage node (`init`, `start`), which can take 30 to 80 seconds on a
//!   cold start. It runs on the same sync thread.
//! - `uploadUrl` and `downloadToUrl` take file paths and answer through the events
//!   `storageUploadDone` and `storageDownloadDone`. Each call writes or reads one temp file.
//! - The caller checks the SHA-256 of every download against the chain. This file does not.
//!
//! Fallback, not built: if the core path loses replies under Basecamp (storage#88: after `start()`
//! the module host can drop the reply of a cross-module call), the UI can make the Storage calls
//! itself. `Main.qml` would call `logos.callModuleAsync("storage_module", "uploadUrl", [...])` as
//! `filesharing-ui` does, and hand the CID to the core. This needs a new module method that takes
//! a CID, and a QML change. Do that only if the check of step 2.5 fails.
//!
//! The node reads the config from `~/.logos_storage/config.json` (`loadConfigOrDefault`) and `init`
//! writes it back. `PUKAAR_STORAGE_CFG` holds a JSON object that overrides keys of that config (for
//! a local test: `nat`, `bootstrap-node`, `data-dir`). A test must redirect `HOME` first.

use logos_rust_sdk::{EventData, LogosError, LogosModuleSDK, PluginProxy};
use pukaar_logic::fetch::Store;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const MODULE: &str = "storage_module";
/// A plain call (not `init` or `start`) answers fast or not at all.
const CALL: Duration = Duration::from_secs(10);
/// `init` and `start` on a cold start.
const START: Duration = Duration::from_secs(120);
/// From the call to the `...Done` event.
const TRANSFER: Duration = Duration::from_secs(30);
const CHUNK: u64 = 65536;
/// Events waiting for a transfer. A shared module may emit the events of other callers too.
const MAX_EVENTS: usize = 64;

/// The `storageUploadDone` and `storageDownloadDone` events that arrived, oldest first.
#[derive(Default)]
struct Events {
    queue: Mutex<Vec<(&'static str, Value)>>,
    arrived: Condvar,
}

impl Events {
    fn push(&self, kind: &'static str, payload: Value) {
        let mut q = self.queue.lock().unwrap();
        if q.len() >= MAX_EVENTS {
            q.remove(0);
        }
        q.push((kind, payload));
        self.arrived.notify_all();
    }

    fn clear(&self, kind: &str) {
        self.queue.lock().unwrap().retain(|(k, _)| *k != kind);
    }

    /// The first `kind` event for session `sid`. An empty `sid`, or an event without one, matches
    /// any: on macOS the reply of `uploadUrl` has been seen empty, and one transfer runs at a time.
    fn wait(&self, kind: &str, sid: &str, limit: Duration) -> Result<Value, String> {
        let end = Instant::now() + limit;
        let mut q = self.queue.lock().unwrap();
        loop {
            let hit = q.iter().position(|(k, v)| *k == kind && (sid.is_empty() || v["sessionId"].as_str().is_none_or(|s| s == sid)));
            if let Some(i) = hit {
                return Ok(q.remove(i).1);
            }
            let left = end.checked_duration_since(Instant::now()).ok_or("timed out waiting for Storage")?;
            q = self.arrived.wait_timeout(q, left).unwrap().0;
        }
    }
}

/// The payload of a Storage event: one JSON string (LIDL: `payload: tstr`), or already an object.
fn payload(ev: &EventData) -> Option<Value> {
    let first = ev.data.as_array().and_then(|a| a.first()).unwrap_or(&ev.data);
    match first {
        Value::String(s) => serde_json::from_str(s).ok(),
        Value::Object(_) => Some(first.clone()),
        _ => None,
    }
}

/// `{"success", "value", "error"}` is the answer of a `result` method. Anything else is the value.
fn value_of(v: Value) -> Result<Value, String> {
    match v.get("success") {
        Some(Value::Bool(false)) => Err(v["error"].as_str().unwrap_or("refused").to_string()),
        Some(_) => Ok(v.get("value").cloned().unwrap_or(Value::Null)),
        None => Ok(v),
    }
}

fn truthy(v: &Value) -> bool {
    matches!(v, Value::Bool(true)) || matches!(v.get("value"), Some(Value::Bool(true)))
}

fn call(p: &PluginProxy, method: &str, args: Value, limit: Duration) -> Result<Value, String> {
    let r = p.call_json_with_timeout(method, &args, limit).map_err(|e: LogosError| format!("{method}: {e}"))?;
    value_of(r).map_err(|e| format!("{method}: {e}"))
}

/// The proxy and the events. `ready` is true once `init` and `start` worked. The subscriptions are
/// made once: a failed start is retried without new listener threads.
struct Client {
    proxy: PluginProxy,
    events: Arc<Events>,
    ready: bool,
}

pub struct LogosStore {
    /// Temp files for the transfers.
    dir: PathBuf,
    seq: AtomicU64,
    /// Held for a whole transfer: one at a time, as `uploadUrl` needs.
    node: Mutex<Option<Client>>,
}

impl Client {
    fn subscribe() -> Result<Client, String> {
        let mut proxy = LogosModuleSDK::new().plugin(MODULE);
        let events = Arc::new(Events::default());
        for (name, kind) in [("storageUploadDone", "upload"), ("storageDownloadDone", "download")] {
            let sub = proxy.on(name).map_err(|e| format!("subscribe {name}: {e}"))?;
            let events = events.clone();
            std::thread::spawn(move || {
                for ev in sub {
                    if let Some(v) = payload(&ev) {
                        events.push(kind, v);
                    }
                }
            });
        }
        Ok(Client { proxy, events, ready: false })
    }

    /// Load the config, `init`, `start`, and wait until the node runs. `init` refuses a second
    /// call, and the module is shared with other modules, so "already" is not a failure.
    fn bring_up(&mut self) -> Result<(), String> {
        let p = &self.proxy;
        let mut cfg = match call(p, "loadConfigOrDefault", json!([]), CALL)? {
            Value::String(s) => serde_json::from_str(&s).map_err(|e| format!("loadConfigOrDefault: {e}"))?,
            v => v,
        };
        if let Ok(over) = std::env::var("PUKAAR_STORAGE_CFG") {
            let over: Value = serde_json::from_str(&over).map_err(|e| format!("PUKAAR_STORAGE_CFG: {e}"))?;
            for (k, v) in over.as_object().into_iter().flatten() {
                cfg[k] = v.clone();
            }
        }
        // A refused `init` answers a bare `false` (seen: a config key it does not know). The node may
        // also be set up by another module, so go on to `start`, which then reports the real failure.
        match call(p, "init", json!([cfg.to_string()]), START) {
            Ok(v) if !matches!(v, Value::Bool(false)) => {}
            Err(e) if e.contains("already") => {}
            Ok(_) => eprintln!("pukaar: storage init was refused"),
            Err(e) => eprintln!("pukaar: storage init: {e}"),
        }
        if matches!(call(p, "start", json!([]), START)?, Value::Bool(false)) {
            return Err("storage start was refused".into());
        }
        let end = Instant::now() + START;
        while !truthy(&p.call_json_with_timeout("isRunning", &json!([]), CALL).map_err(|e| format!("isRunning: {e}"))?) {
            if Instant::now() > end {
                return Err("the Storage node did not start".into());
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        self.ready = true;
        Ok(())
    }
}

impl LogosStore {
    pub fn new(dir: PathBuf) -> LogosStore {
        LogosStore { dir, seq: AtomicU64::new(0), node: Mutex::new(None) }
    }

    fn with_node<T>(&self, f: impl FnOnce(&PluginProxy, &Events) -> Result<T, String>) -> Result<T, String> {
        let mut guard = self.node.lock().unwrap();
        if guard.is_none() {
            *guard = Some(Client::subscribe()?);
        }
        let node = guard.as_mut().expect("set above");
        if !node.ready {
            node.bring_up()?;
        }
        f(&node.proxy, &node.events)
    }

    fn temp(&self, tag: &str) -> Result<PathBuf, String> {
        std::fs::create_dir_all(&self.dir).map_err(|e| format!("{}: {e}", self.dir.display()))?;
        Ok(self.dir.join(format!("{tag}-{}-{}", std::process::id(), self.seq.fetch_add(1, Ordering::Relaxed))))
    }
}

impl Store for LogosStore {
    fn upload(&self, bytes: &[u8]) -> Result<String, String> {
        let path = self.temp("up")?;
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        let r = self.with_node(|p, ev| {
            ev.clear("upload");
            let sid = call(p, "uploadUrl", json!([path.to_string_lossy(), CHUNK, true]), CALL)?;
            let done = ev.wait("upload", sid.as_str().unwrap_or(""), TRANSFER)?;
            match (done["success"].as_bool(), done["cid"].as_str()) {
                (Some(true), Some(cid)) | (None, Some(cid)) => Ok(cid.to_string()),
                _ => Err(format!("upload: {}", done["error"].as_str().unwrap_or("no CID in the answer"))),
            }
        });
        let _ = std::fs::remove_file(&path);
        r
    }

    fn download(&self, cid: &str) -> Result<Vec<u8>, String> {
        let path = self.temp("down")?;
        let r = self.with_node(|p, ev| {
            ev.clear("download");
            let sid = call(p, "downloadToUrl", json!([cid, path.to_string_lossy(), false, CHUNK, false, true]), CALL)?;
            let done = ev.wait("download", sid.as_str().unwrap_or(""), TRANSFER)?;
            if done["success"].as_bool() == Some(false) {
                return Err(format!("download {cid}: {}", done["error"].as_str().unwrap_or("failed")));
            }
            std::fs::read(&path).map_err(|e| format!("download {cid}: {e}"))
        });
        let _ = std::fs::remove_file(&path);
        r
    }
}
