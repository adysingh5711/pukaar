#!/usr/bin/env bash
# Guarded wallet for the PUBLIC LEZ testnet. Allowlist: anything not listed needs I_APPROVE_TESTNET_WRITE=1.
export LEE_WALLET_HOME_DIR=/Users/aditya/Dev/Hackathons/logos-works/wallet-homes/testnet
ok=0
for a in "$@"; do case "$a" in -h|--help|-V|--version) ok=1;; esac; done
case "${1:-}" in
  ""|help|check-health|chain-info) ok=1;;
  account) case "${2:-}" in get|id|list|ls|show-keys) ok=1;; esac;;
  config) case "${2:-}" in get|description) ok=1;; esac;;
  group) case "${2:-}" in list|ls) ok=1;; esac;;
  keycard) case "${2:-}" in available) ok=1;; esac;;
esac
# ponytail: allowlist, so new/unknown subcommands (writes, signing, bridge, loader, change-network) are refused by default.
if [ "$ok" != 1 ] && [ "${I_APPROVE_TESTNET_WRITE:-}" != 1 ]; then
  echo "lez-testnet.sh: refused '$*' (may write or send). Set I_APPROVE_TESTNET_WRITE=1 to allow." >&2
  exit 2
fi
exec wallet "$@"
