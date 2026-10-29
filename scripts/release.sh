#!/usr/bin/env bash
# Make the signed release packages for one tag. See "Make a release" in docs/developing.md.
# Usage: scripts/release.sh v<version> [<linux-dir>]
#   <linux-dir> holds lgx-linux-amd64/ and lgx-linux-arm64/, as `gh run download` writes them.
#   Without it, the script downloads them with `gh` from the successful CI run of the tag.
# Inputs: the macOS build in dist/ (scripts/build-lgx.sh all) and the two Linux builds from CI.
# Output: dist/release/ with pukaar-core.lgx, pukaar-ui.lgx and SHA256SUMS. Each .lgx holds
#   darwin-arm64, linux-amd64 and linux-arm64, is signed, and its signer is in logos-repo.json.
#   Then public/install/index.json is rebuilt. All work happens in a temp dir: on any failure
#   nothing is written, and dist/release is replaced as a whole, so it never holds an unsigned
#   or an old package. The script never uploads anything.
set -euo pipefail

REPO=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
TAG=${1:-}
LINUX=${2:-}
LGX=${LGX:-$REPO/lgx/bin/lgx}
KEYS=${PUKAAR_KEYS_DIR:-$HOME/.config/pukaar-keys} # holds publisher.jwk; never printed
OUT=$REPO/dist/release
ALL="darwin-arm64 linux-amd64 linux-arm64"

die() { echo "release: FAILED: $*" >&2; exit 1; }
json() { python3 -I -c 'import json,sys; d=json.load(sys.stdin); exec(sys.argv[1])' "$1"; }
variants() { tar tzf "$1" | awk -F/ '$1 == "variants" && $2 != "" {print $2}' | sort -u | xargs; }

[[ $TAG =~ ^v[0-9]+\.[0-9]+\.[0-9]+ ]] || { echo "usage: $0 v<version> [<linux-dir>]" >&2; exit 2; }
VER=${TAG#v}
[ -x "$LGX" ] || die "no lgx at $LGX. Run: nix build 'github:logos-co/logos-package#lgx' --out-link ./lgx"
[ -f "$KEYS/publisher.jwk" ] || die "no signing key publisher.jwk in $KEYS"
SHA=$(git -C "$REPO" rev-parse -q --verify "refs/tags/$TAG^{commit}") || die "no git tag $TAG"
[ "$(git -C "$REPO" rev-parse HEAD)" = "$SHA" ] || echo "release: note: HEAD is not $TAG"

for m in core ui; do
  v=$(git -C "$REPO" show "$TAG:modules/pukaar_$m/metadata.json" | json 'print(d["version"])')
  [ "$v" = "$VER" ] || die "modules/pukaar_$m/metadata.json at $TAG says $v, the tag says $VER"
done
TRUSTED=$(json 'print("\n".join(s["did"] for s in d["trustedSigners"]))' <"$REPO/public/install/logos-repo.json")
[ -n "$TRUSTED" ] || die "logos-repo.json lists no trustedSigners"

mkdir -p "$REPO/dist"
tmp=$(mktemp -d "$REPO/dist/.release.XXXXXX") # in dist/, so the final mv is one rename
trap 'rm -rf "$tmp"' EXIT
mkdir "$tmp/out" "$tmp/keyring"
i=0
while read -r did; do "$LGX" keyring add "signer$((i += 1))" "$did" --dir "$tmp/keyring" >/dev/null; done <<<"$TRUSTED"

if [ -z "$LINUX" ]; then
  command -v gh >/dev/null || die "gh not found. Download the CI artifacts by hand and pass their directory"
  run=$(gh run list --workflow ci.yml --commit "$SHA" --status success --json databaseId,headBranch \
    -q "map(select(.headBranch == \"$TAG\"))[0].databaseId // empty")
  [ -n "$run" ] || die "no successful CI run for $TAG yet. Wait for the lgx job, or pass a directory"
  echo "release: downloading the Linux builds from CI run $run"
  gh run download "$run" -n lgx-linux-amd64 -n lgx-linux-arm64 -D "$tmp/linux"
  LINUX=$tmp/linux
fi

for m in core ui; do
  if [ $m = core ]; then src=logos-pukaar_core-module-lib.lgx; else src=logos-pukaar_ui-module.lgx; fi
  dst=$tmp/out/pukaar-$m.lgx
  inputs=()
  for v in $ALL; do
    if [ "$v" = darwin-arm64 ]; then f=$REPO/dist/$src; else f=$LINUX/lgx-$v/$src; fi
    [ -f "$f" ] || die "missing $v build of pukaar_$m: $f"
    [ "$(variants "$f")" = "$v" ] || die "$f should hold only $v, it holds: $(variants "$f")"
    got=$("$LGX" manifest "$f" --json | json 'print(d["name"], d["version"])')
    [ "$got" = "pukaar_$m $VER" ] || die "$f is $got, expected pukaar_$m $VER"
    inputs+=("$f")
  done
  # The macOS build comes from the working tree, so make sure it is the tagged UI (a stale build shipped once).
  if [ $m = ui ]; then
    "$LGX" extract "${inputs[0]}" -v darwin-arm64 -o "$tmp/qml" >/dev/null
    git -C "$REPO" show "$TAG:modules/pukaar_ui/Main.qml" | cmp -s - "$tmp/qml/darwin-arm64/Main.qml" ||
      die "the macOS pukaar_ui build in dist/ is not the $TAG Main.qml. Run scripts/build-lgx.sh ui on the tag"
  fi

  "$LGX" merge "${inputs[@]}" -o "$dst" -y >/dev/null || die "lgx merge failed for pukaar_$m"
  [ "$(variants "$dst")" = "$ALL" ] || die "merged pukaar_$m holds: $(variants "$dst")"
  "$LGX" sign "$dst" --key publisher --keys-dir "$KEYS" >/dev/null || die "lgx sign failed for pukaar_$m"

  did=$("$LGX" signature "$dst" | json 'print(d["did"])') || die "pukaar_$m has no signature"
  grep -qxF "$did" <<<"$TRUSTED" || die "pukaar_$m is signed by a key that is not in logos-repo.json trustedSigners"
  # lgx verify exits 0 for an untrusted signer, so read its report as well.
  report=$("$LGX" verify "$dst" --keyring-dir "$tmp/keyring" 2>&1) || die "lgx verify failed for pukaar_$m: $report"
  { grep -q '^Signature is valid' <<<"$report" && grep -q '^Signer is trusted' <<<"$report"; } ||
    die "lgx verify did not accept the signature of pukaar_$m: $report"
  echo "release: pukaar-$m.lgx $VER ($ALL) signed and verified"
done

(cd "$tmp/out" && shasum -a 256 pukaar-core.lgx pukaar-ui.lgx >SHA256SUMS)
rm -rf "$OUT"
mv "$tmp/out" "$OUT"
python3 "$REPO/public/install/make-index.py"
echo "release: done. Attach the three files in $OUT to the GitHub release $TAG:"
cat "$OUT/SHA256SUMS"
