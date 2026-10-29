#!/usr/bin/env bash
# Build the Pukaar .lgx packages with progress that you can follow.
# Usage: scripts/build-lgx.sh core|ui|all [--dry-run]
# Logs go outside the repo. Read <module>-status.txt to see the state.
set -uo pipefail

REPO=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
LOGS=$REPO/dist/pukaar-build-logs
DEST=$REPO/dist
WHAT=${1:-}
DRY=0
[ "${2:-}" = "--dry-run" ] && DRY=1
case $WHAT in core | ui | all) ;; *) echo "usage: $0 core|ui|all [--dry-run]" >&2; exit 2 ;; esac

mkdir -p "$LOGS"
HB=""
trap '[ -n "$HB" ] && kill "$HB" 2>/dev/null' EXIT

el() { local s=$((SECONDS - T0)); printf '%02d:%02d' $((s / 60)) $((s % 60)); }
out() { printf '[%s] %s\n' "$(el)" "$1" | tee -a "$LOG"; }
say() { out "$1"; printf '%s' "$1" | cut -c1-160 >"$LASTF"; }
# Write the status file in one step: write a tmp file, then move it.
status() {
  { echo "STATE=$1"; echo "STEP=$(cat "$STEPF" 2>/dev/null)"; echo "STARTED=$STARTED"
    echo "ELAPSED=$(el)"; echo "LAST=$(cat "$LASTF" 2>/dev/null)"; [ -n "${2:-}" ] && echo "$2"; } >"$STATUS.tmp"
  mv "$STATUS.tmp" "$STATUS"
}
mtime() { stat -f %m "$1" 2>/dev/null || stat -c %Y "$1"; }
heartbeat() {
  while sleep 10; do
    status running
    if [ $(($(date +%s) - $(mtime "$LASTF"))) -ge 10 ]; then
      out "... still building (elapsed $(el), last: $(cat "$LASTF"))"
    fi
  done
}

# Stop if a nix build already runs in this module directory.
busy() {
  local pid
  for pid in $(pgrep -f 'nix.* build'); do
    [ "$pid" = "$$" ] && continue
    if lsof -a -p "$pid" -d cwd -Fn 2>/dev/null | grep -qx "n$1"; then
      echo "A nix build is already running in $1 (PID $pid). Wait for it to end."
      return 0
    fi
  done
  return 1
}

fail() { # fail <step number> <step name> <exit code>
  say "FAILED at step $1 ($2), exit code $3"
  echo "$2" >"$STEPF"
  kill "$HB" 2>/dev/null; HB=""
  status failed "EXIT=$3"
  say "BUILD FAILED step=$1 $MOD"
  exit 1
}

run_module() {
  MOD=$1
  local dir=$REPO/modules/pukaar_$MOD lgx_name link cmd ts src sha size
  if [ "$MOD" = core ]; then lgx_name=logos-pukaar_core-module-lib.lgx; else lgx_name=logos-pukaar_ui-module.lgx; fi
  link=result-lgx-portable
  cmd="nix build --accept-flake-config '.#lgx-portable' --out-link $link -L"
  ts=$(date +%Y%m%d-%H%M%S)
  LOG=$LOGS/$MOD-$ts.log STATUS=$LOGS/$MOD-status.txt STEPF=$LOGS/.$MOD-step LASTF=$LOGS/.$MOD-last
  STARTED=$(date +%Y-%m-%dT%H:%M:%S) T0=$SECONDS
  : >"$LOG"; : >"$LASTF"; echo start >"$STEPF"

  if [ "$DRY" = 1 ]; then
    echo plan >"$STEPF"
    out "DRY RUN for $MOD. Nothing will be built."
    out "log:    $LOG"
    out "latest: $LOGS/$MOD-latest.log"
    out "status: $STATUS"
    out "[1/4] cd $dir && $cmd"
    if [ "$MOD" = core ]; then out "[2/4] scripts/check-core-lgx.sh $dir/$link/$lgx_name"
    else out "[2/4] extract the lgx and diff -q its Main.qml against $dir/Main.qml"; fi
    out "[3/4] copy to $DEST/$lgx_name (old file kept in $LOGS as $lgx_name.prev-<timestamp>)"
    out "[4/4] print sha256 and size"
    status dry-run
    rm -f "$LOG" "$LASTF" "$STEPF"
    return
  fi

  ln -sf "$LOG" "$LOGS/$MOD-latest.log"
  say "Open another terminal and run:"
  say "  tail -f $LOGS/$MOD-latest.log"
  say "  cat $STATUS"

  if msg=$(busy "$dir"); then say "$msg"; echo preflight >"$STEPF"; status failed EXIT=1; say "BUILD FAILED step=0 $MOD"; exit 1; fi
  untracked=$(cd "$REPO" && git status --short modules/pukaar_core modules/pukaar_ui | grep '^??' || true)
  if [ -n "$untracked" ]; then
    say "WARNING: Nix only sees git-tracked files. Untracked files found:"
    while read -r _ f; do say "  git add -N $f"; done <<<"$untracked"
  fi

  echo nix-build >"$STEPF"; status running
  heartbeat & HB=$!
  say "== [1/4] nix build $MOD =="
  (cd "$dir" && eval "$cmd" 2>&1) | while IFS= read -r l; do say "$l"; done
  rc=${PIPESTATUS[0]}
  [ "$rc" = 0 ] || fail 1 nix-build "$rc"
  src=$dir/$link/$lgx_name
  [ -f "$src" ] || fail 1 nix-build 1

  if [ "$MOD" = core ]; then
    echo check >"$STEPF"; status running
    say "== [2/4] check core lgx =="
    "$REPO/scripts/check-core-lgx.sh" "$src" 2>&1 | while IFS= read -r l; do say "$l"; done
    rc=${PIPESTATUS[0]}
    [ "$rc" = 0 ] || fail 2 check "$rc"
  else
    echo diff-qml >"$STEPF"; status running
    say "== [2/4] diff Main.qml =="
    t=$(mktemp -d)
    tar xzf "$src" -C "$t" && q=$(find "$t" -name Main.qml | head -1) && [ -n "$q" ] && diff -q "$q" "$dir/Main.qml" 2>&1 | while IFS= read -r l; do say "$l"; done
    rc=${PIPESTATUS[0]}
    [ -n "${q:-}" ] || rc=1
    rm -rf "$t"
    [ "$rc" = 0 ] || fail 2 diff-qml "$rc"
    say "Main.qml is identical to the repo copy"
  fi

  echo copy >"$STEPF"; status running
  say "== [3/4] copy to $DEST =="
  mkdir -p "$DEST"
  if [ -f "$DEST/$lgx_name" ]; then
    cp -p "$DEST/$lgx_name" "$LOGS/$lgx_name.prev-$ts" || fail 3 copy 1
    say "kept old file as $LOGS/$lgx_name.prev-$ts"
  fi
  cp "$src" "$DEST/$lgx_name" || fail 3 copy 1

  echo hash >"$STEPF"; status running
  say "== [4/4] sha256 and size =="
  sha=$(shasum -a 256 "$DEST/$lgx_name" | cut -d' ' -f1)
  size=$(stat -f %z "$DEST/$lgx_name" 2>/dev/null || stat -c %s "$DEST/$lgx_name")
  say "sha256 $sha"
  say "size   $size bytes"
  kill "$HB" 2>/dev/null; HB=""
  echo done >"$STEPF"
  status ok "EXIT=0"
  say "BUILD OK $MOD ${sha:0:12} $size"
}

for m in $([ "$WHAT" = all ] && echo "core ui" || echo "$WHAT"); do run_module "$m"; done
exit 0
