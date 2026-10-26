#!/usr/bin/env bash
# Build the pukaar_sites guest reproducibly (Docker) and copy the ELF to ./pukaar_sites.bin.
# Needs cargo-risczero 3.0.5 and a running Docker. Always run this before a test: a host build never rebuilds the guest.
#   build.sh --fast   native dev build with the local risc0 toolchain, no Docker (about 20 s cold, 3 s warm) -> ./pukaar_sites.fast.bin.
#                     Dev chain only: the image id differs from the Docker build. Never deploy it to the testnet.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$HERE"
if [ "${1:-}" = "--fast" ]; then
  case "${LEE_WALLET_HOME_DIR:-}" in *testnet*) echo "refusing --fast with a testnet wallet home" >&2; exit 1;; esac
  echo "FAST BUILD: dev only, image id differs from the Docker build"
  cargo build -q --release --manifest-path fastbuild/Cargo.toml
  cp "$(fastbuild/target/release/fastbuild | tail -1)" pukaar_sites.fast.bin
  ls -l pukaar_sites.fast.bin
  exit 0
fi
# r0.1.91.1 is the image LEZ v0.3.1 builds with; the default r0.1.88 is too old for its deps.
RISC0_DOCKER_CONTAINER_TAG="${RISC0_DOCKER_CONTAINER_TAG:-r0.1.91.1}" cargo risczero build --manifest-path guest/Cargo.toml
BIN=$(find target -path '*riscv32im-risc0-zkvm-elf/docker/*' -name 'pukaar_sites.bin' | head -1)
[ -n "$BIN" ] || { echo "pukaar_sites.bin not found under target/" >&2; exit 1; }
cp "$BIN" "$HERE/pukaar_sites.bin"
ls -l "$HERE/pukaar_sites.bin"
