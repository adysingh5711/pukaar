//! Thin adapter: the rules live in `pukaar_sites_rules`. Build with ../build.sh, never `cargo build`.
//!
//! Every call lists each account once, in any order: the signer (the one authorized account), the
//! clock shard, then PDAs. The plan finds each PDA by the id it derives itself with `for_public_pda`
//! and refuses a call with a missing, wrong or extra account.
//!   anchor        signer, signer Member, checkpoint, clock                       (4)
//!   create_site   signer, Site, creator Member, clock                            (4)
//!   set_details, set_places, set_keyring, super_admin_remove
//!                 signer, Site, signer Member, clock                             (4)
//!   approve_batch, set_role_batch, revoke_batch, set_profile_batch
//!                 the four above + one Member per target (the signer's own once) (4 + N)
//!   super_admin_add, super_admin_transfer: the four above + the target Member    (5)
//! Probe (feature `probe` only, never in a release build) takes, in this order: the CLOCK_01 clock shard if flagged, one native balance shard
//! if flagged, then `n_members` member PDAs. The fee payer co-signs outside the account list.

use clock_core::{CLOCK_01_PROGRAM_ACCOUNT_ID, clock_account_id};
use lee_core::{
    account::AccountId,
    program::{AccountMeta, PdaSeed, Plan, PlanInput, run_program},
};
#[cfg(feature = "probe")]
use lee_core::native_token::NATIVE_TOKEN_PROGRAM_ID;
use pukaar_sites_rules::*;

#[cfg(test)]
mod tests;

fn main() {
    run_program(plan, apply)
}

fn pda(input: &PlanInput, seed: [u8; 32]) -> AccountId {
    AccountId::for_public_pda(&input.self_account_id, &PdaSeed::new(seed))
}

/// The accounts of one call: each plan step takes its account by id and the call ends only if
/// the plan used all of them.
struct Ctx<'a> {
    input: &'a PlanInput,
    plan: Plan,
    used: Vec<AccountId>,
}

impl<'a> Ctx<'a> {
    fn new(input: &'a PlanInput) -> Self {
        Self { input, plan: Plan::new(input), used: vec![] }
    }

    fn acct(&mut self, id: AccountId, what: &str) -> AccountMeta {
        let m = self.input.accounts.iter().find(|m| m.account_id == id).unwrap_or_else(|| panic!("missing account: {what}"));
        if !self.used.contains(&id) {
            self.used.push(id);
        }
        m.clone()
    }

    fn pda_acct(&mut self, seed: [u8; 32], what: &str) -> AccountMeta {
        let id = pda(self.input, seed);
        self.acct(id, what)
    }

    fn site(&mut self, site_id: &[u8; 32]) -> AccountMeta {
        self.pda_acct(seed::site(site_id), "Site PDA")
    }

    fn member(&mut self, site_id: &[u8; 32], user: &[u8; 32]) -> AccountMeta {
        self.pda_acct(seed::member(site_id, user), "Member PDA")
    }

    /// The one authorized account.
    fn signer(&mut self) -> AccountId {
        let mut signed = self.input.accounts.iter().filter(|m| m.is_authorized);
        let id = signed.next().expect("no signer").account_id;
        assert!(signed.next().is_none(), "more than one signer");
        self.acct(id, "signer").account_id
    }

    fn clock(&mut self, now_ms: u64) {
        let clock = self.acct(CLOCK_01_PROGRAM_ACCOUNT_ID, "CLOCK_01");
        self.plan.inspect(&clock, clock_account_id(), &Effect::ClockIs(now_ms));
    }

    /// The start of every role instruction: clock claim, the signer's `Member` and the `Site` account.
    fn open(&mut self, auth: &Auth) -> (Actor, AccountMeta) {
        let signer = self.signer().into_value();
        let actor = Actor { account: signer, claimed: auth.claimed };
        self.clock(auth.now_ms);
        let m = self.member(&auth.site_id, &signer);
        self.plan.effect(&m, &Effect::Signer { site_id: auth.site_id, actor, now_ms: auth.now_ms });
        let site = self.site(&auth.site_id);
        (actor, site)
    }

    fn finish(self) -> Plan {
        assert_eq!(self.used.len(), self.input.accounts.len(), "too many accounts");
        self.plan
    }
}

fn nonempty<T>(items: &[T]) {
    assert!(!items.is_empty(), "plan: {}", RuleError::EmptyBatch);
}

fn plan(input: &PlanInput, instruction: Instruction) -> Plan {
    // A chained call from another program acts with the signer's authority, so a role instruction refuses it.
    // `caller_account_id` is `None` for a direct call of a user (LEZ v0.3.1).
    assert!(input.caller_account_id.is_none(), "plan: a call from another program is refused");
    #[cfg(feature = "probe")]
    if let Instruction::Probe(p) = instruction {
        return probe(input, p);
    }
    let mut c = Ctx::new(input);
    match instruction {
        #[cfg(feature = "probe")]
        Instruction::Probe(_) => unreachable!("handled above"),
        #[cfg(not(feature = "probe"))]
        Instruction::Reserved(_) => panic!("plan: instruction 1 is reserved: the probe is not in this build"),
        Instruction::Anchor(AnchorIx { record, now_ms }) => {
            check_anchor(&record).unwrap_or_else(|e| panic!("anchor: {e}"));
            let signer = c.signer().into_value();
            c.clock(now_ms);
            let m = c.member(&record.site_id, &signer);
            c.plan.effect(&m, &Effect::AnchorSigner { record: record.clone(), signer, now_ms });
            let cp = c.pda_acct(checkpoint_seed(&record.site_id, &record.heads_root), "checkpoint PDA");
            c.plan.effect(&cp, &Effect::Anchor(record));
        }
        Instruction::CreateSite(CreateSiteIx { args, now_ms }) => {
            assert_eq!(c.signer().into_value(), args.creator, "create_site: the creator must sign");
            c.clock(now_ms);
            let site = c.site(&args.site_id);
            let member = c.member(&args.site_id, &args.creator);
            c.plan.effect(&site, &Effect::CreateSite { args: args.clone(), now_ms });
            c.plan.effect(&member, &Effect::CreateMember { args, now_ms });
        }
        Instruction::SetDetails(ix) => set_pointer(&mut c, Pointer::Details, ix),
        Instruction::SetPlaces(ix) => set_pointer(&mut c, Pointer::Places, ix),
        Instruction::SetKeyring(ix) => set_pointer(&mut c, Pointer::Keyring, ix),
        Instruction::ApproveBatch(ix) => {
            nonempty(&ix.items);
            let (actor, site) = c.open(&ix.auth);
            let new_members = ix.items.iter().filter(|e| e.is_new).count();
            c.plan.effect(
                &site,
                &Effect::ApproveSite {
                    site_id: ix.auth.site_id,
                    actor,
                    roster_len: ix.roster_len,
                    new_members: u32::try_from(new_members).expect("batch fits u32"),
                    keyring_cid: ix.keyring_cid,
                    keyring_sha256: ix.keyring_sha256,
                    now_ms: ix.auth.now_ms,
                },
            );
            for e in ix.items {
                let body = JoinBody::parse(e.item.request[..JOIN_BODY_LEN].try_into().expect("162 bytes"))
                    .unwrap_or_else(|| panic!("approve: {}", RuleError::BadJoinRequest));
                let m = c.member(&ix.auth.site_id, &body.user_account);
                c.plan.effect(&m, &Effect::Approve { site_id: ix.auth.site_id, actor, item: e.item, is_new: e.is_new, now_ms: ix.auth.now_ms });
            }
        }
        Instruction::SetRoleBatch(ix) => {
            nonempty(&ix.items);
            let (actor, site) = c.open(&ix.auth);
            let not_super = ix.items.iter().map(|i| i.user).collect();
            c.plan.effect(&site, &Effect::SiteGuard { site_id: ix.auth.site_id, actor, not_super });
            for i in ix.items {
                let m = c.member(&ix.auth.site_id, &i.user);
                c.plan.effect(&m, &Effect::SetRole { actor, role: i.role, now_ms: ix.auth.now_ms });
            }
        }
        Instruction::RevokeBatch(ix) => {
            nonempty(&ix.items);
            let (actor, site) = c.open(&ix.auth);
            let targets = ix.items.iter().map(|i| i.user).collect();
            c.plan.effect(
                &site,
                &Effect::RevokeSite {
                    site_id: ix.auth.site_id,
                    actor,
                    targets,
                    keyring_cid: ix.keyring_cid,
                    keyring_sha256: ix.keyring_sha256,
                    now_ms: ix.auth.now_ms,
                },
            );
            for i in ix.items {
                let m = c.member(&ix.auth.site_id, &i.user);
                c.plan.effect(&m, &Effect::Revoke { actor, reason_sha256: i.reason_sha256, now_ms: ix.auth.now_ms });
            }
        }
        Instruction::SetProfileBatch(ix) => {
            nonempty(&ix.items);
            let (actor, site) = c.open(&ix.auth);
            c.plan.effect(&site, &Effect::SiteGuard { site_id: ix.auth.site_id, actor, not_super: vec![] });
            for i in ix.items {
                let m = c.member(&ix.auth.site_id, &i.user);
                c.plan.effect(&m, &Effect::SetProfile { actor, cid: i.cid, sha256: i.sha256, version: i.version, now_ms: ix.auth.now_ms });
            }
        }
        Instruction::SuperAdminAdd(ix) => {
            let (actor, site, target) = super_admin(&mut c, &ix, true);
            c.plan.effect(&site, &Effect::SuperAdd { site_id: ix.auth.site_id, actor, target: ix.target, now_ms: ix.auth.now_ms });
            c.plan.effect(&target.expect("target member"), &Effect::Promote { actor, now_ms: ix.auth.now_ms });
        }
        Instruction::SuperAdminRemove(ix) => {
            let (actor, site, _) = super_admin(&mut c, &ix, false);
            c.plan.effect(&site, &Effect::SuperRemove { site_id: ix.auth.site_id, actor, target: ix.target, now_ms: ix.auth.now_ms });
        }
        Instruction::SuperAdminTransfer(ix) => {
            let (actor, site, target) = super_admin(&mut c, &ix, true);
            c.plan.effect(&site, &Effect::SuperTransfer { site_id: ix.auth.site_id, actor, target: ix.target, now_ms: ix.auth.now_ms });
            c.plan.effect(&target.expect("target member"), &Effect::Promote { actor, now_ms: ix.auth.now_ms });
        }
    }
    c.finish()
}

fn set_pointer(c: &mut Ctx, kind: Pointer, ix: SetPointerIx) {
    let (actor, site) = c.open(&ix.auth);
    c.plan.effect(
        &site,
        &Effect::SetPointer { kind, site_id: ix.auth.site_id, actor, cid: ix.cid, sha256: ix.sha256, now_ms: ix.auth.now_ms },
    );
}

/// The common start of the three super admin instructions. `with_target` takes the target's `Member`.
fn super_admin(c: &mut Ctx, ix: &SuperAdminIx, with_target: bool) -> (Actor, AccountMeta, Option<AccountMeta>) {
    let (actor, site) = c.open(&ix.auth);
    let target = with_target.then(|| c.member(&ix.auth.site_id, &ix.target));
    (actor, site, target)
}

#[cfg(feature = "probe")]
fn probe(input: &PlanInput, p: Probe) -> Plan {
    let mut plan = Plan::new(input);
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
    plan
}

fn apply(effect: Effect, pre: &[u8]) -> Option<Vec<u8>> {
    apply_effect(&effect, pre).unwrap_or_else(|e| panic!("apply: {e}"))
}
