#!/usr/bin/env python3
"""Rebuild index.json from the signed release packages in ../../dist/release. Standard library only.

scripts/release.sh writes those packages (each holds every platform variant, merged and
signed) and then runs this script. To run it by hand:
    python3 public/install/make-index.py
Then check the result with `index.py validate --full` from
https://github.com/logos-co/logos-modules-release-tool before publishing.
"""
import hashlib, json, os, sys, tarfile
from datetime import datetime, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
DIST = os.path.join(HERE, "..", "..", "dist", "release")
REPO = "adysingh5711/pukaar"
FILES = ["pukaar-core.lgx", "pukaar-ui.lgx"]  # the names scripts/release.sh writes

def member(tar, name):
    try:
        return tar.extractfile(name).read()
    except KeyError:
        return None

packages = []
now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
for f in FILES:
    path = os.path.join(DIST, f)
    raw = open(path, "rb").read()
    with tarfile.open(path) as t:
        manifest = json.loads(member(t, "manifest.json"))
        sig = member(t, "manifest.sig")
    ver = manifest["version"]
    url = f"https://github.com/{REPO}/releases/download/v{ver}/{f}"
    entry = {
        "releasedAt": now,
        "publisherRef": f"{manifest['name']}-v{ver}",
        "url": url,
        "urls": [url],
        "size": len(raw),
        "sha256": hashlib.sha256(raw).hexdigest(),
        "rootHash": manifest["hashes"]["root"],
        "manifest": manifest,
    }
    if not sig:  # Basecamp checks the file against this signature, so an unsigned entry is never published
        sys.exit(f"{f} is not signed. Make the release packages with scripts/release.sh")
    entry["signature"] = json.loads(sig)
    packages.append({"name": manifest["name"], "versions": [entry]})

names = [p["name"] for p in packages]
if len(set(names)) != len(names):
    sys.exit(f"each package must be listed once, got {names}")
index = {"schemaVersion": 2, "repositoryName": "pukaar", "generatedAt": now, "packages": packages}
with open(os.path.join(HERE, "index.json"), "w") as out:
    json.dump(index, out, indent=2)
    out.write("\n")
# One .lgx per module holds every platform as a variant (`lgx merge`), so list them to catch a missing one.
for p in packages:
    m = p["versions"][0]["manifest"]
    print(f"{p['name']} {m['version']}: {', '.join(sorted(k[9:] for k in m['hashes'] if k.startswith('variants/')))}")
print("wrote index.json")
