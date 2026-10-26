//! Thin adapter: the rules live in `pukaar_sites_rules`. Build with ../build.sh, never `cargo build`.
//!
//! Anchor takes one account: the checkpoint PDA of (this program, `checkpoint_seed`).
//! Probe (test only) takes, in this order: the CLOCK_01 clock shard if flagged, one native balance shard
//! if flagged, then `n_members` member PDAs. The fee payer co-signs outside the account list.

use clock_core::{CLOCK_01_PROGRAM_ACCOUNT_ID, clock_account_id};
use lee_core::{
    account::AccountId,
    native_token::NATIVE_TOKEN_PROGRAM_ID,
    program::{AccountMeta, PdaSeed, Plan, PlanInput, run_program},
};
use pukaar_sites_rules::*;

fn main() {
    run_program(plan, apply)
}

fn pda(input: &PlanInput, seed: [u8; 32]) -> AccountId {
    AccountId::for_public_pda(&input.self_account_id, &PdaSeed::new(seed))
}

fn plan(input: &PlanInput, instruction: Instruction) -> Plan {
    let mut plan = Plan::new(input);
    match instruction {
        Instruction::Anchor(rec) => {
            check_anchor(&rec).unwrap_or_else(|e| panic!("anchor: {e}"));
            let [record] = input.accounts.as_slice() else {
                panic!("anchor takes exactly one account: the checkpoint");
            };
            assert_eq!(
                record.account_id,
                pda(input, checkpoint_seed(&rec.site_id, &rec.heads_root)),
                "anchor: account is not the checkpoint PDA of (site, root)"
            );
            plan.effect(record, &Effect::Anchor(rec));
        }
        Instruction::Probe(p) => {
            check_probe(&p).unwrap_or_else(|e| panic!("probe: {e}"));
            let mut rest = input.accounts.iter();
            let mut next = |what: &str| -> &AccountMeta { rest.next().unwrap_or_else(|| panic!("probe: missing {what}")) };
            if p.flags & PROBE_CLOCK != 0 {
                let clock = next("clock");
                assert_eq!(clock.account_id, CLOCK_01_PROGRAM_ACCOUNT_ID, "probe: not CLOCK_01");
                plan.inspect(clock, clock_account_id(), &Effect::ClockIs(p.now_ms));
            }
            if p.flags & PROBE_BALANCE != 0 {
                plan.inspect(next("balance"), NATIVE_TOKEN_PROGRAM_ID, &Effect::BalanceAtLeast(p.min_balance));
            }
            if p.flags & PROBE_MEMBERS != 0 {
                for i in 0..p.n_members {
                    let m = next("member");
                    assert_eq!(m.account_id, pda(input, member_seed(&p.salt, i)), "probe: not member PDA {i}");
                    plan.effect(m, &Effect::Member(member_record(&p.salt, i)));
                }
            }
            assert!(rest.next().is_none(), "probe: too many accounts");
        }
    }
    plan
}

fn apply(effect: Effect, pre: &[u8]) -> Option<Vec<u8>> {
    apply_effect(&effect, pre).unwrap_or_else(|e| panic!("apply: {e}"))
}
