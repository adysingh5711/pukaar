//! The real `ChainWriter`: the LEZ wallet crate (tag v0.3.1) runs inside this module. It signs and
//! sends the writes itself, so the module needs no `lez_core`. The `ChainWriter` trait, the queue and the
//! builders of the instruction bytes live in `logic`. This file is the only code that talks to the wallet.
//!
//! Rules:
//! - The key lives in memory only. The wallet is built with `new_init_storage` and `store_persistent_data`
//!   is never called, so no secret reaches the disk. `wallet_config.json` holds the node URL and the gas
//!   limit and nothing else. The wallet home is never `from_env` (that is `~/.lee`).
//! - The wallet is async. One runtime is kept in the writer and the worker thread of `lib.rs` calls
//!   `block_on`. That thread is not inside a runtime.
//! - Every call has a time limit. `poll` makes one request (`get_transaction`), not the wallet loop of
//!   `poll_transaction` (up to 48 s). The queue reads the account back to see a program refusal.
//! - A failed call costs about 4M lepta, so the gas limit is low (500000, the default is 2000000).

use lee::AccountId;
use pukaar_logic::queue::{ChainWriter, Poll};
use pukaar_logic::writes::Tx;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use wallet::config::{SequencerConnectionData, WalletConfigOverrides};
use wallet::{AccountIdentity, WalletCore};

const GAS_LIMIT: u64 = 500_000;
/// Signing and sending read nonces and fees and then send: a few requests.
const SEND: Duration = Duration::from_secs(30);
/// One request.
const POLL: Duration = Duration::from_secs(15);

pub struct WalletWriter {
    dir: PathBuf,
    rpc: String,
    rt: tokio::runtime::Runtime,
    wallet: Mutex<Option<WalletCore>>,
    last_error: Mutex<String>,
}

impl WalletWriter {
    /// `dir` holds `wallet_config.json` only. `rpc` is the URL of the sequencer.
    pub fn new(dir: PathBuf, rpc: String) -> WalletWriter {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
        WalletWriter { dir, rpc, rt, wallet: Mutex::new(None), last_error: Mutex::new(String::new()) }
    }

    /// The last error of a send or a poll, for the status text. Empty when there was none.
    pub fn last_error(&self) -> String {
        self.last_error.lock().unwrap().clone()
    }

    fn fail(&self, e: String) -> String {
        *self.last_error.lock().unwrap() = e.clone();
        e
    }

    fn run<T, E: std::fmt::Display>(&self, limit: Duration, f: impl Future<Output = Result<T, E>>) -> Result<T, String> {
        match self.rt.block_on(async { tokio::time::timeout(limit, f).await }) {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => Err(self.fail(format!("{e:#}"))),
            Err(_) => Err(self.fail("timed out".into())),
        }
    }

    fn build(&self) -> Result<WalletCore, String> {
        let overrides = WalletConfigOverrides {
            sequencer: Some(SequencerConnectionData {
                sequencer_addr: self.rpc.parse().map_err(|e| format!("bad node url: {e}"))?,
                basic_auth: None,
            }),
            gas_limit: Some(GAS_LIMIT),
            ..Default::default()
        };
        // The mnemonic of this throw-away wallet is not used: the only key is the one we import.
        WalletCore::new_init_storage(self.dir.join("wallet_config.json"), self.dir.join("storage.json"), Some(overrides), "")
            .map(|(w, _)| w)
            .map_err(|e| format!("wallet start failed: {e:#}"))
    }

    #[cfg(test)]
    fn has_key(&self, id: [u8; 32]) -> bool {
        let g = self.wallet.lock().unwrap();
        g.as_ref().is_some_and(|w| w.get_account_public_signing_key(AccountId::new(id)).is_some())
    }
}

impl ChainWriter for WalletWriter {
    fn import_key(&self, secret_hex: &str) -> Result<(), String> {
        let sk: [u8; 32] = hex::decode(secret_hex).ok().and_then(|b| b.try_into().ok()).ok_or("key import failed: the key is not 64 hex characters")?;
        let sk = lee::PrivateKey::try_new(sk).map_err(|_| "key import failed: not a valid secp256k1 key".to_string())?;
        let mut g = self.wallet.lock().unwrap();
        if g.is_none() {
            *g = Some(self.build()?);
        }
        // Importing the same key twice replaces the same entry.
        g.as_mut().unwrap().storage_mut().key_chain_mut().add_imported_public_account(sk);
        Ok(())
    }

    fn send(&self, tx: &Tx) -> Result<String, String> {
        let g = self.wallet.lock().unwrap();
        let w = g.as_ref().ok_or("the wallet has no key yet")?;
        let mentions = tx
            .accounts
            .iter()
            .map(|m| {
                let id = AccountId::new(m.id);
                let who = if m.sign { AccountIdentity::Public(id) } else { AccountIdentity::PublicNoSign(id) };
                who.select_program_shard(AccountId::new(m.shard))
            })
            .collect();
        let send = w.send_pub_tx_paid_by(mentions, tx.ix.clone(), AccountId::new(tx.program), Some(AccountId::new(tx.payer)));
        self.run(SEND, send).map(|h| h.to_string()).map_err(|e| format!("send failed: {e}"))
    }

    fn poll(&self, tx_hash: &str) -> Poll {
        let g = self.wallet.lock().unwrap();
        let (Some(w), Ok(hash)) = (g.as_ref(), tx_hash.parse()) else { return Poll::Pending };
        // Some = in a block, None = not yet. An error is "not yet" and is kept in `last_error`.
        match self.run(POLL, w.get_transaction(hash)) {
            Ok(Some(_)) => Poll::Included,
            _ => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pukaar_logic::lezkey::{derive_lez_secret, lez_account_id};
    use pukaar_logic::writes::Mention;
    use std::time::Instant;

    /// A fresh temp folder for `wallet_config.json`, removed by the test.
    fn temp() -> PathBuf {
        let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("pukaar-wallet-test-{}-{n}", std::process::id()))
    }

    // Fixed TEST seed 00 01 .. 1f, never a real key. The account is what the wallet printed for it.
    fn seed_key() -> ([u8; 32], String) {
        let seed: [u8; 32] = std::array::from_fn(|i| i as u8);
        let sk = derive_lez_secret(&seed);
        (sk, hex::encode(sk))
    }

    #[test]
    fn imported_key_gives_the_account_of_lezkey() {
        let dir = temp();
        // A closed port: nothing here talks to a node.
        let w = WalletWriter::new(dir.clone(), "http://127.0.0.1:1".into());
        let (sk, hex_sk) = seed_key();
        w.import_key(&hex_sk).unwrap();
        w.import_key(&hex_sk).unwrap();
        let id = lez_account_id(&sk);
        assert_eq!(bs58::encode(id).into_string(), "AVi7Fm67WkAqJwLVD7PeA7KisBNuA2o6ew1CxmdtGrGP");
        assert!(w.has_key(id), "the wallet holds no key for the account of lezkey");
        assert!(!w.has_key([7; 32]));
        assert!(!dir.join("storage.json").exists(), "the key must not reach the disk");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn an_unreachable_node_is_an_error_not_a_hang() {
        let dir = temp();
        let w = WalletWriter::new(dir.clone(), "http://127.0.0.1:1".into());
        let (sk, hex_sk) = seed_key();
        assert!(w.import_key("zz").is_err());
        assert!(w.send(&Tx { program: [1; 32], ix: vec![], accounts: vec![], payer: [2; 32] }).is_err(), "no key yet");
        w.import_key(&hex_sk).unwrap();
        let me = lez_account_id(&sk);
        let tx = Tx { program: [1; 32], ix: vec![0], accounts: vec![Mention { id: me, sign: true, shard: [1; 32] }], payer: me };
        let t = Instant::now();
        let err = w.send(&tx).unwrap_err();
        assert!(err.starts_with("send failed: ") && err.len() > 14, "{err}");
        assert!(matches!(w.poll(&"ab".repeat(32)), Poll::Pending));
        assert!(!w.last_error().is_empty());
        assert!(t.elapsed() < SEND + POLL, "took {:?}", t.elapsed());
        std::fs::remove_dir_all(&dir).ok();
    }
}
