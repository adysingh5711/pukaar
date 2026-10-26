//! The member profile file and the site keyring (port-plan "Cross-cutting work", Encryption).
//!
//! `profile.json` holds {display_name, bio, experience} encrypted with XChaCha20-Poly1305 under a
//! random content key. The author signs the header with ed25519. The content key is wrapped in
//! `wraps`, which the signature does not cover: a symmetric wrap under the site key of `key_epoch`
//! (`to` = "site"), or a sealed box to the X25519 key of one admin (`to` = hex Pukaar key).
//! `keyring.json` holds the site key of the newest epoch sealed to each member, and every older
//! key encrypted under the newest. A revoke makes a new epoch and leaves the removed member out.
//! The chain keeps the CID and the SHA-256 of the whole file: the caller checks that hash.

use crate::lezkey::{open as unseal, seal, x25519_public};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const SCHEMA: &str = "pukaar.member.profile/2";
pub const KEYRING_SCHEMA: &str = "pukaar.keyring/1";
const SITE: &str = "site";

pub type Key = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Body {
    pub display_name: String,
    pub bio: String,
    pub experience: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wrap {
    pub to: String,
    pub ct: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub schema: String,
    pub site_id: String,
    /// The LEZ account of the author.
    pub user: String,
    pub pukaar_key: String,
    pub version: u64,
    pub key_epoch: u64,
    pub nonce: String,
    pub ciphertext: String,
    pub sig_ed: String,
    pub wraps: Vec<Wrap>,
}

fn rand<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    getrandom::getrandom(&mut b).expect("os rng");
    b
}

fn unhex<const N: usize>(s: &str) -> Result<[u8; N], String> {
    hex::decode(s).ok().and_then(|b| b.try_into().ok()).ok_or_else(|| format!("bad {N}-byte hex field"))
}

/// nonce (24) ‖ ciphertext.
fn encrypt(key: &Key, plain: &[u8]) -> Vec<u8> {
    let nonce = rand::<24>();
    let ct = XChaCha20Poly1305::new(key.into()).encrypt(XNonce::from_slice(&nonce), plain).expect("encrypt");
    [&nonce[..], &ct].concat()
}

fn decrypt(key: &Key, boxed: &[u8]) -> Result<Vec<u8>, String> {
    let (nonce, ct) = boxed.split_at_checked(24).ok_or("box too short")?;
    XChaCha20Poly1305::new(key.into()).decrypt(XNonce::from_slice(nonce), ct).map_err(|_| "wrong key or damaged data".to_string())
}

/// What the author signs: the schema as domain, then the fixed fields. Not `wraps`.
fn sig_message(e: &Envelope) -> Result<Vec<u8>, String> {
    let ct = hex::decode(&e.ciphertext).map_err(|_| "bad ciphertext hex")?;
    Ok([
        SCHEMA.as_bytes(),
        &[0],
        &unhex::<32>(&e.site_id)?,
        &unhex::<32>(&e.user)?,
        &e.version.to_be_bytes(),
        &e.key_epoch.to_be_bytes(),
        &unhex::<24>(&e.nonce)?,
        &Sha256::digest(ct),
    ]
    .concat())
}

pub fn build(
    author: &SigningKey,
    site_id: &Key,
    user: &Key,
    version: u64,
    key_epoch: u64,
    body: &Body,
    site_key: Option<&Key>,
    admins: &[Key],
) -> Result<Envelope, String> {
    let content_key = rand::<32>();
    let mut wraps = Vec::new();
    if let Some(k) = site_key {
        wraps.push(Wrap { to: SITE.into(), ct: hex::encode(encrypt(k, &content_key)) });
    }
    for a in admins {
        let vk = VerifyingKey::from_bytes(a).map_err(|_| "bad admin key")?;
        wraps.push(Wrap { to: hex::encode(a), ct: hex::encode(seal(&x25519_public(&vk), &content_key)) });
    }
    if wraps.is_empty() {
        return Err("no reader: give a site key or an admin".into());
    }
    let boxed = encrypt(&content_key, &serde_json::to_vec(body).expect("json"));
    let mut e = Envelope {
        schema: SCHEMA.into(),
        site_id: hex::encode(site_id),
        user: hex::encode(user),
        pukaar_key: hex::encode(author.verifying_key().to_bytes()),
        version,
        key_epoch,
        // The body nonce is the first 24 bytes of `boxed`, kept apart so the signature can cover it.
        nonce: hex::encode(&boxed[..24]),
        ciphertext: hex::encode(&boxed[24..]),
        sig_ed: String::new(),
        wraps,
    };
    e.sig_ed = hex::encode(author.sign(&sig_message(&e)?).to_bytes());
    Ok(e)
}

pub fn to_bytes(e: &Envelope) -> Vec<u8> {
    serde_json::to_vec(e).expect("json")
}

/// The version rule of the file: a reader accepts only the schema it knows.
pub fn from_bytes(b: &[u8]) -> Result<Envelope, String> {
    let e: Envelope = serde_json::from_slice(b).map_err(|e| format!("bad profile: {e}"))?;
    if e.schema != SCHEMA {
        return Err(format!("unsupported profile schema {}", e.schema));
    }
    Ok(e)
}

/// The author signature. Anyone can check it, no key needed.
pub fn verify(e: &Envelope) -> Result<(), String> {
    if e.schema != SCHEMA {
        return Err(format!("unsupported profile schema {}", e.schema));
    }
    let vk = VerifyingKey::from_bytes(&unhex(&e.pukaar_key)?).map_err(|_| "bad author key")?;
    let sig = Signature::from_bytes(&unhex(&e.sig_ed)?);
    vk.verify_strict(&sig_message(e)?, &sig).map_err(|_| "bad profile signature".to_string())
}

/// The newest valid profile of one author in one site. The version rule: the highest version with a
/// valid signature wins, from the chain pointer or from Delivery alike.
pub fn newest<'a>(all: &'a [Envelope], site_id: &Key, user: &Key, author: &Key) -> Option<&'a Envelope> {
    let (s, u, a) = (hex::encode(site_id), hex::encode(user), hex::encode(author));
    all.iter().filter(|e| e.site_id == s && e.user == u && e.pukaar_key == a && verify(e).is_ok()).max_by_key(|e| e.version)
}

fn content_key(e: &Envelope, me: &SigningKey, site_keys: &BTreeMap<u64, Key>) -> Result<Key, String> {
    let mine = hex::encode(me.verifying_key().to_bytes());
    let key = if let Some(w) = e.wraps.iter().find(|w| w.to == mine) {
        unseal(me, &hex::decode(&w.ct).map_err(|_| "bad wrap hex")?).ok_or("admin wrap does not open")?
    } else {
        let w = e.wraps.iter().find(|w| w.to == SITE).ok_or("no wrap for this reader")?;
        let sk = site_keys.get(&e.key_epoch).ok_or_else(|| format!("no site key for epoch {}", e.key_epoch))?;
        decrypt(sk, &hex::decode(&w.ct).map_err(|_| "bad wrap hex")?)?
    };
    key.try_into().map_err(|_| "content key has the wrong length".to_string())
}

/// Check the signature, then read the body with the admin wrap of `me` or the site key.
pub fn open(e: &Envelope, me: &SigningKey, site_keys: &BTreeMap<u64, Key>) -> Result<Body, String> {
    verify(e)?;
    let key = content_key(e, me, site_keys)?;
    let boxed = [hex::decode(&e.nonce).map_err(|_| "bad nonce hex")?, hex::decode(&e.ciphertext).map_err(|_| "bad ciphertext hex")?].concat();
    serde_json::from_slice(&decrypt(&key, &boxed)?).map_err(|e| format!("bad profile body: {e}"))
}

/// At approval: an admin opens the first profile and adds the wrap under the site key of its epoch.
/// The signature does not cover `wraps`, so it stays valid.
pub fn add_site_wrap(e: &mut Envelope, admin: &SigningKey, site_keys: &BTreeMap<u64, Key>) -> Result<(), String> {
    verify(e)?;
    let sk = *site_keys.get(&e.key_epoch).ok_or_else(|| format!("no site key for epoch {}", e.key_epoch))?;
    let ck = content_key(e, admin, site_keys)?;
    e.wraps.retain(|w| w.to != SITE);
    e.wraps.push(Wrap { to: SITE.into(), ct: hex::encode(encrypt(&sk, &ck)) });
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OldKey {
    pub epoch: u64,
    pub ct: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keyring {
    pub schema: String,
    pub site_id: String,
    /// The newest epoch. `members` hold its key.
    pub epoch: u64,
    pub members: Vec<Wrap>,
    pub old: Vec<OldKey>,
}

/// Make the next epoch (1 for a new site), add its random key to `keys` and seal it to `members`
/// (their Pukaar keys). Use it for `create_site` and for each revoke: pass only the members that stay.
pub fn rotate(site_id: &Key, keys: &mut BTreeMap<u64, Key>, members: &[Key]) -> Result<Keyring, String> {
    let epoch = keys.keys().next_back().map_or(1, |e| e + 1);
    keys.insert(epoch, rand());
    let newest = keys[&epoch];
    let mut wraps = Vec::new();
    for m in members {
        let vk = VerifyingKey::from_bytes(m).map_err(|_| "bad member key")?;
        wraps.push(Wrap { to: hex::encode(m), ct: hex::encode(seal(&x25519_public(&vk), &newest)) });
    }
    let old = keys.iter().filter(|(e, _)| **e < epoch).map(|(e, k)| OldKey { epoch: *e, ct: hex::encode(encrypt(&newest, k)) }).collect();
    Ok(Keyring { schema: KEYRING_SCHEMA.into(), site_id: hex::encode(site_id), epoch, members: wraps, old })
}

pub fn keyring_to_bytes(k: &Keyring) -> Vec<u8> {
    serde_json::to_vec(k).expect("json")
}

pub fn keyring_from_bytes(b: &[u8]) -> Result<Keyring, String> {
    let k: Keyring = serde_json::from_slice(b).map_err(|e| format!("bad keyring: {e}"))?;
    if k.schema != KEYRING_SCHEMA {
        return Err(format!("unsupported keyring schema {}", k.schema));
    }
    Ok(k)
}

/// Every site key this member can read, by epoch. A member who is not in `members` gets an error.
pub fn open_keyring(k: &Keyring, me: &SigningKey) -> Result<BTreeMap<u64, Key>, String> {
    let mine = hex::encode(me.verifying_key().to_bytes());
    let w = k.members.iter().find(|w| w.to == mine).ok_or("not in this keyring")?;
    let newest: Key = unseal(me, &hex::decode(&w.ct).map_err(|_| "bad wrap hex")?)
        .ok_or("keyring wrap does not open")?
        .try_into()
        .map_err(|_| "site key has the wrong length")?;
    let mut keys = BTreeMap::from([(k.epoch, newest)]);
    for o in &k.old {
        let key = decrypt(&newest, &hex::decode(&o.ct).map_err(|_| "bad old key hex")?)?;
        keys.insert(o.epoch, key.try_into().map_err(|_| "old site key has the wrong length")?);
    }
    Ok(keys)
}

/// The signed message that `Wire::ProfileAnnounce` carries: a member tells the site that a new profile
/// file is in Storage, before the chain pointer moves. Readers keep the highest version with a valid
/// signature (the version rule).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Announce {
    pub site: Key,
    /// The Pukaar key of the author.
    pub author: Key,
    pub version: u64,
    pub cid: String,
    /// SHA-256 of the profile file, the same hash the chain keeps.
    pub sha256: [u8; 32],
    /// ed25519 signature, 64 bytes. A `Vec` because serde has no impl for `[u8; 64]`.
    pub sig: Vec<u8>,
}

pub const ANNOUNCE_DOMAIN: &[u8] = b"pukaar.profile.announce/1\0";
pub const MAX_CID: usize = 128;

impl Announce {
    fn message(&self) -> Vec<u8> {
        [ANNOUNCE_DOMAIN, &self.site, &self.author, &self.version.to_be_bytes(), &self.sha256, self.cid.as_bytes()].concat()
    }

    pub fn sign(author: &SigningKey, site: Key, version: u64, cid: &str, sha256: [u8; 32]) -> Announce {
        let mut a = Announce {
            site,
            author: author.verifying_key().to_bytes(),
            version,
            cid: cid.into(),
            sha256,
            sig: Vec::new(),
        };
        a.sig = author.sign(&a.message()).to_bytes().to_vec();
        a
    }

    /// Sizes only: what a receiver checks before it spends a signature check.
    pub fn well_formed(&self) -> bool {
        !self.cid.is_empty() && self.cid.len() <= MAX_CID && self.sig.len() == 64
    }

    pub fn verify(&self) -> Result<(), String> {
        if !self.well_formed() {
            return Err("bad profile announce size".into());
        }
        let vk = VerifyingKey::from_bytes(&self.author).map_err(|_| "bad author key")?;
        let sig = Signature::from_slice(&self.sig).map_err(|_| "bad signature bytes")?;
        vk.verify_strict(&self.message(), &sig).map_err(|_| "bad profile announce signature".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sk(n: u8) -> SigningKey {
        SigningKey::from_bytes(&[n; 32])
    }
    fn pk(n: u8) -> Key {
        sk(n).verifying_key().to_bytes()
    }
    const SITE_ID: Key = [0xAA; 32];
    const USER: Key = [0xBB; 32];

    fn body() -> Body {
        Body { display_name: "Asha".into(), bio: "nurse".into(), experience: "10 years".into() }
    }

    /// Site with an admin (seed 1), members 2 and 3 in epoch 1.
    fn site() -> (BTreeMap<u64, Key>, Keyring) {
        let mut keys = BTreeMap::new();
        let ring = rotate(&SITE_ID, &mut keys, &[pk(1), pk(2), pk(3)]).unwrap();
        (keys, ring)
    }

    #[test]
    fn envelope_round_trip_for_site_key_and_admin_wrap() {
        let (keys, ring) = site();
        let e = build(&sk(2), &SITE_ID, &USER, 1, 1, &body(), Some(&keys[&1]), &[pk(1)]).unwrap();
        let e = from_bytes(&to_bytes(&e)).unwrap();
        verify(&e).unwrap();
        // Member 3 reads with the site key from the keyring, admin 1 with its own wrap.
        assert_eq!(open(&e, &sk(3), &open_keyring(&ring, &sk(3)).unwrap()).unwrap(), body());
        assert_eq!(open(&e, &sk(1), &BTreeMap::new()).unwrap(), body());
    }

    #[test]
    fn first_profile_is_read_by_the_admin_then_gets_a_site_wrap() {
        let (keys, _) = site();
        let mut e = build(&sk(4), &SITE_ID, &USER, 1, 1, &body(), None, &[pk(1)]).unwrap();
        assert!(open(&e, &sk(3), &keys).is_err(), "no site wrap yet");
        add_site_wrap(&mut e, &sk(1), &keys).unwrap();
        verify(&e).unwrap();
        assert_eq!(open(&e, &sk(3), &keys).unwrap(), body());
        assert!(build(&sk(4), &SITE_ID, &USER, 1, 1, &body(), None, &[]).is_err());
    }

    #[test]
    fn wrong_key_fails() {
        let (keys, _) = site();
        let e = build(&sk(2), &SITE_ID, &USER, 1, 1, &body(), Some(&keys[&1]), &[pk(1)]).unwrap();
        assert!(open(&e, &sk(9), &BTreeMap::new()).is_err(), "a stranger has no key");
        let wrong = BTreeMap::from([(1, [0u8; 32])]);
        assert!(open(&e, &sk(3), &wrong).is_err(), "wrong site key");
        assert!(open(&e, &sk(3), &BTreeMap::new()).is_err(), "no site key");
    }

    #[test]
    fn revoked_member_cannot_open_a_post_revoke_profile() {
        let (mut keys, ring1) = site();
        // Member 3 is revoked: epoch 2 is sealed to the others only.
        let ring2 = rotate(&SITE_ID, &mut keys, &[pk(1), pk(2)]).unwrap();
        assert_eq!(ring2.epoch, 2);
        assert!(open_keyring(&ring2, &sk(3)).is_err());
        let old3 = open_keyring(&ring1, &sk(3)).unwrap();
        assert_eq!(old3.keys().collect::<Vec<_>>(), [&1]);
        let e = build(&sk(2), &SITE_ID, &USER, 2, 2, &body(), Some(&keys[&2]), &[]).unwrap();
        assert!(open(&e, &sk(3), &old3).is_err(), "revoked member holds only epoch 1");
        // Member 2 reads it, and still reads an epoch 1 profile through the old keys of the new ring.
        let k2 = open_keyring(&ring2, &sk(2)).unwrap();
        assert_eq!(open(&e, &sk(2), &k2).unwrap(), body());
        let before = build(&sk(1), &SITE_ID, &USER, 1, 1, &body(), Some(&keys[&1]), &[]).unwrap();
        assert_eq!(open(&before, &sk(2), &k2).unwrap(), body());
        assert_eq!(k2, keys);
    }

    #[test]
    fn tampered_data_fails() {
        let (keys, _) = site();
        let e = build(&sk(2), &SITE_ID, &USER, 1, 1, &body(), Some(&keys[&1]), &[]).unwrap();
        let flip = |s: &str| format!("{}{}", &s[..s.len() - 2], if s.ends_with("00") { "01" } else { "00" });
        let mut t = e.clone();
        t.ciphertext = flip(&t.ciphertext);
        assert!(verify(&t).is_err() && open(&t, &sk(3), &keys).is_err());
        let mut t = e.clone();
        t.version = 2;
        assert!(verify(&t).is_err());
        let mut t = e.clone();
        t.user = hex::encode([0xCC; 32]);
        assert!(verify(&t).is_err());
        let mut t = e.clone();
        t.pukaar_key = hex::encode(pk(7));
        assert!(verify(&t).is_err(), "a forged author");
        // Wraps are outside the signature: an admin may change them without breaking it.
        let mut t = e;
        t.wraps.clear();
        verify(&t).unwrap();
    }

    #[test]
    fn version_rule() {
        let (keys, _) = site();
        let mk = |v, a: &SigningKey| build(a, &SITE_ID, &USER, v, 1, &body(), Some(&keys[&1]), &[]).unwrap();
        let (v1, v3, v2) = (mk(1, &sk(2)), mk(3, &sk(2)), mk(2, &sk(2)));
        let mut forged = mk(9, &sk(5));
        forged.pukaar_key = hex::encode(pk(2));
        let mut bad_sig = mk(8, &sk(2));
        bad_sig.version = 8;
        bad_sig.sig_ed = v1.sig_ed.clone();
        let all = [v1, v3, v2, forged, bad_sig];
        assert_eq!(newest(&all, &SITE_ID, &USER, &pk(2)).unwrap().version, 3);
        assert!(newest(&all, &SITE_ID, &[1; 32], &pk(2)).is_none(), "other user");
        // An unknown schema is refused on read.
        let mut j: serde_json::Value = serde_json::from_slice(&to_bytes(&all[0])).unwrap();
        j["schema"] = "pukaar.member.profile/3".into();
        assert!(from_bytes(&serde_json::to_vec(&j).unwrap()).unwrap_err().contains("unsupported"));
        let mut k = serde_json::to_value(&site().1).unwrap();
        k["schema"] = "pukaar.keyring/2".into();
        assert!(keyring_from_bytes(&serde_json::to_vec(&k).unwrap()).is_err());
    }

    #[test]
    fn keyring_round_trip_and_epochs() {
        let (mut keys, ring) = site();
        let ring = keyring_from_bytes(&keyring_to_bytes(&ring)).unwrap();
        assert_eq!(open_keyring(&ring, &sk(2)).unwrap(), keys);
        assert!(open_keyring(&ring, &sk(8)).is_err());
        rotate(&SITE_ID, &mut keys, &[pk(1), pk(2)]).unwrap();
        let ring3 = rotate(&SITE_ID, &mut keys, &[pk(1)]).unwrap();
        assert_eq!((ring3.epoch, ring3.old.len()), (3, 2));
        assert_eq!(open_keyring(&ring3, &sk(1)).unwrap(), keys);
    }

    /// A file built once with this code. It fails if the layout, the signed message or the
    /// ciphers change. Keys: author seed [2;32], admin seed [1;32], user BB.., site AA.., epoch 1.
    const VECTOR: &str = include_str!("../tests/fixtures/profile_vector.json");

    #[test]
    fn fixed_test_vector() {
        let e = from_bytes(VECTOR.as_bytes()).unwrap();
        verify(&e).unwrap();
        assert_eq!(e.pukaar_key, hex::encode(pk(2)));
        assert_eq!((e.version, e.key_epoch), (1, 1));
        assert_eq!(open(&e, &sk(1), &BTreeMap::new()).unwrap(), body());
    }

    #[test]
    fn announce_signs_verifies_and_rejects_tampering() {
        let a = Announce::sign(&sk(1), [7; 32], 3, "zDvZRwzm", [9; 32]);
        a.verify().unwrap();
        for t in [
            Announce { version: 4, ..a.clone() },
            Announce { site: [8; 32], ..a.clone() },
            Announce { cid: "zOther".into(), ..a.clone() },
            Announce { sha256: [1; 32], ..a.clone() },
            Announce { author: pk(2), ..a.clone() },
            Announce { sig: a.sig[..63].to_vec(), ..a.clone() },
            Announce { cid: "x".repeat(MAX_CID + 1), ..a.clone() },
        ] {
            assert!(t.verify().is_err());
        }
    }
}
