# Pukaar

**Civic fault reports that can't be quietly closed.** A [Logos Basecamp](https://github.com/logos-co/logos-basecamp) app for the Field Station civic reporting brief. Every action is a signed event in its author's own append-only log, synced peer-to-peer over Logos Delivery. A report only counts as resolved when the person who filed it (or two other residents) confirms. Checkpoints of all logs are anchored on the Logos Execution Zone (LEZ), so the history can't be rewritten later.

[![ci](https://github.com/adysingh5711/pukaar/actions/workflows/ci.yml/badge.svg)](https://github.com/adysingh5711/pukaar/actions/workflows/ci.yml)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![status](https://img.shields.io/badge/status-in%20development-orange)

| | |
|---|---|
| Demo video | To be added in later version |
| Screenshots | To be added in later version |
| Latest release | To be added in later version |

---

## Contents

- [Why](#why)
- [How it works](#how-it-works)
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
- **Checkpoints.** A Merkle root (RFC 6962 style) over every author's chain head, signed by a member under `logos:pukaar:cp:1\0` and anchored with one SPEL instruction on LEZ. Anyone can recompute the root from their own replica and compare.

## Project status

| Component | Path | Status |
|---|---|---|
| Rules engine (`pukaar_logic`): events, chains, reducer, sync, checkpoints, persistence | `modules/pukaar_core/rust-lib/logic/` | **Done.** 28 tests, clippy `-D warnings` clean, fuzzed in CI |
| Checkpoint registry (`pukaar_registry`), a SPEL program on LEZ | `programs/pukaar_registry/` | **Done.** Deployed and anchored on localnet ([record](programs/pukaar_registry/ANCHOR.md)) |
| Logos core module (`pukaar_core`), a Rust cdylib plus Delivery glue | `modules/pukaar_core/` | In progress |
| QML UI module (`pukaar_ui`) | `modules/pukaar_ui/` | To be added in later version |
| Two-instance demo script and signed `.lgx` packages for Basecamp 0.3.0 | `scripts/`, releases | To be added in later version |
| LAN relay and offline three-laptop dry run | | To be added in later version |

## Architecture

```
┌──────────────────────── Logos Basecamp 0.3.0 ─────────────────────────┐
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
- For the Basecamp modules: Nix with flakes, and [Logos Basecamp 0.3.0](https://github.com/logos-co/logos-basecamp/releases/tag/0.3.0)

### Run the rules engine tests

```bash
cd modules/pukaar_core/rust-lib/logic
cargo test                                   # 28 tests
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

To be added in later version.

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
| Rewriting history (equivocation) | Per-author hash chains. Two events at one `(author, seq)` form a self-contained fork proof, flagged on the board |
| Spam | A deterministic open-report cap per key. The admin can revoke roles, and the reports stay visible |
| Malformed input | Strict decoder with size limits, the signature checked before decoding, and a `cargo-fuzz` target run in CI |
| Junk on-chain checkpoints | One PDA per (site, root). Signature and membership are checked off-chain |
| Flooding the channel | Rate-limited `Heads` answers. Only 3 elected members plus the kiosk re-send others' events (max 200) |

**What Pukaar does not claim:** that a problem was physically fixed (only who said so), anonymity against someone who knows the site, network-level unlinkability, resistance to an admin who controls every device, or that the chain makes inputs honest.

| | |
|---|---|
| Security audit | To be added in later version |
| Bug bounty | To be added in later version |
| Release signing (publisher DID, `lgx verify`) | To be added in later version |
| Key storage at rest (password-sealed key file) | To be added in later version |
| Reporting a vulnerability | Open a private [security advisory](https://github.com/adysingh5711/pukaar/security/advisories/new) on this repository |

## Repository layout

```
modules/pukaar_core/
  rust-lib/               the Logos module crate (in progress)
    logic/                pukaar_logic: every rule, tested with plain cargo,
                          nested here so the Nix build's crate-dir staging
                          sees it (single source of truth, no copy)
      src/event.rs        signed envelope, strict decode
      src/store.rs        per-author chains, pending buffer, fork proofs
      src/checkpoint.rs   Merkle root, checkpoint signature, verification
      src/sync.rs         Wire enum, heads-based anti-entropy
      src/reducer.rs      state machine → State
      src/node.rs         one participant: publish/receive + JSON views
      src/persist.rs      atomic on-disk replica
      tests/              event, store, checkpoint, sync, lifecycle, convergence, persist
      fuzz/               cargo-fuzz decode target
    src/lib.rs            Logos glue: PukaarCoreModule trait + delivery_module wiring
programs/pukaar_registry/  SPEL checkpoint registry (LEZ)
.github/workflows/ci.yml   tests, clippy, 60 s fuzz smoke run
```

## Tech stack

| Layer | Choice |
|---|---|
| Logic | Rust (`ed25519-dalek` 2.1, `postcard` 1.1, `sha2` 0.10, `serde`, `serde_json`, `hex`) |
| Module packaging | Nix flakes, `logos-module-builder` 0.3.1, portable `.lgx` |
| Transport | `delivery_module` v0.3.0-rc.2 (Reliable Channels) |
| UI | Qt 6 QML inside Logos Basecamp 0.3.0 |
| Chain | LEZ, SPEL (`lez-framework`), RISC Zero zkVM, `logos-scaffold` |
| CI | GitHub Actions: `cargo test`, `clippy -D warnings`, `cargo-fuzz` smoke run |

## Roadmap

- **L1:** the full report → claim → confirm loop between two Basecamp instances, and signed portable `.lgx` packages.
- **L1+:** anchoring from the app's Anchor tab, with a ✓ on each timeline row; a testnet anchor.
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
