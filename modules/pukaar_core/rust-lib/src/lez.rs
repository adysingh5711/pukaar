//! The real `ChainWriter`: `pukaar_core` calls `lez_core` (Basecamp, 0.5.1) through the raw SDK, the
//! same way `storage.rs` calls `storage_module`. The `ChainWriter` trait, the queue and the builders of
//! the instruction bytes live in `logic`, so the tests run against a fake wallet. This file is the only
//! code that talks to `lez_core`.
//!
//! Rules:
//! - Only the worker thread of `lib.rs` calls it. `poll_transaction_status` blocks for the
//!   `seq_poll_timeout` of `lez_core` (12 s by default) and the UI gives each call 20 seconds (P0:
//!   that was the "lost reply"). So the poll gets a call timeout of 40 s.
//! - `lez_core` 0.5.1 has no `import_public_account` (the local branch `import-public-account` adds it).
//!   A call that fails as a call (the method does not exist) is reported as "key import not
//!   available" and every write fails with that reason. It never crashes.
//! - `import_public_account` answers `{success, account_id, error, code}`. Code 100 means that the
//!   wallet holds the key already, which is success. Code 16 is a bad key.
//! - The secret key is never logged and never put in an error text.
//! - Assumption, not tested against a real `lez_core` (no write in this step): the instruction bytes
//!   cross as the tagged `{"_bytes": ...}` value of the SDK, as the Delivery calls send their payload.

use logos_rust_sdk::{LogosModuleSDK, PluginProxy};
use pukaar_logic::queue::{ChainWriter, Poll};
use pukaar_logic::writes::Tx;
use serde_json::{json, Value};
use std::sync::Mutex;
use std::time::Duration;

const MODULE: &str = "lez_core";
const CALL: Duration = Duration::from_secs(20);
/// Signing and sending can take a few seconds.
const SEND: Duration = Duration::from_secs(30);
/// Above the 12 s `seq_poll_timeout` of `lez_core` (P0 result), so a reply is never lost.
const POLL: Duration = Duration::from_secs(40);
/// `import_public_account` code: the wallet holds the key already.
const ALREADY_IMPORTED: i64 = 100;

pub struct LezWriter {
    proxy: Mutex<PluginProxy>,
}

/// A C++ method that returns a `std::string` of JSON arrives as a string. Open it.
fn reply(v: Value) -> Value {
    match v {
        Value::String(s) => serde_json::from_str(&s).unwrap_or(Value::String(s)),
        v => v,
    }
}

impl LezWriter {
    pub fn new() -> LezWriter {
        LezWriter { proxy: Mutex::new(LogosModuleSDK::new().plugin(MODULE)) }
    }

    fn call(&self, method: &str, args: Value, limit: Duration) -> Result<Value, String> {
        let p = self.proxy.lock().unwrap();
        p.call_json_with_timeout(method, &args, limit).map(reply).map_err(|e| format!("{method}: {e}"))
    }
}

impl ChainWriter for LezWriter {
    fn import_key(&self, secret_hex: &str) -> Result<(), String> {
        let v = self
            .call("import_public_account", json!([secret_hex]), CALL)
            .map_err(|e| format!("key import not available: {}", e.replace(secret_hex, "<key>")))?;
        let code = v["code"].as_i64();
        if v["success"] == true || code == Some(ALREADY_IMPORTED) {
            return Ok(());
        }
        let why = v["error"].as_str().unwrap_or("refused");
        Err(format!("key import failed: {why} (code {})", code.map_or("none".into(), |c| c.to_string())))
    }

    fn send(&self, tx: &Tx) -> Result<String, String> {
        let ids: Vec<String> = tx.accounts.iter().map(|m| hex::encode(m.id)).collect();
        let signs: Vec<bool> = tx.accounts.iter().map(|m| m.sign).collect();
        let shards: Vec<String> = tx.accounts.iter().map(|m| hex::encode(m.shard)).collect();
        let args = json!([ids, signs, logos_rust_sdk::bytes::encode(&tx.ix), hex::encode(tx.program), hex::encode(tx.payer), shards]);
        let v = self.call("send_generic_public_transaction", args, SEND)?;
        let hash = v["tx_hash"].as_str().unwrap_or("");
        if v["success"] == true && !hash.is_empty() {
            return Ok(hash.to_string());
        }
        Err(format!("send failed: {}", v["error"].as_str().filter(|e| !e.is_empty()).unwrap_or("the wallet refused it")))
    }

    fn poll(&self, tx_hash: &str) -> Poll {
        // `poll_transaction_status` answers whether the transaction is in a block. It cannot say that the
        // program refused it: the queue reads the account back for that. A failed call is "not yet".
        match self.call("poll_transaction_status", json!([tx_hash]), POLL) {
            Ok(Value::Bool(true)) => Poll::Included,
            Ok(Value::String(s)) if s == "true" => Poll::Included,
            Ok(Value::Object(o)) if o.get("result") == Some(&Value::Bool(true)) => Poll::Included,
            _ => Poll::Pending,
        }
    }
}
