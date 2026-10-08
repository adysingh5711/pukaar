# Graph Report - field-study  (2026-10-08)

## Corpus Check
- 57 files · ~247,252 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 16 file(s) not represented in the graph (top: (none) 5, .mdc 4, .lock 2)

## Summary
- 724 nodes · 1426 edges · 35 communities (24 shown, 11 thin omitted)
- Extraction: 89% EXTRACTED · 11% INFERRED · 0% AMBIGUOUS · INFERRED: 153 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Logos Runtime Bindings
- Signing and Checkpoint Fuzzing
- Identity Encryption
- Node State Machine
- Persistence and Site Membership
- Checkpoint JSON Export
- CI and Build Config
- Event Store Dedup
- Lifecycle Tests
- Location Tests
- Core Module API
- Core Module Metadata
- Sync and Resend
- Retire Tests
- UI Module Metadata
- Event Body Variants
- Event Encoding
- Genesis Setup Tests
- Checkpoint Signing
- Registry Anchor Program
- App Features Screens
- Board UI Screenshot
- Decode Errors
- Roles
- Workspace Crates
- Basecamp Script
- Demo Script
- Window Script
- Deploy Methods Example
- Deploy Programs Example
- LEZ Client Gen
- Pukaar Registry

## God Nodes (most connected - your core abstractions)
1. `Node` - 78 edges
2. `Body` - 48 edges
3. `Store` - 34 edges
4. `new_key()` - 31 edges
5. `Pukaar` - 28 edges
6. `PukaarCoreModule` - 26 edges
7. `State` - 24 edges
8. `now()` - 23 edges
9. `genesis_event()` - 19 edges
10. `sign()` - 15 edges

## Surprising Connections (you probably didn't know these)
- `Pukaar app icon (square with diagonal slash, grey on dark slate)` --conceptually_related_to--> `Pukaar Board screenshot: four-column kanban (Reported, In progress, Resolved, Closed without fix)`  [INFERRED]
  modules/pukaar_ui/src/icons/pukaar.png → public/images/01-board.png
- `a_pseudonymous_joiner_still_shows_up_for_approval()` --calls--> `new_key()`  [INFERRED]
  modules/pukaar_core/rust-lib/logic/tests/lifecycle.rs → modules/pukaar_core/rust-lib/logic/src/event.rs
- `staff_are_named_and_residents_choose()` --calls--> `new_key()`  [INFERRED]
  modules/pukaar_core/rust-lib/logic/tests/lifecycle.rs → modules/pukaar_core/rust-lib/logic/src/event.rs
- `logic job (cargo test + clippy)` --conceptually_related_to--> `pukaar_logic (rules engine crate)`  [EXTRACTED]
  .github/workflows/ci.yml → README.md
- `fuzz-smoke job (cargo-fuzz decode, 60 s)` --conceptually_related_to--> `pukaar_logic (rules engine crate)`  [EXTRACTED]
  .github/workflows/ci.yml → README.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Pukaar runtime stack (UI, core, logic, Delivery)** — readme_pukaar_ui, readme_pukaar_core, readme_pukaar_logic, readme_logos_delivery [EXTRACTED 1.00]
- **Checkpoint anchoring flow** — readme_checkpoint, readme_pukaar_registry, readme_spel, readme_lez, programs_pukaar_registry_anchor_spel_inspect [INFERRED 0.85]
- **Tamper-evident history mechanisms** — readme_signed_event_envelope, readme_fork_handling, readme_checkpoint, readme_deterministic_ordering [INFERRED 0.85]
- **Pukaar report lifecycle walkthrough: board, why not closed, how closed** — public_images_01_board_board, public_images_02_why_not_closed_yet_detail_panel, public_images_03_how_it_was_closed_detail_panel [INFERRED 0.85]
- **Trust and integrity screens** — public_images_06_members_members, public_images_07_back_up_identity_identity_backup, public_images_08_anchor_anchor [INFERRED 0.75]
- **Issue lifecycle screens** — public_images_05_report_a_fault_report_form, public_images_04_history_history, public_images_06_members_add_location [INFERRED 0.65]

## Communities (35 total, 11 thin omitted)

### Community 0 - "Logos Runtime Bindings"
Cohesion: 0.06
Nodes (34): T, advance(), ANSWER_EVERY, create_and_listen(), delivered(), delivered_once(), delivery_status(), DELIVERY_TIMEOUT (+26 more)

### Community 1 - "Signing and Checkpoint Fuzzing"
Cohesion: 0.07
Nodes (39): new_key(), sign(), heads_msg(), checkpoint_signs_and_reproduces(), an_event_serialized_before_location_retire_round_trips_unchanged(), edit(), GOLDEN_REPORT, location_edit_and_remove_round_trip() (+31 more)

### Community 2 - "Identity Encryption"
Cohesion: 0.07
Nodes (34): cipher(), export_identity(), import_identity(), import_into(), MIN_PASSWORD, NONCE, PREFIX, SALT (+26 more)

### Community 3 - "Node State Machine"
Cohesion: 0.10
Nodes (15): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), fingerprint(), HEADS_TO_SETTLE, kind_name(), NAME_PLACEHOLDER (+7 more)

### Community 4 - "Persistence and Site Membership"
Cohesion: 0.08
Nodes (29): join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), remove_if_present(), RESTORING (+21 more)

### Community 5 - "Checkpoint JSON Export"
Cohesion: 0.10
Nodes (26): Location, issue_json(), location_json(), removed_json(), stage(), apply(), Config, DAY (+18 more)

### Community 6 - "CI and Build Config"
Cohesion: 0.05
Nodes (42): CI workflow, fuzz-smoke job (cargo-fuzz decode, 60 s), logic job (cargo test + clippy), pukaar_core CMakeLists.txt, LogosModule.cmake helper, metadata.json, AI agent skills index, basecamp skill (+34 more)

### Community 7 - "Event Store Dedup"
Cohesion: 0.13
Nodes (12): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+4 more)

### Community 8 - "Lifecycle Tests"
Cohesion: 0.11
Nodes (23): a_pseudonymous_joiner_still_shows_up_for_approval(), an_event_over_the_size_cap_is_refused(), an_oversize_genesis_is_refused(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline(), claim_reopen_claim_confirm() (+15 more)

### Community 9 - "Location Tests"
Cohesion: 0.13
Nodes (24): a_code_that_was_ever_used_cannot_be_added_again(), a_location_pending_removal_cannot_be_edited(), a_location_with_any_report_cannot_be_removed_even_when_resolved(), a_never_used_location_enters_pending_removal(), a_new_location_is_added_and_listed(), a_report_on_a_location_pending_removal_is_rejected_and_kept(), a_report_racing_a_remove_converges_on_every_replica(), a_retired_never_used_location_can_be_removed_and_undo_makes_it_active() (+16 more)

### Community 11 - "Core Module Metadata"
Cohesion: 0.08
Nodes (23): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+15 more)

### Community 12 - "Sync and Resend"
Cohesion: 0.13
Nodes (16): ANSWERERS, lost_node(), MAX_RESEND, resend(), send_queue(), SendError, Refused, Retry (+8 more)

### Community 13 - "Retire Tests"
Cohesion: 0.17
Nodes (17): a_reopen_cannot_bring_an_issue_back_onto_a_retired_location(), a_report_on_a_retired_location_is_rejected_and_kept(), a_report_racing_a_retire_converges_on_every_replica(), act(), an_admin_retires_a_location_with_no_open_issues(), claim(), confirm(), is_retired() (+9 more)

### Community 14 - "UI Module Metadata"
Cohesion: 0.09
Nodes (21): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+13 more)

### Community 15 - "Event Body Variants"
Cohesion: 0.11
Nodes (19): Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm, Genesis (+11 more)

### Community 16 - "Event Encoding"
Cohesion: 0.16
Nodes (10): decode(), DOMAIN, Event, location_texts(), MAX_EVENT_BYTES, MAX_TEXT, OTHER_LOCATION, Unsigned (+2 more)

### Community 17 - "Genesis Setup Tests"
Cohesion: 0.17
Nodes (6): genesis_from_json(), twice(), a_site_map_over_the_event_limit_is_refused_as_too_long(), check(), draft(), with()

### Community 18 - "Checkpoint Signing"
Cohesion: 0.26
Nodes (9): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), root_for(), verify_anchor() (+1 more)

### Community 19 - "Registry Anchor Program"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 20 - "App Features Screens"
Cohesion: 0.15
Nodes (9): History search and status/when filters, History screen (resolved/closed issues archive), Place picker grouped by water points and bins, Report a fault form (What's wrong?), Add a location form, Members screen (approve, roles, places), Roles: Admin, Steward, Resident, Back up your identity (password-protected export) (+1 more)

### Community 21 - "Board UI Screenshot"
Cohesion: 0.22
Nodes (10): Pukaar app icon (square with diagonal slash, grey on dark slate), Pukaar Board screenshot: four-column kanban (Reported, In progress, Resolved, Closed without fix), Overdue and not-acknowledged warnings (SLA, 12 h), Sidebar navigation: Board, History, Report, Members, Anchor, Identity, Status badges: Reported, In progress, Fix claimed, Resolved, Reopened, Closed won't fix, Three-step closure path: steward acknowledges, steward claims fix, reporter or 2 residents confirm, Report detail side panel with Why it isn't closed yet checklist, Report timeline (reported, comment, acknowledged, update posted) with steward note form (+2 more)

### Community 22 - "Decode Errors"
Cohesion: 0.22
Nodes (9): DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong, TooLarge (+1 more)

### Community 23 - "Roles"
Cohesion: 0.50
Nodes (4): Role, Admin, Resident, Steward

### Community 24 - "Workspace Crates"
Cohesion: 0.67
Nodes (3): pukaar_core, pukaar_logic, pukaar_logic-fuzz

## Knowledge Gaps
- **153 isolated node(s):** `name`, `display_name`, `version`, `type`, `interface` (+148 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 257 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **11 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Node State Machine` to `Logos Runtime Bindings`, `Identity Encryption`, `Persistence and Site Membership`, `Event Store Dedup`, `Lifecycle Tests`, `Location Tests`, `Retire Tests`, `Checkpoint Signing`?**
  _High betweenness centrality (0.345) - this node is a cross-community bridge._
- **Are the 29 inferred relationships involving `new_key()` (e.g. with `checkpoint_signs_and_reproduces()` and `.new()`) actually correct?**
  _`new_key()` has 29 INFERRED edges - model-reasoned connections that need verification._
- **What connects `name`, `display_name`, `version` to the rest of the system?**
  _153 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Logos Runtime Bindings` be split into smaller, more focused modules?**
  _Cohesion score 0.05701592002961866 - nodes in this community are weakly interconnected._
- **Why does `Store` connect `Event Store Dedup` to `Signing and Checkpoint Fuzzing`, `Checkpoint Signing`, `Node State Machine`, `Sync and Resend`?**
  _High betweenness centrality (0.116) - this node is a cross-community bridge._
- **Should `Signing and Checkpoint Fuzzing` be split into smaller, more focused modules?**
  _Cohesion score 0.07067603160667252 - nodes in this community are weakly interconnected._
- **Why does `Body` connect `Event Body Variants` to `Logos Runtime Bindings`, `Signing and Checkpoint Fuzzing`, `Node State Machine`, `Persistence and Site Membership`, `Checkpoint JSON Export`, `Retire Tests`, `Event Encoding`, `Genesis Setup Tests`, `Roles`?**
  _High betweenness centrality (0.101) - this node is a cross-community bridge._