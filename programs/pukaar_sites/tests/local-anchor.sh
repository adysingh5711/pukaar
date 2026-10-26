#!/usr/bin/env bash
# Rebuild the guest, deploy it to the LOCAL sequencer, anchor twice, read the record back.
# Re-run after a sequencer wipe. Local dev chain only: never point this at the testnet wallet home.
#   LEE_WALLET_HOME_DIR  default: the local wallet home
#   PAYER                default: the funded local test account (without the Public/ prefix)
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export LEE_WALLET_HOME_DIR="${LEE_WALLET_HOME_DIR:-/Users/aditya/Dev/Hackathons/logos-works/wallet-homes/local}"
case "$LEE_WALLET_HOME_DIR" in *testnet*) echo "refusing the testnet wallet home" >&2; exit 1;; esac
PAYER="${PAYER:-CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r}"
cd "$HERE"

wallet check-health
./build.sh

# the guest must be rebuilt on purpose: a host build does not do it
( cd runner && cargo build -q )
RUNNER="$HERE/runner/target/debug/runner"

# one segment account per 96 KiB of ELF, plus the header account
SEGS=$(( ( $(wc -c < pukaar_sites.bin) + 98303 ) / 98304 ))
new_account() { wallet account new public --label "$1" | sed -n 's/.*account_id Public\/\([A-Za-z0-9]*\) .*/\1/p'; }
HEADER=$(new_account "pukaar-sites-hdr")
SEG_IDS=()
for i in $(seq 1 "$SEGS"); do SEG_IDS+=("Public/$(new_account "pukaar-sites-seg$i")"); done

# no --immutable: the header stays updatable for bug fixes
# The file holds a kernel ELF too, so SEGS can be one too high. The wallet says how many segments it needs.
deploy() { wallet program-loader deploy --elf pukaar_sites.bin --header "Public/$HEADER" --segments "$@" --payer "Public/$PAYER" 2>&1; }
OUT=$(deploy "${SEG_IDS[@]}" || true)
N=$(echo "$OUT" | sed -n 's/.*splits into \([0-9]*\) segment.*/\1/p')
[ -z "$N" ] || OUT=$(deploy "${SEG_IDS[@]:0:$N}")
echo "$OUT" | grep "Program deployed"
export PUKAAR_PROGRAM_ID="$HEADER" PUKAAR_PAYER="$PAYER"

SITE=$(openssl rand -hex 32)
ROOT=$(openssl rand -hex 32)   # fresh root each run, so a re-run on a live chain does not collide

echo "--- anchor #1"
"$RUNNER" anchor "$SITE" "$ROOT" 5
"$RUNNER" get-checkpoint "$SITE" "$ROOT"
echo "--- anchor #2, same root (must not change state)"
"$RUNNER" anchor "$SITE" "$ROOT" 99 || echo "second anchor refused by the client or sequencer"
"$RUNNER" get-checkpoint "$SITE" "$ROOT"
