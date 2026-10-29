#!/usr/bin/env bash
# Load check for a Pukaar core .lgx. Usage: scripts/check-core-lgx.sh path/to/core.lgx
# Checks every variant that this host can check. A merged package has darwin and linux variants.
# darwin (on macOS): fails if the plugin has an undefined symbol that no linked library provides,
#   or if dlopen fails. The macOS link lets such symbols through ("dynamically looked up").
#   Basecamp then fails at load.
# linux (with readelf): fails if the plugin uses PC/SC (SCard*) but does not link libpcsclite.
set -euo pipefail
lgx=${1:?usage: check-core-lgx.sh <core.lgx>}
# Symbols that the host may provide at load time (Qt and module SDK). Empty today: a working module has none.
ALLOW='^_(QT_|lp_|logos_host_)'
FW=/Applications/LogosBasecamp.app/Contents/Frameworks

tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
tar xzf "$lgx" -C "$tmp"
echo "dependencies: $(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1])).get("dependencies"))' "$tmp/manifest.json")"

fail=0 checked=0
for dir in "$tmp"/variants/*/; do
  v=$(basename "$dir")
  case $v in
    darwin-*)
      [ "$(uname)" = Darwin ] || { echo "SKIP $v: needs macOS"; continue; }
      dylib=$(ls "$dir"*_plugin.dylib)
      bad=$(nm -mu "$dylib" | grep 'dynamically looked up' | awk '{print $3}' | grep -Ev "$ALLOW" || true)
      if [ -n "$bad" ]; then
        echo "FAIL $v: $(echo "$bad" | wc -l | tr -d ' ') undefined symbols that no library provides:"
        echo "$bad" | head -5
        fail=1
      fi
      # dlopen from a neutral directory. The Basecamp Qt frameworks stand in for the host runtime.
      err=$(cd / && DYLD_FRAMEWORK_PATH=$FW python3 -I -c 'import ctypes,sys; ctypes.CDLL(sys.argv[1])' "$dylib" 2>&1 | tail -1 | cut -c1-200) || true
      if echo "$err" | grep -q 'OSError'; then
        echo "FAIL $v: dlopen: $err"
        fail=1
      fi
      ;;
    linux-*)
      command -v readelf >/dev/null || { echo "SKIP $v: needs readelf"; continue; }
      so=$(ls "$dir"*_plugin.so)
      # ponytail: checks only the PC/SC link (the one gap seen so far). A full check needs the Basecamp Qt libs.
      if readelf -W --dyn-syms "$so" | awk '$7 == "UND" {print $8}' | grep -q '^SCard' &&
        ! readelf -d "$so" | grep -q 'NEEDED.*libpcsclite'; then
        echo "FAIL $v: the plugin uses SCard* symbols but does not link libpcsclite"
        fail=1
      fi
      ;;
    *) echo "SKIP $v: unknown variant"; continue ;;
  esac
  checked=$((checked + 1))
done
[ $checked -gt 0 ] || { echo "FAIL: no variant could be checked on this host"; exit 1; }
[ $fail = 0 ] && echo "PASS: $checked variant(s) checked"
exit $fail
