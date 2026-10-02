#!/usr/bin/env bash
# Open one Pukaar window. Never deletes data.
# Usage: scripts/window.sh [name]   new name = new person; existing name = reopen; no name = fresh person
set -euo pipefail
NIX="${NIX:-nix}"
command -v "$NIX" >/dev/null 2>&1 || NIX=/nix/var/nix/profiles/default/bin/nix
DIR="${PUKAAR_DEMO_DIR:-$HOME/.local/share/pukaar-demo}/${1:-fresh-$(date +%s)}"
echo "Pukaar window: $DIR"
cd "$(dirname "$0")/../modules/pukaar_ui"
exec "$NIX" run --accept-flake-config . -- --user-dir "$DIR"
