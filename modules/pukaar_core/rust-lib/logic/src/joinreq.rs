//! The join request: 162-byte body (`pukaar_sites_rules::JoinBody`), then the ed25519 signature of
//! the body (64) and the BIP340 signature of sha256(body) (64), 290 bytes in all. The layout is
//! the rules crate's. This file adds the signing and the check of both signatures.
//! `user_account` is the one LEZ account of the user, the same in every site.

use crate::lezkey::{lez_account_id, lez_public_key, schnorr_sign};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use k256::schnorr;
use pukaar_sites_rules::{request_id, JoinBody, JOIN_BODY_LEN, JOIN_REQ_LEN};
use sha2::{Digest, Sha256};

/// Sign a join request for `site_id` with the Pukaar key and the LEZ secret of the user.
pub fn build(ed: &SigningKey, lez_sk: &[u8; 32], site_id: [u8; 32], nonce: u64, valid_until_ms: u64) -> [u8; JOIN_REQ_LEN] {
    let body = JoinBody {
        site_id,
        user_account: lez_account_id(lez_sk),
        user_schnorr_pk: lez_public_key(lez_sk),
        pukaar_ed_pk: ed.verifying_key().to_bytes(),
        nonce,
        valid_until_ms,
    }
    .encode();
    let sig_schnorr = schnorr_sign(lez_sk, &Sha256::digest(body).into());
    [&body[..], &ed.sign(&body).to_bytes(), &sig_schnorr].concat().try_into().expect("290 bytes")
}

/// Parse a request and check the account binding and both signatures. It does not check the site,
/// the nonce or the time: `pukaar_sites_rules::check_join_request` does. Returns the body and the
/// request id (the 32 bytes that the chain keeps as `Member.request_id`).
pub fn parse(req: &[u8]) -> Result<(JoinBody, [u8; 32]), String> {
    let req: &[u8; JOIN_REQ_LEN] = req.try_into().map_err(|_| format!("a join request has {JOIN_REQ_LEN} bytes"))?;
    let body_bytes: &[u8; JOIN_BODY_LEN] = req[..JOIN_BODY_LEN].try_into().expect("162 bytes");
    let body = JoinBody::parse(body_bytes).ok_or("bad join request domain")?;
    if pukaar_sites_rules::lez_account_id(&body.user_schnorr_pk) != body.user_account {
        return Err("user_account does not belong to the LEZ key".into());
    }
    let ed = VerifyingKey::from_bytes(&body.pukaar_ed_pk).map_err(|_| "bad Pukaar key")?;
    let sig_ed = ed25519_dalek::Signature::from_bytes(req[JOIN_BODY_LEN..JOIN_BODY_LEN + 64].try_into().expect("64 bytes"));
    ed.verify_strict(body_bytes, &sig_ed).map_err(|_| "bad Pukaar signature")?;
    let vk = schnorr::VerifyingKey::from_bytes(&body.user_schnorr_pk).map_err(|_| "bad LEZ key")?;
    let sig = schnorr::Signature::try_from(&req[JOIN_BODY_LEN + 64..]).map_err(|_| "bad LEZ signature bytes")?;
    k256::schnorr::signature::hazmat::PrehashVerifier::verify_prehash(&vk, &Sha256::digest(body_bytes), &sig)
        .map_err(|_| "bad LEZ signature")?;
    Ok((body, request_id(req)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lezkey::derive_lez_secret;
    use pukaar_sites_rules::check_join_request;

    fn keys() -> (SigningKey, [u8; 32]) {
        (SigningKey::from_bytes(&[3; 32]), derive_lez_secret(&[3; 32]))
    }

    #[test]
    fn sizes_are_162_and_290() {
        assert_eq!((JOIN_BODY_LEN, JOIN_REQ_LEN), (162, 290));
        // 18 domain + 4 * 32 (site, user account, LEZ key, Pukaar key) + 8 nonce + 8 valid_until.
        assert_eq!(18 + 4 * 32 + 8 + 8, 162);
        assert_eq!(162 + 64 + 64, 290);
        let (ed, lez) = keys();
        assert_eq!(build(&ed, &lez, [9; 32], 1, 2).len(), 290);
    }

    #[test]
    fn built_request_parses_and_passes_the_rules() {
        let (ed, lez) = keys();
        let site_id = [9; 32];
        let req = build(&ed, &lez, site_id, 1_000, 5_000);
        let (body, id) = parse(&req).unwrap();
        assert_eq!(body.user_account, lez_account_id(&lez));
        assert_eq!(body.pukaar_ed_pk, ed.verifying_key().to_bytes());
        assert_eq!((body.nonce, body.valid_until_ms), (1_000, 5_000));
        assert_eq!(id, request_id(&req));
        assert_eq!(check_join_request(&req, &site_id, None, 2_000).unwrap(), body);
    }

    #[test]
    fn tampering_and_wrong_keys_are_refused() {
        let (ed, lez) = keys();
        let req = build(&ed, &lez, [9; 32], 1, 2);
        assert!(parse(&req[..289]).is_err());
        for i in [0, 20, 60, 100, 140, 150, 160, 170, 225, 230, 289] {
            let mut t = req;
            t[i] ^= 1;
            assert!(parse(&t).is_err(), "byte {i}");
        }
        // The LEZ key of another user, with the right body bytes: the account binding fails.
        let mut other = build(&ed, &derive_lez_secret(&[4; 32]), [9; 32], 1, 2);
        other[18 + 32..18 + 64].copy_from_slice(&lez_account_id(&lez));
        assert!(parse(&other).is_err());
    }
}
