//! What the module methods do, without the glue: each function checks the request against the cached
//! chain state and the rules, then adds an entry to the write queue. None waits for the chain, Storage
//! or the wallet, and none sends anything: the worker (`queue::tick`) does that. A write that the rules
//! would refuse is refused here, because a failed call costs a fee.
//!
//! Held writes (approvals, role changes, revokes) wait for `send_batch`. The others are queued at once.

use crate::chain::b58;
use crate::event::{Key, Location, MAX_TEXT};
use crate::fetch::is_staff;
use crate::joinreq;
use crate::lezkey::{derive_lez_secret, lez_account_id};
use crate::node::{fingerprint, parse_id, Node};
use crate::profile;
use crate::sync::Wire;
use crate::queue::{ApproveReq, Blob, Intent, Prof, ProfReq, Queue, RevokeReq, St, REQUEST_VALID_MS};
use crate::rpc::{self, rank_name};
use crate::writes::{self, Genesis, PlaceOp};
use ed25519_dalek::SigningKey;
use pukaar_sites_rules::{can, request_id, seed, Action, Member, MemberStatus, Rank, Role, Site};
use serde_json::{json, Value};

/// The cached chain state that a method checks before it queues a write.
pub struct View {
    pub site: Site,
    pub members: Vec<Member>,
    /// Our rank on the chain.
    pub mine: Option<Rank>,
}

pub fn view(node: &Node) -> Result<View, String> {
    let c = node.chain.as_ref().ok_or("the chain is not connected")?;
    let (site, members) = c.lock().unwrap().snapshot();
    Ok(View { site: site.ok_or("the chain has not shown this site yet")?, members, mine: node.role_now(&node.me()) })
}

impl View {
    pub fn rank_of(&self, m: &Member) -> Rank {
        if self.site.is_super(&m.user) {
            Rank::SuperAdmin
        } else {
            m.role.into()
        }
    }

    /// The active member with this Pukaar key.
    pub fn active(&self, key: &Key) -> Result<&Member, String> {
        let on = |m: &&Member| m.pukaar_key == *key && m.status == MemberStatus::Active && m.revoked.is_none();
        self.members.iter().find(on).ok_or_else(|| "not an active member of this site".to_string())
    }

    /// Our rank, if the rules let it do `action` on a subject of rank `target`.
    pub fn need(&self, action: Action, target: Option<Rank>) -> Result<Rank, String> {
        let me = self.mine.ok_or("you are not an approved member of this site")?;
        if can(me, action, target) {
            Ok(me)
        } else {
            Err(format!("not allowed: a {} cannot do this", rank_name(me).to_lowercase()))
        }
    }
}

fn reason_text(reason: &str) -> Result<String, String> {
    let r = reason.trim();
    if r.is_empty() {
        return Err("a reason is required".into());
    }
    if r.len() > MAX_TEXT {
        return Err(format!("too long: keep the reason under {MAX_TEXT} bytes"));
    }
    Ok(r.to_string())
}

/// The next profile version: above what we gave out and above what the chain holds for us.
fn next_version(node: &Node, q: &mut Queue) -> u64 {
    let on_chain = node.chain.as_ref().map_or(0, |c| c.lock().unwrap().snapshot().1.iter().find(|m| m.user == node.account).map_or(0, |m| m.profile_version));
    q.profile_version = q.profile_version.max(on_chain) + 1;
    q.profile_version
}

/// Held writes of the same kind and subject are replaced by the newer one.
fn drop_held(q: &mut Queue, site: &Key, same: impl Fn(&Intent) -> bool) {
    q.entries.retain(|e| !(e.site == *site && e.state == St::Held && same(&e.intent)));
}

// ---- site_create, site_join, set_profile ----

/// The new files of a site that `site_create` uploads, and the ids that come with them.
pub struct NewSite {
    pub salt: Key,
    pub site_id: Key,
    /// details.json, places.json and the first keyring (the site key sealed to the creator).
    pub blobs: Vec<Blob>,
}

pub fn new_site(key: &SigningKey, g: &Genesis) -> Result<NewSite, String> {
    let me = lez_account_id(&derive_lez_secret(&key.to_bytes()));
    let salt = crate::event::new_key().to_bytes();
    let site_id = seed::site_id(&me, &salt);
    let ring = profile::rotate(&site_id, &mut Default::default(), &[(me, key.verifying_key().to_bytes())])?;
    let blobs = vec![Blob::new(g.details.to_bytes()), Blob::new(g.places.to_bytes()), Blob::new(profile::keyring_to_bytes(&ring))];
    Ok(NewSite { salt, site_id, blobs })
}

/// Queue `create_site`, then the first profile of the admin. Both run on the worker, in this order.
pub fn create_site(node: &Node, q: &mut Queue, ns: NewSite, admin_name: &str, now: u64) -> u64 {
    let id = q.push(ns.site_id, Intent::CreateSite { salt: ns.salt, pukaar_key: node.me() }, St::Preparing, ns.blobs, now);
    let version = next_version(node, q);
    q.push(ns.site_id, Intent::Publish { name: admin_name.into(), version, bio: String::new(), experience: String::new() }, St::Preparing, vec![], now);
    id
}

/// Sign the join request with both keys and keep it. The caller sends the returned bytes as
/// `Wire::JoinReq`. The first profile (no name) is queued: an admin needs a profile file to approve us.
pub fn join(node: &Node, q: &mut Queue, now: u64) -> (Vec<u8>, u64) {
    let req = joinreq::build(&node.key, &derive_lez_secret(&node.key.to_bytes()), node.store.site, now, now + REQUEST_VALID_MS).to_vec();
    (q.mine, q.declined) = (Some(req.clone()), false);
    let version = next_version(node, q);
    // A first join has no name. A new request keeps the name and the texts that we know of ourselves.
    let me = node.me();
    let (bio, experience) = node.about.get(&me).cloned().unwrap_or_default();
    let name = node.names.get(&me).cloned().unwrap_or_default();
    let id = q.push(node.store.site, Intent::Publish { name, version, bio, experience }, St::Preparing, vec![], now);
    (req, id)
}

/// `join_again`: the user cancels the request that waits, or that was declined or spent (a removed member),
/// and sends a new one with a fresh profile file. The old request stays valid on the admin side, but a
/// request of the same key with a higher nonce replaces it there.
pub fn join_again(node: &Node, q: &mut Queue, now: u64) -> Result<(Vec<u8>, u64), String> {
    if node.role_now(&node.me()).is_some() {
        return Err("you have a role in this site already".into());
    }
    Ok(join(node, q, now))
}

/// A join request that the chain used already: its nonce is at or below `last_req_nonce` of the member.
/// The chain refuses it a second time, so it must not show as waiting or be approved.
fn spent(members: &[Member], b: &pukaar_sites_rules::JoinBody) -> bool {
    members.iter().any(|m| m.user == b.user_account && m.last_req_nonce >= b.nonce)
}

/// A bio or an experience text: no control characters except a new line, at most `MAX_TEXT` characters.
fn about_text(what: &str, text: &str) -> Result<String, String> {
    let t: String = text.chars().filter(|c| *c == '\n' || !c.is_control()).collect::<String>().trim().to_string();
    if t.chars().count() > MAX_TEXT {
        return Err(format!("too long: keep the {what} under {MAX_TEXT} characters"));
    }
    Ok(t)
}

pub fn set_profile(node: &Node, q: &mut Queue, display_name: &str, bio: &str, experience: &str, now: u64) -> Result<u64, String> {
    let name: String = display_name.chars().filter(|c| !c.is_control()).collect::<String>().trim().to_string();
    if name.chars().count() > 64 {
        return Err("too long: keep the name under 64 characters".into());
    }
    let (bio, experience) = (about_text("bio", bio)?, about_text("experience", experience)?);
    // O5: readers ignore the own name of a steward, so refuse it here instead of saving a name nobody sees.
    if view(node).is_ok_and(|v| v.mine == Some(Rank::Steward)) {
        return Err(STAFF_NAME.into());
    }
    let version = next_version(node, q);
    Ok(q.push(node.store.site, Intent::Publish { name, version, bio, experience }, St::Preparing, vec![], now))
}

/// The text when a steward tries to set their own name (O5).
pub const STAFF_NAME: &str = "An admin sets the name of staff. Ask an admin to set your name.";

/// `set_staff_name` (O5): an admin signs the name of a steward or an admin. It is a chain write of one
/// profile pointer, sent at once (it costs one fee, no batch). The old program already takes it.
pub fn set_staff_name(node: &Node, q: &mut Queue, subject_hex: &str, name: &str, now: u64) -> Result<u64, String> {
    let subject = parse_id(subject_hex.trim()).ok_or("bad member key")?;
    let name: String = name.chars().filter(|c| !c.is_control()).collect::<String>().trim().to_string();
    if name.is_empty() {
        return Err("a name is required".into());
    }
    if name.chars().count() > 64 {
        return Err("too long: keep the name under 64 characters".into());
    }
    let (v, site) = (view(node)?, node.store.site);
    let m = v.active(&subject)?;
    if !is_staff(&v.site, m) {
        return Err("a resident sets their own name".into());
    }
    let rank = v.rank_of(m);
    v.need(Action::SetProfile, Some(rank))?;
    if rank >= Rank::Admin && m.user != node.account && v.mine != Some(Rank::SuperAdmin) {
        return Err("only a super admin sets the name of another admin".into());
    }
    Ok(push_staff_name(q, site, m, name, now))
}

/// Queue the signed name. A name that did not start yet is replaced. The version is above the chain pointer
/// and above every pointer that waits.
fn push_staff_name(q: &mut Queue, site: Key, m: &Member, name: String, now: u64) -> u64 {
    q.entries.retain(|e| !(e.site == site && e.state == St::Preparing && matches!(&e.intent, Intent::StaffName { user, .. } if *user == m.user)));
    let queued = q.entries.iter().filter(|e| e.site == site).filter_map(|e| match &e.intent {
        Intent::StaffName { user, version, .. } if *user == m.user => Some(*version),
        Intent::SetProfile { items } => items.iter().filter(|i| i.user == m.user).map(|i| i.prof.version).max(),
        _ => None,
    });
    let version = queued.max().unwrap_or(0).max(m.profile_version) + 1;
    q.push(site, Intent::StaffName { user: m.user, key: m.pukaar_key, name, version }, St::Preparing, vec![], now)
}

// ---- roles ----

/// `grant_role`: a role change for a member, or an approval for a person with a join request.
pub fn grant(node: &Node, q: &mut Queue, subject_hex: &str, role: &str, name: &str, now: u64) -> Result<u64, String> {
    let subject = parse_id(subject_hex.trim()).ok_or("bad member key")?;
    let role = writes::parse_role(role)?;
    let name = name.trim().to_string();
    if role != Role::Resident && name.is_empty() {
        return Err("a steward or admin needs a name".into());
    }
    let (v, site) = (view(node)?, node.store.site);
    if let Ok(m) = v.active(&subject) {
        v.need(Action::SetRole(role), Some(v.rank_of(m)))?;
        if m.role == role {
            return Err("they have this role already".into());
        }
        let user = m.user;
        drop_held(q, &site, |i| matches!(i, Intent::SetRole { items } if items[0].0 == user));
        let id = q.push(site, Intent::SetRole { items: vec![(user, writes::role_code(role))] }, St::Held, vec![], now);
        // O5: a member who becomes staff needs a name that an admin signs (a demotion to resident keeps the old one).
        if role != Role::Resident {
            // The reader accepts an admin-signed name for any rank, so it can land before the role change.
            push_staff_name(q, site, m, name, now);
        }
        return Ok(id);
    }
    let (body, request) = q.request_for(&subject, now).ok_or("no join request from this person, or it expired: ask them to join again")?;
    if spent(&v.members, &body) {
        return Err("no new join request from this person: ask them to join again".into());
    }
    v.need(Action::Approve(role), None)?;
    // The approval opens the file of the joiner, so we must hold its bytes now (Delivery or the fetcher
    // brought them): the write thread never waits for a remote Storage node.
    let arrived = node.fetched.as_ref().is_some_and(|f| {
        let f = f.lock().unwrap();
        f.announce_of(&subject).is_some_and(|a| f.cached(&a.cid, &a.sha256).is_some())
    });
    if !arrived {
        return Err("This person's profile has not arrived yet. Try again in a minute.".into());
    }
    drop_held(q, &site, |i| matches!(i, Intent::Approve { items, .. } if items[0].request == request));
    // O5: for a steward or an admin, `prepare_approve` signs this name into the profile file.
    let item = ApproveReq { request, role: writes::role_code(role), name, profile: None };
    Ok(q.push(site, Intent::Approve { roster_len: 0, items: vec![item] }, St::Held, vec![], now))
}

/// `revoke_role`: the reason text goes to details.json, its hash to the chain, at `send_batch`.
pub fn revoke(node: &Node, q: &mut Queue, subject_hex: &str, reason: &str, now: u64) -> Result<u64, String> {
    let subject = parse_id(subject_hex.trim()).ok_or("bad member key")?;
    let reason = reason_text(reason)?;
    let (v, site) = (view(node)?, node.store.site);
    let m = v.active(&subject)?;
    v.need(Action::Revoke, Some(v.rank_of(m)))?;
    let user = m.user;
    drop_held(q, &site, |i| matches!(i, Intent::Revoke { items } if items[0].user == user));
    let item = RevokeReq { user, key: subject, reason };
    Ok(q.push(site, Intent::Revoke { items: vec![item] }, St::Held, vec![], now))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SuperKind {
    Add,
    Remove,
    Transfer,
}

pub fn super_admin(node: &Node, q: &mut Queue, subject_hex: &str, kind: SuperKind, now: u64) -> Result<u64, String> {
    let subject = parse_id(subject_hex.trim()).ok_or("bad member key")?;
    let v = view(node)?;
    let m = v.active(&subject)?;
    let action = match kind {
        SuperKind::Add => Action::SuperAdminAdd,
        SuperKind::Remove => Action::SuperAdminRemove,
        SuperKind::Transfer => Action::SuperAdminTransfer,
    };
    v.need(action, Some(v.rank_of(m)))?;
    let intent = match kind {
        SuperKind::Add if v.site.super_admins.iter().all(Option::is_some) => return Err("two super admins already, the most allowed".into()),
        SuperKind::Add => Intent::SuperAdd(m.user),
        SuperKind::Remove if m.user == node.account => return Err("you cannot remove yourself: hand your place over instead".into()),
        SuperKind::Remove => Intent::SuperRemove(m.user),
        SuperKind::Transfer => Intent::SuperTransfer(m.user),
    };
    Ok(q.push(node.store.site, intent, St::Queued, vec![], now))
}

// ---- places ----

/// A request of the place methods.
pub enum Place<'a> {
    Add { code: &'a str, label: &'a str, group: &'a str },
    Edit { code: &'a str, label: &'a str, group: &'a str },
    Retire { code: &'a str, reason: &'a str },
    Restore { code: &'a str, reason: &'a str },
    /// Only for a place that no report ever named.
    Remove { code: &'a str, reason: &'a str },
}

fn known_place(node: &Node, code: &str) -> Result<(), String> {
    node.locations.iter().find(|l| l.code == code).map(|_| ()).ok_or_else(|| format!("no active location {code}"))
}

/// An admin changes `places.json`. The worker applies the edit to the newest file in Storage and
/// queues `set_places`, so two quick edits both survive.
pub fn place_edit(node: &Node, q: &mut Queue, req: Place, now: u64) -> Result<u64, String> {
    view(node)?.need(Action::SetSite, None)?;
    let loc = |code: &str, label: &str, group: &str| Location { code: code.trim().into(), label: label.trim().into(), group: group.trim().into() };
    let op = match req {
        Place::Add { code, label, group } => {
            let l = loc(code, label, group);
            writes::check_place(&l)?;
            if node.locations.iter().any(|x| x.code == l.code) {
                return Err(format!("location code {} is used already", l.code));
            }
            PlaceOp::Add(l)
        }
        Place::Edit { code, label, group } => {
            known_place(node, code.trim())?;
            PlaceOp::Edit(loc(code, label, group))
        }
        Place::Retire { code, reason } => {
            let code = code.trim();
            reason_text(reason)?;
            known_place(node, code)?;
            let open = node.state().open_issues_at(code);
            if open > 0 {
                return Err(format!("{code} has {open} open issue{}", if open == 1 { "" } else { "s" }));
            }
            PlaceOp::Retire(code.into())
        }
        Place::Restore { code, reason } => {
            reason_text(reason)?;
            PlaceOp::Restore(code.trim().into())
        }
        Place::Remove { code, reason } => {
            let code = code.trim();
            reason_text(reason)?;
            known_place(node, code)?;
            if node.state().issues.values().any(|i| i.location == code) {
                return Err(format!("{code} has had reports: retire it instead"));
            }
            PlaceOp::Retire(code.into())
        }
    };
    Ok(q.push(node.store.site, Intent::Places { ops: vec![op] }, St::Preparing, vec![], now))
}

// ---- batches and status ----

/// The profile files that members announced and the chain does not point to yet.
pub fn refresh_items(node: &Node, q: &Queue) -> Vec<ProfReq> {
    let (Some(c), Some(f)) = (&node.chain, &node.fetched) else { return vec![] };
    let (Some(v_site), members) = c.lock().unwrap().snapshot() else { return vec![] };
    let announces = f.lock().unwrap().announces();
    let open: Vec<([u8; 32], u64)> = q
        .open(&node.store.site)
        .filter_map(|e| match &e.intent {
            Intent::SetProfile { items } => Some(items.iter().map(|i| (i.user, i.prof.version)).collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()
        .collect();
    members
        .iter()
        .filter(|m| m.status == MemberStatus::Active && m.revoked.is_none())
        .filter(|m| !is_staff(&v_site, m)) // O5: a staff name comes from an admin, not from an announce
        .filter_map(|m| {
            let a = announces.iter().find(|a| a.author == m.pukaar_key && a.version > m.profile_version)?;
            let item = ProfReq { user: m.user, prof: Prof { cid: a.cid.clone(), sha: a.sha256, version: a.version } };
            (!open.contains(&(m.user, a.version))).then_some(item)
        })
        .collect()
}

/// `send_batch`: batches of 10 from the held writes and the refreshes. Admin only.
pub fn send_batch(node: &Node, q: &mut Queue, now: u64) -> Result<Vec<u64>, String> {
    view(node)?.need(Action::SetProfile, None)?;
    let refreshes = refresh_items(node, q);
    let ids = q.make_batches(&node.store.site, refreshes, now);
    if ids.is_empty() {
        return Err("nothing waits for a batch".into());
    }
    Ok(ids)
}

/// `reject_join`: hide the join request of a person and tell them. It writes nothing to the chain and
/// costs no fee. Returns the `Wire::JoinDecline` bytes for the site channel.
pub fn reject(node: &Node, q: &mut Queue, subject_hex: &str, now: u64) -> Result<Vec<u8>, String> {
    let subject = parse_id(subject_hex.trim()).ok_or("bad member key")?;
    view(node)?.need(Action::Approve(Role::Resident), None)?;
    let (_, request) = q.request_for(&subject, now).ok_or("no join request from this person, or it expired")?;
    if q.is_queued_approval(&node.store.site, &subject) {
        return Err("an approval for this person waits to be sent: it cannot be rejected now".into());
    }
    let id = request_id(request.as_slice().try_into().map_err(|_| "bad join request")?);
    q.reject(id);
    let sig = joinreq::sign_decline(&node.key, &node.store.site, &id);
    Ok(Wire::JoinDecline { site: node.store.site, request_id: id, admin: node.me(), sig }.encode())
}

/// A `Wire::JoinDecline` from Delivery. It counts only for the request that we sent, from an active admin
/// of this site, with a good signature. Anything else is ignored, so a stranger or a replay changes
/// nothing, and the check costs one signature at most. True when we accepted it.
pub fn on_decline(node: &Node, q: &mut Queue, site: &Key, request_id: &Key, admin: &Key, sig: &[u8]) -> bool {
    let ours = *site == node.store.site && q.mine_id() == Some(*request_id);
    let by_admin = || node.role_now(admin).is_some_and(|r| r >= Rank::Admin);
    if !(ours && by_admin() && joinreq::check_decline(admin, site, request_id, sig)) {
        return false;
    }
    (q.mine, q.declined) = (None, true);
    q.save();
    true
}

/// Is our join request still waiting? The chain keeps the nonce of the last request that it used
/// (`last_req_nonce`): a request at or below it is spent. An approved and later removed member must
/// see the removal, not "request sent".
pub fn request_waits(node: &Node, q: &Queue) -> bool {
    let Some(nonce) = q.mine_nonce() else { return false };
    let used = node.chain.as_ref().and_then(|c| c.lock().unwrap().snapshot().1.into_iter().find(|m| m.user == node.account)).map_or(0, |m| m.last_req_nonce);
    nonce > used
}

/// How long a join request still holds, in plain words: "45 min", "23 h", "2 d". Empty when it is over.
pub fn left_text(until_ms: u64, now_ms: u64) -> String {
    let s = until_ms.saturating_sub(now_ms) / 1000;
    match s {
        0 => String::new(),
        1..=3599 => format!("{} min", s.div_ceil(60)),
        3600..=172_799 => format!("{} h", s / 3600),
        _ => format!("{} d", s / 86_400),
    }
}

/// `my_identity.pending` of a user without a role, and when our request stops being valid (ms): "requested"
/// while it holds, "expired" after `valid_until` (an admin no longer sees it: the user sends it again),
/// "declined" when an admin turned it down. `None` when no request is out.
pub fn my_pending(node: &Node, q: &Queue, now_ms: u64) -> Option<(&'static str, Option<u64>)> {
    if request_waits(node, q) {
        let until = q.mine_valid_until();
        return Some((if until.is_some_and(|u| u <= now_ms) { "expired" } else { "requested" }, until));
    }
    q.declined.then_some(("declined", None))
}

/// Tell the chain cache about the accounts that the queue knows, so it reads them again after a restart.
pub fn remember_known(node: &Node, q: &Queue) {
    let Some(c) = &node.chain else { return };
    let mut c = c.lock().unwrap();
    for a in q.accounts(&node.store.site) {
        c.hint(a, None);
    }
}

/// `chain_status`. `reason` is plain text, not an error: why we hold no site key, or null.
pub fn status(node: &Node, q: &Queue, now_ms: u64) -> Value {
    let mut v = q.status_json(&node.store.site, now_ms);
    v["refresh_waiting"] = refresh_items(node, q).len().into();
    // The native balance of our account (lepta, null until the first read) and the two limits of the guard.
    v["balance_lepta"] = node.chain.as_ref().and_then(|c| c.lock().unwrap().balance).map(|b| b as u64).into();
    v["floor_lepta"] = (rpc::FLOOR_LEPTA as u64).into();
    v["low_lepta"] = (rpc::LOW_LEPTA as u64).into();
    v["account"] = b58(&node.account).into();
    let me = node.me();
    let removed = node.chain.as_ref().is_some_and(|c| c.lock().unwrap().revocation_of(&me, &node.names).is_some());
    v["reason"] = match (node.role_now(&me), removed) {
        (Some(_), _) => Value::Null,
        (None, true) => "Your role was removed. You can read the board, but not the names and profiles of members.".into(),
        (None, false) => "An admin has not approved you yet. Names and profiles of members stay hidden until then.".into(),
    };
    v
}

/// `site_info.pending`: the join requests that wait for an admin.
pub fn pending_rows(node: &Node, q: &Queue, now: u64) -> Vec<Value> {
    let members = node.chain.as_ref().map_or_else(Vec::new, |c| c.lock().unwrap().snapshot().1);
    q.requests(&node.me(), now)
        .into_iter()
        .filter(|b| node.role_now(&b.pukaar_ed_pk).is_none() && !spent(&members, b))
        .map(|b| {
            json!({
                "key": hex::encode(b.pukaar_ed_pk), "fingerprint": fingerprint(&b.pukaar_ed_pk),
                "name": node.names.get(&b.pukaar_ed_pk), "account": b58(&b.user_account),
                "queued": q.is_queued_approval(&node.store.site, &b.pukaar_ed_pk),
                // When the request stops being valid (unix seconds) and the time left in words.
                "valid_until": b.valid_until_ms / 1000, "left": left_text(b.valid_until_ms, now),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
