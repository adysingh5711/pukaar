#!/usr/bin/env bash
# Load check for a Pukaar core .lgx on macOS. Usage: scripts/check-core-lgx.sh path/to/core.lgx
# Fails if the plugin has an undefined symbol that no linked library provides, or if dlopen fails.
# The macOS link lets such symbols through ("dynamically looked up"). Basecamp then fails at load.
set -euo pipefail
lgx=${1:?usage: check-core-lgx.sh <core.lgx>}
# Symbols that the host may provide at load time (Qt and module SDK). Empty today: a working module has none.
ALLOW='^_(QT_|lp_|logos_host_)'
FW=/Applications/LogosBasecamp.app/Contents/Frameworks

tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
tar xzf "$lgx" -C "$tmp"
echo "dependencies: $(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1])).get("dependencies"))' "$tmp/manifest.json")"
dylib=$(ls "$tmp"/variants/darwin-*/*_plugin.dylib)

bad=$(nm -mu "$dylib" | grep 'dynamically looked up' | awk '{print $3}' | grep -Ev "$ALLOW" || true)
fail=0
if [ -n "$bad" ]; then
  echo "FAIL: $(echo "$bad" | wc -l | tr -d ' ') undefined symbols that no library provides:"
  echo "$bad" | head -5
  fail=1
fi

# dlopen from a neutral directory. The Basecamp Qt frameworks stand in for the host runtime.
err=$(cd / && DYLD_FRAMEWORK_PATH=$FW python3 -I -c 'import ctypes,sys; ctypes.CDLL(sys.argv[1])' "$dylib" 2>&1 | tail -1 | cut -c1-200) || true
if echo "$err" | grep -q 'OSError'; then
  echo "FAIL: dlopen: $err"
  fail=1
fi
[ $fail = 0 ] && echo "PASS: no unresolved symbols, dlopen works"
exit $fail
