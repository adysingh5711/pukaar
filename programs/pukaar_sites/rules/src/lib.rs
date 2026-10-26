//! Pukaar site rules as plain functions. No zkVM, no wallet: the guest and the app share this crate.
//!
//! `anchor` stores one `CheckpointRecord` per (site, root) in the program's own shard of a PDA.
//! Clients check the signature and the root off-chain.

use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256};

mod records;
mod roles;
mod wire;
pub use records::*;
pub use roles::*;
pub use wire::*;

/// Borsh size of `CheckpointRecord`: 32 + 32 + 8 + 32 + 32 + 32.
pub const RECORD_LEN: usize = 168;

const CP_SEED_PREFIX: &[u8] = b"pukaar/cp/v1";

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CheckpointRecord {
    pub site_id: [u8; 32],
    pub heads_root: [u8; 32],
    pub n_events: u64,
    pub signer_pk: [u8; 32],
    pub sig_r: [u8; 32],
    pub sig_s: [u8; 32],
}

/// The program instruction. New variants are appended, never reordered.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum Instruction {
    /// The signer must be an active member of the checkpoint's site.
    Anchor(AnchorIx),
    /// Test-only gas probe (P0), built only with the `probe` feature.
    #[cfg(feature = "probe")]
    Probe(Probe),
    /// Index 1 stays taken so the other indices never move. The default build refuses it.
    #[cfg(not(feature = "probe"))]
    Reserved(Probe),
    CreateSite(CreateSiteIx),
    SetDetails(SetPointerIx),
    SetPlaces(SetPointerIx),
    SetKeyring(SetPointerIx),
    ApproveBatch(ApproveBatchIx),
    SetRoleBatch(SetRoleBatchIx),
    RevokeBatch(RevokeBatchIx),
    SetProfileBatch(SetProfileBatchIx),
    SuperAdminAdd(SuperAdminIx),
    SuperAdminRemove(SuperAdminIx),
    SuperAdminTransfer(SuperAdminIx),
}

/// What the plan hands to `apply` for one account. New variants are appended, never reordered.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum Effect {
    Anchor(CheckpointRecord),
    /// Guard on the clock shard: the claimed `now_ms` is not ahead and at most `CLOCK_TOLERANCE_MS` behind.
    ClockIs(u64),
    /// Guard on a native balance shard.
    BalanceAtLeast(u128),
    /// Write one 64-byte test member record.
    Member([u8; 64]),
    // Site and Member effects (P1), see `wire.rs`. Append only.
    Signer { site_id: [u8; 32], actor: Actor, now_ms: u64 },
    AnchorSigner { record: CheckpointRecord, signer: AccountId, now_ms: u64 },
    SiteGuard { site_id: [u8; 32], actor: Actor, not_super: Vec<AccountId> },
    CreateSite { args: CreateSite, now_ms: u64 },
    CreateMember { args: CreateSite, now_ms: u64 },
    SetPointer { kind: Pointer, site_id: [u8; 32], actor: Actor, cid: String, sha256: [u8; 32], now_ms: u64 },
    Approve { site_id: [u8; 32], actor: Actor, item: ApproveItem, is_new: bool, now_ms: u64 },
    ApproveSite {
        site_id: [u8; 32],
        actor: Actor,
        roster_len: u32,
        new_members: u32,
        keyring_cid: String,
        keyring_sha256: [u8; 32],
        now_ms: u64,
    },
    SetRole { actor: Actor, role: Role, now_ms: u64 },
    Revoke { actor: Actor, reason_sha256: [u8; 32], now_ms: u64 },
    RevokeSite {
        site_id: [u8; 32],
        actor: Actor,
        targets: Vec<AccountId>,
        keyring_cid: String,
        keyring_sha256: [u8; 32],
        now_ms: u64,
    },
    SetProfile { actor: Actor, cid: String, sha256: [u8; 32], version: u64, now_ms: u64 },
    SuperAdd { site_id: [u8; 32], actor: Actor, target: AccountId, now_ms: u64 },
    SuperRemove { site_id: [u8; 32], actor: Actor, target: AccountId, now_ms: u64 },
    SuperTransfer { site_id: [u8; 32], actor: Actor, target: AccountId, now_ms: u64 },
    Promote { actor: Actor, now_ms: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleError {
    EmptyRoot,
    AlreadyAnchored,
    ClockNotSet,
    ClockBehind,
    ClockStale,
    BadBalance,
    LowBalance,
    BadMerkle,
    BadSignature,
    MemberExists,
    // Site and Member rules (P1). Append only.
    WrongSiteId,
    BadCid,
    GatesNotInP1,
    ExpiryNotInP1,
    WrongRecord,
    NotActive,
    ClaimMismatch,
    Forbidden,
    AlreadyMember,
    StaleRoster,
    RosterFull,
    EmptyReason,
    EmptyBatch,
    SuperAdminTarget,
    AlreadySuperAdmin,
    NotSuperAdmin,
    SuperAdminsFull,
    SelfTarget,
    StaleProfile,
    BadJoinRequest,
    JoinWrongSite,
    KeyBinding,
    StaleNonce,
    JoinExpired,
    // wire.rs. Append only.
    RecordExists,
    NoRecord,
    // review fixes. Append only.
    VersionJump,
    NonceTooHigh,
    SignerKey,
}

impl core::fmt::Display for RuleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::EmptyRoot => "heads_root is empty",
            Self::AlreadyAnchored => "this (site, root) is already anchored",
            Self::ClockNotSet => "the clock account is not written yet",
            Self::ClockBehind => "claimed time is ahead of the clock",
            Self::ClockStale => "claimed time is too far behind the clock",
            Self::BadBalance => "native balance shard is not a canonical encoding",
            Self::LowBalance => "native balance is below the threshold",
            Self::BadMerkle => "merkle proof does not fold to the root",
            Self::BadSignature => "ed25519 signature is not valid",
            Self::MemberExists => "member account is already written",
            Self::WrongSiteId => "site_id is not sha256(domain || creator || salt)",
            Self::BadCid => "a CID is 1 to 128 ASCII letters and digits",
            Self::GatesNotInP1 => "gates stay off in P1",
            Self::ExpiryNotInP1 => "access_duration_ms stays 0 in P1",
            Self::WrongRecord => "the member record is not the one of this site and account",
            Self::NotActive => "the member is not active",
            Self::ClaimMismatch => "the claimed role is not the stored one",
            Self::Forbidden => "the role of the signer does not allow this",
            Self::AlreadyMember => "the account is already an active member",
            Self::StaleRoster => "roster_len changed since the client read it",
            Self::RosterFull => "roster_len overflows",
            Self::EmptyReason => "a revoke needs a reason",
            Self::EmptyBatch => "the batch has no item",
            Self::SuperAdminTarget => "a super admin cannot be revoked or have the role changed",
            Self::AlreadySuperAdmin => "already a super admin",
            Self::NotSuperAdmin => "not a super admin",
            Self::SuperAdminsFull => "there are already two super admins",
            Self::SelfTarget => "a super admin never removes or transfers to self",
            Self::StaleProfile => "the profile version does not grow",
            Self::BadJoinRequest => "the join request is not a pukaar/joinreq/v1 body",
            Self::JoinWrongSite => "the join request is for another site",
            Self::KeyBinding => "user_account is not the account id of user_schnorr_pk",
            Self::StaleNonce => "the join request nonce does not grow",
            Self::JoinExpired => "the join request is past valid_until_ms",
            Self::RecordExists => "the record is already written",
            Self::NoRecord => "the shard holds no record",
            Self::VersionJump => "the profile version jumps more than MAX_VERSION_STEP",
            Self::NonceTooHigh => "the join request nonce is more than the clock tolerance ahead of now",
            Self::SignerKey => "signer_pk is not the pukaar key of the member",
        })
    }
}

impl CheckpointRecord {
    pub fn encode(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("borsh serialization is infallible")
    }

    /// `None` unless `bytes` is exactly one record.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        borsh::from_slice(bytes).ok()
    }
}

/// PDA seed of a checkpoint: sha256("pukaar/cp/v1" || site_id || heads_root).
pub fn checkpoint_seed(site_id: &[u8; 32], heads_root: &[u8; 32]) -> [u8; 32] {
    Sha256::new()
        .chain_update(CP_SEED_PREFIX)
        .chain_update(site_id)
        .chain_update(heads_root)
        .finalize()
        .into()
}

/// Plan step: checks that need no chain state.
pub fn check_anchor(rec: &CheckpointRecord) -> Result<(), RuleError> {
    if rec.heads_root == [0; 32] {
        return Err(RuleError::EmptyRoot);
    }
    Ok(())
}

/// Apply step: the new shard bytes. `pre` is the shard now, empty before the first anchor.
pub fn apply_anchor(rec: &CheckpointRecord, pre: &[u8]) -> Result<Vec<u8>, RuleError> {
    check_anchor(rec)?;
    if !pre.is_empty() {
        return Err(RuleError::AlreadyAnchored);
    }
    Ok(rec.encode())
}

/// How far behind the clock a claimed `now` may be. A claim is a hint the client read earlier.
pub const CLOCK_TOLERANCE_MS: u64 = 900_000;

/// `ClockAccountData` is borsh `{ block_id: u64, timestamp: u64 }`: the timestamp is bytes 8..16, little-endian.
/// Zero means the clock account is not written yet.
pub fn check_clock(shard: &[u8], now_ms: u64) -> Result<(), RuleError> {
    let ts = shard.get(8..16).map_or(0, |b| u64::from_le_bytes(b.try_into().expect("8 bytes")));
    if ts == 0 {
        return Err(RuleError::ClockNotSet);
    }
    if ts < now_ms {
        return Err(RuleError::ClockBehind);
    }
    if ts - now_ms > CLOCK_TOLERANCE_MS {
        return Err(RuleError::ClockStale);
    }
    Ok(())
}

/// A native balance shard is empty (zero) or the 16-byte little-endian u128. Same rule as lee_core `decode_balance`.
pub fn check_balance(shard: &[u8], min: u128) -> Result<(), RuleError> {
    let bal = if shard.is_empty() {
        0
    } else {
        u128::from_le_bytes(shard.try_into().map_err(|_| RuleError::BadBalance)?)
    };
    if bal < min { Err(RuleError::LowBalance) } else { Ok(()) }
}

fn sha(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

/// Merkle leaf, as in the app's `checkpoint.rs`: sha256(0x00 || data).
pub fn merkle_leaf(data: &[u8]) -> [u8; 32] {
    sha(&[&[0], data])
}

/// Fold a proof for leaf `idx` of a tree with `n` leaves. Nodes are sha256(0x01 || left || right).
/// An odd node at the end of a level is promoted, not duplicated, so that level takes no proof item.
/// `None` unless the proof has exactly the items the path needs.
pub fn merkle_fold(leaf: [u8; 32], mut idx: u64, mut n: u64, proof: &[[u8; 32]]) -> Option<[u8; 32]> {
    if idx >= n {
        return None;
    }
    let (mut node, mut items) = (leaf, proof.iter());
    while n > 1 {
        if idx % 2 == 1 {
            node = sha(&[&[1], items.next()?, &node]);
        } else if idx + 1 < n {
            node = sha(&[&[1], &node, items.next()?]);
        }
        idx /= 2;
        n = n.div_ceil(2);
    }
    items.next().is_none().then_some(node)
}

#[cfg(feature = "probe")]
pub fn check_ed25519(pk: &[u8; 32], sig: &[u8; 64], msg: &[u8]) -> Result<(), RuleError> {
    let vk = ed25519_dalek::VerifyingKey::from_bytes(pk).map_err(|_| RuleError::BadSignature)?;
    vk.verify_strict(msg, &ed25519_dalek::Signature::from_bytes(sig)).map_err(|_| RuleError::BadSignature)
}

// ---- P0 gas probe: test-only. Each flag switches one step on. ----

pub const PROBE_CLOCK: u8 = 1;
pub const PROBE_BALANCE: u8 = 2;
pub const PROBE_MERKLE: u8 = 4;
pub const PROBE_ED25519: u8 = 8;
pub const PROBE_MEMBERS: u8 = 16;

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Probe {
    pub flags: u8,
    /// Claimed chain time (clock step).
    pub now_ms: u64,
    /// Balance threshold (balance step).
    pub min_balance: u128,
    /// Merkle step: leaf data, leaf index, leaf count, proof, root.
    pub leaf_data: Vec<u8>,
    pub index: u64,
    pub n_leaves: u64,
    pub proof: Vec<[u8; 32]>,
    pub root: [u8; 32],
    /// Ed25519 step.
    pub pk: [u8; 32],
    pub sig: [u8; 64],
    pub msg: Vec<u8>,
    /// Members step: `n_members` PDAs of seed `member_seed(salt, i)`, one 64-byte record each.
    pub salt: [u8; 32],
    pub n_members: u16,
}

/// Plan-side steps that need no chain state: Merkle and ed25519.
#[cfg(feature = "probe")]
pub fn check_probe(p: &Probe) -> Result<(), RuleError> {
    if p.flags & PROBE_MERKLE != 0 {
        let top = merkle_fold(merkle_leaf(&p.leaf_data), p.index, p.n_leaves, &p.proof);
        if top != Some(p.root) {
            return Err(RuleError::BadMerkle);
        }
    }
    if p.flags & PROBE_ED25519 != 0 {
        check_ed25519(&p.pk, &p.sig, &p.msg)?;
    }
    Ok(())
}

/// All levels of the app's `checkpoint.rs::merkle_root` tree, leaves first. For tests and the probe runner.
#[cfg(feature = "probe")]
pub fn merkle_levels(leaves: Vec<[u8; 32]>) -> Vec<Vec<[u8; 32]>> {
    let mut levels = vec![leaves];
    while levels.last().is_some_and(|l| l.len() > 1) {
        let next = levels.last().unwrap().chunks(2).map(|p| if p.len() == 2 { sha(&[&[1], &p[0], &p[1]]) } else { p[0] }).collect();
        levels.push(next);
    }
    levels
}

/// The proof `merkle_fold` expects for leaf `idx`.
#[cfg(feature = "probe")]
pub fn merkle_proof(levels: &[Vec<[u8; 32]>], mut idx: usize) -> Vec<[u8; 32]> {
    let mut proof = vec![];
    for level in &levels[..levels.len() - 1] {
        if idx % 2 == 1 {
            proof.push(level[idx - 1]);
        } else if idx + 1 < level.len() {
            proof.push(level[idx + 1]);
        }
        idx /= 2;
    }
    proof
}

/// RFC 8032 section 7.1, test 2: one-byte message 0x72.
#[cfg(feature = "probe")]
pub const ED_PK: &str = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
#[cfg(feature = "probe")]
pub const ED_SIG: &str = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00";

pub fn unhex<const N: usize>(s: &str) -> [u8; N] {
    let v: Vec<u8> = (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex")).collect();
    v.try_into().expect("length")
}

/// A probe whose steps all pass: a depth-10 proof for leaf 5 of 1024, and one valid signature.
#[cfg(feature = "probe")]
pub fn sample_probe(flags: u8, now_ms: u64, min_balance: u128, n_members: u16, salt: [u8; 32]) -> Probe {
    let n = 1024u64;
    let levels = merkle_levels((0..n).map(|i| merkle_leaf(&i.to_be_bytes())).collect());
    Probe {
        flags,
        now_ms,
        min_balance,
        leaf_data: 5u64.to_be_bytes().to_vec(),
        index: 5,
        n_leaves: n,
        proof: merkle_proof(&levels, 5),
        root: levels[levels.len() - 1][0],
        pk: unhex(ED_PK),
        sig: unhex(ED_SIG),
        msg: vec![0x72],
        salt,
        n_members,
    }
}

pub fn member_seed(salt: &[u8; 32], i: u16) -> [u8; 32] {
    sha(&[b"pukaar/probe-member", salt, &i.to_le_bytes()])
}

/// A 64-byte record of the size of a small Member: 32 bytes of key, 32 bytes of profile hash.
pub fn member_record(salt: &[u8; 32], i: u16) -> [u8; 64] {
    let mut r = [0; 64];
    r[..32].copy_from_slice(&member_seed(salt, i));
    r[32..].copy_from_slice(&sha(&[salt, &i.to_le_bytes()]));
    r
}

/// Apply step: the new shard bytes, or `None` to keep the shard.
pub fn apply_effect(effect: &Effect, pre: &[u8]) -> Result<Option<Vec<u8>>, RuleError> {
    match effect {
        Effect::Anchor(rec) => apply_anchor(rec, pre).map(Some),
        Effect::ClockIs(now) => check_clock(pre, *now).map(|()| None),
        Effect::BalanceAtLeast(min) => check_balance(pre, *min).map(|()| None),
        Effect::Member(rec) if pre.is_empty() => Ok(Some(rec.to_vec())),
        Effect::Member(_) => Err(RuleError::MemberExists),
        other => apply_site_effect(other, pre),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(root: u8) -> CheckpointRecord {
        CheckpointRecord {
            site_id: [1; 32],
            heads_root: [root; 32],
            n_events: 7,
            signer_pk: [3; 32],
            sig_r: [4; 32],
            sig_s: [5; 32],
        }
    }

    #[test]
    fn seed_vector() {
        // python3: sha256(b"pukaar/cp/v1" + [1]*32 + [2]*32)
        let want = "48a1501a8ccc11f268170a5236679b76b7709876158fab21ac79b96e801eb0fc";
        let got: String = checkpoint_seed(&[1; 32], &[2; 32]).iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn record_round_trip_is_168_bytes() {
        let bytes = rec(2).encode();
        assert_eq!(bytes.len(), RECORD_LEN);
        assert_eq!(CheckpointRecord::decode(&bytes), Some(rec(2)));
        assert_eq!(CheckpointRecord::decode(&bytes[..167]), None);
    }

    #[test]
    fn anchor_writes_once() {
        let first = apply_anchor(&rec(2), &[]).unwrap();
        assert_eq!(first, rec(2).encode());
        // same root again, and a different record for the same shard: both rejected
        assert_eq!(apply_anchor(&rec(2), &first), Err(RuleError::AlreadyAnchored));
        assert_eq!(apply_anchor(&rec(9), &first), Err(RuleError::AlreadyAnchored));
    }

    #[test]
    fn anchor_rejects_empty_root() {
        assert_eq!(apply_anchor(&rec(0), &[]), Err(RuleError::EmptyRoot));
        assert_eq!(check_anchor(&rec(0)), Err(RuleError::EmptyRoot));
    }

    // ---- probe steps ----

    fn clock(ts: u64) -> Vec<u8> {
        // borsh ClockAccountData { block_id, timestamp }
        [5u64.to_le_bytes(), ts.to_le_bytes()].concat()
    }

    #[test]
    fn clock_window() {
        assert_eq!(check_clock(&clock(1000), 1000), Ok(()));
        assert_eq!(check_clock(&clock(1_000_000), 1_000_000 - CLOCK_TOLERANCE_MS), Ok(()));
        assert_eq!(check_clock(&clock(1_000_000), 1_000_000 - CLOCK_TOLERANCE_MS - 1), Err(RuleError::ClockStale));
        assert_eq!(check_clock(&clock(1000), 1001), Err(RuleError::ClockBehind));
        assert_eq!(check_clock(&clock(0), 0), Err(RuleError::ClockNotSet));
        assert_eq!(check_clock(&[], 0), Err(RuleError::ClockNotSet));
    }

    #[test]
    fn balance_threshold() {
        let bal = |b: u128| b.to_le_bytes().to_vec();
        assert_eq!(check_balance(&bal(5), 5), Ok(()));
        assert_eq!(check_balance(&bal(4), 5), Err(RuleError::LowBalance));
        assert_eq!(check_balance(&[], 0), Ok(()));
        assert_eq!(check_balance(&[], 1), Err(RuleError::LowBalance));
        assert_eq!(check_balance(&[1, 2, 3], 0), Err(RuleError::BadBalance));
    }

    #[cfg(feature = "probe")]
    #[test]
    fn merkle_fold_matches_checkpoint_scheme() {
        for n in [1usize, 2, 3, 7, 1000, 1024] {
            let leaves: Vec<_> = (0..n).map(|i| merkle_leaf(&(i as u64).to_be_bytes())).collect();
            let levels = merkle_levels(leaves.clone());
            let root = levels.last().unwrap()[0];
            for i in (0..n).step_by(if n > 100 { 37 } else { 1 }).chain([n - 1]) {
                let proof = merkle_proof(&levels, i);
                assert_eq!(merkle_fold(leaves[i], i as u64, n as u64, &proof), Some(root), "n={n} i={i}");
                // a wrong leaf, a short proof and a long proof all fail
                assert_ne!(merkle_fold(merkle_leaf(b"x"), i as u64, n as u64, &proof), Some(root));
                if !proof.is_empty() {
                    assert_eq!(merkle_fold(leaves[i], i as u64, n as u64, &proof[1..]), None);
                }
                let long = [proof.clone(), vec![[0; 32]]].concat();
                assert_eq!(merkle_fold(leaves[i], i as u64, n as u64, &long), None);
            }
        }
    }

    #[cfg(feature = "probe")]
    #[test]
    fn ed25519_strict() {
        let (pk, sig) = (unhex::<32>(ED_PK), unhex::<64>(ED_SIG));
        assert_eq!(check_ed25519(&pk, &sig, &[0x72]), Ok(()));
        assert_eq!(check_ed25519(&pk, &sig, &[0x73]), Err(RuleError::BadSignature));
    }

    #[cfg(feature = "probe")]
    #[test]
    fn probe_runs_each_step() {
        let mut p = sample_probe(PROBE_MERKLE | PROBE_ED25519, 0, 0, 0, [9; 32]);
        assert_eq!(p.proof.len(), 10);
        assert_eq!(check_probe(&p), Ok(()));
        p.root[0] ^= 1;
        assert_eq!(check_probe(&p), Err(RuleError::BadMerkle));
        p.flags = PROBE_ED25519;
        p.msg = vec![1];
        assert_eq!(check_probe(&p), Err(RuleError::BadSignature));
        p.flags = 0;
        assert_eq!(check_probe(&p), Ok(()));
    }

    #[cfg(feature = "probe")]
    #[test]
    fn member_written_once() {
        let rec = member_record(&[9; 32], 3);
        assert_ne!(rec, member_record(&[9; 32], 4));
        assert_eq!(apply_effect(&Effect::Member(rec), &[]), Ok(Some(rec.to_vec())));
        assert_eq!(apply_effect(&Effect::Member(rec), &rec), Err(RuleError::MemberExists));
        assert_eq!(apply_effect(&Effect::ClockIs(1), &clock(1)), Ok(None));
    }
}
