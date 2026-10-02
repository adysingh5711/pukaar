//! Password-sealed identity backup: one copy-pasteable line,
//! `pukaar-id-1:<hex(salt 16 ‖ nonce 24 ‖ ciphertext)>`, where the plaintext is
//! secret key (32) ‖ site id (32). Argon2id (default params) derives the key and
//! XChaCha20-Poly1305 seals it, with the version prefix as associated data.

use crate::event::{Id, Key};
use crate::node::Node;
use crate::persist;
use argon2::Argon2;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use ed25519_dalek::SigningKey;
use std::path::Path;

const PREFIX: &str = "pukaar-id-";
const VERSION: &str = "pukaar-id-1:";
const SALT: usize = 16;
const NONCE: usize = 24;
const MIN_PASSWORD: usize = 8;

fn cipher(password: &str, salt: &[u8]) -> Result<XChaCha20Poly1305, String> {
    let mut k = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut k)
        .map_err(|e| format!("key derivation failed: {e}"))?;
    Ok(XChaCha20Poly1305::new(&k.into()))
}

fn random<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    getrandom::getrandom(&mut b).expect("os rng");
    b
}

pub fn export_identity(key: &SigningKey, site: &Id, password: &str) -> Result<String, String> {
    if password.chars().count() < MIN_PASSWORD {
        return Err(format!(
            "password must be at least {MIN_PASSWORD} characters"
        ));
    }
    let (salt, nonce) = (random::<SALT>(), random::<NONCE>());
    let mut plain = [0u8; 64];
    plain[..32].copy_from_slice(&key.to_bytes());
    plain[32..].copy_from_slice(site);
    let sealed = cipher(password, &salt)?
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: &plain,
                aad: VERSION.as_bytes(),
            },
        )
        .map_err(|_| "could not seal the identity".to_string())?;
    let mut raw = Vec::with_capacity(SALT + NONCE + sealed.len());
    raw.extend(salt);
    raw.extend(nonce);
    raw.extend(sealed);
    Ok(format!("{VERSION}{}", hex::encode(raw)))
}

pub fn import_identity(blob: &str, password: &str) -> Result<(SigningKey, Id), String> {
    let blob = blob.trim();
    let body = blob.strip_prefix(VERSION).ok_or_else(|| {
        if blob.starts_with(PREFIX) {
            "unsupported backup version".to_string()
        } else {
            "not a Pukaar identity backup".to_string()
        }
    })?;
    let raw = hex::decode(body).map_err(|_| "not a Pukaar identity backup")?;
    if raw.len() < SALT + NONCE {
        return Err("not a Pukaar identity backup".into());
    }
    let (salt, rest) = raw.split_at(SALT);
    let (nonce, sealed) = rest.split_at(NONCE);
    let plain = cipher(password, salt)?
        .decrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: sealed,
                aad: VERSION.as_bytes(),
            },
        )
        .map_err(|_| "wrong password or damaged backup".to_string())?;
    let (key, site): (Key, Id) = match (plain.get(..32), plain.get(32..)) {
        (Some(k), Some(s)) => (
            k.try_into().map_err(|_| "damaged backup")?,
            s.try_into().map_err(|_| "damaged backup")?,
        ),
        _ => return Err("damaged backup".into()),
    };
    Ok((SigningKey::from_bytes(&key), site))
}

/// Restore on a fresh device: refuses if `dir` already holds a site, otherwise writes the key
/// and site and returns an empty node that catches up through anti-entropy.
pub fn import_into(dir: &Path, blob: &str, password: &str) -> Result<Node, String> {
    if dir.join("site").exists() {
        return Err("this device already has an identity".into());
    }
    let (key, site) = import_identity(blob, password)?;
    let node = Node::join(key, site);
    persist::save(&node, dir).map_err(|e| e.to_string())?;
    Ok(node)
}
