//! One read-only check against the real testnet site. Run it with
//! `cargo test --features http -- --ignored`. It sends no transaction.
#![cfg(feature = "http")]
use pukaar_logic::chain::AccountId;
use pukaar_logic::lezkey::{derive_lez_secret, lez_account_id};
use pukaar_logic::reducer::Rank;
use pukaar_logic::rpc::{refresh, Chain, HttpRpc};

#[test]
#[ignore = "reads https://testnet.lez.logos.co"]
fn the_real_testnet_site_has_the_expected_roles() {
    let program: AccountId = bs58::decode("Gs1EWbwJKSBGRBiR7xasYtkrfCyUGJ52Vx4ASsDLooVK").into_vec().unwrap().try_into().unwrap();
    let site: [u8; 32] = hex::decode("89169de759a670a419cbe3d179e972c2cd03513f020570b296bd64e5b7904dc0").unwrap().try_into().unwrap();
    let acct = |n: u8| lez_account_id(&derive_lez_secret(&[n; 32]));
    let key = |n: u8| ed25519_dalek::SigningKey::from_bytes(&[n; 32]).verifying_key().to_bytes();
    let c = Chain::shared(program, site, acct(1));
    for n in [2, 3] {
        c.lock().unwrap().track(acct(n));
    }
    refresh(&c, &HttpRpc::new("https://testnet.lez.logos.co"));
    let c = c.lock().unwrap();
    assert!(c.ok, "read failed: {:?}", c.error);
    assert!(c.block > 0);
    assert_eq!(c.role_at(&key(1), u64::MAX), Some(Rank::SuperAdmin));
    assert_eq!(c.role_at(&key(2), u64::MAX), Some(Rank::Admin));
    assert_eq!(c.role_at(&key(3), u64::MAX), None, "t3 was removed");
    assert_eq!(c.status_json()["roster_len"], 4);
}
