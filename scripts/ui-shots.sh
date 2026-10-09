#!/usr/bin/env bash
# Offscreen screenshots of the Pukaar view against fixture data (scripts/ui-shots.qml), in light
# and dark, for every state (not in a site, pending, resident, steward, admin), page and the issue
# pane, at 800, 1280 and 1400 px. Prints QML warnings; exits non-zero on any.
#   scripts/ui-shots.sh [out dir]      default: $TMPDIR/pukaar-shots/<git short hash>
# Compare two runs (fuzz: a line at a fractional x can anti-alias differently from run to run):
#   for f in A/*/*.png; do g=B/${f#A/}; n=$(magick compare -metric AE -fuzz 20% "$f" "$g" null: 2>&1); [ "${n%% *}" = 0 ] || echo "$n $f"; done
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
mod="$here/../modules/pukaar_ui"
out="${1:-${TMPDIR:-/tmp}/pukaar-shots/$(git -C "$here" rev-parse --short HEAD)}"
nix=/nix/var/nix/profiles/default/bin/nix
# The Qt 6.9.2 that logos-module-builder builds the module with (its pinned nixpkgs).
pkgs=github:NixOS/nixpkgs/e9f00bd893984bc8ce46c895c3bf7cac95331127
{ read -r qd; read -r qb; } < <($nix build --accept-flake-config --no-link --print-out-paths "$pkgs#qt6.qtdeclarative" "$pkgs#qt6.qtbase")

grep -q "Application.styleHints.colorScheme === Qt.Dark" "$mod/Main.qml"   # the line the theme pin rewrites
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
ln -s "$mod/src" "$tmp/src"   # the sidebar logo
fail=0
for theme in light dark; do
    mkdir -p "$out/$theme"
    # The offscreen platform ignores the OS colour scheme (and setColorScheme): pin it in a copy.
    [ "$theme" = dark ] && v=true || v=false
    sed "s/Application.styleHints.colorScheme === Qt.Dark/$v/" "$mod/Main.qml" > "$tmp/Main.qml"
    QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software QT_LOGGING_RULES="qt.qpa.*=false;qt.text.*=false" \
    QT_PLUGIN_PATH="$qb/lib/qt-6/plugins:$qd/lib/qt-6/plugins" QML_IMPORT_PATH="$qd/lib/qt-6/qml" \
        "$qd/bin/qml" "$here/ui-shots.qml" -- "$tmp/Main.qml" "$out/$theme" > "$tmp/log" 2>&1 || fail=1
    # Font-database notices are muted above and the offscreen GL notice dropped here: anything left is from QML.
    if grep -v "does not support createPlatformOpenGLContext" "$tmp/log" | grep .; then fail=1; fi
done
echo "$(ls "$out"/*/*.png | wc -l | tr -d ' ') shots in $out"
exit $fail
