//! The LEZ key and the profile encryption primitives (port-plan "Key handling", "Encryption").
//!
//! - LEZ key: secp256k1 secret from HKDF-SHA256(ikm = Pukaar seed, no salt, info = INFO).
//! - Account id: sha256(ACCOUNT_ID_PREFIX ‖ x-only public key), as LEZ v0.3.1
//!   `impl From<&PublicKey> for AccountId` (the prefix is padded to 32 bytes).
//! - Signature: BIP340 over a 32-byte message used as is, as LEZ `Signature::new`.
//! - Sealed box: libsodium `crypto_box_seal` to the X25519 form of the Pukaar ed25519 key.

use crypto_box::aead::OsRng;
use ed25519_dalek::{SigningKey, VerifyingKey};
use hkdf::Hkdf;
use k256::schnorr;
use sha2::{Digest, Sha256};

const INFO: &[u8] = b"pukaar/lez-key/v1";
const ACCOUNT_ID_PREFIX: &[u8; 32] = b"/LEE/v0.3/AccountId/Public/\0\0\0\0\0";

/// The LEZ secret key of a Pukaar seed (the 32-byte ed25519 secret key). If the bytes are 0 or
/// not below the group order (chance about 2^-128), derive again with info ‖ counter (1, 2, ...).
pub fn derive_lez_secret(seed: &[u8; 32]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, seed);
    let mut info = INFO.to_vec();
    let mut counter = 0u8;
    loop {
        let mut out = [0u8; 32];
        hk.expand(&info, &mut out).expect("32 bytes fit HKDF-SHA256");
        if k256::SecretKey::from_bytes(&out.into()).is_ok() {
            return out;
        }
        counter = counter.checked_add(1).expect("255 invalid keys in a row");
        info.truncate(INFO.len());
        info.push(counter);
    }
}

fn schnorr_key(sk: &[u8; 32]) -> schnorr::SigningKey {
    schnorr::SigningKey::from_bytes(sk).expect("a key from derive_lez_secret")
}

/// The x-only public key (BIP340), the form LEZ `PublicKey::new_from_private_key` keeps.
pub fn lez_public_key(sk: &[u8; 32]) -> [u8; 32] {
    schnorr_key(sk).verifying_key().to_bytes().into()
}

/// The public account id; the wallet prints it as `Public/<base58 of these bytes>`.
pub fn lez_account_id(sk: &[u8; 32]) -> [u8; 32] {
    Sha256::new()
        .chain_update(ACCOUNT_ID_PREFIX)
        .chain_update(lez_public_key(sk))
        .finalize()
        .into()
}

/// BIP340 signature of a 32-byte hash, with fresh aux randomness, as LEZ `Signature::new`.
pub fn schnorr_sign(sk: &[u8; 32], hash: &[u8; 32]) -> [u8; 64] {
    let mut aux = [0u8; 32];
    getrandom::getrandom(&mut aux).expect("os rng");
    sign_with_aux(sk, hash, &aux)
}

fn sign_with_aux(sk: &[u8; 32], hash: &[u8; 32], aux: &[u8; 32]) -> [u8; 64] {
    schnorr_key(sk)
        .sign_prehash_with_aux_rand(hash, aux)
        .expect("BIP340 signing")
        .to_bytes()
}

/// The X25519 public key of a Pukaar key (libsodium `crypto_sign_ed25519_pk_to_curve25519`).
pub fn x25519_public(key: &VerifyingKey) -> [u8; 32] {
    key.to_montgomery().to_bytes()
}

/// The X25519 secret: the clamped first half of SHA-512(seed), as libsodium
/// `crypto_sign_ed25519_sk_to_curve25519` (crypto_box clamps the bytes).
fn x25519_secret(key: &SigningKey) -> crypto_box::SecretKey {
    crypto_box::SecretKey::from_bytes(key.to_scalar_bytes())
}

/// Sealed box (ephemeral X25519, XSalsa20-Poly1305, nonce = BLAKE2b(epk ‖ pk)): 48 bytes longer.
pub fn seal(recipient_x25519: &[u8; 32], plaintext: &[u8]) -> Vec<u8> {
    crypto_box::PublicKey::from_bytes(*recipient_x25519)
        .seal(&mut OsRng, plaintext)
        .expect("seal")
}

/// Open a sealed box with the Pukaar key of the recipient; `None` for a wrong key or a bad box.
pub fn open(recipient: &SigningKey, sealed: &[u8]) -> Option<Vec<u8>> {
    x25519_secret(recipient).unseal(sealed).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chacha20poly1305::aead::{Aead, KeyInit};
    use chacha20poly1305::{XChaCha20Poly1305, XNonce};
    use k256::schnorr::signature::hazmat::PrehashVerifier;

    fn h<const N: usize>(s: &str) -> [u8; N] {
        hex::decode(s).unwrap().try_into().unwrap()
    }

    // Fixed TEST seed 00 01 .. 1f, never a real key. lez_sk from Python stdlib HMAC (HKDF by
    // hand); the account id is what `wallet account import public` (LEZ v0.3.1) printed for it.
    const SEED: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    const LEZ_SK: &str = "6f4cd00b05182f2061911d286ea71b947d880c0f3ed793456962b40316394f23";
    const ACCOUNT_B58: &str = "AVi7Fm67WkAqJwLVD7PeA7KisBNuA2o6ew1CxmdtGrGP";

    #[test]
    fn lez_key_and_account_id_match_the_wallet() {
        let sk = derive_lez_secret(&h(SEED));
        assert_eq!(hex::encode(sk), LEZ_SK);
        let id = lez_account_id(&sk);
        assert_eq!(bs58::encode(id).into_string(), ACCOUNT_B58);
        assert_eq!(
            hex::encode(id),
            "8d13ecff3bf91d2ad05a8f68aa6be75d63dd1fc0008db38f20f7965ff2c383e8"
        );
    }

    #[test]
    fn bip340_vectors_sign_and_verify() {
        // bitcoin/bips bip-0340/test-vectors.csv rows 0..=2 (also in LEZ bip340_test_vectors.rs).
        let rows = [
            ("0000000000000000000000000000000000000000000000000000000000000003",
             "F9308A019258C31049344F85F89D5229B531C845836F99B08601F113BCE036F9",
             "0000000000000000000000000000000000000000000000000000000000000000",
             "0000000000000000000000000000000000000000000000000000000000000000",
             "E907831F80848D1069A5371B402410364BDF1C5F8307B0084C55F1CE2DCA821525F66A4A85EA8B71E482A74F382D2CE5EBEEE8FDB2172F477DF4900D310536C0"),
            ("B7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF",
             "DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
             "0000000000000000000000000000000000000000000000000000000000000001",
             "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
             "6896BD60EEAE296DB48A229FF71DFE071BDE413E6D43F917DC8DCF8C78DE33418906D11AC976ABCCB20B091292BFF4EA897EFCB639EA871CFA95F6DE339E4B0A"),
            ("C90FDAA22168C234C4C6628B80DC1CD129024E088A67CC74020BBEA63B14E5C9",
             "DD308AFEC5777E13121FA72B9CC1B7CC0139715309B086C960E18FD969774EB8",
             "C87AA53824B4D7AE2EB035A2B5BBBCCC080E76CDC6D1692C4B0B62D798E6D906",
             "7E2D58D8B3BCDF1ABADEC7829054F90DDA9805AAB56C77333024B9D0A508B75C",
             "5831AAEED7B44BB74E5EAB94BA9D4294C49BCF2A60728D8B4C200F50DD313C1BAB745879A5AD954A72C45A91C3A51D3C7ADEA98D82F8481E0E1E03674A6F3FB7"),
        ];
        for (sk, pk, aux, msg, sig) in rows {
            let (sk, pk) = (h::<32>(sk), h::<32>(pk));
            assert_eq!(lez_public_key(&sk), pk);
            assert_eq!(sign_with_aux(&sk, &h(msg), &h(aux)), h::<64>(sig));
        }
    }

    #[test]
    fn signature_verifies_as_lez_does() {
        // LEZ `Signature::is_valid_for`: VerifyingKey::from_bytes(x-only) + verify_prehash.
        let sk = derive_lez_secret(&h(SEED));
        let msg = Sha256::digest(b"join request").into();
        let sig = schnorr_sign(&sk, &msg);
        let vk = schnorr::VerifyingKey::from_bytes(&lez_public_key(&sk)).unwrap();
        let ok = |m: &[u8; 32], s: &[u8; 64]| {
            vk.verify_prehash(m, &schnorr::Signature::try_from(&s[..]).unwrap()).is_ok()
        };
        assert!(ok(&msg, &sig));
        assert!(!ok(&[0; 32], &sig));
        assert_ne!(sig, schnorr_sign(&sk, &msg), "aux randomness must be fresh");
    }

    #[test]
    fn x25519_keys_match_libsodium() {
        // From libsodium crypto_sign_seed_keypair + crypto_sign_ed25519_{pk,sk}_to_curve25519.
        let key = SigningKey::from_bytes(&h(SEED));
        assert_eq!(
            hex::encode(key.verifying_key().to_bytes()),
            "03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8"
        );
        let xpk = x25519_public(&key.verifying_key());
        assert_eq!(
            hex::encode(xpk),
            "4701d08488451f545a409fb58ae3e58581ca40ac3f7f114698cd71deac73ca01"
        );
        // crypto_box keeps the unclamped bytes; libsodium returns them clamped.
        let mut xsk = x25519_secret(&key).to_bytes();
        (xsk[0], xsk[31]) = (xsk[0] & 248, xsk[31] & 127 | 64);
        assert_eq!(
            hex::encode(xsk),
            "3894eea49c580aef816935762be049559d6d1440dede12e6a125f1841fff8e6f"
        );
        assert_eq!(x25519_secret(&key).public_key().to_bytes(), xpk);
    }

    #[test]
    fn sealed_box_round_trip_wrong_key_and_libsodium() {
        let key = SigningKey::from_bytes(&h(SEED));
        let site_key = [7u8; 32];
        let sealed = seal(&x25519_public(&key.verifying_key()), &site_key);
        assert_eq!(sealed.len(), 32 + 48);
        assert_eq!(open(&key, &sealed).unwrap(), site_key);
        assert!(open(&SigningKey::from_bytes(&[9; 32]), &sealed).is_none());
        let mut bad = sealed.clone();
        bad[40] ^= 1;
        assert!(open(&key, &bad).is_none());
        // Sealed by libsodium crypto_box_seal to the same X25519 key.
        let libsodium: Vec<u8> = hex::decode("161d63134e54c5328f6bb9561fa6a89922458c0b2198ee2def46aa18bc8c921a5a853da85cc277b27836c07ca1d226281eff1094b39441e69a57653ea98e80ee9500972ae932b756b4e2d481ecef8ef1").unwrap();
        assert_eq!(open(&key, &libsodium).unwrap(), site_key);
    }

    #[test]
    fn xchacha_profile_body_matches_libsodium() {
        // libsodium crypto_aead_xchacha20poly1305_ietf_encrypt, key 01*32, nonce 02*24, no AD.
        let body = br#"{"display_name":"Asha","bio":"nurse"}"#;
        let c = XChaCha20Poly1305::new(&[1u8; 32].into());
        let n = XNonce::from_slice(&[2u8; 24]);
        let ct = c.encrypt(n, &body[..]).unwrap();
        assert_eq!(hex::encode(&ct), "b7445286d907902cd93e0ff01f355ba4ef9482c054c40735220ed34c4bf391ede20ba0f31b6ad5c377535bbbc08d1037b841ee1749");
        assert_eq!(c.decrypt(n, &ct[..]).unwrap(), body);
    }
}
