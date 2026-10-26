//! One participant: their key plus their replica of the site. Everything the
//! Logos module exposes is a thin wrapper over this, so it's all testable with `cargo test`.

use crate::checkpoint::{checkpoint_now, n_events, root_for};
use crate::event::{
    decode, sign, Body, DecodeError, Event, Id, Key, Location, Unsigned, MAX_EVENT_BYTES, MAX_TEXT,
    VERSION,
};
use crate::fetch::SharedFetched;
use crate::lezkey::{derive_lez_secret, lez_account_id};
use crate::profile::Announce;
use crate::reducer::{issue_of, reduce, Config, Issue, RoleFn, State, Status};
use crate::rpc::{rank_name, SharedChain};
use crate::store::{Accept, Store};
use ed25519_dalek::SigningKey;
use pukaar_sites_rules::Rank;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Heads messages to hear after a restore before trusting that our history is back.
pub const HEADS_TO_SETTLE: u32 = 3;
/// Seconds after a restore before the user may skip the wait (history lost with the old device).
pub const SKIP_SYNC_AFTER: u64 = 600;
const SYNCING: &str = "still syncing your history, try again shortly";

/// A restored identity's chain lives on other devices: signing before it's back would reuse
/// a seq we already signed and fork our own chain. Set by a restore, persisted, and cleared
/// once we hold at least one of our events, have heard `HEADS_TO_SETTLE` Heads messages, and
/// hold our chain up to the highest seq any of them named for us.
// ponytail: Heads carry no sender, so 3 messages may be one peer three times (or our own
// echo); a peer that holds a later event and stays silent still forks us, flagged, not lost.
// Count distinct signed Heads if that bites.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Restore {
    /// When the restore happened (our clock): the skip timeout counts from here.
    pub since: u64,
    heads_seen: u32,
    /// Highest seq any Heads message named for us.
    claimed: Option<u64>,
}

impl Restore {
    #[must_use]
    pub fn new(since: u64) -> Self {
        Restore {
            since,
            heads_seen: 0,
            claimed: None,
        }
    }
}

pub struct Node {
    pub key: SigningKey,
    pub store: Store,
    /// `Some` while a restored identity waits for its own history.
    pub restore: Option<Restore>,
    /// The newest chain block we know. Each event we sign names it as `chain_block`.
    pub chain_block: u64,
    /// The chain role of an author at a block. Nobody has a role until `attach_chain`.
    pub roles: RoleFn,
    /// The cached chain reads that back `roles`.
    pub chain: Option<SharedChain>,
    /// Our own LEZ account. Each event we sign names it as `author_account`.
    pub account: Key,
    pub config: Config,         // from details.json
    pub locations: Vec<Location>, // from places.json
    /// The hidden places of places.json (retired or removed): an admin can restore them.
    pub retired: Vec<Location>,
    pub names: BTreeMap<Key, String>, // from the profiles
    /// The bio and the experience of each member, from the profiles.
    pub about: BTreeMap<Key, (String, String)>,
    /// The text of each removal reason, by the SHA-256 of the text (from details.json).
    pub reasons: BTreeMap<Key, String>,
    /// What the Storage fetchers learned, and the version of it that we copied last.
    pub fetched: Option<SharedFetched>,
    fetched_version: u64,
}

impl Node {
    /// A replica of `site`, the site id that the chain gave (a format 2 log has no genesis).
    pub fn join(key: SigningKey, site: Id) -> Node {
        Node {
            account: lez_account_id(&derive_lez_secret(&key.to_bytes())),
            key,
            store: Store::new(site),
            restore: None,
            chain_block: 0,
            roles: Box::new(|_, _, _| None),
            chain: None,
            config: Config::default(),
            locations: Vec::new(),
            retired: Vec::new(),
            names: BTreeMap::new(),
            about: BTreeMap::new(),
            reasons: BTreeMap::new(),
            fetched: None,
            fetched_version: 0,
        }
    }

    /// Join with an identity that already has history here: publish nothing until it's back.
    pub fn join_restored(key: SigningKey, site: Id, now: u64) -> Node {
        Node {
            restore: Some(Restore::new(now)),
            ..Node::join(key, site)
        }
    }

    /// Back `roles` with the cached chain reads. Our events name the newest block it has read.
    pub fn attach_chain(&mut self, c: SharedChain) {
        let for_roles = c.clone();
        self.roles = Box::new(move |k, a, block| for_roles.lock().unwrap().role_of_author(k, a, block));
        self.chain = Some(c);
    }

    /// Show what the Storage fetchers learned. `apply_fetched` copies it in.
    pub fn attach_fetched(&mut self, f: SharedFetched) {
        self.fetched = Some(f);
    }

    /// Copy the settings, places, names and reasons that Storage delivered, when they changed. Call
    /// it before a read. A field that Storage has not delivered keeps its value (empty at first).
    pub fn apply_fetched(&mut self) {
        let Some(f) = &self.fetched else { return };
        let f = f.lock().unwrap();
        if f.version == self.fetched_version {
            return;
        }
        self.fetched_version = f.version;
        self.config = f.config.clone().unwrap_or_else(|| self.config.clone());
        self.locations = f.places.clone().unwrap_or_else(|| self.locations.clone());
        self.retired = f.retired.clone();
        self.names = f.names.clone();
        self.about = f.about.clone();
        self.reasons = f.reasons.clone();
    }

    /// A join request from Delivery names the LEZ account of a user. Read it from the chain, so a
    /// member that nobody else knows gets a role once an admin approves. True when it was new.
    /// A request that is not ours, or not signed twice, is ignored.
    pub fn on_join_req(&self, req: &[u8]) -> bool {
        let Ok((body, _)) = crate::joinreq::parse(req) else { return false };
        let ours = body.site_id == self.store.site;
        ours && self.chain.as_ref().is_some_and(|c| c.lock().unwrap().hint(body.user_account, Some(&body.pukaar_ed_pk)))
    }

    /// A profile announce from Delivery: keep it for the fetchers when its signature holds. They
    /// read the file, show the name and learn the account of an author that the chain does not
    /// list. True when it was kept.
    pub fn on_announce(&self, a: Announce) -> bool {
        a.site == self.store.site && self.fetched.as_ref().is_some_and(|f| f.lock().unwrap().note_announce(a))
    }

    /// The sorted keys of the approved members. They elect who answers Heads.
    #[must_use]
    pub fn approved_keys(&self) -> Vec<Key> {
        self.chain.as_ref().map_or_else(Vec::new, |c| c.lock().unwrap().approved_keys().into_iter().collect())
    }

    #[must_use]
    pub fn me(&self) -> Key {
        self.key.verifying_key().to_bytes()
    }

    /// Sign and apply our own event. Validity is the reducer's call, not ours:
    /// an event the rules reject is still recorded, visibly, as rejected.
    pub fn publish(&mut self, body: Body, ts: u64) -> Result<Event, String> {
        if self.restore.is_some() {
            return Err(SYNCING.into());
        }
        let me = self.me();
        let (seq, prev) = self.store.next_seq(&me);
        if let Some(c) = &self.chain {
            self.chain_block = self.chain_block.max(c.lock().unwrap().block);
        }
        let u = Unsigned {
            v: VERSION,
            site: self.store.site,
            author: me,
            seq,
            prev,
            lamport: self.store.max_lamport() + 1,
            ts,
            chain_block: self.chain_block,
            author_account: self.account,
            body,
        };
        let e = sign_within_limits(&self.key, u)?;
        assert_eq!(
            self.store.insert(e.clone()),
            Accept::New,
            "own chain is always valid"
        );
        Ok(e)
    }

    pub fn receive(&mut self, bytes: &[u8]) -> Result<Accept, DecodeError> {
        let e = decode(bytes)?;
        // An author that the chain roster does not know yet: read the account that the event names.
        // The record there must hold the author key, or the reducer rejects the event.
        if e.u.site == self.store.site {
            if let Some(c) = &self.chain {
                c.lock().unwrap().hint(e.u.author_account, Some(&e.u.author));
            }
        }
        let r = self.store.insert(e);
        self.settle();
        Ok(r)
    }

    /// Note a peer's Heads message. True when it ended a restore's wait (time to save).
    pub fn observe_heads(&mut self, theirs: &[(Key, u64)]) -> bool {
        let me = self.me();
        let Some(r) = self.restore.as_mut() else {
            return false;
        };
        r.heads_seen += 1;
        let named = theirs.iter().filter(|(a, _)| *a == me).map(|(_, s)| *s);
        r.claimed = r.claimed.into_iter().chain(named).max();
        self.settle();
        self.restore.is_none()
    }

    fn settle(&mut self) {
        let Some(r) = &self.restore else { return };
        let (next, _) = self.store.next_seq(&self.me());
        if next > 0 && r.heads_seen >= HEADS_TO_SETTLE && r.claimed.is_none_or(|c| next > c) {
            self.restore = None;
        }
    }

    /// The user's way out when the old history is gone for good (it never left the old device).
    pub fn skip_history_sync(&mut self, now: u64) -> Result<(), String> {
        if let Some(r) = &self.restore {
            let wait = (r.since + SKIP_SYNC_AFTER).saturating_sub(now);
            if wait > 0 {
                return Err(format!(
                    "your earlier events may still arrive: you can skip in {} minute(s)",
                    wait.div_ceil(60)
                ));
            }
            self.restore = None;
        }
        Ok(())
    }

    #[must_use]
    pub fn state(&self) -> State {
        reduce(self.store.ordered(), &*self.roles, &self.config, &self.locations)
    }

    /// The newest chain role of `k`.
    #[must_use]
    pub fn role_now(&self, k: &Key) -> Option<Rank> {
        (self.roles)(k, None, u64::MAX)
    }

    /// Is (author, seq) inside a reproducible checkpoint that names a LEZ tx?
    /// Gives the tx and who recorded it (any member can; only the root is checked here).
    #[must_use]
    pub fn anchored<'s>(&self, s: &'s State, author: &Key, seq: u64) -> Option<(&'s str, Key)> {
        s.checkpoints.iter().find_map(|c| {
            let covers = c.heads.iter().any(|(a, h)| a == author && *h >= seq);
            (covers && !c.lez_tx.is_empty() && root_for(&self.store, &c.heads).is_some())
                .then_some((c.lez_tx.as_str(), c.by))
        })
    }

    /// Fill `reason` of a removal row from the text in details.json. The row has the hash.
    fn with_reason(&self, mut row: Value) -> Value {
        let text = row["reason_sha256"].as_str().and_then(|h| hex::decode(h).ok()).and_then(|h| Key::try_from(h).ok()).and_then(|h| self.reasons.get(&h));
        if let Some(t) = text {
            row["reason"] = json!(t);
        }
        row
    }

    /// Our identity, with the chain role we hold now.
    #[must_use]
    pub fn identity_json(&self) -> String {
        let me = self.me();
        let role = self.role_now(&me);
        json!({
            "key": hex::encode(me),
            "fingerprint": fingerprint(&me),
            "role": role.map(rank_name),
            "super_admin": role == Some(Rank::SuperAdmin),
            "name": self.names.get(&me),
            "bio": self.about.get(&me).map(|a| &a.0),
            "experience": self.about.get(&me).map(|a| &a.1),
            "revoked": self.chain.as_ref().and_then(|c| c.lock().unwrap().revocation_of(&me, &self.names)).map(|r| self.with_reason(r)),
            // 2.3b: "requested" once a join request of ours is known
            "pending": "none",
            "site": hex::encode(self.store.site),
            "syncing_own_history": self.restore.is_some(),
        })
        .to_string()
    }

    #[must_use]
    pub fn site_info_json(&self) -> String {
        let s = self.state();
        let place = |l: &Location| {
            let mut v = json!(l);
            v["state"] = json!("active");
            v["open_issues"] = json!(s.open_issues_at(&l.code));
            // A place that no report ever named can be removed (the same hiding as retire).
            v["ever_used"] = json!(s.issues.values().any(|i| i.location == l.code));
            v
        };
        // events no issue timeline shows (a rejected checkpoint), newest first
        let mut rejected: Vec<&Event> = self
            .store
            .events
            .values()
            .filter(|e| s.rejected.contains_key(&e.id) && issue_of(&e.u.body).is_none())
            .collect();
        rejected.sort_by_key(|e| std::cmp::Reverse(e.u.ts));
        let (mut people, chain) = match &self.chain {
            Some(c) => {
                let c = c.lock().unwrap();
                (c.people_json(&self.names), c.status_json())
            }
            None => (
                json!({ "members": [], "super_admins": [], "revoked": [] }),
                json!({ "block": 0, "ok": false, "error": null }),
            ),
        };
        let gone: Vec<Value> = people["revoked"].as_array().into_iter().flatten().map(|r| self.with_reason(r.clone())).collect();
        people["revoked"] = json!(gone);
        let storage = self.fetched.as_ref().map_or(json!({ "pending": 0, "error": null }), |f| f.lock().unwrap().status_json());
        json!({
            "site": hex::encode(self.store.site),
            "storage": storage,
            "name": s.config.name,
            "categories": s.config.categories,
            "locations": s.locations.iter().map(place).collect::<Vec<_>>(),
            "retired_locations": self.retired.iter().map(|l| json!(l)).collect::<Vec<_>>(),
            "members": people["members"], "super_admins": people["super_admins"], "revoked": people["revoked"],
            "pending": [], // 2.3b: the join requests that Delivery brought
            "chain": chain,
            "rejected": rejected.into_iter().map(|e| json!({
                "id": hex::encode(e.id), "kind": kind_name(&e.u.body), "author": hex::encode(e.u.author),
                "author_name": self.names.get(&e.u.author), "ts": e.u.ts, "reason": s.rejected[&e.id],
            })).collect::<Vec<_>>(),
            "sla_ack_h": s.config.sla_ack_h,
            "sla_fix_h": s.config.sla_fix_h,
            "events": self.store.events.len(),
            "forks": self.store.fork_count(),
            "forked_authors": self.store.forked_authors().iter().map(hex::encode).collect::<Vec<_>>(),
            // newest by the recorder's clock; on a tie, max_by_key keeps the later-applied one
            "last_anchor": s.checkpoints.iter().max_by_key(|c| c.ts).map(|c| json!({
                "ts": c.ts,
                "tx": c.lez_tx,
                "by": hex::encode(c.by),
                "by_name": self.names.get(&c.by),
                "events_covered": n_events(&c.heads),
                "reproducible": root_for(&self.store, &c.heads).is_some(),
            })),
        })
        .to_string()
    }

    /// Everything needed to anchor, including the exact `spel` command to run.
    #[must_use]
    pub fn checkpoint_json(&self) -> String {
        let cp = checkpoint_now(&self.store, &self.key);
        let h = |b: &[u8]| hex::encode(b);
        let heads: Vec<Value> = cp.heads.iter().map(|(a, s)| json!([h(a), s])).collect();
        // SPEL CLI: snake_case args become --kebab-case; [u8; 32] args take 64 hex chars.
        let spel = format!(
            "spel anchor --site-id {} --heads-root {} --n-events {} --signer-pk {} --sig-r {} --sig-s {} --payer <YOUR_PUBLIC_ACCOUNT>",
            h(&self.store.site), h(&cp.heads_root), cp.n_events, h(&cp.signer), h(&cp.sig[..32]), h(&cp.sig[32..])
        );
        json!({ "heads": heads, "heads_root": h(&cp.heads_root), "n_events": cp.n_events,
                "signer": h(&cp.signer), "sig": h(&cp.sig), "spel": spel })
        .to_string()
    }

    /// After `spel anchor` succeeds: publish the checkpoint so every client can check it.
    pub fn record_anchor(
        &mut self,
        heads_json: &str,
        lez_ref: &str,
        ts: u64,
    ) -> Result<Event, String> {
        let raw: Vec<(String, u64)> =
            serde_json::from_str(heads_json).map_err(|e| e.to_string())?;
        let heads = raw
            .into_iter()
            .map(|(a, s)| parse_id(&a).map(|k| (k, s)).ok_or("bad author key"))
            .collect::<Result<Vec<_>, _>>()?;
        if lez_ref.trim().is_empty() {
            return Err("missing LEZ reference".into());
        }
        self.publish(
            Body::Checkpoint {
                heads,
                lez_tx: lez_ref.trim().into(),
            },
            ts,
        )
    }

    /// Oldest report first. `now` (unix seconds) drives the overdue flags.
    #[must_use]
    pub fn issues_json(&self, now: u64) -> String {
        let s = self.state();
        let mut v: Vec<&Issue> = s.issues.values().collect();
        v.sort_by_key(|i| (i.reported_ts, i.id));
        Value::Array(v.into_iter().map(|i| issue_json(self, &s, i, now)).collect()).to_string()
    }

    #[must_use]
    pub fn timeline_json(&self, issue_hex: &str, now: u64) -> String {
        let s = self.state();
        let Some(i) = parse_id(issue_hex).and_then(|id| s.issues.get(&id)) else {
            return "null".into();
        };
        let events: Vec<Value> = i
            .timeline
            .iter()
            .filter_map(|id| self.store.events.values().find(|e| &e.id == id))
            .map(|e| {
                let anchor = self.anchored(&s, &e.u.author, e.u.seq);
                json!({
                    "id": hex::encode(e.id),
                    "kind": kind_name(&e.u.body),
                    "author": hex::encode(e.u.author),
                    "author_name": self.names.get(&e.u.author),
                    "former": self.role_now(&e.u.author).is_none(),
                    "ts": e.u.ts,
                    "body": body_text(&e.u.body),
                    "rejected": s.rejected.get(&e.id),
                    "anchored_tx": anchor.map(|(tx, _)| tx),
                    "anchored_by": anchor.map(|(_, by)| hex::encode(by)),
                    "anchored_by_name": anchor.and_then(|(_, by)| self.names.get(&by)),
                })
            })
            .collect();
        json!({ "issue": issue_json(self, &s, i, now), "events": events }).to_string()
    }
}

/// Sign, then apply every check a receiver's `decode` applies (and the text cap to every text
/// field, not just a report's): an event peers drop would stall every later event of ours.
fn sign_within_limits(key: &SigningKey, u: Unsigned) -> Result<Event, String> {
    if u.body.texts().iter().any(|t| t.len() > MAX_TEXT) {
        return Err(format!("too long: keep each text under {MAX_TEXT} bytes"));
    }
    let e = sign(key, u);
    match decode(&e.bytes) {
        Ok(_) => Ok(e),
        Err(DecodeError::TooLarge) => Err(format!(
            "too long: the event is over {MAX_EVENT_BYTES} bytes"
        )),
        Err(other) => Err(format!("cannot sign: {other:?}")),
    }
}

/// UI action name -> event body. `text` is the note/reason, or the target issue id for
/// `mark_duplicate`. `next_step` and `eta_h` are used by `update` (and `eta_h` by `acknowledge`).
pub fn action_body(
    issue: Id,
    action: &str,
    text: &str,
    next_step: &str,
    eta_h: u32,
) -> Result<Body, String> {
    let t = text.to_string();
    Ok(match action {
        "acknowledge" => Body::Acknowledge {
            issue,
            eta_h,
            note: t,
        },
        "update" => Body::Update {
            issue,
            note: t,
            next_step: next_step.to_string(),
            eta_h,
        },
        "claim_resolved" => Body::ClaimResolved { issue, note: t },
        "confirm" => Body::Confirm { issue, note: t },
        "reopen" => Body::Reopen { issue, reason: t },
        "close_wontfix" => Body::CloseWontfix { issue, reason: t },
        "comment" => Body::Comment { issue, text: t },
        "mark_duplicate" => Body::MarkDuplicate {
            issue,
            of: parse_id(text).ok_or("bad duplicate id")?,
        },
        other => return Err(format!("unknown action {other}")),
    })
}

/// 6 hex chars, read aloud at check-in to match a pending member.
#[must_use]
pub fn fingerprint(k: &Key) -> String {
    hex::encode(&k[..3])
}

#[must_use]
pub fn parse_id(h: &str) -> Option<[u8; 32]> {
    hex::decode(h).ok()?.try_into().ok()
}

/// The brief's three stages, plus the honest detail in between.
#[must_use]
pub fn stage(st: Status) -> &'static str {
    match st {
        Status::Open | Status::Acknowledged => "Reported",
        Status::InProgress => "In progress",
        Status::AwaitingConfirmation => "Fix claimed, awaiting confirmation",
        Status::ConfirmedResolved => "Resolved",
        Status::ClosedWontfix => "Closed: won't fix",
        Status::Duplicate => "Closed: duplicate",
    }
}

/// A claimed fix nobody has confirmed for this long is flagged for follow-up.
const CONFIRM_WAIT_H: u32 = 48;

/// `ts` plus `h` hours; None when `h` is 0 (no deadline set).
fn deadline(ts: u64, h: u32) -> Option<u64> {
    (h > 0).then(|| ts + u64::from(h) * 3600)
}

fn past(now: u64, ts: u64, h: u32) -> bool {
    deadline(ts, h).is_some_and(|d| now > d)
}

fn issue_json(n: &Node, s: &State, i: &Issue, now: u64) -> Value {
    let location_label = s
        .locations
        .iter()
        .find(|l| l.code == i.location)
        .map(|l| l.label.clone());
    let progress = i.progress.as_ref().map(|p| {
        json!({
            "by": hex::encode(p.by),
            "by_name": n.names.get(&p.by),
            "ts": p.ts,
            "note": p.note,
            "next_step": p.next_step,
            "eta_h": p.eta_h,
            // the UI compares this with its clock to show "overdue"; 0 = no ETA given
            "due_ts": deadline(p.ts, p.eta_h).unwrap_or(0),
        })
    });
    json!({
        "id": hex::encode(i.id),
        "status": format!("{:?}", i.status),
        "stage": stage(i.status),
        "category": i.category,
        "location": i.location,
        "location_label": location_label,
        "location_retired": false,
        "landmark": i.landmark,
        "text": i.text,
        "progress": progress,
        "reporter": hex::encode(i.reporter),
        "reporter_name": n.names.get(&i.reporter),
        "former": n.role_now(&i.reporter).is_none(),
        "claimant": i.claimant.map(hex::encode),
        "confirms": i.confirms.len(),
        "reopen_count": i.reopen_count,
        "reported_ts": i.reported_ts,
        "acked_ts": i.acked_ts,
        "claimed_ts": i.claimed_ts,
        "ack_overdue": i.status == Status::Open && past(now, i.reported_ts, s.config.sla_ack_h),
        "fix_overdue": i.status.awaits_staff()
            && past(now, i.reported_ts, s.config.sla_fix_h),
        "awaiting_48h": i.status == Status::AwaitingConfirmation
            && i.progress.as_ref().is_some_and(|p| past(now, p.ts, CONFIRM_WAIT_H)),
    })
}

#[must_use]
pub fn kind_name(b: &Body) -> &'static str {
    use Body::*;
    match b {
        Report { .. } => "report",
        Acknowledge { .. } => "acknowledge",
        Update { .. } => "update",
        ClaimResolved { .. } => "claim_resolved",
        Confirm { .. } => "confirm",
        Reopen { .. } => "reopen",
        CloseWontfix { .. } => "close_wontfix",
        MarkDuplicate { .. } => "mark_duplicate",
        Comment { .. } => "comment",
        Checkpoint { .. } => "checkpoint",
    }
}

fn body_text(b: &Body) -> String {
    use Body::*;
    match b {
        Comment { text, .. } => text.clone(),
        Report { text, landmark, .. } if landmark.is_empty() => text.clone(),
        Acknowledge { note, eta_h, .. } => format!("ETA {eta_h} h. {note}"),
        Update {
            note,
            next_step,
            eta_h,
            ..
        } => {
            let mut t = note.clone();
            if !next_step.is_empty() {
                t += &format!(" Next: {next_step}.");
            }
            if *eta_h > 0 {
                t += &format!(" ETA {eta_h} h.");
            }
            t
        }
        Report { text, landmark, .. } => format!("{text} ({landmark})"),
        ClaimResolved { note, .. } | Confirm { note, .. } => note.clone(),
        Reopen { reason, .. } | CloseWontfix { reason, .. } => reason.clone(),
        MarkDuplicate { of, .. } => format!("duplicate of {}", hex::encode(of)),
        _ => String::new(),
    }
}
