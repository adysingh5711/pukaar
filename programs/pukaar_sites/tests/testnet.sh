#!/usr/bin/env bash
# Happy path of tests/local.sh on the PUBLIC LEZ testnet (v0.3.1). Every send is real and costs lepta.
#   I_APPROVE_TESTNET_WRITE=1 tests/testnet.sh
#   LEE_WALLET_HOME_DIR     default: the testnet wallet home
#   PUKAAR_PROGRAM_ID       default: the deployed header account (without the Public/ prefix)
#   PAYER                   default: pukaar-payer (pays and signs as the site creator)
#   FLOOR                   stop before a call that could leave the payer under this balance (default 4000000)
#   PROGRESS                one line per step (default ./.testnet-progress.txt)
# Member keys t1.. are throwaway keys derived from test seeds. Only t1 signs, so only t1 is imported.
# No negative tests: a failed tx costs about 4,006,680 lepta at gas_limit 500000.
# The scenario stops at the first failed or unconfirmed call and never resends. One tx at a time.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
[ "${I_APPROVE_TESTNET_WRITE:-}" = 1 ] || { echo "set I_APPROVE_TESTNET_WRITE=1: this sends paid txs to the public testnet" >&2; exit 2; }
export LEE_WALLET_HOME_DIR="${LEE_WALLET_HOME_DIR:-/Users/aditya/Dev/Hackathons/logos-works/wallet-homes/testnet}"
case "$LEE_WALLET_HOME_DIR" in *testnet*) ;; *) echo "not the testnet wallet home" >&2; exit 1;; esac
export PUKAAR_PROGRAM_ID="${PUKAAR_PROGRAM_ID:-Gs1EWbwJKSBGRBiR7xasYtkrfCyUGJ52Vx4ASsDLooVK}"
PAYER="${PAYER:-9SvEHsoxeUmZRRCHHKvsJBVRoqGjVQZQbs3Kco9iMPK3}"
export PUKAAR_PAYER="$PAYER"
FLOOR="${FLOOR:-4000000}"
PROGRESS="${PROGRESS:-$HERE/.testnet-progress.txt}"
GUARD="$HERE/../../scripts/lez-testnet.sh"
RUNNER="$HERE/runner/target/release/runner"
cd "$HERE"

PASS=0; SCALE=100; B0=0; HASH=""; INC=""; SITE=""
R() { "$RUNNER" "$@"; }
site() { R get-site "$SITE" "$@"; }
mem() { R get-member "$SITE" "$1"; }
bal() { "$GUARD" account get --account-id "Public/$PAYER" | sed -n 's/^Balance \([0-9]*\),.*/\1/p'; }
log() { echo "$*" | tee -a "$PROGRESS"; }
die() { log "FAIL $*"; log "=== $PASS PASS, 1 FAIL (site ${SITE:-none}, program $PUKAAR_PROGRAM_ID, balance $(bal))"; exit 1; }

# step NAME EST SIGNER ARGS..: send one instruction (at most 10 min), never resend. EST is the expected fee in lepta.
step() {
  local name=$1 est=$2 s=$3; shift 3
  BASE=$est; B0=$(bal); est=$(( est * SCALE / 100 ))
  [ $(( B0 - est )) -ge "$FLOOR" ] || die "$name: balance $B0 minus the expected fee $est would be under $FLOOR, not sent"
  OUT=$(PUKAAR_SIGNER=$s timeout 600 "$RUNNER" "$@" 2>&1) || true
  HASH=$(sed -n 's/^tx: //p' <<<"$OUT"); INC=$(sed -n 's/^included in block //p' <<<"$OUT")
  [ -n "$HASH" ] || die "$name: no tx hash, nothing was sent: $OUT"
}
# check NAME CMD PATTERN..: poll the chain every 20 s (30 times at most) until CMD prints every pattern (extended regex).
# If the runner saw the tx in a block, the state must match after 3 reads, else the tx failed.
check() {
  local name=$1 cmd=$2 out p ok t=0; shift 2
  while :; do
    out=$(eval "$cmd" 2>&1) || true; ok=1
    for p in "$@"; do grep -Eq -- "$p" <<<"$out" || ok=0; done
    [ $ok = 1 ] && return 0
    t=$((t+1))
    if [ $t -ge 30 ] || { [ -n "$INC" ] && [ $t -ge 3 ]; }; then die "$name tx=${HASH:0:10} included=${INC:-no}: state does not match /$*/: $out"; fi
    sleep 20
  done
}
# fin NAME: print the step line and learn the fee scale
fin() {
  local b1 fee pct; b1=$(bal); fee=$((B0 - b1)); PASS=$((PASS+1)); pct=$(( fee * 100 / BASE ))
  [ "$pct" -le "$SCALE" ] || SCALE=$pct   # a call dearer than expected makes every later estimate dearer
  log "PASS $1 tx=${HASH:0:10} before=$B0 after=$b1 fee=$fee"
}

: > "$PROGRESS"
[ -x "$RUNNER" ] || { echo "build the runner: cd runner && cargo build --release" >&2; exit 1; }
"$GUARD" check-health >/dev/null
CLOCK=$(R clock); log "start: clock=$CLOCK payer_balance=$(bal) program=$PUKAAR_PROGRAM_ID"
ID=()
for i in 1 2 3; do read -r KEY ACC < <(R member-key "t$i"); ID[$i]=$ACC; [ "$i" = 1 ] && K1=$KEY; done
"$GUARD" account import public --private-key "$K1" >/dev/null 2>&1 || true   # a second import of the same key is refused: ignore

# ---- scenario: tests/local.sh without the negative tests ----
SALT=$(openssl rand -hex 32)
step create_site 1000000 "$PAYER" create-site "$SALT"
SITE=$(sed -n 's/^site_id: //p' <<<"$OUT"); log "site_id=$SITE"
check create_site 'site "$PAYER"' 'roster_len: 1,' 'key_epoch: 1,' "super_admins: \[Some\(\"$PAYER\"\), None\]" 'role: Admin,' 'status: Active,'
fin create_site

step approve_batch_3 1500000 "$PAYER" approve "$SITE" t1:resident:prof1:1 t2:resident:prof2:1 t3:resident:prof3:1 roster=1 keyring=keyring2
check approve_batch 'site' 'roster_len: 4,' 'keyring_cid: "keyring2"'
for i in 1 2 3; do check "approve_batch t$i" "mem t$i" 'role: Resident,' 'status: Active,' "profile_cid: \"prof$i\"" 'profile_version: 1,' 'last_req_nonce: 1 }'; done
fin approve_batch_3

step set_role_batch_1 800000 "$PAYER" set-role "$SITE" t2:steward
check set_role_batch 'mem t2' 'role: Steward,' 'status: Active,'
check "set_role_batch t1 unchanged" 'mem t1' 'role: Resident,'
fin set_role_batch_1

step revoke_batch_1 800000 "$PAYER" revoke "$SITE" t3 keyring=keyring3
check revoke_batch 'site' 'key_epoch: 2,' 'keyring_cid: "keyring3"' 'roster_len: 4,'
check "revoke_batch t3" 'mem t3' 'status: Removed,' 'revoked: Some'
fin revoke_batch_1

step set_profile_batch_2 1000000 "$PAYER" set-profile "$SITE" t1:prof1b:2 t2:prof2b:2
check "set_profile_batch t1" 'mem t1' 'profile_cid: "prof1b"' 'profile_version: 2,'
check "set_profile_batch t2" 'mem t2' 'profile_cid: "prof2b"' 'profile_version: 2,'
fin set_profile_batch_2

step set_details 800000 "$PAYER" set-details "$SITE" details2
check set_details 'site' 'details_cid: "details2"'; fin set_details
step set_places 800000 "$PAYER" set-places "$SITE" places2
check set_places 'site' 'places_cid: "places2"'; fin set_places
step set_keyring 800000 "$PAYER" set-keyring "$SITE" keyring4
check set_keyring 'site' 'keyring_cid: "keyring4"' 'key_epoch: 2,'; fin set_keyring

step super_admin_add 800000 "$PAYER" super-add "$SITE" t2
check super_admin_add 'site' "super_admins: \[Some\(\"$PAYER\"\), Some\(\"${ID[2]}\"\)\]"
check "super_admin_add promotes t2" 'mem t2' 'role: Admin,'
fin super_admin_add
step super_admin_transfer 800000 "$PAYER" super-transfer "$SITE" t1
check super_admin_transfer 'site' "super_admins: \[Some\(\"${ID[1]}\"\), Some\(\"${ID[2]}\"\)\]"
check "super_admin_transfer: A stays admin" 'mem "$PAYER"' 'role: Admin,' 'status: Active,'
fin super_admin_transfer
step super_admin_remove 800000 t1 super-remove "$SITE" t2
check super_admin_remove 'site' "super_admins: \[Some\(\"${ID[1]}\"\), None\]"
fin super_admin_remove

ROOT=$(openssl rand -hex 32)
step anchor 800000 t1 anchor "$SITE" "$ROOT" 5
check "anchor by active member t1" 'R get-checkpoint "$SITE" "$ROOT"' 'n_events: 5,'
fin anchor

log "=== $PASS PASS, 0 FAIL (site $SITE, program $PUKAAR_PROGRAM_ID, balance $(bal))"
