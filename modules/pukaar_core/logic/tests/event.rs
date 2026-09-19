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
