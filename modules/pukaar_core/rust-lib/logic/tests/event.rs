use ed25519_dalek::{Signer, SigningKey};
use pukaar_logic::event::*;
fn report(key: &SigningKey, text: &str) -> Unsigned {
    Unsigned {
        v: VERSION,
        site: [7; 32],
        author: key.verifying_key().to_bytes(),
        seq: 0,
        prev: ZERO,
        lamport: 1,
        ts: 0,
        chain_block: 1234,
        author_account: [9; 32],
        body: Body::Report {
            category: "water".into(),
            location: "W-03".into(),
            landmark: String::new(),
            text: text.into(),
        },
    }
}

#[test]
fn round_trip() {
    let k = new_key();
    let e = sign(&k, report(&k, "tap leaking"));
    assert_eq!(decode(&e.bytes).unwrap(), e);
}

#[test]
fn tampered_byte_fails_signature() {
    let k = new_key();
    let mut b = sign(&k, report(&k, "tap leaking")).bytes;
    let last = b.len() - 1;
    b[last] ^= 1;
    assert_eq!(decode(&b), Err(DecodeError::BadSig));
}

#[test]
fn trailing_bytes_rejected() {
    let k = new_key();
    let mut u = report(&k, "x");
    u.seq = 0;
    let e = sign(&k, u);
    // re-sign payload with junk appended: valid signature, invalid encoding
    let mut payload = e.bytes[64..].to_vec();
    payload.push(0);
    let mut b = k.sign(&payload).to_bytes().to_vec();
    b.extend(payload);
    assert_eq!(decode(&b), Err(DecodeError::Malformed));
}

#[test]
fn size_limits() {
    let k = new_key();
    let e = sign(&k, report(&k, &"a".repeat(MAX_TEXT + 1)));
    assert_eq!(decode(&e.bytes), Err(DecodeError::TextTooLong));
    assert_eq!(
        decode(&vec![0; MAX_EVENT_BYTES + 1]),
        Err(DecodeError::TooLarge)
    );
    assert_eq!(decode(&[0; 10]), Err(DecodeError::TooShort));
}

const GOLDEN_REPORT: &str = "4f243ab1bc1a7ba4bb35ce7f62d296e6326a367b7adf40bb254a2a8acadd4134b063a45dd073a2f0d46de8ea461a4b348819201366527fa0ee0d3ad31a797a0b6c6f676f733a70756b6161723a32000207070707070707070707070707070707070707070707070707070707070707078a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c0000000000000000000000000000000000000000000000000000000000000000000100d20909090909090909090909090909090909090909090909090909090909090909090005776174657204572d3033000b746170206c65616b696e67";

/// Wire compatibility of format 2: the same content signs to the same bytes, and they decode back.
#[test]
fn a_format_2_event_round_trips_unchanged() {
    let bytes = hex::decode(GOLDEN_REPORT).unwrap();
    let e = decode(&bytes).unwrap();
    assert_eq!(e.bytes, bytes);
    let k = SigningKey::from_bytes(&[1u8; 32]);
    assert_eq!(e.u, report(&k, "tap leaking"));
    assert_eq!(sign(&k, report(&k, "tap leaking")).bytes, bytes);
}

#[test]
fn the_domain_and_version_are_format_2() {
    assert_eq!(DOMAIN, b"logos:pukaar:2\0");
    assert_eq!(VERSION, 2);
    let k = new_key();
    let e = sign(&k, report(&k, "x"));
    assert_eq!(&e.bytes[64..64 + DOMAIN.len()], DOMAIN);
    // a format 1 event (old domain) is refused before anything else
    let mut old = e.bytes.clone();
    old[64 + 13] = b'1';
    assert_eq!(decode(&old), Err(DecodeError::BadDomain));
    // right domain, wrong version byte, valid signature: the version rule
    let mut u = report(&k, "x");
    u.v = 1;
    assert_eq!(decode(&sign(&k, u).bytes), Err(DecodeError::BadVersion));
}

#[test]
fn the_chain_block_is_signed_and_survives_the_wire() {
    let k = new_key();
    let e = sign(&k, report(&k, "x"));
    assert_eq!(decode(&e.bytes).unwrap().u.chain_block, 1234);
    // the block id sits in the signed payload: changing it breaks the signature
    let mut u = report(&k, "x");
    u.chain_block = 1235;
    let other = sign(&k, u);
    assert_ne!(e.bytes[64..], other.bytes[64..]);
    let mut forged = e.bytes.clone();
    let at = (0..forged.len()).rev().find(|&i| forged[i..].starts_with(&[0xd2, 0x09])).unwrap();
    forged[at] ^= 1; // 1234 as a varint is d2 09
    assert_eq!(decode(&forged), Err(DecodeError::BadSig));
}

#[test]
fn only_issue_events_and_checkpoints_are_bodies() {
    // The variant index is on the wire. Report is first, Checkpoint is last.
    let k = new_key();
    let u = |body| Unsigned { body, ..report(&k, "x") };
    for body in [
        Body::Comment { issue: [1; 32], text: "hi".into() },
        Body::MarkDuplicate { issue: [1; 32], of: [2; 32] },
        Body::Checkpoint { heads: vec![([3; 32], 7)], lez_tx: "tx".into() },
    ] {
        let e = sign(&k, u(body));
        assert_eq!(decode(&e.bytes).unwrap(), e);
    }
    let tag = |b: &Body| postcard::to_allocvec(b).unwrap()[0];
    assert_eq!(tag(&report(&k, "x").body), 0);
    assert_eq!(tag(&Body::Checkpoint { heads: vec![], lez_tx: String::new() }), 9);
}

#[test]
fn every_text_is_capped_on_receipt() {
    let k = new_key();
    let long = "a".repeat(MAX_TEXT + 1);
    for body in [
        Body::Comment { issue: [1; 32], text: long.clone() },
        Body::Update { issue: [1; 32], note: "ok".into(), next_step: long.clone(), eta_h: 1 },
        Body::Reopen { issue: [1; 32], reason: long.clone() },
        Body::Checkpoint { heads: vec![], lez_tx: long.clone() },
    ] {
        let e = sign(&k, Unsigned { body, ..report(&k, "x") });
        assert_eq!(decode(&e.bytes), Err(DecodeError::TextTooLong));
    }
    assert!(Body::MarkDuplicate { issue: [1; 32], of: [2; 32] }.texts().is_empty());
}

#[test]
fn the_author_account_is_signed_and_survives_the_wire() {
    let k = new_key();
    let e = sign(&k, report(&k, "x"));
    assert_eq!(decode(&e.bytes).unwrap().u.author_account, [9; 32]);
    // The account sits in the signed payload: another account is another event, and a flipped byte fails.
    let other = sign(&k, Unsigned { author_account: [8; 32], ..report(&k, "x") });
    assert_ne!(e.bytes[64..], other.bytes[64..]);
    let mut forged = e.bytes.clone();
    let at = (0..forged.len()).find(|&i| forged[i..].starts_with(&[9; 32])).unwrap();
    forged[at] = 8;
    assert_eq!(decode(&forged), Err(DecodeError::BadSig));
}
