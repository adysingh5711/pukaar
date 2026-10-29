# Security

The threat model, what Pukaar does not claim, release signing, key storage, how to report a vulnerability and how to check an anchor yourself. [Back to the README](../README.md).

## Threat model and claims

**Threat model (summary).**

| Attack | Mitigation |
|---|---|
| Deleting an embarrassing report | No server and no delete event. Replicas on every device. Anchored heads |
| Closing a ticket that isn't fixed | A claim isn't resolution. It needs the reporter, or 2 residents, and never the claimant. Reopen is always allowed |
| Backdating an acknowledgement | `ts` is informational. Order comes from the chain and the Lamport clock. Checkpoints bound when an event existed |
| Rewriting history (equivocation) | Per-author hash chains. Two events at one `(author, seq)` form a self-contained fork proof, kept as evidence and flagged on the board. Every replica keeps the same branch (lowest id wins), so boards still converge |
| One identity used on two devices | Treated like a fork: both versions kept, one deterministic winner everywhere, a fork warning on the board. A restored identity can't publish until its own history has synced back |
| Removing an inconvenient place | Only places nobody ever reported can be removed, with a reason, after a 30-day pending period anyone can see; places with reports can only be retired |
| Spam | A deterministic open-report cap per key. The admin can revoke roles, and the reports stay visible |
| Malformed input | Strict decoder with size limits, the signature checked before decoding, and a `cargo-fuzz` target run in CI |
| Junk on-chain checkpoints | One PDA per (site, root). Signature and membership are checked off-chain |
| Flooding the channel | Rate-limited `Heads` answers. Only 3 elected members plus the kiosk re-send others' events (max 200) |

**What Pukaar does not claim:** that a problem was physically fixed (only who said so), anonymity against someone who knows the site, network-level unlinkability, resistance to an admin who controls every device, or that the chain makes inputs honest.

| | |
|---|---|
| Security audit | To be added in later version |
| Bug bounty | To be added in later version |
| Release signing | Packages are signed with `lgx`. Publisher DID: `did:jwk:eyJjcnYiOiJFZDI1NTE5Iiwia3R5IjoiT0tQIiwieCI6IlpfZkxKcnVWR3UyYnBiR0VNMlhMTElmY2FzdTFycVkycHJZM1Z0cklRR28ifQ`. Verify with `lgx keyring add publisher "<DID>" --dir ./trusted-keys` then `lgx verify <file>.lgx --keyring-dir ./trusted-keys`. The official index ships `trustedSigners: []`, so Basecamp doesn't enforce signatures yet |
| Key storage at rest | The key file is `0600` in the host's data directory. A password-sealed **backup** is available (Profile page). Sealing the live key file itself: to be added in later version |
| Reporting a vulnerability | Open a private [security advisory](https://github.com/adysingh5711/pukaar/security/advisories/new) on this repository |

## Verifying an anchor yourself

```bash
cd programs/pukaar_registry
spel inspect <RECORD_PDA> --idl idl/pukaar_registry.json --type CheckpointRecord
```

The `heads_root` and `n_events` it prints must equal the values that your own replica computes for that checkpoint. A mismatch means the anchored history and your history disagree.

The Integrity page in the app reports the newest anchor recorded in the site's own history, and whether this device can rebuild its root. It does not read LEZ, so only `spel inspect` confirms what is on chain.
