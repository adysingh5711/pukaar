#!/usr/bin/env python3
"""Rebuild index.json from the .lgx files in ../../dist. Standard library only.

Run it after the final signed build, and again after any version bump:
    python3 public/install/make-index.py
Then check the result with `index.py validate --full` from
https://github.com/logos-co/logos-modules-release-tool before publishing.
"""
import hashlib, json, os, sys, tarfile
from datetime import datetime, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
DIST = os.path.join(HERE, "..", "..", "dist")
REPO = "adysingh5711/pukaar"
FILES = ["logos-pukaar_core-module-lib.lgx", "logos-pukaar_ui-module.lgx"]

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
    if sig:  # omit the key for unsigned packages
        entry["signature"] = json.loads(sig)
    packages.append({"name": manifest["name"], "versions": [entry]})

index = {"schemaVersion": 2, "repositoryName": "pukaar", "generatedAt": now, "packages": packages}
with open(os.path.join(HERE, "index.json"), "w") as out:
    json.dump(index, out, indent=2)
    out.write("\n")
print("wrote index.json:", ", ".join(f"{p['name']} {p['versions'][0]['manifest']['version']}" for p in packages))
if not all("signature" in p["versions"][0] for p in packages):
    print("warning: unsigned package(s); sign the .lgx files first for a release", file=sys.stderr)
