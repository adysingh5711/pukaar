//! Private 1:1 chat (chat-module.md). `pukaar_core` calls `chat_module` 0.3.0 through the raw SDK,
//! the same way `storage.rs` calls Storage. The module needs `delivery_module` and a key service
//! (`devnet.chat-kc.logos.co`): `init` publishes the new installation to it, so `init` can take a
//! few seconds and fails when the key service is down.

use crate::storage::call;
use logos_rust_sdk::LogosModuleSDK;
use serde_json::{json, Value};
use std::time::Duration;

const MODULE: &str = "chat_module";
/// Pukaar's Delivery preset. The first module that creates the shared node decides its settings
/// (chat defaults to `logos.test`), so chat must ask for the same one.
const PRESET: &str = "logos.dev";
/// `init` does one HTTPS round trip to the key service.
const CALL: Duration = Duration::from_secs(20);

/// Start the chat module and read its address. `init` answers at once while the Delivery node
/// comes up (`delivery` is `online` later), and a second `init` while running is a no-op, so the
/// UI may call this on every poll until `delivery` is `online`.
pub fn start() -> Result<Value, String> {
    let p = LogosModuleSDK::new().plugin(MODULE);
    call(&p, "init", json!([{ "delivery_preset": PRESET }]), CALL)?;
    let address = call(&p, "get_address", json!([]), CALL)?;
    let address = address
        .as_str()
        .filter(|a| !a.is_empty())
        .ok_or("chat_module has no address yet")?;
    let status = call(&p, "status", json!([]), CALL)?;
    Ok(json!({
        "address": address,
        "delivery": status["delivery_state"],
        "detail": status["detail"],
        "adopted": status["delivery_adopted"],
        "preset": PRESET,
    }))
}
