# Pukaar

**Civic fault reports that can't be quietly closed.** A [Logos Basecamp](https://github.com/logos-co/logos-basecamp) app for the Field Station civic reporting brief. Every action is a signed event in its author's own append-only log, synced peer-to-peer over Logos Delivery. A report only counts as resolved when the person who filed it (or two other residents) confirms. Checkpoints of all logs are anchored on the Logos Execution Zone (LEZ), so the history can't be rewritten later.

[![ci](https://github.com/adysingh5711/pukaar/actions/workflows/ci.yml/badge.svg)](https://github.com/adysingh5711/pukaar/actions/workflows/ci.yml)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![status](https://img.shields.io/badge/status-pilot--ready%20v0.1-blue)

| | |
|---|---|
| Demo video | To be added in later version |
| Screenshots | To be added in later version |
| Latest release | [v0.2.1](https://github.com/adysingh5711/pukaar/releases/tag/v0.2.1): signed `.lgx` packages for Logos Basecamp 0.3.1 |

---

## Contents

- [Why](#why)
- [How it works](#how-it-works)
- [Features](#features)
- [Project status](#project-status)
- [Architecture](#architecture)
- [On-chain component: `pukaar_registry`](#on-chain-component-pukaar_registry)
- [Deployments](#deployments)
- [Getting started](#getting-started)
- [Verifying an anchor yourself](#verifying-an-anchor-yourself)
- [Security](#security)
- [Repository layout](#repository-layout)
- [Tech stack](#tech-stack)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Why

Taking in a fault report is easy. Closing it is where fault reporting fails: whoever runs the system decides what counts as "resolved". In FixMyStreet, the best-known open-source tool, whether the public can even reopen a closed report is a per-council setting.

Pukaar takes that power away from the operator:

- **No server and no delete.** Every member holds a full replica. There's no delete event, action or button anywhere. An invalid event is kept, marked `rejected`, and shown.
- **Two-key closure.** A steward can *claim* a fix. Only the reporter, or two distinct residents who aren't the claimant, can *confirm* it. The reporter can always reopen.
- **Staff are named, residents choose.** Stewards and admins are always named, by the admin's grant. Residents are named or pseudonymous, and never named by the admin. Every staff update and fix claim must say what was done.
- **Tamper-evident history.** Per-author hash chains expose any rewrite (a fork proof). Checkpoints anchored on LEZ fix the history even for people who join later.

## How it works

```
 report ─▶ OPEN ─acknowledge─▶ ACKNOWLEDGED ─update─▶ IN_PROGRESS ⟲ update
             │                                              │
             └───────── claim_resolved (needs a note) ──────┴─▶ AWAITING_CONFIRMATION
                                                                     │
                     confirm: the reporter, or 2 residents, never the claimant
                                                                     ▼
                                                            CONFIRMED_RESOLVED
   close_wontfix / mark_duplicate from any open state.  reopen: reporter or 2 residents.
```

- **Events.** Each action is an ed25519-signed, postcard-encoded envelope (`author`, `seq`, `prev` hash, Lamport clock, body) under the domain `logos:pukaar:1\0`. The decoder is strict: events are at most 4 KiB, text fields at most 500 B, trailing bytes are rejected, and the signature is checked before the body is decoded.
- **Ordering.** Events apply in `(lamport, author, id)` order, so every replica computes the same board no matter what order messages arrive in. A property test replays 60 random histories in 20 shuffled delivery orders each and checks that the final states are identical.
- **Transport.** Live events go over `delivery_module` Reliable Channels on the content topic `/pukaar/1/site-<site_hex>/proto`. Catch-up is heads-based anti-entropy with a capped resend, not Store queries, so late joiners, restarts and LAN-only sites work without a store node.
- **Forks.** If one identity ever signs two different events at the same position (one person on two devices, or a cheat), every replica keeps the **same** branch: at each position the event with the lowest id wins, whatever order the copies arrive in. The losing branch is kept and saved as evidence, never applied, and the board shows a fork warning.
- **Checkpoints.** A Merkle root (RFC 6962 style) over every author's chain head, signed by a member under `logos:pukaar:cp:1\0` and anchored with one SPEL instruction on LEZ. Anyone can recompute the root from their own replica and compare.

## Features

- **Report** what's wrong and where: a category, one line, and a place picked from the site's grouped, searchable location list (or "Other" with a description).
- **Staff workflow:** acknowledge, post updates with the next step and an ETA, claim a fix (a note is required), close as won't-fix, or mark a duplicate. Overdue and waiting-for-48 h cards are highlighted.
- **Two-key closure:** only the reporter, or two residents who aren't the claimant, can confirm a fix. The reporter can always reopen.
- **Members:** join with the site id; the admin approves each person after hearing their fingerprint read aloud. Staff are always named by the admin, and residents choose a name or stay pseudonymous. The admin can revoke a role, with a reason.
- **Site setup:** the admin creates a site from a guided form: site name, their own name, editable categories, and a places table that starts empty (or from the Dhun sample). Service rules (acknowledge and fix targets in hours, open reports per person) are plain fields with explanations.
- **Locations:** the admin adds places and can **edit** a place's name or group (its code never changes; old issues show "renamed from …"). A place in use can be **retired** (blocked while it has open issues), and a retired place offers "Report as Other at this spot". A place nobody ever reported can be **removed**: it shows as pending for 30 days with Undo, then is hidden from every list and listed in the admin's change log. A duplicate code is refused.
- **Identity backup:** export a password-sealed copy of your identity (Argon2id + XChaCha20-Poly1305) and restore it on a new install. A restored identity waits for its own history before it can publish, so it can't fork its own chain.
- **Anchor:** compute a checkpoint of everyone's history and anchor it on LEZ with the printed `spel` command. Covered events show who recorded the anchor.
- **Nothing is ever deleted:** rejected actions are kept and shown. "Removed" places are only hidden; their signed events stay in everyone's log. A forked chain is flagged, and every replica settles on the same branch.
- **One identity, one device:** your key is your identity (there are no accounts). Run it on one device at a time; a restored identity waits for its own history before it can act.

## Project status

| Component | Path | Status |
|---|---|---|
| Rules engine (`pukaar_logic`): events, chains, reducer, sync, checkpoints, persistence, identity backup | `modules/pukaar_core/rust-lib/logic/` | **Done.** 111 tests, clippy `-D warnings` clean, fuzzed in CI |
| Checkpoint registry (`pukaar_registry`), a SPEL program on LEZ | `programs/pukaar_registry/` | **Done.** Deployed and anchored on localnet ([record](programs/pukaar_registry/ANCHOR.md)) |
| Logos core module (`pukaar_core`), a Rust cdylib plus Delivery glue | `modules/pukaar_core/` | **Done.** Runs in the standalone Logos host and inside Basecamp 0.3.1 |
| QML UI module (`pukaar_ui`) | `modules/pukaar_ui/` | **Done.** Board, Report, Members, Anchor and Identity tabs |
| Demo scripts and signed `.lgx` packages for Basecamp 0.3.1 | `scripts/`, [releases](https://github.com/adysingh5711/pukaar/releases) | **Done.** Packages signed with `lgx` (see Security) |
| LAN relay and offline three-laptop dry run | | To be added in later version |

## Architecture

```
┌──────────────────────── Logos Basecamp 0.3.1 ─────────────────────────┐
│  pukaar_ui (QML only)  ──logos.callModule──▶  pukaar_core (Rust cdylib)│
│  polls every 2 s, JSON in/out                 │                        │
│                                               ├─ pukaar_logic (all rules)
│                                               │   event · store · sync  │
│                                               │   reducer · node        │
│                                               │   checkpoint · persist  │
│                                               └─ delivery_module        │
│                                                  Reliable Channels      │
└────────────────────────────────────────┬──────────────────────────────┘
                                         │ spel anchor (IDL-generated CLI)
                                         ▼
                           LEZ ── pukaar_registry: CheckpointRecord PDAs
```

All the logic lives in `pukaar_logic`, a plain Rust crate that you can test with `cargo test` and no Nix. The module crate only moves bytes between the `Node`, the disk and Delivery. Structured data crosses the module boundary as JSON strings. A failing call returns a string that starts with `error: `, or with `rejected: ` when the rules made the event inert.

**Wire format (frozen).** Domains `logos:pukaar:1\0` and `logos:pukaar:cp:1\0`. The field order of the unsigned envelope is fixed. `Body` is an append-only enum: new variants are added at the end only.

## On-chain component: `pukaar_registry`

A one-instruction [SPEL](https://github.com/logos-co/spel) program ([source](programs/pukaar_registry/methods/guest/src/bin/pukaar_registry.rs), [IDL](programs/pukaar_registry/idl/pukaar_registry.json)).

| Item | Value |
|---|---|
| Instruction | `anchor(site_id, heads_root, n_events, signer_pk, sig_r, sig_s)` with accounts `record` (init) and `payer` (signer) |
| Account type | `CheckpointRecord { site_id, heads_root, n_events, signer_pk, sig_r, sig_s }` (Borsh) |
| PDA seeds | `["cp", site_id, heads_root]`: one record per (site, root) |
| Replay / DoS | Re-anchoring the same root fails at `init`. Nothing is sequential, so a junk anchor can't block an honest one |
| Verification | Off-chain: recompute the root from your replica, verify `sig_r‖sig_s` against `signer_pk`, and check that the signer is a site member |
| Client | The IDL-generated `spel` CLI is the only chain client. The app prints the exact `spel anchor …` command |

## Deployments

| Network | Program ID | Status |
|---|---|---|
| LEZ localnet (`logos-scaffold`, `RISC0_DEV_MODE=1`) | `23365d872303f303c7ca56cdfbca1356e38911facd08b37bdefe5e9eb9490890` | Deployed. First anchor tx `9cca68dd…d5b6`, record PDA `Fy4ehfAjFJvrZgWnPu5fGC2bLRgbbpp3at4CjL9e35Yu` ([full record and `spel inspect` output](programs/pukaar_registry/ANCHOR.md)) |
| LEZ testnet | To be added in later version | To be added in later version |
| Mainnet | To be added in later version | To be added in later version |

> **Localnet is not permanent.** Its chain state is discarded when the local sequencer stops. The hosted LEZ testnet was also reset around 8 Sept 2026, so a testnet anchor is only as permanent as the testnet.

## Getting started

### Prerequisites

- Rust 1.96+ (`rustup`)
- For the chain program: Docker, [`logos-scaffold`](https://github.com/logos-co/scaffold) (`cargo install logos-scaffold`, which provides `lgs`), and the RISC Zero toolchain (`rzup install rust`, `rzup install r0vm`)
- For the Basecamp modules: Nix with flakes, and [Logos Basecamp 0.3.1](https://github.com/logos-co/logos-basecamp/releases/tag/0.3.1)
- Hindi text in reports uses the system's Devanagari font, so a bare Linux kiosk should install `fonts-noto-core`

### Run the rules engine tests

```bash
cd modules/pukaar_core/rust-lib/logic
cargo test                                   # 111 tests
cargo clippy --all-targets -- -D warnings
cargo +nightly fuzz run decode -- -max_total_time=60   # needs cargo-fuzz
```

### Build and deploy the registry on localnet

```bash
cd programs/pukaar_registry
lgs run          # builds the guest, starts localnet, funds the wallet, deploys
spel --idl idl/pukaar_registry.json --program <PROGRAM_ID> -- anchor \
  --site-id <64 hex> --heads-root <64 hex> --n-events <n> \
  --signer-pk <64 hex> --sig-r <64 hex> --sig-s <64 hex> --payer Public/<ACCOUNT>
```

### Run Pukaar in Basecamp

1. Download `logos-pukaar_core-module-lib.lgx` and `logos-pukaar_ui-module.lgx` from the [latest release](https://github.com/adysingh5711/pukaar/releases/latest), and check them (see Security → Release signing).
2. In Basecamp 0.3.1: **Package Manager → Install Local Package**. Install the core package first, then the UI package.
3. Open Pukaar. Everyone except the admin pastes the site id under **Join a site**, then reads their fingerprint aloud at the kiosk so the admin can approve them. The admin opens **Create a site** instead, fills in the form, and clicks **Create site**. **Restore your identity** brings back an exported identity on a new install.

One Basecamp profile holds one Pukaar identity. To run several people on one machine, give each its own Basecamp data directory, side by side:

```bash
scripts/basecamp.sh default    # your normal Basecamp profile
scripts/basecamp.sh asha       # a separate profile (install Pukaar once in it)
```

### Try it from source (developers)

```bash
scripts/demo.sh --clean        # two Pukaar windows, admin and resident, built from source with Nix
scripts/window.sh <name>       # one more window as a new or existing person; never deletes data
```

## Verifying an anchor yourself

```bash
cd programs/pukaar_registry
spel inspect <RECORD_PDA> --idl idl/pukaar_registry.json --type CheckpointRecord
```

The `heads_root` and `n_events` it prints must equal the values that your own replica computes for that checkpoint. A mismatch means the anchored history and your history disagree.

## Security

**Threat model (summary).**

| Attack | Mitigation |
|---|---|
| Deleting an embarrassing report | No server and no delete event. Replicas on every device. Anchored heads |
| Closing a ticket that isn't fixed | A claim isn't resolution. It needs the reporter, or 2 residents, and never the claimant. Reopen is always allowed |
| Backdating an acknowledgement | `ts` is informational. Order comes from the chain and the Lamport clock. Checkpoints bound when an event existed |
| Rewriting history (equivocation) | Per-author hash chains. Two events at one `(author, seq)` form a self-contained fork proof, kept as evidence and flagged on the board. Every replica keeps the same branch (lowest id wins), so boards still converge |
| One identity used on two devices | Treated like a fork: both versions kept, one deterministic winner everywhere, a fork warning on the board. A restored identity can't publish until its own history has synced back |
| Removing an inconvenient place | Only places nobody ever reported can be removed, with a reason, after a 30-day pending period anyone can see; places with reports can only be retired |
| Spam | A deterministic open-report cap per key. The admin can revoke roles, and the reports stay visible |
| Malformed input | Strict decoder with size limits, the signature checked before decoding, and a `cargo-fuzz` target run in CI |
| Junk on-chain checkpoints | One PDA per (site, root). Signature and membership are checked off-chain |
| Flooding the channel | Rate-limited `Heads` answers. Only 3 elected members plus the kiosk re-send others' events (max 200) |

**What Pukaar does not claim:** that a problem was physically fixed (only who said so), anonymity against someone who knows the site, network-level unlinkability, resistance to an admin who controls every device, or that the chain makes inputs honest.

| | |
|---|---|
| Security audit | To be added in later version |
| Bug bounty | To be added in later version |
| Release signing | Packages are signed with `lgx`. Publisher DID: `did:jwk:eyJjcnYiOiJFZDI1NTE5Iiwia3R5IjoiT0tQIiwieCI6IlpfZkxKcnVWR3UyYnBiR0VNMlhMTElmY2FzdTFycVkycHJZM1Z0cklRR28ifQ`. Verify with `lgx keyring add publisher "<DID>" --dir ./trusted-keys` then `lgx verify <file>.lgx --keyring-dir ./trusted-keys`. The official index ships `trustedSigners: []`, so Basecamp doesn't enforce signatures yet |
| Key storage at rest | The key file is `0600` in the host's data directory. A password-sealed **backup** is available (Identity tab). Sealing the live key file itself: to be added in later version |
| Reporting a vulnerability | Open a private [security advisory](https://github.com/adysingh5711/pukaar/security/advisories/new) on this repository |

## Repository layout

```
modules/pukaar_core/
  rust-lib/               the Logos module crate
    logic/                pukaar_logic: every rule, tested with plain cargo,
                          nested here so the Nix build's crate-dir staging
                          sees it (single source of truth, no copy)
      src/event.rs        signed envelope, strict decode
      src/store.rs        per-author chains, pending buffer, deterministic fork choice + evidence
      src/checkpoint.rs   Merkle root, checkpoint signature, verification
      src/sync.rs         Wire enum, heads-based anti-entropy
      src/reducer.rs      state machine → State
      src/node.rs         one participant: publish/receive + JSON views
      src/persist.rs      atomic on-disk replica, leave/rejoin
      src/identity.rs     password-sealed identity export/import
      tests/              event, store, checkpoint, sync, lifecycle, convergence, persist, identity, retire, location, setup
      fuzz/               cargo-fuzz decode target
    src/lib.rs            Logos glue: PukaarCoreModule trait + delivery_module wiring
modules/pukaar_ui/Main.qml  the QML UI (one call funnel, shared components, one colour palette)
programs/pukaar_registry/  SPEL checkpoint registry (LEZ)
scripts/                   demo.sh, window.sh, basecamp.sh
.github/workflows/ci.yml   tests, clippy, 60 s fuzz smoke run
```

## Tech stack

| Layer | Choice |
|---|---|
| Logic | Rust (`ed25519-dalek` 2.1, `postcard` 1.1, `sha2` 0.10, `serde`, `serde_json`, `hex`; `argon2` 0.5 and `chacha20poly1305` 0.10 for the identity backup) |
| Module packaging | Nix flakes, `logos-module-builder` 0.3.1, portable `.lgx` |
| Transport | `delivery_module` v0.3.0-rc.2 (Reliable Channels) |
| UI | Qt 6 QML (Controls Basic) inside Logos Basecamp 0.3.1 |
| Chain | LEZ, SPEL (`lez-framework`), RISC Zero zkVM, `logos-scaffold` |
| CI | GitHub Actions: `cargo test`, `clippy -D warnings`, `cargo-fuzz` smoke run |

## Roadmap

- **L1 (done in v0.1.0):** the full report → claim → confirm loop between instances over Logos Delivery, running inside Basecamp, with signed portable `.lgx` packages.
- **v0.1.1:** guided site setup, edit and remove locations (30-day pending removal), a change log, service rules as form fields, deterministic fork resolution.
- **v0.2.0:** UI revamp: light and dark themes, a collapsible sidebar, redesigned board, cards and issue pane (with a "Why it isn't closed yet" card), a board history window with a History page, and Hindi user content.
- **v0.2.1:** fix: calls no longer hang for 20 s when Logos Delivery is unreachable (5 s Delivery timeouts plus a 10 s back-off), and clearer error text when the core does not answer.
- **L1+:** anchoring from the app's Anchor tab on localnet and a testnet anchor; an "identity active on another device" warning; Heads messages that carry head ids, so anti-entropy also repairs equal-length forks.
- **L2:** LAN relay for sites without internet, a password-sealed key file, a site-health view (category level only, no per-person ranking), export and verify bundles, evidence photos on Logos Storage (EXIF stripped), UI tests, and a steward guide in Hindi and English.
- **L3:** an anonymous reporting lane with an RLN rate limit, a phone path, and 2-of-3 admin grants.

Dates and milestones beyond L1: to be added in later version.

## Contributing

Issues and pull requests are welcome. Before opening a PR, run `cargo test`, `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` in `modules/pukaar_core/rust-lib/logic`. Changes to the wire format must keep `Body` append-only.

Contribution guidelines and code of conduct: to be added in later version.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

## Acknowledgements

Built on [Logos](https://logos.co) (Basecamp, Delivery, LEZ, SPEL and `logos-scaffold`) for the Field Station civic reporting brief.
