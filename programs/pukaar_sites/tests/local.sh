#!/usr/bin/env bash
# Scenario and negative tests for pukaar_sites on the LOCAL sequencer (127.0.0.1:3040, scripts/lez-local.sh).
# Deploys a new copy of the program, then drives every instruction and reads the state back after each call.
#   tests/local.sh [--rebuild|--fast]   --rebuild runs the slow Docker guest build; --fast the native dev build
#   (pukaar_sites.fast.bin, dev chain only); the default reuses ./pukaar_sites.bin
#   LEE_WALLET_HOME_DIR  default: the local wallet home (the testnet home is refused)
#   PAYER                default: the funded local test account (without the Public/ prefix)
#   GAS_LIMIT            wallet gas_limit during the scenario (default 500000), restored on exit
# Test members t1..t5 (Pukaar seed = N repeated, LEZ key derived from it) are imported into the local wallet.
# Fast chain: scripts/lez-local.sh start (2s blocks) and seq_poll_timeout 3s in the local wallet config.
# The txs run one after another: each one is paid and co-signed by the same payer, so its nonce needs the last block.
# Prints PASS or FAIL per step. Exits non-zero if any step fails.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export LEE_WALLET_HOME_DIR="${LEE_WALLET_HOME_DIR:-/Users/aditya/Dev/Hackathons/logos-works/wallet-homes/local}"
case "$LEE_WALLET_HOME_DIR" in *testnet*) echo "refusing the testnet wallet home" >&2; exit 1;; esac
PAYER="${PAYER:-CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r}"
cd "$HERE"

wallet check-health
ELF=pukaar_sites.bin
if [ "${1:-}" = "--rebuild" ]; then ./build.sh; fi
if [ "${1:-}" = "--fast" ]; then ./build.sh --fast; ELF=pukaar_sites.fast.bin; fi
[ -f "$ELF" ] || { echo "no $ELF: run with --rebuild or --fast" >&2; exit 1; }
[ -z "$(find guest/src guest/Cargo.toml rules/src rules/Cargo.toml -newer "$ELF" -print -quit)" ] ||
  echo "WARNING: the sources are newer than $ELF, run with --rebuild or --fast"
( cd runner && cargo build -q )
RUNNER="$HERE/runner/target/debug/runner"

# ---- deploy a new copy (one segment per 96 KiB of ELF, plus the header account) ----
SEGS=$(( ( $(wc -c < "$ELF") + 98303 ) / 98304 ))
new_account() { wallet account new public --label "$1" | sed -n 's/.*account_id Public\/\([A-Za-z0-9]*\) .*/\1/p'; }
TAG=$(date +%m%d-%H%M%S)   # wallet labels are unique, so each run gets a new copy
HEADER=$(new_account "pukaar-sites-hdr-$TAG")
SEG_IDS=()
for i in $(seq 1 "$SEGS"); do SEG_IDS+=("Public/$(new_account "pukaar-sites-seg$i-$TAG")"); done
deploy() { wallet program-loader deploy --elf "$ELF" --header "Public/$HEADER" --segments "$@" --payer "Public/$PAYER" 2>&1; }
OUT=$(deploy "${SEG_IDS[@]}" || true)
N=$(echo "$OUT" | sed -n 's/.*splits into \([0-9]*\) segment.*/\1/p')   # the wallet says how many segments it needs
[ -z "$N" ] || OUT=$(deploy "${SEG_IDS[@]:0:$N}")
echo "$OUT" | grep "Program deployed"
export PUKAAR_PROGRAM_ID="$HEADER" PUKAAR_PAYER="$PAYER"

# ---- test members: import their LEZ keys (a second import of the same key is refused: ignore) ----
ID=()
for i in 1 2 3 4 5; do
  read -r KEY ACC < <("$RUNNER" member-key "t$i")
  wallet account import public --private-key "$KEY" >/dev/null 2>&1 || true
  ID[$i]=$ACC
done

# ---- a lower gas_limit makes a failed tx cheaper (it pays the full hold). Local home only, restored on exit. ----
OLD_GAS=$(wallet config get gas_limit | awk 'END{print $NF}')
trap 'wallet config set gas_limit "$OLD_GAS" >/dev/null' EXIT
wallet config set gas_limit "${GAS_LIMIT:-500000}" >/dev/null

# ---- helpers ----
PASS=0; FAIL=0; FEES=()
R() { "$RUNNER" "$@"; }
site() { R get-site "$SITE" "$@"; }
mem() { R get-member "$SITE" "$1"; }
snap() { site; for u in "$PAYER" t1 t2 t3 t4 t5; do mem "$u"; done; }
pass() { echo "PASS $1${2:+ ($2)}"; PASS=$((PASS+1)); }
fail() { echo "FAIL $1: $2"; FAIL=$((FAIL+1)); }
# expect NAME OUTPUT PATTERN..: every pattern (extended regex) must be in OUTPUT
expect() {
  local name=$1 out=$2 p; shift 2
  for p in "$@"; do
    grep -Eq -- "$p" <<<"$out" || { fail "$name" "missing /$p/"; sed 's/^/    /' <<<"$out"; return; }
  done
  pass "$name" "fee=${FEE:-?}"
}
# tx SIGNER ARGS..: send one instruction as SIGNER; sets OUT and FEE
tx() { local s=$1; shift; OUT=$(PUKAAR_SIGNER=$s timeout 120 "$RUNNER" "$@" 2>&1) || true; FEE=$(sed -n 's/^fee=//p' <<<"$OUT"); }
# neg NAME SIGNER ARGS..: the tx must land and pay a fee, and the state must not change
neg() {
  local name=$1 s=$2 before after; shift 2
  before=$(snap); tx "$s" "$@"; after=$(snap)
  if [ "$before" != "$after" ]; then fail "neg $name" "state changed"; diff <(echo "$before") <(echo "$after") | head -5
  elif [ "${FEE:-0}" -le 0 ]; then fail "neg $name" "no fee, the tx did not land: $OUT"
  else pass "neg $name" "state unchanged, fee=$FEE"; FEES+=("$name=$FEE"); fi
}
NOW() { R clock; }

# ---- scenario ----
echo "--- 1. create_site"
SALT=$(openssl rand -hex 32)
tx "$PAYER" create-site "$SALT"
SITE=$(sed -n 's/^site_id: //p' <<<"$OUT")
expect "create_site" "$(site "$PAYER")" 'roster_len: 1,' 'key_epoch: 1,' "super_admins: \[Some\(\"$PAYER\"\), None\]" 'role: Admin,' 'status: Active,'

echo "--- 2. approve_batch: three members with profiles"
tx "$PAYER" approve "$SITE" t1:resident:prof1:1 t2:resident:prof2:1 t3:resident:prof3:1 roster=1 keyring=keyring2
expect "approve_batch site" "$(site)" 'roster_len: 4,' 'keyring_cid: "keyring2"'
for i in 1 2 3; do
  expect "approve_batch t$i" "$(mem "t$i")" 'role: Resident,' 'status: Active,' "profile_cid: \"prof$i\"" 'profile_version: 1,' 'last_req_nonce: 1 }'
done

echo "--- 3. set_role_batch: t2 resident to steward"
tx "$PAYER" set-role "$SITE" t2:steward
expect "set_role_batch t2" "$(mem t2)" 'role: Steward,' 'status: Active,'
expect "set_role_batch t1 unchanged" "$(mem t1)" 'role: Resident,'

echo "--- 4. revoke_batch: t3, key_epoch+1 and a new keyring"
tx "$PAYER" revoke "$SITE" t3 keyring=keyring3
expect "revoke_batch site" "$(site)" 'key_epoch: 2,' 'keyring_cid: "keyring3"' 'roster_len: 4,'
expect "revoke_batch t3" "$(mem t3)" 'status: Removed,' 'revoked: Some'

echo "--- 5. set_profile_batch: t1 and t2, version grows"
tx "$PAYER" set-profile "$SITE" t1:prof1b:2 t2:prof2b:2
expect "set_profile_batch t1" "$(mem t1)" 'profile_cid: "prof1b"' 'profile_version: 2,'
expect "set_profile_batch t2" "$(mem t2)" 'profile_cid: "prof2b"' 'profile_version: 2,'

echo "--- 6. set_details, set_places, set_keyring"
tx "$PAYER" set-details "$SITE" details2; expect "set_details" "$(site)" 'details_cid: "details2"'
tx "$PAYER" set-places "$SITE" places2; expect "set_places" "$(site)" 'places_cid: "places2"'
tx "$PAYER" set-keyring "$SITE" keyring4; expect "set_keyring" "$(site)" 'keyring_cid: "keyring4"' 'key_epoch: 2,'

echo "--- negative tests: each tx lands, pays its fee, and leaves the state unchanged"
neg "non-admin approves"          t1 approve "$SITE" t4:resident:prof4:1 roster=4
neg "profile version not growing" "$PAYER" set-profile "$SITE" t1:profx:2
neg "wrong PDA"                   "$PAYER" set-profile "$SITE" t1:profy:3 bad=pda
neg "stale roster_len"            "$PAYER" approve "$SITE" t4:resident:prof4:1 roster=1
neg "clock claim too old"         "$PAYER" set-details "$SITE" detailsz now=$(( $(NOW) - 1000000 ))
neg "bad item in set_profile_batch" "$PAYER" set-profile "$SITE" t1:prof1c:3 t2:prof2c:2
neg "bad item in approve_batch"   "$PAYER" approve "$SITE" t4:resident:prof4:1 t1:resident:prof1d:3 roster=4

echo "--- 7. super admins: add t2, transfer A to t1, t1 removes t2"
tx "$PAYER" super-add "$SITE" t2
expect "super_admin_add" "$(site)" "super_admins: \[Some\(\"$PAYER\"\), Some\(\"${ID[2]}\"\)\]"
expect "super_admin_add promotes t2" "$(mem t2)" 'role: Admin,'
tx "$PAYER" super-transfer "$SITE" t1
expect "super_admin_transfer" "$(site)" "super_admins: \[Some\(\"${ID[1]}\"\), Some\(\"${ID[2]}\"\)\]"
expect "super_admin_transfer: A stays admin" "$(mem "$PAYER")" 'role: Admin,' 'status: Active,'
tx t1 super-remove "$SITE" t2
expect "super_admin_remove" "$(site)" "super_admins: \[Some\(\"${ID[1]}\"\), None\]"

neg "admin revokes a super admin" "$PAYER" revoke "$SITE" t1 keyring=keyring5

echo "--- 8. anchor as an active member"
ROOT=$(openssl rand -hex 32)
tx t1 anchor "$SITE" "$ROOT" 5
expect "anchor by active member t1" "$(R get-checkpoint "$SITE" "$ROOT")" 'n_events: 5,'
for who in t5 t3; do   # t5 never joined, t3 was revoked
  ROOT=$(openssl rand -hex 32)
  neg "anchor by $who (non-member / removed)" "$who" anchor "$SITE" "$ROOT" 9
  expect "anchor by $who wrote no checkpoint" "$(R get-checkpoint "$SITE" "$ROOT")" 'no record'
done

echo "--- 9. backup pointer: made by the first backup, overwritten at the same address by the next"
tx t1 set-backup "$SITE" backupone
expect "set_backup makes the record" "$(R get-backup "$SITE" t1)" 'cid: "backupone"' 'rev: 1,'
tx t1 set-backup "$SITE" backuptwo
expect "set_backup overwrites the same record" "$(R get-backup "$SITE" t1)" 'cid: "backuptwo"' 'rev: 2,'
neg "backup by a non-member" t5 set-backup "$SITE" backupx
expect "backup by a non-member wrote no pointer" "$(R get-backup "$SITE" t5)" 'None'

echo "--- failed-tx fees at gas_limit ${GAS_LIMIT:-500000}: ${FEES[*]}"
echo "=== $PASS PASS, $FAIL FAIL (site $SITE, program $HEADER)"
[ "$FAIL" -eq 0 ]
