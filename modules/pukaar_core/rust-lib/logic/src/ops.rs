//! What the module methods do, without the glue: each function checks the request against the cached
//! chain state and the rules, then adds an entry to the write queue. None waits for the chain, Storage
//! or the wallet, and none sends anything: the worker (`queue::tick`) does that. A write that the rules
//! would refuse is refused here, because a failed call costs a fee.
//!
//! Held writes (approvals, role changes, revokes) wait for `send_batch`. The others are queued at once.

use crate::chain::b58;
use crate::event::{Key, Location, MAX_TEXT};
use crate::joinreq;
use crate::lezkey::{derive_lez_secret, lez_account_id};
use crate::node::{fingerprint, parse_id, Node};
use crate::profile;
use crate::queue::{ApproveReq, Blob, Intent, Prof, ProfReq, Queue, RevokeReq, St, REQUEST_VALID_MS};
use crate::rpc::rank_name;
use crate::writes::{self, Genesis, PlaceOp};
use ed25519_dalek::SigningKey;
use pukaar_sites_rules::{can, seed, Action, Member, MemberStatus, Rank, Role, Site};
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
    q.mine = Some(req.clone());
    let version = next_version(node, q);
    let id = q.push(node.store.site, Intent::Publish { name: String::new(), version, bio: String::new(), experience: String::new() }, St::Preparing, vec![], now);
    (req, id)
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
    let version = next_version(node, q);
    Ok(q.push(node.store.site, Intent::Publish { name, version, bio, experience }, St::Preparing, vec![], now))
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
        return Ok(q.push(site, Intent::SetRole { items: vec![(user, writes::role_code(role))] }, St::Held, vec![], now));
    }
    let (_, request) = q.request_for(&subject, now).ok_or("no join request from this person, or it expired: ask them to join again")?;
    v.need(Action::Approve(role), None)?;
    if node.fetched.as_ref().is_none_or(|f| f.lock().unwrap().announce_of(&subject).is_none()) {
        return Err("their profile has not arrived yet, try again shortly".into());
    }
    drop_held(q, &site, |i| matches!(i, Intent::Approve { items, .. } if items[0].request == request));
    // ponytail: a steward or admin name that an admin signs (O5) is not built: readers do not accept
    // such a profile yet. The name is kept here for 2.3c.
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
    let members = c.lock().unwrap().snapshot().1;
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

/// `chain_status`.
pub fn status(node: &Node, q: &Queue) -> Value {
    let mut v = q.status_json(&node.store.site);
    v["refresh_waiting"] = refresh_items(node, q).len().into();
    v
}

/// `site_info.pending`: the join requests that wait for an admin.
pub fn pending_rows(node: &Node, q: &Queue, now: u64) -> Vec<Value> {
    q.requests(&node.me(), now)
        .into_iter()
        .filter(|b| node.role_now(&b.pukaar_ed_pk).is_none())
        .map(|b| {
            json!({
                "key": hex::encode(b.pukaar_ed_pk), "fingerprint": fingerprint(&b.pukaar_ed_pk),
                "name": node.names.get(&b.pukaar_ed_pk), "account": b58(&b.user_account),
                "queued": q.is_queued_approval(&node.store.site, &b.pukaar_ed_pk),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
