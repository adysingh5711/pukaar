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
//! The node starts from `loadConfigOrDefault` (the file `~/.logos_storage/config.json`, or the
//! defaults). `node_config` then gives this profile its own `data-dir` (`<module data>/storage`):
//! the metadata store of a data dir is locked, so two Storage nodes on one data dir cannot both
//! start ("Should create metadata store!"). The listen port is 0 (the OS picks a free one) and the
//! REST and metrics servers are off, so no port can clash. `PUKAAR_STORAGE_CFG` holds a JSON object
//! that overrides keys of the result (for a local test: `nat`, `bootstrap-node`, `data-dir`).
//!
//! A successful `init` of storage_module 3.0.2 writes the config to `$HOME/.logos_storage/config.json`
//! and no key, variable or argument moves that file. That file belongs to the default Basecamp
//! profile, so `keep_file` puts the old content back after `init`.

use logos_rust_sdk::{EventData, LogosError, LogosModuleSDK, PluginProxy};
use pukaar_logic::fetch::Store;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
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

/// First wait after a failed start, and the longest wait.
const RETRY_FIRST: Duration = Duration::from_secs(5);
const RETRY_MAX: Duration = Duration::from_secs(60);

/// The wait after a failure: 5 s, then twice the last one, at most 60 s.
fn next_wait(last: Option<Duration>) -> Duration {
    last.map_or(RETRY_FIRST, |w| (w * 2).min(RETRY_MAX))
}

/// The config of this node: `base` (what the module loaded) with its own data dir, a free listen
/// port and no REST or metrics server, then the keys of `over` (`PUKAAR_STORAGE_CFG`).
fn node_config(mut base: Value, data_dir: &Path, over: Option<&str>) -> Result<Value, String> {
    let o = base.as_object_mut().ok_or("loadConfigOrDefault: not an object")?;
    o.insert("data-dir".into(), json!(data_dir.to_string_lossy()));
    o.insert("listen-port".into(), json!(0));
    o.remove("api-bindaddr");
    o.remove("metrics");
    if let Some(over) = over {
        let over: Value = serde_json::from_str(over).map_err(|e| format!("PUKAAR_STORAGE_CFG: {e}"))?;
        for (k, v) in over.as_object().into_iter().flatten() {
            o.insert(k.clone(), v.clone());
        }
    }
    Ok(base)
}

/// Run `f`, then put `path` back as it was before: the old bytes, or no file if there was none.
/// ponytail: no lock, so a write by another process inside `f` is lost. Add a lock if that shows up.
fn keep_file<T>(path: &Path, f: impl FnOnce() -> T) -> T {
    let old = std::fs::read(path).ok();
    let r = f();
    match old {
        Some(b) => {
            let _ = std::fs::write(path, b);
        }
        None => {
            let _ = std::fs::remove_file(path);
        }
    }
    r
}

/// The file that `init` writes: `$HOME/.logos_storage/config.json`.
fn user_config_file() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").filter(|h| !h.is_empty())?;
    Some(PathBuf::from(home).join(".logos_storage").join("config.json"))
}

/// The proxy and the events. `ready` is true once `init` and `start` worked. The subscriptions are
/// made once: a failed start is retried without new listener threads, after `hold` (the last error
/// and the time of the next try, so a dead Storage is not asked every 10 s).
struct Client {
    proxy: PluginProxy,
    events: Arc<Events>,
    ready: bool,
    hold: Option<(Instant, String)>,
    wait: Option<Duration>,
}

/// What the UI shows as the Storage state: true once `init` and `start` worked (a call to the Storage
/// client went through), else the text of the last failed bring-up. One static: the node is one per module.
static STATUS: Mutex<(bool, Option<String>)> = Mutex::new((false, None));

/// `(ready, error)` of the Storage node. `ready` is false until the first bring-up works.
pub fn status() -> (bool, Option<String>) {
    STATUS.lock().unwrap().clone()
}

fn note_bring_up(r: &Result<(), String>) {
    *STATUS.lock().unwrap() = (r.is_ok(), r.as_ref().err().cloned());
}

/// `site_info.storage`: the fetch result (`pending`, `error`) plus `ready`. A fetch error wins over
/// the bring-up error, because it came later.
pub fn merge_status(storage: &mut Value, ready: bool, error: Option<String>) {
    storage["ready"] = json!(ready);
    if storage["error"].is_null() {
        storage["error"] = json!(error);
    }
}

pub struct LogosStore {
    /// The module data dir of this profile: `storage-tmp` holds the temp files of the transfers and
    /// `storage` the data of the Storage node.
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
        Ok(Client { proxy, events, ready: false, hold: None, wait: None })
    }

    /// Load the config, `init`, `start`, and wait until the node runs. `init` refuses a second
    /// call, and the module is shared with other modules, so "already" is not a failure.
    fn bring_up(&mut self, data_dir: &Path) -> Result<(), String> {
        let p = &self.proxy;
        let base = match call(p, "loadConfigOrDefault", json!([]), CALL)? {
            Value::String(s) => serde_json::from_str(&s).map_err(|e| format!("loadConfigOrDefault: {e}"))?,
            v => v,
        };
        let data = data_dir.join("storage");
        std::fs::create_dir_all(&data).map_err(|e| format!("{}: {e}", data.display()))?;
        let cfg = node_config(base, &data, std::env::var("PUKAAR_STORAGE_CFG").ok().as_deref())?;
        // A refused `init` answers a bare `false`, and the module writes the reason only to its log
        // (seen: a config key it does not know, or a data dir that another node holds). The node may
        // also be set up by another module, so go on to `start`, which then reports the failure.
        let run = || call(p, "init", json!([cfg.to_string()]), START);
        let init = match user_config_file() {
            Some(f) => keep_file(&f, run),
            None => run(),
        };
        let mut note = String::new();
        match init {
            Ok(v) if !matches!(v, Value::Bool(false)) => {}
            Err(e) if e.contains("already") => {}
            Ok(_) => note = format!("init was refused (see the storage_module log; data dir {})", data.display()),
            Err(e) => note = e,
        }
        if matches!(call(p, "start", json!([]), START)?, Value::Bool(false)) {
            return Err(format!("storage start was refused{}", if note.is_empty() { String::new() } else { format!(": {note}") }));
        }
        let end = Instant::now() + START;
        while !truthy(&p.call_json_with_timeout("isRunning", &json!([]), CALL).map_err(|e| format!("isRunning: {e}"))?) {
            if Instant::now() > end {
                return Err(format!("the Storage node did not start{}", if note.is_empty() { String::new() } else { format!(": {note}") }));
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
            if let Some((at, err)) = &node.hold {
                if Instant::now() < *at {
                    return Err(err.clone());
                }
            }
            let up = node.bring_up(&self.dir);
            note_bring_up(&up);
            if let Err(e) = up {
                let wait = next_wait(node.wait);
                node.wait = Some(wait);
                let e = format!("{e} (next try in {} s)", wait.as_secs());
                node.hold = Some((Instant::now() + wait, e.clone()));
                note_bring_up(&Err(e.clone()));
                return Err(e);
            }
            node.wait = None;
        }
        f(&node.proxy, &node.events)
    }

    fn temp(&self, tag: &str) -> Result<PathBuf, String> {
        let dir = self.dir.join("storage-tmp");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        Ok(dir.join(format!("{tag}-{}-{}", std::process::id(), self.seq.fetch_add(1, Ordering::Relaxed))))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Value {
        json!({"config-version": 3, "data-dir": "/home/u/.logos_storage/data", "listen-port": 8070, "api-bindaddr": "127.0.0.1", "metrics": true, "nat": "auto"})
    }

    #[test]
    fn config_gets_own_data_dir_and_free_ports() {
        let c = node_config(base(), Path::new("/m/spike-a/storage"), None).unwrap();
        assert_eq!(c["data-dir"], "/m/spike-a/storage");
        assert_eq!(c["listen-port"], 0);
        assert!(c.get("api-bindaddr").is_none() && c.get("metrics").is_none());
        assert_eq!(c["nat"], "auto");
        let d = node_config(base(), Path::new("/m/spike-b/storage"), None).unwrap();
        assert_ne!(c["data-dir"], d["data-dir"]);
    }

    #[test]
    fn env_override_wins_and_bad_json_is_an_error() {
        let c = node_config(base(), Path::new("/m/s"), Some(r#"{"data-dir":"/t","nat":"extip:1.2.3.4"}"#)).unwrap();
        assert_eq!((c["data-dir"].as_str(), c["nat"].as_str()), (Some("/t"), Some("extip:1.2.3.4")));
        assert!(node_config(base(), Path::new("/m/s"), Some("{")).unwrap_err().starts_with("PUKAAR_STORAGE_CFG"));
        assert!(node_config(json!("x"), Path::new("/m/s"), None).is_err());
    }

    #[test]
    fn ready_only_after_a_good_bring_up() {
        assert_eq!(status(), (false, None), "nothing tried yet");
        note_bring_up(&Err("storage start was refused (next try in 5 s)".into()));
        let (ready, error) = status();
        let mut s = json!({ "pending": 0, "error": null });
        merge_status(&mut s, ready, error);
        assert_eq!((s["ready"].as_bool(), s["error"].as_str()), (Some(false), Some("storage start was refused (next try in 5 s)")));
        note_bring_up(&Ok(()));
        let (ready, error) = status();
        let mut s = json!({ "pending": 2, "error": null });
        merge_status(&mut s, ready, error);
        assert_eq!((s["ready"].as_bool(), s["pending"].as_u64(), s["error"].is_null()), (Some(true), Some(2), true));
        let mut s = json!({ "pending": 1, "error": "storage: download failed" });
        merge_status(&mut s, false, Some("old".into()));
        assert_eq!(s["error"], "storage: download failed");
    }

    #[test]
    fn wait_grows_to_one_minute() {
        let w: Vec<u64> = std::iter::successors(Some(next_wait(None)), |w| Some(next_wait(Some(*w)))).take(6).map(|d| d.as_secs()).collect();
        assert_eq!(w, [5, 10, 20, 40, 60, 60]);
    }

    #[test]
    fn keep_file_restores_or_removes() {
        let dir = std::env::temp_dir().join(format!("pukaar-keep-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("config.json");
        keep_file(&f, || std::fs::write(&f, "ours").unwrap());
        assert!(!f.exists(), "no file before: no file after");
        std::fs::write(&f, "theirs").unwrap();
        keep_file(&f, || std::fs::write(&f, "ours").unwrap());
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "theirs");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
