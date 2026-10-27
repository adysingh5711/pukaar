//! Instruction builders: the bytes and the accounts of every chain write. Pure, no network.
//!
//! The reference is `programs/pukaar_sites/runner/src/main.rs`. An instruction is the Borsh form of
//! `pukaar_sites_rules::Instruction`. The accounts are, in this order: the signer (signs, in the
//! shard of the program), the clock `CLOCK_01` (in the shard of the clock program), then the PDAs
//! that the instruction touches (the Site, the signer's Member, the Member of every target), with
//! doubles removed. The signer pays the fee (O1: the derived account is the one account).
//! The caller reads the chain clock and the claimed rank, and gives them in `Ctx`.

use crate::chain::{checkpoint_pda, member_pda, site_pda, AccountId};
use crate::event::{sha256, Location, OTHER_LOCATION};
use crate::fetch::{Details, Places, DETAILS_SCHEMA};
use pukaar_sites_rules::{
    seed, AnchorIx, ApproveBatchIx, ApproveEntry, ApproveItem, Auth, CheckpointRecord, CreateSite, CreateSiteIx, GateConfig, Instruction,
    JoinBody, Pointer, ProfileItem, Rank, RevokeBatchIx, RevokeItem, Role, RoleItem, SetPointerIx, SetProfileBatchIx, SetRoleBatchIx,
    SuperAdminIx, JOIN_BODY_LEN,
};
use sha2::{Digest, Sha256};

/// The most items one batch carries until the gas is measured (plan: batches of 10).
pub const BATCH: usize = 10;

/// `CLOCK_01_PROGRAM_ACCOUNT_ID` of LEZ v0.3.1 (`clock_core`).
pub const CLOCK_ACCOUNT: AccountId = *b"/LEZ/ClockProgramAccount/0000001";

/// The id of the clock program, the shard that the clock account is read in:
/// sha256("/LEE-BuiltinProgram/v1/AccountId" ‖ "clock") (`AccountId::from_builtin_program_name`).
pub fn clock_program() -> AccountId {
    Sha256::new().chain_update(b"/LEE-BuiltinProgram/v1/AccountId").chain_update(b"clock").finalize().into()
}

/// One account of a transaction. `shard` is the program whose shard of the account is meant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mention {
    pub id: AccountId,
    pub sign: bool,
    pub shard: AccountId,
}

/// A transaction that is ready to send.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tx {
    pub program: AccountId,
    pub ix: Vec<u8>,
    pub accounts: Vec<Mention>,
    /// Pays the fee: the signer.
    pub payer: AccountId,
}

/// What every builder needs from the chain and from the user.
#[derive(Clone, Debug)]
pub struct Ctx {
    pub program: AccountId,
    pub site_id: [u8; 32],
    /// Our LEZ account.
    pub signer: AccountId,
    /// The rank we hold on the chain: a super admin claims `SuperAdmin`.
    pub claimed: Rank,
    /// The `now_ms` claim (`rpc::claim_ms`): the chain clock minus a margin.
    pub now_ms: u64,
}

impl Ctx {
    fn auth(&self) -> Auth {
        Auth { site_id: self.site_id, now_ms: self.now_ms, claimed: self.claimed }
    }

    /// The Site, our Member, then the Member of every target: the accounts of a role instruction.
    fn site_accounts(&self, targets: &[AccountId]) -> Vec<AccountId> {
        let mut v = vec![site_pda(&self.program, &self.site_id), member_pda(&self.program, &self.site_id, &self.signer)];
        v.extend(targets.iter().map(|t| member_pda(&self.program, &self.site_id, t)));
        v
    }

    fn tx(&self, ix: Instruction, pdas: Vec<AccountId>) -> Tx {
        let mut uniq: Vec<AccountId> = vec![];
        for p in pdas {
            if !uniq.contains(&p) {
                uniq.push(p);
            }
        }
        let mut accounts = vec![
            Mention { id: self.signer, sign: true, shard: self.program },
            Mention { id: CLOCK_ACCOUNT, sign: false, shard: clock_program() },
        ];
        accounts.extend(uniq.into_iter().map(|id| Mention { id, sign: false, shard: self.program }));
        Tx { program: self.program, ix: borsh::to_vec(&ix).expect("borsh"), accounts, payer: self.signer }
    }
}

/// A file in Storage that the chain points to: its CID and the SHA-256 of its bytes.
pub type Pointed = (String, [u8; 32]);

/// `create_site`: the signer makes the site and becomes its first member (Admin and super admin).
pub fn create_site(ctx: &Ctx, salt: [u8; 32], pukaar_key: [u8; 32], details: Pointed, places: Pointed, keyring: Pointed) -> Tx {
    let creator = ctx.signer;
    let site_id = seed::site_id(&creator, &salt);
    let args = CreateSite {
        creator,
        salt,
        site_id,
        pukaar_key,
        details_cid: details.0,
        details_sha256: details.1,
        places_cid: places.0,
        places_sha256: places.1,
        keyring_cid: keyring.0,
        keyring_sha256: keyring.1,
        gates: GateConfig::default(),
        access_duration_ms: 0,
        renew_window_ms: 0,
    };
    let pdas = vec![site_pda(&ctx.program, &site_id), member_pda(&ctx.program, &site_id, &creator)];
    ctx.tx(Instruction::CreateSite(CreateSiteIx { args, now_ms: ctx.now_ms }), pdas)
}

/// The LEZ account that a join request names.
pub fn requester(item: &ApproveItem) -> Result<AccountId, String> {
    let body: &[u8; JOIN_BODY_LEN] = item.request[..JOIN_BODY_LEN].try_into().expect("162 bytes");
    Ok(JoinBody::parse(body).ok_or("bad join request")?.user_account)
}

/// `approve_batch`: `items` are (item, is_new). `roster_len` is the one read from the Site, and the
/// keyring is the new one that holds the Seat of every approved member.
pub fn approve_batch(ctx: &Ctx, roster_len: u32, keyring: Pointed, items: Vec<(ApproveItem, bool)>) -> Result<Tx, String> {
    let targets = items.iter().map(|(i, _)| requester(i)).collect::<Result<Vec<_>, _>>()?;
    let items = items.into_iter().map(|(item, is_new)| ApproveEntry { item, is_new }).collect();
    let ix = ApproveBatchIx { auth: ctx.auth(), roster_len, keyring_cid: keyring.0, keyring_sha256: keyring.1, items };
    Ok(ctx.tx(Instruction::ApproveBatch(ix), ctx.site_accounts(&targets)))
}

pub fn set_role_batch(ctx: &Ctx, items: Vec<(AccountId, Role)>) -> Tx {
    let targets: Vec<AccountId> = items.iter().map(|(u, _)| *u).collect();
    let items = items.into_iter().map(|(user, role)| RoleItem { user, role }).collect();
    ctx.tx(Instruction::SetRoleBatch(SetRoleBatchIx { auth: ctx.auth(), items }), ctx.site_accounts(&targets))
}

/// `revoke_batch`: `items` are (account, SHA-256 of the reason text). The text goes to `details.json`.
pub fn revoke_batch(ctx: &Ctx, keyring: Pointed, items: Vec<(AccountId, [u8; 32])>) -> Tx {
    let targets: Vec<AccountId> = items.iter().map(|(u, _)| *u).collect();
    let items = items.into_iter().map(|(user, reason_sha256)| RevokeItem { user, reason_sha256 }).collect();
    let ix = RevokeBatchIx { auth: ctx.auth(), keyring_cid: keyring.0, keyring_sha256: keyring.1, items };
    ctx.tx(Instruction::RevokeBatch(ix), ctx.site_accounts(&targets))
}

pub fn set_profile_batch(ctx: &Ctx, items: Vec<ProfileItem>) -> Tx {
    let targets: Vec<AccountId> = items.iter().map(|i| i.user).collect();
    ctx.tx(Instruction::SetProfileBatch(SetProfileBatchIx { auth: ctx.auth(), items }), ctx.site_accounts(&targets))
}

/// `set_details`, `set_places` or `set_keyring`.
pub fn set_pointer(ctx: &Ctx, kind: Pointer, to: Pointed) -> Tx {
    let ix = SetPointerIx { auth: ctx.auth(), cid: to.0, sha256: to.1 };
    let ix = match kind {
        Pointer::Details => Instruction::SetDetails(ix),
        Pointer::Places => Instruction::SetPlaces(ix),
        Pointer::Keyring => Instruction::SetKeyring(ix),
    };
    ctx.tx(ix, ctx.site_accounts(&[]))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuperOp {
    Add,
    Remove,
    Transfer,
}

/// `super_admin_add`, `super_admin_remove` and `super_admin_transfer`. A remove names no Member.
pub fn super_admin(ctx: &Ctx, op: SuperOp, target: AccountId) -> Tx {
    let ix = SuperAdminIx { auth: ctx.auth(), target };
    let ix = match op {
        SuperOp::Add => Instruction::SuperAdminAdd(ix),
        SuperOp::Remove => Instruction::SuperAdminRemove(ix),
        SuperOp::Transfer => Instruction::SuperAdminTransfer(ix),
    };
    let targets: &[AccountId] = if op == SuperOp::Remove { &[] } else { &[target] };
    ctx.tx(ix, ctx.site_accounts(targets))
}

/// `anchor`: `signer_pk` is the Pukaar key of our Member. The signature fields stay zero: clients
/// check the checkpoint off-chain (as the runner does). Used by 2.5 with the anchor rule of O10.
pub fn anchor(ctx: &Ctx, heads_root: [u8; 32], n_events: u64, signer_pk: [u8; 32]) -> Tx {
    let record = CheckpointRecord { site_id: ctx.site_id, heads_root, n_events, signer_pk, sig_r: [0; 32], sig_s: [0; 32] };
    let pdas = vec![member_pda(&ctx.program, &ctx.site_id, &ctx.signer), checkpoint_pda(&ctx.program, &ctx.site_id, &heads_root)];
    ctx.tx(Instruction::Anchor(AnchorIx { record, now_ms: ctx.now_ms }), pdas)
}

/// The SHA-256 of a revoke reason: the chain keeps it, `details.json` keeps the text.
pub fn reason_hash(reason: &str) -> [u8; 32] {
    sha256(reason.as_bytes())
}

pub fn role_code(r: Role) -> u8 {
    match r {
        Role::Resident => 0,
        Role::Steward => 1,
        Role::Admin => 2,
    }
}

pub fn role_from_code(c: u8) -> Role {
    match c {
        1 => Role::Steward,
        2 => Role::Admin,
        _ => Role::Resident,
    }
}

pub fn parse_role(s: &str) -> Result<Role, String> {
    match s.trim().to_ascii_lowercase().as_str() {
        "resident" => Ok(Role::Resident),
        "steward" => Ok(Role::Steward),
        "admin" => Ok(Role::Admin),
        other => Err(format!("unknown role {other}: use resident, steward or admin")),
    }
}

// ---- the first files of a site ----

/// What the Create form sends: `{"Genesis": {...}}`. It becomes `details.json` and `places.json`.
#[derive(Debug)]
pub struct Genesis {
    pub details: Details,
    pub places: Places,
    /// The name of the admin who creates the site: the first profile.
    pub admin_name: String,
}

/// Parse and check the genesis of a new site. The checks are the ones that the old genesis event had.
pub fn genesis(json: &str) -> Result<Genesis, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let g = v.get("Genesis").ok_or("not a genesis body")?;
    let text = |k: &str| g.get(k).and_then(|x| x.as_str()).map(|s| s.trim().to_string()).ok_or_else(|| format!("missing {k}"));
    let num = |k: &str| g.get(k).and_then(|x| x.as_u64()).and_then(|n| u32::try_from(n).ok()).ok_or_else(|| format!("missing or bad {k}"));
    let (name, admin_name) = (text("name")?, text("admin_name")?);
    if admin_name.is_empty() {
        return Err("the admin must be named".into());
    }
    if name.is_empty() {
        return Err("the site needs a name".into());
    }
    let categories: Vec<String> = serde_json::from_value(g.get("categories").cloned().unwrap_or_default()).map_err(|_| "bad categories")?;
    let categories: Vec<String> = categories.iter().map(|c| c.trim().to_string()).collect();
    if categories.is_empty() {
        return Err("add at least one category".into());
    }
    if categories.iter().any(|c| c.is_empty()) {
        return Err("a category can't be empty".into());
    }
    if let Some(c) = categories.iter().enumerate().find(|(i, c)| categories[..*i].contains(c)) {
        return Err(format!("category {} is listed twice", c.1));
    }
    let locations: Vec<Location> = serde_json::from_value(g.get("locations").cloned().unwrap_or_default()).map_err(|_| "bad locations")?;
    let mut places = Places::new(vec![]);
    for l in &locations {
        apply_place_op(&mut places, &PlaceOp::Add(l.clone())).map_err(|e| e.replace("is used already", "is used twice"))?;
    }
    let details = Details {
        schema: DETAILS_SCHEMA.into(),
        name,
        categories,
        sla_ack_h: num("sla_ack_h")?,
        sla_fix_h: num("sla_fix_h")?,
        max_open_per_author: num("max_open_per_author")?,
        revoke_reasons: Default::default(),
    };
    Ok(Genesis { details, places, admin_name })
}

// ---- places.json edits ----

/// One edit of `places.json`. `Retire` and `Remove` both move the place to `retired` (hidden, the
/// signed events stay), `Restore` brings it back.
// ponytail: format 2 has no 30-day removal window, so `remove` hides like `retire` and `undo_remove`
// restores. Add a removal date to Places if the window comes back.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlaceOp {
    Add(Location),
    Edit(Location),
    Retire(String),
    Restore(String),
}

fn trimmed(l: &Location) -> Location {
    Location { code: l.code.trim().into(), label: l.label.trim().into(), group: l.group.trim().into() }
}

/// A place needs a code, a name and a group. "other" is the code of a place that is not on the list.
pub fn check_place(l: &Location) -> Result<(), String> {
    let l = trimmed(l);
    if l.code.is_empty() {
        return Err("every location needs a code".into());
    }
    if l.code == OTHER_LOCATION {
        return Err(format!("the code {OTHER_LOCATION} is reserved for places not on the list"));
    }
    if l.label.is_empty() || l.group.is_empty() {
        return Err(format!("location {} needs a name and a group", l.code));
    }
    Ok(())
}

/// Apply one edit. A code is an identity: an edit changes the name and the group, never the code.
pub fn apply_place_op(p: &mut Places, op: &PlaceOp) -> Result<(), String> {
    let find = |v: &[Location], code: &str| v.iter().position(|l| l.code == code.trim());
    match op {
        PlaceOp::Add(l) => {
            check_place(l)?;
            let l = trimmed(l);
            if find(&p.places, &l.code).or_else(|| find(&p.retired, &l.code)).is_some() {
                return Err(format!("location code {} is used already", l.code));
            }
            p.places.push(l);
        }
        PlaceOp::Edit(l) => {
            check_place(l)?;
            let l = trimmed(l);
            let list = if find(&p.places, &l.code).is_some() { &mut p.places } else { &mut p.retired };
            let i = find(list, &l.code).ok_or_else(|| format!("no location {}", l.code))?;
            list[i] = l;
        }
        PlaceOp::Retire(code) => {
            let i = find(&p.places, code).ok_or_else(|| format!("no active location {code}"))?;
            let l = p.places.remove(i);
            p.retired.push(l);
        }
        PlaceOp::Restore(code) => {
            let i = find(&p.retired, code).ok_or_else(|| format!("{code} is not retired"))?;
            let l = p.retired.remove(i);
            p.places.push(l);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pukaar_sites_rules::{ApproveItem, JOIN_REQ_LEN};

    const PROGRAM: AccountId = [7; 32];
    const SIGNER: AccountId = [1; 32];
    const SITE: [u8; 32] = [9; 32];

    fn ctx() -> Ctx {
        Ctx { program: PROGRAM, site_id: SITE, signer: SIGNER, claimed: Rank::SuperAdmin, now_ms: 1_700_000_000_000 }
    }

    fn ptr(n: u8) -> Pointed {
        (format!("cid{n}"), [n; 32])
    }

    /// The runner builds each instruction as `borsh::to_vec(&Instruction::X(..))` with these fields
    /// (`runner/src/main.rs`, `run` and `Cx::send`). The runner cannot print instruction bytes without
    /// a wallet and a node, so the expected bytes are built here by hand from the same rules types, and
    /// the first bytes of each are pinned: a change of the rules crate (a new deploy) breaks them.
    fn by_hand(ix: Instruction) -> Vec<u8> {
        borsh::to_vec(&ix).unwrap()
    }

    fn auth() -> Auth {
        Auth { site_id: SITE, now_ms: 1_700_000_000_000, claimed: Rank::SuperAdmin }
    }

    #[test]
    fn instruction_bytes_equal_the_runner_construction() {
        let c = ctx();
        let salt = [5; 32];
        let t = create_site(&c, salt, [0xAA; 32], ptr(1), ptr(2), ptr(3));
        let args = CreateSite {
            creator: SIGNER,
            salt,
            site_id: seed::site_id(&SIGNER, &salt),
            pukaar_key: [0xAA; 32],
            details_cid: "cid1".into(),
            details_sha256: [1; 32],
            places_cid: "cid2".into(),
            places_sha256: [2; 32],
            keyring_cid: "cid3".into(),
            keyring_sha256: [3; 32],
            gates: GateConfig::default(),
            access_duration_ms: 0,
            renew_window_ms: 0,
        };
        assert_eq!(t.ix, by_hand(Instruction::CreateSite(CreateSiteIx { args, now_ms: c.now_ms })));
        // variant index 2 (Anchor 0, Reserved 1, CreateSite 2), then the creator
        assert_eq!(&t.ix[..5], &[2, 1, 1, 1, 1]);

        let item = RoleItem { user: [4; 32], role: Role::Steward };
        let t = set_role_batch(&c, vec![([4; 32], Role::Steward)]);
        assert_eq!(t.ix, by_hand(Instruction::SetRoleBatch(SetRoleBatchIx { auth: auth(), items: vec![item] })));
        assert_eq!(t.ix[0], 7);

        let t = revoke_batch(&c, ptr(4), vec![([4; 32], [8; 32])]);
        let ix = RevokeBatchIx { auth: auth(), keyring_cid: "cid4".into(), keyring_sha256: [4; 32], items: vec![RevokeItem { user: [4; 32], reason_sha256: [8; 32] }] };
        assert_eq!(t.ix, by_hand(Instruction::RevokeBatch(ix)));
        assert_eq!(t.ix[0], 8);

        let pi = ProfileItem { user: [4; 32], cid: "p".into(), sha256: [6; 32], version: 3 };
        let t = set_profile_batch(&c, vec![pi.clone()]);
        assert_eq!(t.ix, by_hand(Instruction::SetProfileBatch(SetProfileBatchIx { auth: auth(), items: vec![pi] })));
        assert_eq!(t.ix[0], 9);

        for (kind, ix_of, idx) in [
            (Pointer::Details, Instruction::SetDetails as fn(SetPointerIx) -> Instruction, 3u8),
            (Pointer::Places, Instruction::SetPlaces, 4),
            (Pointer::Keyring, Instruction::SetKeyring, 5),
        ] {
            let t = set_pointer(&c, kind, ptr(5));
            assert_eq!(t.ix, by_hand(ix_of(SetPointerIx { auth: auth(), cid: "cid5".into(), sha256: [5; 32] })));
            assert_eq!(t.ix[0], idx);
        }

        for (op, ix_of, idx) in [
            (SuperOp::Add, Instruction::SuperAdminAdd as fn(SuperAdminIx) -> Instruction, 10u8),
            (SuperOp::Remove, Instruction::SuperAdminRemove, 11),
            (SuperOp::Transfer, Instruction::SuperAdminTransfer, 12),
        ] {
            let t = super_admin(&c, op, [4; 32]);
            assert_eq!(t.ix, by_hand(ix_of(SuperAdminIx { auth: auth(), target: [4; 32] })));
            assert_eq!(t.ix[0], idx);
        }

        let t = anchor(&c, [3; 32], 12, [0xCC; 32]);
        let record = CheckpointRecord { site_id: SITE, heads_root: [3; 32], n_events: 12, signer_pk: [0xCC; 32], sig_r: [0; 32], sig_s: [0; 32] };
        assert_eq!(t.ix, by_hand(Instruction::Anchor(AnchorIx { record, now_ms: c.now_ms })));
        assert_eq!(t.ix[0], 0);
    }

    #[test]
    fn approve_batch_names_the_requester_and_the_claims() {
        let c = ctx();
        let (ed, lez) = (ed25519_dalek::SigningKey::from_bytes(&[3; 32]), crate::lezkey::derive_lez_secret(&[3; 32]));
        let request = crate::joinreq::build(&ed, &lez, SITE, 1, 2).to_vec();
        let item = ApproveItem { request: request.clone().try_into().unwrap(), role: Role::Resident, profile_cid: "pc".into(), profile_sha256: [2; 32], profile_version: 1 };
        assert_eq!(request.len(), JOIN_REQ_LEN);
        let t = approve_batch(&c, 4, ptr(6), vec![(item.clone(), true)]).unwrap();
        let ix = ApproveBatchIx { auth: auth(), roster_len: 4, keyring_cid: "cid6".into(), keyring_sha256: [6; 32], items: vec![ApproveEntry { item, is_new: true }] };
        assert_eq!(t.ix, by_hand(Instruction::ApproveBatch(ix)));
        assert_eq!(t.ix[0], 6);
        // The target Member is the PDA of the account that the request names.
        let user = crate::lezkey::lez_account_id(&lez);
        assert!(t.accounts.iter().any(|m| m.id == member_pda(&PROGRAM, &SITE, &user)));
    }

    #[test]
    fn accounts_follow_the_runner_order() {
        let c = ctx();
        let t = set_role_batch(&c, vec![([4; 32], Role::Admin), ([4; 32], Role::Steward), (SIGNER, Role::Admin)]);
        let ids: Vec<AccountId> = t.accounts.iter().map(|m| m.id).collect();
        assert_eq!(ids[0], SIGNER, "the signer comes first");
        assert_eq!(ids[1], CLOCK_ACCOUNT);
        assert_eq!(ids[2], site_pda(&PROGRAM, &SITE));
        assert_eq!(ids[3], member_pda(&PROGRAM, &SITE, &SIGNER));
        assert_eq!(ids[4], member_pda(&PROGRAM, &SITE, &[4; 32]));
        assert_eq!(ids.len(), 5, "doubles are removed");
        assert_eq!((t.accounts[0].sign, t.accounts[0].shard), (true, PROGRAM));
        assert_eq!((t.accounts[1].sign, t.accounts[1].shard), (false, clock_program()));
        assert!(t.accounts[2..].iter().all(|m| !m.sign && m.shard == PROGRAM));
        assert_eq!(t.payer, SIGNER);
        // remove names no target Member
        assert_eq!(super_admin(&c, SuperOp::Remove, [4; 32]).accounts.len(), 4);
        assert_eq!(super_admin(&c, SuperOp::Add, [4; 32]).accounts.len(), 5);
        // anchor: our Member, then the checkpoint
        let a = anchor(&c, [3; 32], 1, [0; 32]);
        assert_eq!(a.accounts[2].id, member_pda(&PROGRAM, &SITE, &SIGNER));
        assert_eq!(a.accounts[3].id, checkpoint_pda(&PROGRAM, &SITE, &[3; 32]));
    }

    #[test]
    fn the_clock_program_id_is_the_builtin_hash() {
        // sha256("/LEE-BuiltinProgram/v1/AccountId" ‖ "clock"): pinned from LEZ v0.3.1.
        let again = Sha256::digest([&b"/LEE-BuiltinProgram/v1/AccountId"[..], b"clock"].concat());
        assert_eq!(clock_program(), <[u8; 32]>::from(again));
        assert_eq!(&CLOCK_ACCOUNT[..4], b"/LEZ");
    }

    #[test]
    fn place_edits() {
        let l = |c: &str| Location { code: c.into(), label: "Tap".into(), group: "Water".into() };
        let mut p = Places::new(vec![l("W-01")]);
        apply_place_op(&mut p, &PlaceOp::Add(l(" W-02 "))).unwrap();
        assert_eq!(p.places[1].code, "W-02");
        assert!(apply_place_op(&mut p, &PlaceOp::Add(l("W-01"))).is_err(), "a code is used once");
        assert!(apply_place_op(&mut p, &PlaceOp::Add(l("other"))).is_err());
        assert!(apply_place_op(&mut p, &PlaceOp::Add(Location { code: "X".into(), label: " ".into(), group: "g".into() })).is_err());
        apply_place_op(&mut p, &PlaceOp::Retire("W-01".into())).unwrap();
        assert_eq!((p.places.len(), p.retired.len()), (1, 1));
        assert!(apply_place_op(&mut p, &PlaceOp::Add(l("W-01"))).is_err(), "a retired code stays taken");
        let mut e = l("W-01");
        e.label = "New".into();
        apply_place_op(&mut p, &PlaceOp::Edit(e)).unwrap();
        assert_eq!(p.retired[0].label, "New");
        apply_place_op(&mut p, &PlaceOp::Restore("W-01".into())).unwrap();
        assert_eq!((p.places.len(), p.retired.len()), (2, 0));
        assert!(apply_place_op(&mut p, &PlaceOp::Restore("W-01".into())).is_err());
        assert!(apply_place_op(&mut p, &PlaceOp::Retire("nope".into())).is_err());
    }

    #[test]
    fn genesis_is_checked() {
        let ok = r#"{"Genesis":{"name":" Dhun ","admin_name":"Asha","categories":["water","bins"],"locations":[{"code":"W-01","label":"Tap","group":"Water"}],"sla_ack_h":24,"sla_fix_h":72,"max_open_per_author":5}}"#;
        let g = genesis(ok).unwrap();
        assert_eq!((g.details.name.as_str(), g.admin_name.as_str(), g.places.places.len()), ("Dhun", "Asha", 1));
        assert_eq!(g.details.max_open_per_author, 5);
        let bad = |from: &str, to: &str| genesis(&ok.replace(from, to)).unwrap_err();
        assert!(bad("\"Asha\"", "\" \"").contains("named"));
        assert!(bad("\" Dhun \"", "\"\"").contains("name"));
        assert!(bad("[\"water\",\"bins\"]", "[]").contains("category"));
        assert!(bad("[\"water\",\"bins\"]", "[\"water\",\"water\"]").contains("twice"));
        assert!(bad("\"W-01\"", "\"other\"").contains("reserved"));
        assert!(bad("\"sla_fix_h\":72", "\"sla_fix_h\":-1").contains("sla_fix_h"));
        let twice = ok.replace("}],", "},{\"code\":\"W-01\",\"label\":\"x\",\"group\":\"y\"}],");
        assert!(genesis(&twice).unwrap_err().contains("used twice"));
        assert!(genesis("{}").is_err() && genesis("nope").is_err());
    }

    #[test]
    fn roles_and_reasons() {
        for r in [Role::Resident, Role::Steward, Role::Admin] {
            assert_eq!(role_from_code(role_code(r)), r);
        }
        assert_eq!(parse_role(" Steward ").unwrap(), Role::Steward);
        assert!(parse_role("boss").is_err());
        assert_eq!(reason_hash("x"), sha256(b"x"));
    }
}
