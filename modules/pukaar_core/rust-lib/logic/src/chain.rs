//! Read side of the `pukaar_sites` program: PDA addresses, record decoders and the `Roster` cache.
//!
//! Pure functions, no network. Step 2.3 fetches JSON-RPC `getAccount` and hands the text to
//! `shard_of`. The record layouts and the PDA seeds are the types of `pukaar_sites_rules`.

use pukaar_sites_rules::{seed, CheckpointRecord, Effective, Member, Rank, Site, RECORD_VERSION};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// A LEZ account id, 32 bytes.
pub type AccountId = [u8; 32];

/// LEZ v0.3.1 `AccountId::for_public_pda` (`lee_core` `program/mod.rs`): the rules crate returns
/// only the seed, because the hash there needs risc0.
const PDA_PREFIX: &[u8; 32] = b"/LEE/v0.2/AccountId/PDA/\0\0\0\0\0\0\0\0";

/// sha256(prefix ‖ program account id ‖ seed).
pub fn pda(program: &AccountId, seed: &[u8; 32]) -> AccountId {
    Sha256::new().chain_update(PDA_PREFIX).chain_update(program).chain_update(seed).finalize().into()
}

pub fn site_pda(program: &AccountId, site_id: &[u8; 32]) -> AccountId {
    pda(program, &seed::site(site_id))
}

/// One account per (site, user). `user_account` is the LEZ account of the user.
pub fn member_pda(program: &AccountId, site_id: &[u8; 32], user_account: &AccountId) -> AccountId {
    pda(program, &seed::member(site_id, user_account))
}

pub fn checkpoint_pda(program: &AccountId, site_id: &[u8; 32], heads_root: &[u8; 32]) -> AccountId {
    pda(program, &seed::checkpoint(site_id, heads_root))
}

/// The base58 form of an account id, as the wallet and the RPC print it.
pub fn b58(id: &AccountId) -> String {
    bs58::encode(id).into_string()
}

/// The bytes of this program's shard in a `getAccount` (or `getAccountView`) response. An account
/// that holds no record gives an empty vector. The shape, from the real testnet:
/// `{"jsonrpc":"2.0","id":1,"result":{"nonce":0,"data":{"shards":{"<program b58>":[1,137,...]}}}}`.
pub fn shard_of(response: &str, program: &AccountId) -> Result<Vec<u8>, String> {
    let v: serde_json::Value = serde_json::from_str(response).map_err(|e| format!("bad JSON: {e}"))?;
    if let Some(e) = v.get("error") {
        return Err(format!("rpc error: {e}"));
    }
    let shards = v
        .pointer("/result/data/shards")
        .and_then(|s| s.as_object())
        .ok_or("no result.data.shards in the response")?;
    let Some(shard) = shards.get(&b58(program)) else { return Ok(Vec::new()) };
    shard
        .as_array()
        .ok_or("shard is not an array")?
        .iter()
        .map(|b| b.as_u64().and_then(|b| u8::try_from(b).ok()).ok_or_else(|| "shard byte out of range".to_string()))
        .collect()
}

fn check_version(v: u8) -> Result<(), String> {
    if v == RECORD_VERSION { Ok(()) } else { Err(format!("unknown record version {v}")) }
}

pub fn decode_site(data: &[u8]) -> Result<Site, String> {
    let s = Site::decode(data).ok_or("not a Site record")?;
    check_version(s.version).map(|()| s)
}

pub fn decode_member(data: &[u8]) -> Result<Member, String> {
    let m = Member::decode(data).ok_or("not a Member record")?;
    check_version(m.version).map(|()| m)
}

pub fn decode_checkpoint(data: &[u8]) -> Result<CheckpointRecord, String> {
    CheckpointRecord::decode(data).ok_or_else(|| "not a Checkpoint record".to_string())
}

/// The records of one site that the app has read: the `Site` and every `Member`.
pub struct Roster {
    program: AccountId,
    site_id: [u8; 32],
    pub site: Option<Site>,
    /// By member PDA, so `apply` needs no user id and an emptied shard removes the row.
    members: HashMap<AccountId, Member>,
}

impl Roster {
    pub fn new(program: AccountId, site_id: [u8; 32]) -> Self {
        Self { program, site_id, site: None, members: HashMap::new() }
    }

    /// Take the shard bytes that the chain holds at `account_id`. Empty bytes remove the row. A
    /// record that does not belong to this site and this address is an error and changes nothing.
    pub fn apply(&mut self, account_id: &AccountId, data: &[u8]) -> Result<(), String> {
        if *account_id == site_pda(&self.program, &self.site_id) {
            if data.is_empty() {
                self.site = None;
                return Ok(());
            }
            let s = decode_site(data)?;
            if s.site_id != self.site_id {
                return Err("Site record of another site".into());
            }
            self.site = Some(s);
            return Ok(());
        }
        if data.is_empty() {
            self.members.remove(account_id);
            return Ok(());
        }
        let m = decode_member(data)?;
        if m.site_id != self.site_id || member_pda(&self.program, &m.site_id, &m.user) != *account_id {
            return Err("Member record does not belong to this address".into());
        }
        self.members.insert(*account_id, m);
        Ok(())
    }

    pub fn member(&self, user_account: &AccountId) -> Option<&Member> {
        self.members.get(&member_pda(&self.program, &self.site_id, user_account))
    }

    pub fn members(&self) -> impl Iterator<Item = &Member> {
        self.members.values()
    }

    /// The rank of an active member, found by LEZ account or by Pukaar key. `None` for a stranger
    /// or a removed member. A super admin (a slot in `Site`) has `Rank::SuperAdmin`.
    pub fn role_of(&self, who: &[u8; 32], now_ms: u64) -> Option<Rank> {
        // ponytail: scan by Pukaar key, O(members). Add a key index if rosters pass thousands.
        let m = self.member(who).or_else(|| self.members.values().find(|m| m.pukaar_key == *who))?;
        self.rank_of(m, now_ms)
    }

    /// The rank of the author of an event. With `account`, the `Member` record at that account
    /// must hold this Pukaar key: an event that names the wrong account has no role.
    pub fn role_for(&self, key: &[u8; 32], account: Option<&AccountId>, now_ms: u64) -> Option<Rank> {
        let Some(a) = account else { return self.role_of(key, now_ms) };
        let m = self.member(a).filter(|m| m.pukaar_key == *key)?;
        self.rank_of(m, now_ms)
    }

    fn rank_of(&self, m: &Member, now_ms: u64) -> Option<Rank> {
        let Effective::Active(role) = pukaar_sites_rules::effective_status(m, now_ms) else { return None };
        match &self.site {
            Some(s) if s.is_super(&m.user) => Some(Rank::SuperAdmin),
            _ => Some(role.into()),
        }
    }

    /// Drop each member whose Pukaar key differs from the key that `expect` holds for its account.
    pub fn retain_matching(&mut self, expect: &std::collections::BTreeMap<AccountId, [u8; 32]>) {
        self.members.retain(|_, m| expect.get(&m.user).is_none_or(|k| *k == m.pukaar_key));
    }

    /// Does any member hold this Pukaar key?
    pub fn has_key(&self, key: &[u8; 32]) -> bool {
        self.members.values().any(|m| m.pukaar_key == *key)
    }

    /// Active admins and super admins: the readers of a first profile (before the site key wrap).
    pub fn admins(&self, now_ms: u64) -> Vec<&Member> {
        self.members().filter(|m| self.role_of(&m.user, now_ms).is_some_and(|r| r >= Rank::Admin)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real testnet read, public chain data (`getAccount` of the Site PDA, program Public/Gs1EW...).
    const SITE_RESPONSE: &str = include_str!("../tests/fixtures/getaccount_site.json");
    const PROGRAM_B58: &str = "Gs1EWbwJKSBGRBiR7xasYtkrfCyUGJ52Vx4ASsDLooVK";
    const SITE_ID: &str = "89169de759a670a419cbe3d179e972c2cd03513f020570b296bd64e5b7904dc0";

    fn program() -> AccountId {
        bs58::decode(PROGRAM_B58).into_vec().unwrap().try_into().unwrap()
    }

    #[test]
    fn decodes_the_real_site_account() {
        let data = shard_of(SITE_RESPONSE, &program()).unwrap();
        let site = decode_site(&data).unwrap();
        assert_eq!(hex::encode(site.site_id), SITE_ID);
        // The Site lives at the address the rules seed gives: the PDA derivation is right.
        assert_eq!(b58(&site_pda(&program(), &site.site_id)), "Ag2ED4fjdAxuMUStZxnCXMMTdLRogtH6sewHNtUPYrPU");
        assert!(site.super_admins.iter().any(|a| a.is_some()));
        assert_eq!(Site::decode(&data).unwrap().encode(), data, "layout round trip");
    }

    #[test]
    fn shard_of_handles_empty_error_and_junk() {
        let other = [5u8; 32];
        assert!(shard_of(SITE_RESPONSE, &other).unwrap().is_empty());
        assert!(shard_of(r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"x"}}"#, &other).is_err());
        assert!(shard_of("nope", &other).is_err());
        assert!(shard_of(r#"{"result":{"data":{"shards":{}}}}"#, &other).unwrap().is_empty());
        assert!(decode_site(&[]).is_err());
        assert!(decode_member(&[1, 2, 3]).is_err());
    }

    #[test]
    fn pda_hash_is_the_lez_one() {
        // The hash input, spelled out. The real-site test shows it equals the address on the chain.
        let id = pda(&[1; 32], &[2; 32]);
        let mut input = b"/LEE/v0.2/AccountId/PDA/".to_vec();
        input.extend([0u8; 8]);
        input.extend([1u8; 32]);
        input.extend([2u8; 32]);
        assert_eq!(id, <[u8; 32]>::from(Sha256::digest(&input)));
        assert_ne!(site_pda(&[1; 32], &[2; 32]), member_pda(&[1; 32], &[2; 32], &[3; 32]));
        assert_ne!(checkpoint_pda(&[1; 32], &[2; 32], &[3; 32]), site_pda(&[1; 32], &[2; 32]));
    }

    fn member(site_id: [u8; 32], user: u8, key: u8, role: pukaar_sites_rules::Role) -> Member {
        Member {
            version: RECORD_VERSION,
            site_id,
            user: [user; 32],
            pukaar_key: [key; 32],
            role,
            status: pukaar_sites_rules::MemberStatus::Active,
            granted_by: [0; 32],
            granted_at_ms: 0,
            expires_at_ms: 0,
            notice: None,
            revoked: None,
            rights_override: 0,
            profile_cid: String::new(),
            profile_sha256: [0; 32],
            profile_version: 0,
            gate_attestation: None,
            request_id: [0; 32],
            requested_at_ms: 0,
            last_req_nonce: 0,
        }
    }

    #[test]
    fn roster_applies_records_and_answers_role_of() {
        use pukaar_sites_rules::Role;
        let prog = program();
        let site: Site = decode_site(&shard_of(SITE_RESPONSE, &prog).unwrap()).unwrap();
        let mut r = Roster::new(prog, site.site_id);
        r.apply(&site_pda(&prog, &site.site_id), &site.encode()).unwrap();
        let root = site.super_admins.iter().flatten().next().copied().unwrap();

        let put = |r: &mut Roster, m: &Member| r.apply(&member_pda(&prog, &m.site_id, &m.user), &m.encode());
        let mut sup = member(site.site_id, 0, 10, Role::Admin);
        sup.user = root;
        let res = member(site.site_id, 2, 20, Role::Resident);
        let mut gone = member(site.site_id, 3, 30, Role::Steward);
        gone.status = pukaar_sites_rules::MemberStatus::Removed;
        for m in [&sup, &res, &gone] {
            put(&mut r, m).unwrap();
        }
        assert_eq!(r.role_of(&root, 0), Some(Rank::SuperAdmin));
        assert_eq!(r.role_of(&[10; 32], 0), Some(Rank::SuperAdmin), "by Pukaar key");
        assert_eq!(r.role_of(&[2; 32], 0), Some(Rank::Resident));
        assert_eq!(r.role_of(&[20; 32], 0), Some(Rank::Resident));
        assert_eq!(r.role_of(&[3; 32], 0), None, "removed");
        assert_eq!(r.role_of(&[99; 32], 0), None, "stranger");
        assert_eq!(r.admins(0).len(), 1);

        // A promotion arrives as a new shard for the same address.
        let mut up = res.clone();
        up.role = Role::Steward;
        put(&mut r, &up).unwrap();
        assert_eq!(r.role_of(&[2; 32], 0), Some(Rank::Steward));

        // Wrong address, wrong site, junk: refused and nothing changes.
        assert!(r.apply(&[7; 32], &res.encode()).is_err());
        let foreign = member([1; 32], 4, 40, Role::Admin);
        assert!(put(&mut r, &foreign).is_err());
        assert!(r.apply(&member_pda(&prog, &site.site_id, &[4; 32]), &[1, 2]).is_err());
        assert_eq!(r.members().count(), 3);
        // An emptied shard removes the row.
        r.apply(&member_pda(&prog, &site.site_id, &[2; 32]), &[]).unwrap();
        assert_eq!(r.role_of(&[2; 32], 0), None);
    }

    /// The Site and the three members of the testnet scenario: t1 is the super admin after the
    /// transfer, t2 an admin, t3 revoked (member seed N is `[N; 32]`).
    #[test]
    fn real_accounts_fill_a_roster() {
        let prog = program();
        let site = decode_site(&shard_of(SITE_RESPONSE, &prog).unwrap()).unwrap();
        let mut r = Roster::new(prog, site.site_id);
        r.apply(&site_pda(&prog, &site.site_id), &site.encode()).unwrap();
        let files = [
            (1u8, include_str!("../tests/fixtures/getaccount_member_t1.json")),
            (2, include_str!("../tests/fixtures/getaccount_member_t2.json")),
            (3, include_str!("../tests/fixtures/getaccount_member_t3_revoked.json")),
        ];
        let acct = |n: u8| crate::lezkey::lez_account_id(&crate::lezkey::derive_lez_secret(&[n; 32]));
        for (n, f) in files {
            // The fixture sits at the member PDA of that user: `apply` checks it.
            let pda = member_pda(&prog, &site.site_id, &acct(n));
            r.apply(&pda, &shard_of(f, &prog).unwrap()).unwrap();
        }
        let key = |n: u8| ed25519_dalek::SigningKey::from_bytes(&[n; 32]).verifying_key().to_bytes();
        assert_eq!(r.role_of(&acct(1), 0), Some(Rank::SuperAdmin));
        assert_eq!(r.role_of(&key(2), 0), Some(Rank::Admin));
        assert_eq!(r.role_of(&acct(3), 0), None);
        let t3 = r.member(&acct(3)).unwrap();
        assert!(t3.revoked.is_some() && t3.profile_version == 1);
        assert_eq!(r.member(&acct(2)).unwrap().profile_version, 2);
        assert_eq!(r.admins(0).len(), 2);
        assert_eq!((site.key_epoch, site.roster_len), (2, 4));
    }
}
