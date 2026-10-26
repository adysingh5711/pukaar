//! Host runner for pukaar_sites: one command per instruction. Run `runner --help` for the list.
//! Wallet and sequencer come from the environment:
//!   LEE_WALLET_HOME_DIR  wallet home (use the local one for local work)
//!   PUKAAR_PROGRAM_ID    header account id of the deployed program (base58)
//!   PUKAAR_PAYER         funded public account that pays the fee (base58)
//!   PUKAAR_SIGNER        who signs: a base58 id or `tN` (default: the payer). The wallet must hold its key.
//!
//! The spike sets signer_pk of an anchor to the pukaar key of the signer's Member and leaves sig_r and sig_s at zero: clients verify them off-chain.

use anyhow::{Context as _, Result, anyhow, bail};
use clock_core::{CLOCK_01_PROGRAM_ACCOUNT_ID, clock_account_id};
use ed25519_dalek::{Signer as _, SigningKey};
use hkdf::Hkdf;
use lee::{AccountId, PrivateKey, ProgramShardSelector, PublicKey, Signature};
use lee_core::program::PdaSeed;
use pukaar_sites_rules::{
    AnchorIx, ApproveBatchIx, ApproveEntry, ApproveItem, Auth, CheckpointRecord, CreateSite, CreateSiteIx, GateConfig, Instruction, JOIN_REQ_LEN,
    JoinBody, Member, ProfileItem, Rank, RevokeBatchIx, RevokeItem, Role, RoleItem, SetPointerIx, SetProfileBatchIx, SetRoleBatchIx, Site,
    SuperAdminIx, checkpoint_seed, member_seed, merkle_leaf, sample_probe, seed,
};
use sha2::{Digest as _, Sha256};
use wallet::{AccountIdentity, WalletCore};

const USAGE: &str = "usage: runner <command> [args] [key=value ...]
A `user` is a base58 account id or `tN` (test member N: Pukaar seed = N repeated 32 times, LEZ key derived from it).
Options for every instruction below: now=<ms> (claimed chain time, default the clock), claim=resident|steward|admin|super
(default: read from the chain), bad=pda (swap the last account for a wrong PDA).
Instructions (the signer is PUKAAR_SIGNER; the payer pays):
  create-site <salt_hex>                                  the signer creates a site and becomes its first member
  approve <site> <user>:<role>:<profile_cid>:<version> .. [roster=<n>] [keyring=<cid>]   approve_batch (user must be tN: it signs its join request)
  set-role <site> <user>:<role> ..                        set_role_batch
  revoke <site> <user> .. [keyring=<cid>]                 revoke_batch (key_epoch+1 and the new keyring pointer)
  set-profile <site> <user>:<cid>:<version> ..            set_profile_batch
  set-details|set-places|set-keyring <site> <cid>         the SHA-256 is random test bytes
  super-add|super-remove|super-transfer <site> <user>
  anchor <site> <heads_root_hex> [n_events]               the signer must be an active member
Reads and tools:
  get-site <site> [user]      Site, and the Member of `user` if given
  get-member <site> <user>
  get-checkpoint <site> <heads_root_hex>
  build-join-request <tN|seed_hex> <site> [nonce=<n>] [valid=<ms>]   prints the 290-byte request as hex
  member-key <tN>             prints the LEZ private key (hex) and account id, for `wallet account import public`
  clock                       chain time in ms
  probe <flags> [now=<ms>] [min=<balance>] [members=<n>] [salt=<hex32>] [bad=clock|balance|merkle|sig]   test-only gas probe, needs a guest built with `--features probe` (the default build refuses it)
  block-txs <from_block> <to_block>   serialized size of every tx in those blocks
Every instruction prints the tx hash, its block, and `fee=<lepta>` (payer balance before minus after).
A failed instruction still lands and pays: read the state back with get-site and get-member.";

fn id_env(name: &str) -> Result<AccountId> {
    std::env::var(name)
        .with_context(|| format!("set {name}"))?
        .parse()
        .map_err(|e| anyhow!("{name}: {e:?}"))
}

fn hex32(s: &str) -> Result<[u8; 32]> {
    hex::decode(s)?.try_into().map_err(|_| anyhow!("expected 64 hex chars, got {s}"))
}

/// The chain time the clock account holds, in ms.
async fn clock_ms(wallet: &WalletCore) -> Result<u64> {
    let clock_program = clock_account_id();
    let clock = wallet.get_account_view(ProgramShardSelector::new(CLOCK_01_PROGRAM_ACCOUNT_ID, clock_program)).await?;
    Ok(u64::from_le_bytes(clock.data.shard(clock_program).as_ref().get(8..16).context("clock not written")?.try_into()?))
}

fn pda(program: &AccountId, seed: [u8; 32]) -> AccountId {
    AccountId::for_public_pda(program, &PdaSeed::new(seed))
}

// ---- test members: the keys of modules/pukaar_core/rust-lib/logic/src/lezkey.rs, from a fixed seed ----

struct Keys {
    ed: SigningKey,
    lez: PrivateKey,
    schnorr_pk: [u8; 32],
    account: AccountId,
}

/// HKDF-SHA256(seed, no salt, info "pukaar/lez-key/v1"), as `derive_lez_secret`; the account id is as `lez_account_id`.
fn keys(seed: &[u8; 32]) -> Keys {
    const INFO: &[u8] = b"pukaar/lez-key/v1";
    let hk = Hkdf::<Sha256>::new(None, seed);
    let mut info = INFO.to_vec();
    let mut counter = 0u8;
    let lez = loop {
        let mut out = [0u8; 32];
        hk.expand(&info, &mut out).expect("32 bytes fit HKDF-SHA256");
        if let Ok(k) = PrivateKey::try_new(out) {
            break k;
        }
        counter += 1;
        info.truncate(INFO.len());
        info.push(counter);
    };
    let pk = PublicKey::new_from_private_key(&lez);
    Keys { ed: SigningKey::from_bytes(seed), schnorr_pk: *pk.value(), account: AccountId::from(&pk), lez }
}

/// `tN`: the Pukaar seed of test member N.
fn test_seed(spec: &str) -> Option<[u8; 32]> {
    Some([spec.strip_prefix('t')?.parse().ok()?; 32])
}

fn user(spec: &str) -> Result<AccountId> {
    match test_seed(spec) {
        Some(s) => Ok(keys(&s).account),
        None => spec.parse().map_err(|e| anyhow!("user {spec}: {e:?}")),
    }
}

/// The 290-byte join request: body, ed25519 signature of the body, BIP340 signature of sha256(body).
fn join_request(k: &Keys, site_id: [u8; 32], nonce: u64, valid_until_ms: u64) -> Result<[u8; JOIN_REQ_LEN]> {
    let body = JoinBody {
        site_id,
        user_account: k.account.into_value(),
        user_schnorr_pk: k.schnorr_pk,
        pukaar_ed_pk: k.ed.verifying_key().to_bytes(),
        nonce,
        valid_until_ms,
    }
    .encode();
    let sig_schnorr = Signature::new(&k.lez, &Sha256::digest(body).into()).value;
    [&body[..], &k.ed.sign(&body).to_bytes(), &sig_schnorr].concat().try_into().map_err(|_| anyhow!("request is not {JOIN_REQ_LEN} bytes"))
}

/// Random test bytes for a SHA-256 field.
fn rnd() -> [u8; 32] {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Sha256::digest([&nanos.to_le_bytes()[..], &n.to_le_bytes()].concat()).into()
}

fn role(s: &str) -> Result<Role> {
    Ok(match s {
        "resident" => Role::Resident,
        "steward" => Role::Steward,
        "admin" => Role::Admin,
        _ => bail!("role {s}: resident|steward|admin"),
    })
}

/// The value of `key=value` among the arguments.
fn opt<'a>(args: &'a [String], k: &str) -> Option<&'a str> {
    args.iter().find_map(|a| a.strip_prefix(k)?.strip_prefix('='))
}

// ---- one transaction ----

struct Cx {
    wallet: WalletCore,
    program: AccountId,
    payer: AccountId,
    signer: AccountId,
}

impl Cx {
    fn new() -> Result<Self> {
        let payer = id_env("PUKAAR_PAYER")?;
        let signer = std::env::var("PUKAAR_SIGNER").map_or(Ok(payer), |s| user(&s))?;
        Ok(Self { wallet: WalletCore::from_env()?, program: id_env("PUKAAR_PROGRAM_ID")?, payer, signer })
    }

    async fn shard(&self, id: AccountId) -> Result<Vec<u8>> {
        let acc = self.wallet.get_account_view(ProgramShardSelector::new(id, self.program)).await?;
        Ok(acc.data.shard(self.program).as_ref().to_vec())
    }

    fn site_pda(&self, site_id: &[u8; 32]) -> AccountId {
        pda(&self.program, seed::site(site_id))
    }

    fn member_pda(&self, site_id: &[u8; 32], user: &AccountId) -> AccountId {
        pda(&self.program, seed::member(site_id, &user.into_value()))
    }

    async fn site(&self, site_id: &[u8; 32]) -> Result<Option<Site>> {
        Ok(Site::decode(&self.shard(self.site_pda(site_id)).await?))
    }

    async fn member(&self, site_id: &[u8; 32], user: &AccountId) -> Result<Option<Member>> {
        Ok(Member::decode(&self.shard(self.member_pda(site_id, user)).await?))
    }

    /// The signer's claim: `now` and `claim` from the arguments, else the clock and the rank on the chain.
    async fn auth(&self, site_id: [u8; 32], args: &[String]) -> Result<Auth> {
        let now_ms = match opt(args, "now") {
            Some(n) => n.parse()?,
            None => clock_ms(&self.wallet).await?,
        };
        let claimed = match opt(args, "claim") {
            Some("super") => Rank::SuperAdmin,
            Some(r) => role(r)?.into(),
            None if self.site(&site_id).await?.is_some_and(|s| s.is_super(&self.signer.into_value())) => Rank::SuperAdmin,
            None => self.member(&site_id, &self.signer).await?.map_or(Rank::Resident, |m| m.role.into()),
        };
        Ok(Auth { site_id, now_ms, claimed })
    }

    /// Sign with the signer, mention the clock and the PDAs and wait until the tx is in a block. The payer pays.
    async fn send(&self, mut pdas: Vec<AccountId>, ix: Instruction, args: &[String]) -> Result<()> {
        if opt(args, "bad") == Some("pda") {
            *pdas.last_mut().context("no account to swap")? = self.member_pda(&[0xEE; 32], &AccountId::new([0xEE; 32]));
        }
        let mut uniq: Vec<AccountId> = vec![];
        for p in pdas {
            if !uniq.contains(&p) {
                uniq.push(p);
            }
        }
        let mut mentions = vec![
            AccountIdentity::Public(self.signer).select_program_shard(self.program),
            AccountIdentity::PublicNoSign(CLOCK_01_PROGRAM_ACCOUNT_ID).select_program_shard(clock_account_id()),
        ];
        mentions.extend(uniq.iter().map(|p| AccountIdentity::PublicNoSign(*p).select_program_shard(self.program)));
        let before = self.wallet.get_account_balance(self.payer).await?;
        let hash = self
            .wallet
            .send_pub_tx_paid_by(mentions, borsh::to_vec(&ix)?, self.program, Some(self.payer))
            .await
            .map_err(|e| anyhow!("send failed: {e:?}"))?;
        println!("tx: {hash}");
        // Included does not mean applied: a panicking program still lands, so the caller reads the account back.
        let (_, block) = self.wallet.poll_transaction(hash).await?;
        println!("included in block {block}");
        println!("fee={}", before - self.wallet.get_account_balance(self.payer).await?);
        Ok(())
    }

    /// A role instruction: the Site and the signer's Member, then the Member of every target.
    async fn send_site(&self, site_id: [u8; 32], targets: &[AccountId], args: &[String], ix: impl FnOnce(Auth) -> Instruction) -> Result<()> {
        let mut pdas = vec![self.site_pda(&site_id), self.member_pda(&site_id, &self.signer)];
        pdas.extend(targets.iter().map(|t| self.member_pda(&site_id, t)));
        self.send(pdas, ix(self.auth(site_id, args).await?), args).await
    }
}

/// Send one probe and print the numbers the gas formula needs.
async fn probe(wallet: &WalletCore, program: AccountId, rest: &[String]) -> Result<()> {
    let opt = |k: &str| rest.iter().skip(1).find_map(|a| a.strip_prefix(&format!("{k}=")));
    let flags: u8 = rest.first().context("probe needs <flags>")?.parse()?;
    let payer = id_env("PUKAAR_PAYER")?;
    let clock_program = clock_account_id();
    let clock_ms = clock_ms(wallet).await?;
    let n_members: u16 = opt("members").map_or(Ok(0), str::parse)?;
    let salt = match opt("salt") {
        Some(h) => hex32(h)?,
        None => merkle_leaf(&std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos().to_le_bytes()),
    };
    let mut p = sample_probe(
        flags,
        opt("now").map_or(Ok(clock_ms), str::parse)?,
        opt("min").map_or(Ok(1), str::parse)?,
        n_members,
        salt,
    );
    match opt("bad") {
        Some("clock") => p.now_ms = clock_ms + 1_000_000,
        Some("balance") => p.min_balance = u128::MAX,
        Some("merkle") => p.root[0] ^= 1,
        Some("sig") => p.msg = vec![1],
        Some(other) => bail!("unknown bad={other}"),
        None => {}
    }
    let mut mentions = vec![];
    if flags & 1 != 0 {
        mentions.push(AccountIdentity::PublicNoSign(CLOCK_01_PROGRAM_ACCOUNT_ID).select_program_shard(clock_program));
    }
    if flags & 2 != 0 {
        mentions.push(AccountIdentity::PublicNoSign(id_env("PUKAAR_BALANCE_ACCT")?).balance());
    }
    let mut member_pdas = vec![];
    if flags & 16 != 0 {
        for i in 0..n_members {
            let pda = AccountId::for_public_pda(&program, &PdaSeed::new(member_seed(&salt, i)));
            mentions.push(AccountIdentity::PublicNoSign(pda).select_program_shard(program));
            member_pdas.push(pda);
        }
    }
    let data = borsh::to_vec(&Instruction::Probe(p))?;
    let before = wallet.get_account_balance(payer).await?;
    let hash = wallet
        .send_pub_tx_paid_by(mentions, data, program, Some(payer))
        .await
        .map_err(|e| anyhow::anyhow!("send failed: {e:?}"))?;
    let (tx, block) = wallet.poll_transaction(hash).await?;
    let after = wallet.get_account_balance(payer).await?;
    println!("flags={flags} members={n_members} salt={}", hex::encode(salt));
    println!("tx: {hash} block {block}");
    println!("tx_bytes={}", borsh::to_vec(&tx)?.len());
    println!("payer_before={before} payer_after={after} fee={}", before - after);
    if let Some(last) = member_pdas.last() {
        let acc = wallet.get_account_view(ProgramShardSelector::new(*last, program)).await?;
        println!("last member shard length: {}", acc.data.shard(program).as_ref().len());
    }
    Ok(())
}

/// `N` fields per item, separated by `:`.
fn items<'a, const N: usize>(pos: &[&'a str]) -> Result<Vec<[&'a str; N]>> {
    pos.iter().map(|s| s.split(':').collect::<Vec<_>>().try_into().map_err(|_| anyhow!("item {s}: expected {N} fields separated by :"))).collect()
}

fn users<const N: usize>(items: &[[&str; N]]) -> Result<Vec<AccountId>> {
    items.iter().map(|i| user(i[0])).collect()
}

/// One instruction or read.
async fn run(cx: &Cx, cmd: &str, args: &[String]) -> Result<()> {
    let pos: Vec<&str> = args.iter().filter(|a| !a.contains('=')).map(String::as_str).collect();
    let h = hex32(pos.first().context("missing the first argument, see --help")?)?;
    let rest = &pos[1..];
    let keyring = opt(args, "keyring").unwrap_or("keyringtest").to_string();
    match cmd {
        "create-site" => {
            let (creator, salt) = (cx.signer.into_value(), h);
            let site_id = seed::site_id(&creator, &salt);
            let args_ = CreateSite {
                creator,
                salt,
                site_id,
                pukaar_key: [0xAA; 32],
                details_cid: "details1".into(),
                details_sha256: [11; 32],
                places_cid: "places1".into(),
                places_sha256: [12; 32],
                keyring_cid: "keyring1".into(),
                keyring_sha256: [13; 32],
                gates: GateConfig::default(),
                access_duration_ms: 0,
                renew_window_ms: 0,
            };
            println!("site_id: {}", hex::encode(site_id));
            let pdas = vec![cx.site_pda(&site_id), cx.member_pda(&site_id, &cx.signer)];
            cx.send(pdas, Instruction::CreateSite(CreateSiteIx { args: args_, now_ms: cx.auth(site_id, args).await?.now_ms }), args).await
        }
        "approve" => {
            let site = cx.site(&h).await?.context("no such site")?;
            let valid_until_ms = clock_ms(&cx.wallet).await? + 3_600_000;
            let (mut entries, mut targets) = (vec![], vec![]);
            for [who, r, cid, ver] in items::<4>(rest)? {
                let k = keys(&test_seed(who).with_context(|| format!("approve needs a tN user, got {who}"))?);
                let old = cx.member(&h, &k.account).await?;
                let request = join_request(&k, h, old.as_ref().map_or(1, |m| m.last_req_nonce + 1), valid_until_ms)?;
                let item = ApproveItem { request, role: role(r)?, profile_cid: cid.into(), profile_sha256: rnd(), profile_version: ver.parse()? };
                entries.push(ApproveEntry { item, is_new: old.is_none() });
                targets.push(k.account);
            }
            let roster_len = opt(args, "roster").map_or(Ok(site.roster_len), str::parse)?;
            cx.send_site(h, &targets, args, |auth| {
                Instruction::ApproveBatch(ApproveBatchIx { auth, roster_len, keyring_cid: keyring, keyring_sha256: rnd(), items: entries })
            })
            .await
        }
        "set-role" => {
            let it = items::<2>(rest)?;
            let targets = users(&it)?;
            let items = it.iter().zip(&targets).map(|([_, r], t)| Ok(RoleItem { user: t.into_value(), role: role(r)? })).collect::<Result<_>>()?;
            cx.send_site(h, &targets, args, |auth| Instruction::SetRoleBatch(SetRoleBatchIx { auth, items })).await
        }
        "revoke" => {
            let targets = users(&items::<1>(rest)?)?;
            let items = targets.iter().map(|t| RevokeItem { user: t.into_value(), reason_sha256: rnd() }).collect();
            cx.send_site(h, &targets, args, |auth| {
                Instruction::RevokeBatch(RevokeBatchIx { auth, keyring_cid: keyring, keyring_sha256: rnd(), items })
            })
            .await
        }
        "set-profile" => {
            let it = items::<3>(rest)?;
            let targets = users(&it)?;
            let items = it
                .iter()
                .zip(&targets)
                .map(|([_, cid, ver], t)| Ok(ProfileItem { user: t.into_value(), cid: (*cid).into(), sha256: rnd(), version: ver.parse()? }))
                .collect::<Result<_>>()?;
            cx.send_site(h, &targets, args, |auth| Instruction::SetProfileBatch(SetProfileBatchIx { auth, items })).await
        }
        "set-details" | "set-places" | "set-keyring" => {
            let ix = |auth| SetPointerIx { auth, cid: rest.first().unwrap_or(&"").to_string(), sha256: rnd() };
            cx.send_site(h, &[], args, |auth| match cmd {
                "set-details" => Instruction::SetDetails(ix(auth)),
                "set-places" => Instruction::SetPlaces(ix(auth)),
                _ => Instruction::SetKeyring(ix(auth)),
            })
            .await
        }
        "super-add" | "super-remove" | "super-transfer" => {
            let target = user(rest.first().context("missing <user>")?)?;
            let sa = |auth| SuperAdminIx { auth, target: target.into_value() };
            let targets: &[AccountId] = if cmd == "super-remove" { &[] } else { &[target] };
            cx.send_site(h, targets, args, |auth| match cmd {
                "super-add" => Instruction::SuperAdminAdd(sa(auth)),
                "super-remove" => Instruction::SuperAdminRemove(sa(auth)),
                _ => Instruction::SuperAdminTransfer(sa(auth)),
            })
            .await
        }
        "anchor" | "get-checkpoint" => {
            let heads_root = hex32(rest.first().context("missing <heads_root_hex>")?)?;
            let cp = pda(&cx.program, checkpoint_seed(&h, &heads_root));
            println!("checkpoint account: {cp}");
            if cmd == "get-checkpoint" {
                let bytes = cx.shard(cp).await?;
                println!("shard length: {}", bytes.len());
                match CheckpointRecord::decode(&bytes) {
                    Some(r) => println!("{r:?}"),
                    None => println!("no record"),
                }
                return Ok(());
            }
            // signer_pk must be the pukaar key of the signer's Member. A stranger has none: the program refuses it.
            let signer_pk = cx.member(&h, &cx.signer).await?.map_or([0; 32], |m| m.pukaar_key);
            let record = CheckpointRecord {
                site_id: h,
                heads_root,
                n_events: rest.get(1).map_or(Ok(0), |n| n.parse())?,
                signer_pk,
                sig_r: [0; 32],
                sig_s: [0; 32],
            };
            let ix = Instruction::Anchor(AnchorIx { record, now_ms: cx.auth(h, args).await?.now_ms });
            cx.send(vec![cx.member_pda(&h, &cx.signer), cp], ix, args).await
        }
        "get-site" => {
            let site = cx.site(&h).await?;
            println!("{site:?}");
            if let Some(s) = site {
                println!("super_admins: {:?}", s.super_admins.map(|a| a.map(|a| AccountId::new(a).to_string())));
            }
            if let Some(u) = rest.first() {
                println!("{:?}", cx.member(&h, &user(u)?).await?);
            }
            Ok(())
        }
        "get-member" => {
            println!("{:?}", cx.member(&h, &user(rest.first().context("missing <user>")?)?).await?);
            Ok(())
        }
        other => bail!("unknown command {other}, see --help"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => bail!("{USAGE}"),
        [h] if matches!(h.as_str(), "--help" | "-h" | "help") => println!("{USAGE}"),
        [cmd, who] if cmd == "member-key" => {
            let k = keys(&test_seed(who).context("member-key <tN>")?);
            println!("{} {}", k.lez, k.account);
        }
        [cmd] if cmd == "clock" => println!("{}", clock_ms(&WalletCore::from_env()?).await?),
        [cmd, who, site, rest @ ..] if cmd == "build-join-request" => {
            let seed = test_seed(who).map_or_else(|| hex32(who), Ok)?;
            let valid = match opt(rest, "valid") {
                Some(v) => v.parse()?,
                None => clock_ms(&WalletCore::from_env()?).await? + 3_600_000,
            };
            println!("{}", hex::encode(join_request(&keys(&seed), hex32(site)?, opt(rest, "nonce").map_or(Ok(1), str::parse)?, valid)?));
        }
        [cmd, rest @ ..] if cmd == "probe" => probe(&WalletCore::from_env()?, id_env("PUKAAR_PROGRAM_ID")?, rest).await?,
        [cmd, from, to] if cmd == "block-txs" => {
            let wallet = WalletCore::from_env()?;
            for id in from.parse::<u64>()?..=to.parse()? {
                for tx in wallet.get_block(id).await?.context("no such block")?.body.transactions {
                    println!("block {id} tx_bytes={}", borsh::to_vec(&tx)?.len());
                }
            }
        }
        [cmd, rest @ ..] => run(&Cx::new()?, cmd, rest).await?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pukaar_sites_rules::{check_join_request, seed};

    #[test]
    fn key_derivation_matches_lezkey_rs() {
        // lezkey.rs test vector: seed 00 01 .. 1f gives this account id.
        let seed: [u8; 32] = std::array::from_fn(|i| i as u8);
        assert_eq!(keys(&seed).account.to_string(), "AVi7Fm67WkAqJwLVD7PeA7KisBNuA2o6ew1CxmdtGrGP");
    }

    #[test]
    fn join_request_passes_the_rules_and_both_signatures_verify() {
        let k = keys(&[3; 32]);
        let site_id = seed::site_id(&[9; 32], &[8; 32]);
        let req = join_request(&k, site_id, 1, 1_000).unwrap();
        let body = check_join_request(&req, &site_id, None, 500).unwrap();
        assert_eq!(body.user_account, k.account.into_value());
        k.ed.verifying_key().verify_strict(&req[..162], &ed25519_dalek::Signature::from_bytes(req[162..226].try_into().unwrap())).unwrap();
        let sig = Signature { value: req[226..].try_into().unwrap() };
        assert!(sig.is_valid_for(&Sha256::digest(&req[..162]).into(), &PublicKey::try_new(k.schnorr_pk).unwrap()));
    }
}
