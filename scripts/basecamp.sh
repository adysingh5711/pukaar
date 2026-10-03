#!/usr/bin/env bash
# Open Logos Basecamp as one person, with its own data dir (modules, Pukaar identity, replica). Never deletes data.
# Usage: scripts/basecamp.sh <name>     new name = new profile (install Pukaar once via Package Manager); existing = reopen
#        scripts/basecamp.sh default    your normal Basecamp profile (the one the Dock icon opens)
# Each profile can run alongside the others; one profile can't run twice.
set -euo pipefail
[[ $# -eq 1 ]] || { echo "usage: $0 <name>|default" >&2; exit 2; }
APP="${BASECAMP_APP:-/Applications/LogosBasecamp.app}/Contents/MacOS/LogosBasecamp"
if [[ "$1" == "default" ]]; then
  echo "Basecamp profile: default (~/Library/Application Support/Logos/LogosBasecamp)"
  exec "$APP"
fi
DIR="${PUKAAR_BASECAMP_DIR:-$HOME/.local/share/pukaar-basecamp}/$1"
mkdir -p "$DIR"
echo "Basecamp profile: $DIR"
exec "$APP" --user-dir "$DIR"
