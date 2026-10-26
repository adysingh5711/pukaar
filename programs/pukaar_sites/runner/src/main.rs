//! Host runner for pukaar_sites. Wallet and sequencer come from the environment:
//!   LEE_WALLET_HOME_DIR  wallet home (use the local one for local work)
//!   PUKAAR_PROGRAM_ID    header account id of the deployed program (base58)
//!   PUKAAR_PAYER         funded public account that pays the fee (base58)
//!
//!   runner anchor <site_id_hex> <heads_root_hex> [n_events]
//!   runner get-checkpoint <site_id_hex> <heads_root_hex>
//!   runner probe <flags> [now=<ms>] [min=<balance>] [members=<n>] [salt=<hex32>] [bad=clock|balance|merkle|sig]
//!     Test-only gas probe. flags: 1 clock, 2 native balance, 4 merkle, 8 ed25519, 16 members (add them up).
//!     The balance step reads PUKAAR_BALANCE_ACCT (any funded public account that is not the payer).
//!     Prints payer balance before and after, the fee, and the serialized tx size, for the gas formula.
//!   runner block-txs <from_block> <to_block>   serialized size of every tx in those blocks
//!
//! The spike leaves signer_pk, sig_r and sig_s at zero: clients verify them off-chain.

use anyhow::{Context as _, Result, bail};
use clock_core::{CLOCK_01_PROGRAM_ACCOUNT_ID, clock_account_id};
use lee::{AccountId, ProgramShardSelector};
use lee_core::program::PdaSeed;
use pukaar_sites_rules::{CheckpointRecord, Instruction, checkpoint_seed, member_seed, merkle_leaf, sample_probe};
use wallet::{AccountIdentity, WalletCore};

fn id_env(name: &str) -> Result<AccountId> {
    std::env::var(name)
        .with_context(|| format!("set {name}"))?
        .parse()
        .map_err(|e| anyhow::anyhow!("{name}: {e:?}"))
}

fn hex32(s: &str) -> Result<[u8; 32]> {
    hex::decode(s)?.try_into().map_err(|_| anyhow::anyhow!("expected 64 hex chars, got {s}"))
}

/// Send one probe and print the numbers the gas formula needs.
async fn probe(wallet: &WalletCore, program: AccountId, rest: &[String]) -> Result<()> {
    let opt = |k: &str| rest.iter().skip(1).find_map(|a| a.strip_prefix(&format!("{k}=")));
    let flags: u8 = rest.first().context("probe needs <flags>")?.parse()?;
    let payer = id_env("PUKAAR_PAYER")?;
    let clock_program = clock_account_id();
    let clock = wallet.get_account_view(ProgramShardSelector::new(CLOCK_01_PROGRAM_ACCOUNT_ID, clock_program)).await?;
    let clock_ms = u64::from_le_bytes(clock.data.shard(clock_program).as_ref().get(8..16).context("clock not written")?.try_into()?);
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

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let [cmd, rest @ ..] = args.as_slice() && cmd == "probe" {
        return probe(&WalletCore::from_env()?, id_env("PUKAAR_PROGRAM_ID")?, rest).await;
    }
    if let [cmd, from, to] = args.as_slice() && cmd == "block-txs" {
        let wallet = WalletCore::from_env()?;
        for id in from.parse::<u64>()?..=to.parse()? {
            for tx in wallet.get_block(id).await?.context("no such block")?.body.transactions {
                println!("block {id} tx_bytes={}", borsh::to_vec(&tx)?.len());
            }
        }
        return Ok(());
    }
    let [cmd, site, root, rest @ ..] = args.as_slice() else {
        bail!("usage: runner anchor|get-checkpoint <site_id_hex> <heads_root_hex> [n_events]");
    };
    let (site_id, heads_root) = (hex32(site)?, hex32(root)?);
    let program = id_env("PUKAAR_PROGRAM_ID")?;
    let pda = AccountId::for_public_pda(&program, &PdaSeed::new(checkpoint_seed(&site_id, &heads_root)));
    let wallet = WalletCore::from_env()?;

    match cmd.as_str() {
        "anchor" => {
            let rec = CheckpointRecord {
                site_id,
                heads_root,
                n_events: rest.first().map_or(Ok(0), |n| n.parse())?,
                signer_pk: [0; 32],
                sig_r: [0; 32],
                sig_s: [0; 32],
            };
            let data = borsh::to_vec(&Instruction::Anchor(rec))?;
            let mention = AccountIdentity::PublicNoSign(pda).select_program_shard(program);
            let hash = wallet
                .send_pub_tx_paid_by(vec![mention], data, program, Some(id_env("PUKAAR_PAYER")?))
                .await
                .map_err(|e| anyhow::anyhow!("send failed: {e:?}"))?;
            println!("checkpoint account: {pda}");
            println!("tx: {hash}");
            // Included does not mean applied: a panicking program still lands, so read the account back.
            let (_, block) = wallet.poll_transaction(hash).await?;
            println!("included in block {block}");
        }
        "get-checkpoint" => {
            let account = wallet.get_account_view(ProgramShardSelector::new(pda, program)).await?;
            let bytes: &[u8] = account.data.shard(program).as_ref();
            println!("checkpoint account: {pda}");
            println!("shard length: {}", bytes.len());
            match CheckpointRecord::decode(bytes) {
                Some(r) => println!("{r:?}"),
                None => println!("no record"),
            }
        }
        other => bail!("unknown command {other}"),
    }
    Ok(())
}
