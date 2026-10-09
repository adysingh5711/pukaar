# Reliable Channels: a message whose SDS causal dependencies never arrive is parked forever, which blocks that sender's whole channel

**Related issues:** none found as a duplicate (searched logos-messaging/logos-delivery, logos-co/logos-delivery-module and logos-messaging/nim-sds for SDS, causal/missing dependencies, retrieval hint, "irretrievably lost", sweep). Background only:
- logos-messaging/logos-delivery#3799 (SDS integration epic)
- logos-messaging/nim-sds#11 (retrieval hints in causal history, closed)
- logos-messaging/nim-sds#25 (tracking SDS spec changes, open; does not mention the incoming-buffer sweep)
- logos-co/logos-delivery-module#151 (RLN-on `logos.test` blocking consumer apps; the trigger of our case, see "How it arose")

## Summary

Scalable Data Sync (SDS) in the Reliable Channel holds every incoming message until the last N messages in its causal history (default `sdsCausalHistorySize` = 2) are in the receiver's history. If those dependencies never arrive, the message stays parked indefinitely. Nothing times it out, marks the dependencies as lost and delivers anyway, or fetches them from Store.

A parked message is not added to the receiver's history, so the sender's next message lists it as a dependency and is parked too. The channel is permanently blocked for that sender, and the app never receives `messageReceived` for it.

## Environment

- Logos Basecamp 0.3.1
- delivery_module 0.3.2 (bundles logos-delivery v0.39.1); also reproduced on delivery_module 0.3.0
- liblogos_rln_module 0.10.0
- macOS (Darwin 27)
- Preset `logos.dev` (cluster 3, RLN off), Edge mode, 6 of 6 peers connected
- Source checked: logos-delivery `v0.39.1` (commit `0f2188c`), which pins nim-sds `4b08d50` ([logos_delivery.nimble#L77](https://github.com/logos-messaging/logos-delivery/blob/v0.39.1/logos_delivery.nimble#L77)). `master` (`fb1a79b`, 2026-10-09) has an identical `scalable_data_sync.nim`, and nim-sds `master` is the same revision as the pin.

## Steps to reproduce

Headless `logoscore` nodes using Basecamp's installed Delivery engine.

1. Nodes A and B exchange messages on `logos.test`. (From 2026-09-30 `logos.test` sends only with a funded RLN membership, so these sends are accepted locally but never reach the network. Each node's local `sds.db` records them as sent history, about 1000 entries.)
2. Move both nodes to `logos.dev`. Messages now propagate.
3. Start a fresh node C and import B's identity.
4. A sends new messages. Their causal history cites the earlier, never-propagated messages.

Result: C buffers all 11 of A's messages and delivers none. The same happens on delivery_module 0.3.0.

Control: with `channelsOverrides: {"sdsCausalHistorySize": 0}`, C buffers 0 messages and restores in about 133 s.

## Expected vs actual

Expected: when dependencies cannot be recovered within some period, the message is delivered (or surfaced as having a gap) rather than withheld indefinitely. The SDS spec describes this: unmet dependencies "MAY" be fetched from Store and, after a predetermined time, marked "irretrievably lost" (see Root cause).

Actual: the receiver logs the following repeatedly (14 times per side in our run), and never logs `SDS releasing buffered message, dependencies met`:

```
DBG SDS message has missing dependencies  topics="sds-handler" channelId=... messageId=... missing=2
```

Transport is healthy: every send logs `Message propagated via Lightpush`, 6 of 6 peers are connected, and the receiver's node does receive the messages. Only the SDS layer withholds them.

## Root cause

All links below are pinned to `v0.39.1` (logos-delivery) or nim-sds `4b08d50`.

1. Park on any missing dependency. `handleIncoming` stores the payload in `pendingContent` and returns nothing whenever `unwrapped.missingDeps.len > 0` ([scalable_data_sync.nim#L220-L232](https://github.com/logos-messaging/logos-delivery/blob/v0.39.1/logos_delivery/channels/scalable_data_sync/scalable_data_sync.nim#L220-L232)). The payload is released only from `onMessageReady` once nim-sds reports the dependencies met ([#L84-L96](https://github.com/logos-messaging/logos-delivery/blob/v0.39.1/logos_delivery/channels/scalable_data_sync/scalable_data_sync.nim#L84-L96)).
2. The missing-dependencies callback only logs. Its comment reads "Recovery via SDS sync / SDS-R for now; targeted store fetch by retrieval hint is a planned follow-up" ([#L103-L109](https://github.com/logos-messaging/logos-delivery/blob/v0.39.1/logos_delivery/channels/scalable_data_sync/scalable_data_sync.nim#L103-L109)). So no Store retrieval exists yet.
3. Dependencies are checked against the receiver's `messageHistory`, which only holds delivered messages ([sds_utils.nim#L341-L357](https://github.com/logos-messaging/nim-sds/blob/4b08d508dbfa69c0e2e3883db67adf1fe5a0c994/sds/sds_utils.nim#L341-L357)). A parked message is not in it, so the block is transitive for that sender.
4. The incoming buffer is never swept. nim-sds's `periodicBufferSweep` only resends or expires the outgoing buffer and cleans the bloom filter ([sds.nim#L437-L497](https://github.com/logos-messaging/nim-sds/blob/4b08d508dbfa69c0e2e3883db67adf1fe5a0c994/sds/sds.nim#L437-L497)). The only code that re-evaluates `incomingBuffer` runs when a new message arrives (`unwrapReceivedMessage`, [#L242-L369](https://github.com/logos-messaging/nim-sds/blob/4b08d508dbfa69c0e2e3883db67adf1fe5a0c994/sds/sds.nim#L242-L369)) or when the app calls `markDependenciesMet` ([#L374-L411](https://github.com/logos-messaging/nim-sds/blob/4b08d508dbfa69c0e2e3883db67adf1fe5a0c994/sds/sds.nim#L374-L411)). There is no age or timeout field on buffered entries. logos-delivery never calls `markDependenciesMet` (grep over the whole v0.39.1 tree).
5. The causal-history window is configurable but undocumented beyond a one-line field comment ([channels_conf.nim#L28](https://github.com/logos-messaging/logos-delivery/blob/v0.39.1/logos_delivery/api/conf/channels_conf.nim#L28)). It feeds nim-sds `maxCausalHistory` ([channel_lifecycle.nim#L65-L71](https://github.com/logos-messaging/logos-delivery/blob/v0.39.1/logos_delivery/channels/api/channel_lifecycle.nim#L65-L71)).

### What the spec says

The SDS spec, section "Periodic Incoming Buffer Sweep" ([sds.md#L266-L280](https://github.com/logos-co/logos-lips/blob/fddc2c1911626f7b1949360b8924bb6c05252bff/docs/anoncomms/raw/sds.md#L266-L280)):

- "The participant MUST periodically check causal dependencies for each message in the incoming buffer."
- It "MAY attempt to retrieve missing dependencies from the Store node (high-availability cache) or other peers", optionally using the `retrieval_hint`.
- "If a message's causal dependencies have failed to be met after a predetermined amount of time, the participant MAY mark them as **irretrievably lost**."

Lost-marking and Store retrieval are optional (MAY), so omitting them is not a spec violation. The periodic check of the incoming buffer is a MUST, and no such periodic pass exists. Without lost-marking or a configurable ceiling, a single unrecoverable message is unrecoverable for the sender's whole stream.

### Mechanisms that exist but did not rescue us (honest caveats)

- SDS-R (repair) is implemented. A parked message does queue its missing dependencies in the outgoing repair buffer ([sds.nim#L350-L362](https://github.com/logos-messaging/nim-sds/blob/4b08d508dbfa69c0e2e3883db67adf1fe5a0c994/sds/sds.nim#L350-L362)), but a repair request only leaves the receiver attached to one of its own outgoing messages or periodic sync messages. The receiver in our repro (C) was receive-only. Outgoing repair entries are dropped once they are more than `repairTMax` (300 s) past their request time ([#L546-L554](https://github.com/logos-messaging/nim-sds/blob/4b08d508dbfa69c0e2e3883db67adf1fe5a0c994/sds/sds.nim#L546-L554)). We did not verify at the wire level whether repair requests were sent in the live case, so we cannot say SDS-R never fires; we only observed that nothing was released.
- Periodic sync messages are not wired. nim-sds's `periodicSyncMessage` calls `rm.onPeriodicSync` ([sds.nim#L499-L507](https://github.com/logos-messaging/nim-sds/blob/4b08d508dbfa69c0e2e3883db67adf1fe5a0c994/sds/sds.nim#L499-L507)), but logos-delivery never sets it (no `onPeriodicSync` in the v0.39.1 tree), so SDS never emits the sync messages that carry repair requests when idle. It also never sets `onRetrievalHint`, so causal-history entries carry no retrieval hint ([sds_utils.nim#L313-L335](https://github.com/logos-messaging/nim-sds/blob/4b08d508dbfa69c0e2e3883db67adf1fe5a0c994/sds/sds_utils.nim#L313-L335)).
- The stash in logos-delivery is bounded, so "parked forever" is precise only for the nim-sds buffer entry. `MaxPendingContent = 32` ([#L30-L33](https://github.com/logos-messaging/logos-delivery/blob/v0.39.1/logos_delivery/channels/scalable_data_sync/scalable_data_sync.nim#L30-L33)); when it is full, `handleIncoming` drops the oldest payload with a `warn` ([#L221-L228](https://github.com/logos-messaging/logos-delivery/blob/v0.39.1/logos_delivery/channels/scalable_data_sync/scalable_data_sync.nim#L221-L228)). The nim-sds entry for that message stays, so if its dependencies later arrive, `onMessageReady` finds no payload (`messageId in self.pendingContent` is false, [#L93](https://github.com/logos-messaging/logos-delivery/blob/v0.39.1/logos_delivery/channels/scalable_data_sync/scalable_data_sync.nim#L93)) and the message is silently never delivered. Our repro (11 messages) stayed under 32, so we did not hit this. Likewise `pendingContent` is in memory only, while nim-sds persists and restores `incomingBuffer` ([sds_utils.nim#L444-L445](https://github.com/logos-messaging/nim-sds/blob/4b08d508dbfa69c0e2e3883db67adf1fe5a0c994/sds/sds_utils.nim#L444-L445)); after a restart a message parked earlier looks buffered to SDS but has no payload to release. We did not test the restart case.

## How it arose

- Nodes first sent on `logos.test`, which began requiring a funded RLN membership on 2026-09-30. Sends were accepted locally but never reached the network, and the local `sds.db` recorded them as history.
- After the nodes moved to `logos.dev`, new messages listed those never-propagated messages as causal dependencies, and receivers waited on them forever.
- The same can happen whenever any message is lost: a node offline for longer than Store retention, a dropped send, or a network/preset switch with the same data directory or identity.

## Impact

- The channel is silently and permanently one-directional for the affected sender. No error or event reaches the app, only a `debug`-level log line.
- It is not recoverable by restarting, since SDS state is persisted.
- It also fails closed in a confusing way: transport is healthy and sends report as propagated.
- Apps cannot work around it without either disabling causal history or implementing their own catch-up.

## Suggested fixes

1. Timeout, then mark as lost, then deliver. Add a periodic pass over `incomingBuffer` (the spec's Periodic Incoming Buffer Sweep) that, after a configurable age, treats still-missing dependencies as irretrievably lost, drops them from `missingDeps`, and releases the message through the normal `onMessageReady` path. Log it at `warn`.
2. Store retrieval of missing dependencies, using the existing `retrieval_hint` plumbing: set `onRetrievalHint` and act on `onMissingDependencies` with a Store query. Fall back to fix 1 if the query finds nothing.
3. A documented config knob: document `sdsCausalHistorySize` (including that 0 disables the dependency check) and add an age limit for parked messages, e.g. `sdsMissingDependencyTimeoutMs`.
4. Clear or ignore stale history on a network or preset change, or record unconfirmed sends so they are not cited as causal dependencies until acknowledged or propagated.
5. Make the stash and SDS buffer consistent: when `MaxPendingContent` evicts a payload, also drop its nim-sds `incomingBuffer` entry (or persist the payloads), so a later dependency arrival cannot silently lose a message.

## Our workaround

We create the node with:

```json
{"mode":"Edge","preset":"logos.dev","channelsOverrides":{"sdsCausalHistorySize":0}}
```

With a window of 0, messages carry no dependencies, so nothing is parked. We do our own heads-based catch-up above this. Risks:

- Gap detection and ordering guarantees from SDS are given up.
- The key is documented only by a field comment. At v0.39.1 unknown keys inside `channelsOverrides` are rejected at parse time ([tests/api/test_conf.nim#L332-L334](https://github.com/logos-messaging/logos-delivery/blob/v0.39.1/tests/api/test_conf.nim#L332-L334)), so a rename would fail loudly rather than be ignored. A change in the default or in how `0` is interpreted in a later release would be silent, so we re-test after each update.
