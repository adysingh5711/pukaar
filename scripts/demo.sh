#!/usr/bin/env bash
# Two Pukaar instances on one machine, each with its own key and replica.
# Usage: scripts/demo.sh [--clean]   (--clean wipes only the two demo dirs)
set -euo pipefail
ROOT="${PUKAAR_DEMO_DIR:-$HOME/.local/share/pukaar-demo}"
if [[ "${1:-}" == "--clean" ]]; then rm -rf -- "$ROOT/resident-a" "$ROOT/steward-b"; fi
W="$(dirname "$0")/window.sh"
"$W" resident-a & PID_A=$!
"$W" steward-b & PID_B=$!
trap 'kill "$PID_A" "$PID_B" 2>/dev/null || true' EXIT
wait
