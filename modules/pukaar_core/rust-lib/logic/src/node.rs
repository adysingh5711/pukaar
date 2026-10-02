//! One participant: their key plus their replica of the site. Everything the
//! Logos module exposes is a thin wrapper over this, so it's all testable with `cargo test`.

use crate::checkpoint::{checkpoint_now, root_for};
use crate::event::{decode, sign, Body, DecodeError, Event, Id, Key, Unsigned, VERSION, ZERO};
use crate::reducer::{reduce, Issue, State, Status};
use crate::store::{Accept, Store};
use ed25519_dalek::SigningKey;
use serde_json::{json, Value};

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
}

impl Node {
    pub fn create_site(key: SigningKey, genesis: Body, ts: u64) -> Node {
        let u = Unsigned {
            v: VERSION,
            site: ZERO,
            author: key.verifying_key().to_bytes(),
            seq: 0,
            prev: ZERO,
            lamport: 1,
            ts,
            body: genesis,
        };
        let e = sign(&key, u);
        let mut store = Store::new(e.id);
        assert_eq!(store.insert(e), Accept::New);
        Node {
            key,
            store,
            restore: None,
        }
    }

    pub fn join(key: SigningKey, site: Id) -> Node {
        Node {
            key,
            store: Store::new(site),
            restore: None,
        }
    }

    /// Join with an identity that already has history here: publish nothing until it's back.
    pub fn join_restored(key: SigningKey, site: Id, now: u64) -> Node {
        Node {
            restore: Some(Restore::new(now)),
            ..Node::join(key, site)
        }
    }

    /// Join and announce ourselves with an empty profile, so the admin sees us as a pending
    /// member even if we never set a name (staying a pseudonym).
    pub fn join_announced(key: SigningKey, site: Id, ts: u64) -> Node {
        let mut n = Node::join(key, site);
        n.publish(Body::Profile { display_name: None }, ts)
            .expect("a fresh node publishes freely");
        n
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
        let u = Unsigned {
            v: VERSION,
            site: self.store.site,
            author: me,
            seq,
            prev,
            lamport: self.store.max_lamport() + 1,
            ts,
            body,
        };
        let e = sign(&self.key, u);
        assert_eq!(
            self.store.insert(e.clone()),
            Accept::New,
            "own chain is always valid"
        );
        Ok(e)
    }

    pub fn receive(&mut self, bytes: &[u8]) -> Result<Accept, DecodeError> {
        let r = self.store.insert(decode(bytes)?);
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
        reduce(self.store.ordered())
    }

    /// Is (author, seq) inside a reproducible checkpoint that names a LEZ tx?
    #[must_use]
    pub fn anchored(&self, s: &State, author: &Key, seq: u64) -> Option<String> {
        s.checkpoints.iter().find_map(|(_, heads, tx)| {
            let covers = heads.iter().any(|(a, h)| a == author && *h >= seq);
            (covers && !tx.is_empty() && root_for(&self.store, heads).is_some()).then(|| tx.clone())
        })
    }

    #[must_use]
    pub fn identity_json(&self) -> String {
        let s = self.state();
        let me = self.me();
        json!({
            "key": hex::encode(me),
            "fingerprint": fingerprint(&me),
            "role": s.roles.get(&me).map(|r| format!("{r:?}")),
            "name": s.names.get(&me),
            "site": hex::encode(self.store.site),
            "syncing_own_history": self.restore.is_some(),
        })
        .to_string()
    }

    #[must_use]
    pub fn site_info_json(&self) -> String {
        let s = self.state();
        let member = |k: &Key| json!({ "key": hex::encode(k), "fingerprint": fingerprint(k), "name": s.names.get(k) });
        json!({
            "site": hex::encode(self.store.site),
            "name": s.config.name,
            "categories": s.config.categories,
            "locations": s.locations.iter().map(|l| json!(l)).collect::<Vec<_>>(),
            "members": s.roles.iter().map(|(k, r)| { let mut m = member(k); m["role"] = json!(format!("{r:?}")); m }).collect::<Vec<_>>(),
            "pending": s.pending_members.iter().map(member).collect::<Vec<_>>(),
            "events": self.store.events.len(),
            "forks": self.store.forks.len(),
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

    #[must_use]
    pub fn issues_json(&self) -> String {
        let s = self.state();
        let v: Vec<Value> = s.issues.values().map(|i| issue_json(&s, i)).collect();
        Value::Array(v).to_string()
    }

    #[must_use]
    pub fn timeline_json(&self, issue_hex: &str) -> String {
        let s = self.state();
        let Some(i) = parse_id(issue_hex).and_then(|id| s.issues.get(&id)) else {
            return "null".into();
        };
        let events: Vec<Value> = i
            .timeline
            .iter()
            .filter_map(|id| self.store.events.values().find(|e| &e.id == id))
            .map(|e| {
                json!({
                    "id": hex::encode(e.id),
                    "kind": kind_name(&e.u.body),
                    "author": hex::encode(e.u.author),
                    "author_name": s.names.get(&e.u.author),
                    "ts": e.u.ts,
                    "body": body_text(&e.u.body),
                    "rejected": s.rejected.get(&e.id),
                    "anchored_tx": self.anchored(&s, &e.u.author, e.u.seq),
                })
            })
            .collect();
        json!({ "issue": issue_json(&s, i), "events": events }).to_string()
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

/// `{"Genesis": {...}}`, the serde shape of `Body::Genesis`.
/// The create form's placeholder: a genesis still carrying it names nobody.
const NAME_PLACEHOLDER: &str = "<your name>";

/// Staff are always named, so a new site's admin must be too. Checked here, at creation,
/// not in the reducer: replaying an already-shared genesis must never change its outcome.
pub fn genesis_from_json(json: &str) -> Result<Body, String> {
    match serde_json::from_str::<Body>(json) {
        Ok(Body::Genesis { ref admin_name, .. })
            if matches!(admin_name.trim(), "" | NAME_PLACEHOLDER) =>
        {
            Err("the admin must be named".into())
        }
        Ok(b @ Body::Genesis { .. }) => Ok(b),
        Ok(_) => Err("not a genesis body".into()),
        Err(e) => Err(e.to_string()),
    }
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

fn issue_json(s: &State, i: &Issue) -> Value {
    let location_label = s
        .locations
        .iter()
        .find(|l| l.code == i.location)
        .map(|l| l.label.clone());
    let progress = i.progress.as_ref().map(|p| {
        json!({
            "by": hex::encode(p.by),
            "by_name": s.names.get(&p.by),
            "ts": p.ts,
            "note": p.note,
            "next_step": p.next_step,
            "eta_h": p.eta_h,
            // the UI compares this with its clock to show "overdue"; 0 = no ETA given
            "due_ts": if p.eta_h > 0 { p.ts + p.eta_h as u64 * 3600 } else { 0 },
        })
    });
    json!({
        "id": hex::encode(i.id),
        "status": format!("{:?}", i.status),
        "stage": stage(i.status),
        "category": i.category,
        "location": i.location,
        "location_label": location_label,
        "landmark": i.landmark,
        "text": i.text,
        "progress": progress,
        "reporter": hex::encode(i.reporter),
        "reporter_name": s.names.get(&i.reporter),
        "claimant": i.claimant.map(hex::encode),
        "confirms": i.confirms.len(),
        "reopen_count": i.reopen_count,
        "reported_ts": i.reported_ts,
    })
}

#[must_use]
pub fn kind_name(b: &Body) -> &'static str {
    use Body::*;
    match b {
        Genesis { .. } => "genesis",
        RoleGrant { .. } => "role_grant",
        RoleRevoke { .. } => "role_revoke",
        Profile { .. } => "profile",
        LocationsAdd { .. } => "locations_add",
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
