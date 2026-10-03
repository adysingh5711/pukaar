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

const GOLDEN_REPORT: &str = "6cab53e9cd4262147a2c6fa2304f89c542f133c6a3fea67a400c3141d1cdabfffa913bb6fc43380a0e67ff6f4efb409ffd66f41d2cd266a0d1ce8d16d88fd1016c6f676f733a70756b6161723a31000107070707070707070707070707070707070707070707070707070707070707078a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c00000000000000000000000000000000000000000000000000000000000000000001000505776174657204572d3033000b746170206c65616b696e67";

/// Wire compatibility: bytes signed before `LocationRetire` existed still decode to the
/// same event, and re-signing the same content reproduces them bit for bit.
#[test]
fn an_event_serialized_before_location_retire_round_trips_unchanged() {
    let bytes = hex::decode(GOLDEN_REPORT).unwrap();
    let e = decode(&bytes).unwrap();
    assert_eq!(e.bytes, bytes);
    let k = SigningKey::from_bytes(&[1u8; 32]);
    let mut u = report(&k, "tap leaking");
    u.site = [7; 32];
    assert_eq!(e.u, u);
    assert_eq!(sign(&k, u).bytes, bytes);
}

fn retire(reason: &str) -> Body {
    Body::LocationRetire {
        code: "W-03".into(),
        retired: true,
        reason: reason.into(),
    }
}

/// Postcard encodes the variant index first: the new variant must come after Checkpoint.
#[test]
fn location_retire_is_appended_after_the_last_variant() {
    let tag = |b: &Body| postcard::to_allocvec(b).unwrap()[0];
    let checkpoint = Body::Checkpoint {
        heads: vec![],
        lez_tx: String::new(),
    };
    assert_eq!(tag(&checkpoint), 14);
    assert_eq!(tag(&retire("dry")), 15);
}

#[test]
fn location_retire_round_trips() {
    let k = new_key();
    let mut u = report(&k, "x");
    u.body = retire("tap removed");
    let e = sign(&k, u);
    assert_eq!(decode(&e.bytes).unwrap(), e);
}

#[test]
fn location_retire_reason_over_the_text_cap_is_dropped() {
    let k = new_key();
    let mut u = report(&k, "x");
    u.body = retire(&"r".repeat(MAX_TEXT + 1));
    assert_eq!(decode(&sign(&k, u).bytes), Err(DecodeError::TextTooLong));
}

#[test]
fn location_retire_texts_include_the_reason() {
    assert_eq!(retire("dry").texts(), vec!["W-03", "dry"]);
}
