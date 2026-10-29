# Developing Pukaar

How to test the rules engine, build the registry, try Pukaar from source, find your way around the repository, contribute and make a release. [Back to the README](../README.md).

## Run the rules engine tests

```bash
cd modules/pukaar_core/rust-lib/logic
cargo test                                   # 144 tests
cargo clippy --all-targets -- -D warnings
cargo +nightly fuzz run decode -- -max_total_time=60   # needs cargo-fuzz
```

## Build and deploy the registry on localnet

```bash
cd programs/pukaar_registry
lgs run          # builds the guest, starts localnet, funds the wallet, deploys
spel --idl idl/pukaar_registry.json --program <PROGRAM_ID> -- anchor \
  --site-id <64 hex> --heads-root <64 hex> --n-events <n> \
  --signer-pk <64 hex> --sig-r <64 hex> --sig-s <64 hex> --payer Public/<ACCOUNT>
```

## Try it from source (developers)

```bash
scripts/demo.sh --clean        # two Pukaar windows, admin and resident, built from source with Nix
scripts/window.sh <name>       # one more window as a new or existing person; never deletes data
```

Each window keeps its data in a directory in `PUKAAR_DEMO_DIR`, and the default is `~/.local/share/pukaar-demo`. `--clean` deletes only the `resident-a` and `steward-b` directories there.

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
      tests/              event, store, checkpoint, sync, lifecycle, convergence, persist, identity, retire, location, setup, super_admin
        fixtures/         v0_2_4_log.json: a v0.2.4 log that must replay to the same state
      fuzz/               cargo-fuzz decode target
    src/lib.rs            Logos glue: PukaarCoreModule trait + delivery_module wiring
modules/pukaar_ui/Main.qml  the QML UI (one call funnel, shared components, one colour palette)
programs/pukaar_registry/  SPEL checkpoint registry (LEZ)
  README.md                logos-scaffold template guide: build, deploy, IDL
  ANCHOR.md                localnet deployment and first anchor record
scripts/                   demo.sh, window.sh, basecamp.sh
  ui-shots.sh, .qml        offscreen screenshots of every page against fixture data
public/brand/              Pukaar logos (SVG, PNG)
public/images/             README screenshots
LICENSE-MIT, LICENSE-APACHE  the two licences
THIRD_PARTY_LICENSES       Lucide icon licence
.github/workflows/ci.yml   tests, clippy, 60 s fuzz smoke run, version match
```

## Tech stack

| Layer | Choice |
|---|---|
| Logic | Rust (`ed25519-dalek` 2.1, `postcard` 1.1, `sha2` 0.10, `serde`, `serde_json`, `hex`; `argon2` 0.5 and `chacha20poly1305` 0.10 for the identity backup) |
| Module packaging | Nix flakes, `logos-module-builder` 0.3.1, portable `.lgx` |
| Transport | `delivery_module` v0.3.0-rc.2 (Reliable Channels) |
| UI | Qt 6 QML (Controls Basic) inside Logos Basecamp 0.3.1, [Lucide](https://lucide.dev) icons |
| Chain | LEZ, SPEL (`lez-framework`), RISC Zero zkVM, `logos-scaffold` |
| CI | GitHub Actions: `cargo test`, `clippy -D warnings`, `cargo-fuzz` smoke run, `version` job. If the two `metadata.json` versions differ or a `v*` tag does not match them, the `version` job fails |

## Project status

| Component | Path | Status |
|---|---|---|
| Rules engine (`pukaar_logic`): events, chains, reducer, sync, checkpoints, persistence, identity backup | `modules/pukaar_core/rust-lib/logic/` | **Done.** 144 tests, clippy `-D warnings` clean, fuzzed in CI |
| Checkpoint registry (`pukaar_registry`), a SPEL program on LEZ | `programs/pukaar_registry/` | **Done.** Deployed and anchored on localnet ([record](../programs/pukaar_registry/ANCHOR.md)) |
| Logos core module (`pukaar_core`), a Rust cdylib plus Delivery glue | `modules/pukaar_core/` | **Done.** Runs in the standalone Logos host and inside Basecamp 0.3.1 |
| QML UI module (`pukaar_ui`) | `modules/pukaar_ui/` | **Done.** Home, Issue Tracker, Report, History, People, Messages, Meetings, Places, Profile and Integrity pages. History and Places are tables, with super admin and admin controls on People and Places. Offscreen screenshot harness (`scripts/ui-shots.sh`) |
| Demo scripts and signed `.lgx` packages for Basecamp 0.3.1 | `scripts/`, [releases](https://github.com/adysingh5711/pukaar/releases) | **Done.** Packages signed with `lgx` (see [Security](security.md)) |
| LAN relay and offline three-laptop dry run | | To be added in later version |

## Contributing

Issues and pull requests are welcome. Before opening a PR, run `cargo test`, `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` in `modules/pukaar_core/rust-lib/logic`. CI runs the tests and clippy, but CI does not run `cargo fmt --check`. Changes to the wire format must keep `Body` append-only.

Contribution guidelines and code of conduct: to be added in later version.

## Make a release

<details>
<summary>Release steps</summary>

The release packages are the portable `.lgx` builds. The plain `.#lgx` output is a `-dev` build, and release Basecamp rejects it. The signing key is outside the repo, in `~/.config/pukaar-keys`.

1. Set the same `version` in `modules/pukaar_core/metadata.json` and `modules/pukaar_ui/metadata.json`, then commit.
2. Build the portable package in each module, then copy both packages to `dist/`.
3. Sign each package with `lgx`.
4. Make sure that each signature is valid with `lgx verify`. This step needs the `trusted-keys` keyring from [Security](security.md).
5. Tag the commit as `v<version>`, then push the tag. If the tag and the two versions differ, the CI `version` job fails.
6. On GitHub, make a release from the tag. Attach both `.lgx` files and a `SHA256SUMS` file.

```bash
(cd modules/pukaar_core && nix build --accept-flake-config .#lgx-portable --out-link result-lgx-portable)
(cd modules/pukaar_ui && nix build --accept-flake-config .#lgx-portable --out-link result-lgx-portable)
mkdir -p dist && cp modules/*/result-lgx-portable/*.lgx dist/ && chmod u+w dist/*.lgx
nix build 'github:logos-co/logos-package#lgx' --out-link ./lgx
for f in dist/*.lgx; do ./lgx/bin/lgx sign "$f" --key publisher --keys-dir ~/.config/pukaar-keys; done
for f in dist/*.lgx; do ./lgx/bin/lgx verify "$f" --keyring-dir ./trusted-keys; done
git tag v<version> && git push origin v<version>
```

</details>
