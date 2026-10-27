#!/usr/bin/env bash
# Local two-node Storage test: spike-a and spike-b connect directly on loopback, not through relays.
# Prints commands for you to paste in your own terminal. Never starts Basecamp, never writes
# anything except the SPR file below. Reads the two basecamp.log files.
# Usage: scripts/storage-direct.sh a | spr | b | check
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BASE="${PUKAAR_BASECAMP_DIR:-$HOME/.local/share/pukaar-basecamp}"
SPR_FILE=/Users/aditya/Dev/Hackathons/logos-works/storage-direct-spr-a.txt
PORT_A=18211 PORT_B=18212

pid_of() { pgrep -f -- "--user-dir $BASE/$1\$" | head -1 || true; }
refuse_if_running() { [[ -z "$(pid_of "$1")" ]] || { echo "refused: Basecamp $1 is running (pid $(pid_of "$1")). Quit it first." >&2; exit 1; }; }
# Start time of a running profile as "YYYY-MM-DD HH:MM:SS" (the log lines sort as text), else the epoch.
since() {
  local p; p="$(pid_of "$1")"
  [[ -n "$p" ]] || { echo "0000-00-00 00:00:00"; return; }
  date -j -f "%a %b %e %T %Y" "$(ps -o lstart= -p "$p")" +"%F %T" 2>/dev/null
}
# Log lines of profile $1 written at or after its start.
log_after() { awk -v s="$(since "$1")" 'substr($0,2,19) >= s' "$BASE/$1/logs/basecamp.log"; }

case "${1:-}" in
a)
  refuse_if_running spike-a
  echo "PUKAAR_STORAGE_CFG='{\"listen-port\":$PORT_A,\"nat\":\"extip:127.0.0.1\",\"no-bootstrap-node\":true}' $ROOT/scripts/basecamp.sh spike-a"
  ;;
spr)
  [[ -n "$(pid_of spike-a)" ]] || { echo "spike-a is not running. Start it with the command from '$0 a'." >&2; exit 1; }
  for _ in $(seq 40); do
    line="$(log_after spike-a | grep -a "Signed peer record updated" | grep -a "/ip4/127.0.0.1/tcp/$PORT_A" | tail -1 || true)"
    if [[ -n "$line" ]]; then
      echo "$line" | grep -ao 'spr=spr:[^ ]*' | tail -1 | cut -d= -f2 > "$SPR_FILE"
      echo "saved $SPR_FILE (address 127.0.0.1:$PORT_A is in the record)"; exit 0
    fi
    sleep 3
  done
  echo "no SPR line with 127.0.0.1:$PORT_A after 120 s. The override may not have applied." >&2; exit 1
  ;;
b)
  refuse_if_running spike-b
  [[ -s "$SPR_FILE" ]] || { echo "no saved SPR. Run '$0 spr' first." >&2; exit 1; }
  echo "PUKAAR_STORAGE_CFG='{\"listen-port\":$PORT_B,\"nat\":\"extip:127.0.0.1\",\"bootstrap-node\":[\"$(cat "$SPR_FILE")\"]}' $ROOT/scripts/basecamp.sh spike-b"
  ;;
check)
  for p in spike-a spike-b; do
    l="$(log_after "$p" || true)"
    echo "$p: since $(since "$p")"
    echo "  download ok lines : $(grep -ac 'downloadToUrl.*\(ok\|success\|true\)\|storageDownloadDone' <<<"$l" || true)"
    echo "  127.0.0.1 in SPR  : $(grep -a 'Signed peer record updated' <<<"$l" | grep -ac "/ip4/127.0.0.1/tcp/" || true)"
    echo "  blockexc peer     : $(grep -a blockexc <<<"$l" | grep -avc 'Exception\|Detaching' || true)"
    echo "  read loop drops   : $(grep -ac 'Detaching read loop' <<<"$l" || true)"
  done
  if log_after spike-b | grep -aq 'downloadToUrl.*\(ok\|success\|true\)\|storageDownloadDone'; then echo "verdict: a download finished on spike-b"
  else echo "verdict: no finished download seen yet"; fi
  ;;
*) echo "usage: $0 a|spr|b|check" >&2; exit 2 ;;
esac
