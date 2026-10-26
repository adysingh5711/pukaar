//! The `Site` and `Member` records, the PDA seeds and the join request body.
//!
//! Layouts are Borsh, as `CheckpointRecord`. Enums are append-only: a new variant goes at the end,
//! so the bytes of every record written before stay valid. P3 and P4 fill fields that P1 leaves
//! at their zero value; they add no field.

use crate::{RuleError, sha};
use borsh::{BorshDeserialize, BorshSerialize};

/// A LEZ account id: the 32 bytes of `lee_core::account::AccountId` (Borsh `{ value: [u8; 32] }`).
pub type AccountId = [u8; 32];

/// The `version` byte of every record that this crate writes.
pub const RECORD_VERSION: u8 = 1;

/// The longest CID a record holds, in bytes. A CID is ASCII letters and digits (base32 or base58).
/// CIDv1 base32 of a SHA-256 is 59 bytes, so 128 leaves room for longer multihashes.
pub const CID_MAX: usize = 128;

/// LEZ v0.3.1 `DATA_MAX_LENGTH` (`lee/state_machine/core/src/account/data.rs`): 100 KiB per shard.
pub const SHARD_MAX: usize = 100 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, BorshSerialize, BorshDeserialize)]
pub enum Role {
    Resident,
    Steward,
    Admin,
}

/// Stored status. P4 appends `OnNotice` and `Expired`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum MemberStatus {
    Active,
    Removed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum NoticeOutcome {
    Steward,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Notice {
    pub outcome: NoticeOutcome,
    pub deadline_ms: u64,
    pub by: AccountId,
    pub reason_sha256: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Revocation {
    pub by: AccountId,
    pub at_ms: u64,
    pub reason_sha256: [u8; 32],
}

/// An admin record that a gate passed. It holds no amount (Decision 10).
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Attestation {
    pub gate_id: u8,
    pub at_ms: u64,
    pub by: AccountId,
}

pub const GATE_NATIVE: u8 = 0;
pub const GATE_TOKEN: u8 = 1;
pub const GATE_NFT: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum GateMode {
    OnChain,
    Attested,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct TokenGate {
    pub definition: AccountId,
    pub min: u128,
    pub mode: GateMode,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct NftGate {
    pub definition: AccountId,
    pub mode: GateMode,
}

/// Every gate is off (`None`) in P1. P3 switches them on.
#[derive(Debug, Clone, Default, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct GateConfig {
    pub native_min: Option<u128>,
    pub token: Option<TokenGate>,
    pub nft: Option<NftGate>,
    pub token_program: Option<AccountId>,
    pub ata_program: Option<AccountId>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Site {
    pub version: u8,
    pub site_id: [u8; 32],
    pub creator: AccountId,
    pub salt: [u8; 32],
    pub super_admins: [Option<AccountId>; 2],
    pub details_cid: String,
    pub details_sha256: [u8; 32],
    pub places_cid: String,
    pub places_sha256: [u8; 32],
    pub key_epoch: u64,
    pub keyring_cid: String,
    pub keyring_sha256: [u8; 32],
    pub gates: GateConfig,
    /// 0 means no expiry. Always 0 in P1.
    pub access_duration_ms: u64,
    pub renew_window_ms: u64,
    pub roster_len: u32,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Member {
    pub version: u8,
    pub site_id: [u8; 32],
    /// The LEZ account of the user (one per user, all sites).
    pub user: AccountId,
    /// The ed25519 Pukaar key, from the join request.
    pub pukaar_key: [u8; 32],
    /// A super admin holds `Admin` here; the super admin slots are in `Site`.
    pub role: Role,
    pub status: MemberStatus,
    pub granted_by: AccountId,
    pub granted_at_ms: u64,
    /// 0 means never.
    pub expires_at_ms: u64,
    pub notice: Option<Notice>,
    pub revoked: Option<Revocation>,
    pub rights_override: u32,
    pub profile_cid: String,
    pub profile_sha256: [u8; 32],
    pub profile_version: u64,
    pub gate_attestation: Option<Attestation>,
    pub request_id: [u8; 32],
    /// The anchor time of the join request (P3). 0 in P1: P1 has no `anchor_requests`.
    pub requested_at_ms: u64,
    pub last_req_nonce: u64,
}

impl Site {
    pub fn encode(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("borsh serialization is infallible")
    }

    /// `None` unless `bytes` is exactly one record.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        borsh::from_slice(bytes).ok()
    }

    pub fn is_super(&self, a: &AccountId) -> bool {
        self.super_admins.contains(&Some(*a))
    }
}

impl Member {
    pub fn encode(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("borsh serialization is infallible")
    }

    /// `None` unless `bytes` is exactly one record.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        borsh::from_slice(bytes).ok()
    }
}

/// A CID is 1 to `CID_MAX` ASCII letters and digits.
pub fn check_cid(cid: &str) -> Result<(), RuleError> {
    let ok = !cid.is_empty() && cid.len() <= CID_MAX && cid.bytes().all(|b| b.is_ascii_alphanumeric());
    if ok { Ok(()) } else { Err(RuleError::BadCid) }
}

// ---- PDA seeds: sha256(domain ‖ parts). The program turns each into an address with
// `AccountId::for_public_pda` (risc0 only), so this crate returns only the seed. ----

pub mod seed {
    use super::AccountId;
    use crate::sha;

    pub const SITE_ID: &[u8] = b"pukaar/site-id/v1";
    pub const SITE: &[u8] = b"pukaar/site/v1";
    pub const MEMBER: &[u8] = b"pukaar/member/v1";
    pub const ROSTER: &[u8] = b"pukaar/roster/v1";
    /// Reserved for P3 (`anchor_requests`).
    pub const JOIN_BATCH: &[u8] = b"pukaar/joinbatch/v1";
    /// Reserved for P4 (place removal).
    pub const PLACE_REMOVAL: &[u8] = b"pukaar/place-rm/v1";

    /// sha256("pukaar/site-id/v1" ‖ creator ‖ salt). Not a seed: the id that the other seeds take.
    pub fn site_id(creator: &AccountId, salt: &[u8; 32]) -> [u8; 32] {
        sha(&[SITE_ID, creator, salt])
    }

    pub fn site(site_id: &[u8; 32]) -> [u8; 32] {
        sha(&[SITE, site_id])
    }

    pub fn member(site_id: &[u8; 32], user: &AccountId) -> [u8; 32] {
        sha(&[MEMBER, site_id, user])
    }

    pub fn roster(site_id: &[u8; 32], page: u32) -> [u8; 32] {
        sha(&[ROSTER, site_id, &page.to_le_bytes()])
    }

    pub fn checkpoint(site_id: &[u8; 32], heads_root: &[u8; 32]) -> [u8; 32] {
        crate::checkpoint_seed(site_id, heads_root)
    }

    pub fn join_batch(site_id: &[u8; 32], root: &[u8; 32]) -> [u8; 32] {
        sha(&[JOIN_BATCH, site_id, root])
    }

    pub fn place_removal(site_id: &[u8; 32], code: &str) -> [u8; 32] {
        sha(&[PLACE_REMOVAL, site_id, &sha(&[code.as_bytes()])])
    }
}

// ---- The join request. Fixed layout, big-endian integers. ----

pub const JOIN_DOMAIN: &[u8; 18] = b"pukaar/joinreq/v1\0";
/// 18 domain + 4 * 32 (site_id, user_account, user_schnorr_pk, pukaar_ed_pk) + 8 nonce + 8 valid_until_ms.
pub const JOIN_BODY_LEN: usize = 162;
/// Body + sig_ed (64, ed25519 over the body) + sig_schnorr (64, BIP340 over sha256(body)).
pub const JOIN_REQ_LEN: usize = 290;
const REQ_ID_DOMAIN: &[u8] = b"pukaar/joinreq-id/v1";
/// LEZ v0.3.1 `public_key.rs`: the prefix is padded with 5 zero bytes to 32.
const ACCOUNT_ID_PREFIX: &[u8; 32] = b"/LEE/v0.3/AccountId/Public/\0\0\0\0\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinBody {
    pub site_id: [u8; 32],
    pub user_account: AccountId,
    pub user_schnorr_pk: [u8; 32],
    pub pukaar_ed_pk: [u8; 32],
    pub nonce: u64,
    pub valid_until_ms: u64,
}

impl JoinBody {
    pub fn encode(&self) -> [u8; JOIN_BODY_LEN] {
        let parts: [&[u8]; 7] = [
            JOIN_DOMAIN,
            &self.site_id,
            &self.user_account,
            &self.user_schnorr_pk,
            &self.pukaar_ed_pk,
            &self.nonce.to_be_bytes(),
            &self.valid_until_ms.to_be_bytes(),
        ];
        parts.concat().try_into().expect("162 bytes")
    }

    /// `None` unless `b` is a body with the right domain.
    pub fn parse(b: &[u8; JOIN_BODY_LEN]) -> Option<Self> {
        let f = |i: usize| -> [u8; 32] { b[18 + 32 * i..50 + 32 * i].try_into().expect("32 bytes") };
        let n = |i: usize| u64::from_be_bytes(b[i..i + 8].try_into().expect("8 bytes"));
        (b[..18] == JOIN_DOMAIN[..]).then(|| Self {
            site_id: f(0),
            user_account: f(1),
            user_schnorr_pk: f(2),
            pukaar_ed_pk: f(3),
            nonce: n(146),
            valid_until_ms: n(154),
        })
    }
}

/// The LEZ public account id of an x-only secp256k1 key: sha256(prefix ‖ pk), as `lezkey.rs`.
pub fn lez_account_id(x_only_pk: &[u8; 32]) -> AccountId {
    sha(&[ACCOUNT_ID_PREFIX, x_only_pk])
}

/// sha256("pukaar/joinreq-id/v1" ‖ the whole 290-byte request).
pub fn request_id(req: &[u8; JOIN_REQ_LEN]) -> [u8; 32] {
    sha(&[REQ_ID_DOMAIN, req])
}

/// P1 checks of a join request against the target record (`None` before the first approval).
/// It does not verify either signature (O11 is off): the admin app and the auditor do that.
/// `site_id` is the one the program derived the `Site` and `Member` PDAs from.
pub fn check_join_request(
    req: &[u8; JOIN_REQ_LEN],
    site_id: &[u8; 32],
    member: Option<&Member>,
    now_ms: u64,
) -> Result<JoinBody, RuleError> {
    let body = JoinBody::parse(req[..JOIN_BODY_LEN].try_into().expect("162 bytes")).ok_or(RuleError::BadJoinRequest)?;
    if body.site_id != *site_id {
        return Err(RuleError::JoinWrongSite);
    }
    if lez_account_id(&body.user_schnorr_pk) != body.user_account {
        return Err(RuleError::KeyBinding);
    }
    if member.is_some_and(|m| m.user != body.user_account || m.site_id != *site_id) {
        return Err(RuleError::WrongRecord);
    }
    if body.nonce <= member.map_or(0, |m| m.last_req_nonce) {
        return Err(RuleError::StaleNonce);
    }
    if now_ms > body.valid_until_ms {
        return Err(RuleError::JoinExpired);
    }
    Ok(body)
}
