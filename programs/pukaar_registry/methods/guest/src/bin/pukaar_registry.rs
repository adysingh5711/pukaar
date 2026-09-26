#![no_main]

use spel_framework::prelude::*;

#[cfg(not(test))]
risc0_zkvm::guest::entry!(main);

/// One anchored checkpoint. The chain stores it; clients verify it off-chain
/// (root reproduces from their own log copy, sig_r||sig_s verifies, signer is a member).
#[account_type]
#[derive(Debug, Clone, Default, BorshSerialize, BorshDeserialize)]
pub struct CheckpointRecord {
    pub site_id: [u8; 32],
    pub heads_root: [u8; 32],
    pub n_events: u64,
    pub signer_pk: [u8; 32],
    pub sig_r: [u8; 32],
    pub sig_s: [u8; 32],
}

#[lez_program]
mod pukaar_registry {
    #[allow(unused_imports)]
    use super::*;

    /// One PDA per (site, root): anchoring the same root twice fails at `init`,
    /// and nothing is sequential, so a junk anchor can't block an honest one.
    #[instruction]
    pub fn anchor(
        #[account(init, pda = [literal("cp"), arg("site_id"), arg("heads_root")])]
        mut record: AccountWithMetadata,
        #[account(signer)]
        payer: AccountWithMetadata,
        site_id: [u8; 32],
        heads_root: [u8; 32],
        n_events: u64,
        signer_pk: [u8; 32],
        sig_r: [u8; 32],
        sig_s: [u8; 32],
    ) -> SpelResult {
        let rec = CheckpointRecord { site_id, heads_root, n_events, signer_pk, sig_r, sig_s };
        let bytes = borsh::to_vec(&rec).map_err(|e| SpelError::SerializationError { message: e.to_string() })?;
        record.account.data = bytes.try_into().unwrap();
        Ok(SpelOutput::execute(vec![record, payer], vec![]))
    }
}
