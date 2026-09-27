#!/usr/bin/env bash
# Two Pukaar instances on one machine, each with its own key and replica.
# Usage: scripts/demo.sh [--clean]   (--clean wipes only the two demo dirs)
set -euo pipefail
NIX="${NIX:-nix}"
command -v "$NIX" >/dev/null 2>&1 || NIX=/nix/var/nix/profiles/default/bin/nix
ROOT="${PUKAAR_DEMO_DIR:-$HOME/.local/share/pukaar-demo}"
A="$ROOT/resident-a"; B="$ROOT/steward-b"
if [[ "${1:-}" == "--clean" ]]; then rm -rf -- "$A" "$B"; fi
cd "$(dirname "$0")/../modules/pukaar_ui"
"$NIX" build --accept-flake-config .   # build once so both launches start fast
"$NIX" run --accept-flake-config . -- --user-dir "$A" & PID_A=$!
"$NIX" run --accept-flake-config . -- --user-dir "$B" & PID_B=$!
trap 'kill "$PID_A" "$PID_B" 2>/dev/null || true' EXIT
wait
