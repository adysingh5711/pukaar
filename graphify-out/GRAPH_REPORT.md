# Graph Report - field-study  (2026-10-11)

## Corpus Check
- 19 files · ~90,524 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 883 nodes · 1792 edges · 49 communities (30 shown, 19 thin omitted)
- Extraction: 91% EXTRACTED · 9% INFERRED · 0% AMBIGUOUS · INFERRED: 160 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Logos Runtime Bindings
- Node Logic
- Sync and Checkpoint Fuzzing
- Decode Errors
- Checkpoint Logic
- Agent Skills and Docs
- Test Helpers
- Super Admin Tests
- Lifecycle Tests
- Core Module API
- Location Tests
- Event Body Variants
- Core Module Metadata
- Signing Tests
- Retire Tests
- UI Module Metadata
- Identity Sync Tests
- Identity Encryption
- Persistence
- History Screenshot
- People and Places Screens
- Identity Import Tests
- Registry Anchor Program
- Event Encoding
- Board Screenshot Features
- Place Change Kinds
- CI Workflow
- Roles and Notice Outcome
- Persist Tests
- Brand Logo and Icon
- Build Script
- Core CMake Build
- Workspace Crates
- Member Approval
- Place Lifecycle
- Basecamp Script
- Demo Script
- UI Screenshot Script
- Window Script
- Deploy Methods Example
- Deploy Programs Example
- LEZ Client Gen
- Pukaar Registry

## God Nodes (most connected - your core abstractions)
1. `Node` - 96 edges
2. `Body` - 59 edges
3. `Event` - 41 edges
4. `State` - 40 edges
5. `Pukaar` - 34 edges
6. `Store` - 32 edges
7. `new_key()` - 32 edges
8. `PukaarCoreModule` - 32 edges
9. `now()` - 31 edges
10. `deputy_site()` - 25 edges

## Surprising Connections (you probably didn't know these)
- `Generated IDL (idl/pukaar_registry.json)` --implements--> `pukaar_registry (SPEL program on LEZ)`  [INFERRED]
  programs/pukaar_registry/README.md → README.md
- `Pukaar app icon (bell, orange-magenta gradient, 256px)` --references--> `Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)`  [INFERRED]
  modules/pukaar_ui/src/icons/pukaar.png → public/brand/pukaar-logo.svg
- `Pukaar app icon (bell, orange-magenta gradient, 256px)` --conceptually_related_to--> `Pukaar color logo (PNG)`  [INFERRED]
  modules/pukaar_ui/src/icons/pukaar.png → public/brand/pukaar-logo-color.png
- `blob()` --calls--> `export_identity()`  [INFERRED]
  modules/pukaar_core/rust-lib/logic/tests/identity.rs → modules/pukaar_core/rust-lib/logic/src/identity.rs
- `short_password_is_rejected()` --calls--> `export_identity()`  [INFERRED]
  modules/pukaar_core/rust-lib/logic/tests/identity.rs → modules/pukaar_core/rust-lib/logic/src/identity.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Rust logic crate quality checks** — github_workflows_ci_logic_job, github_workflows_ci_fuzz_smoke_job, github_workflows_ci_rust_logic_crate [EXTRACTED 1.00]
- **Pukaar bell logo variants** — modules_pukaar_ui_src_icons_pukaar_app_icon, public_brand_pukaar_logo_color_logo, public_brand_pukaar_logo_color_svg_logo, public_brand_pukaar_logo_logo, public_brand_pukaar_logo_svg_logo [INFERRED 0.85]
- **Tamper-evident history mechanisms** — readme_signed_event_envelope, readme_fork_resolution, readme_checkpoint_merkle_root, readme_pukaar_registry [EXTRACTED 1.00]
- **Pukaar module stack** — readme_pukaar_ui, readme_pukaar_core, readme_pukaar_logic, readme_logos_delivery [EXTRACTED 1.00]

## Communities (49 total, 19 thin omitted)

### Community 0 - "Logos Runtime Bindings"
Cohesion: 0.05
Nodes (35): action_body(), advance(), ANSWER_EVERY, create_and_listen(), delivered(), delivered_once(), delivery_status(), DELIVERY_TIMEOUT (+27 more)

### Community 1 - "Node Logic"
Cohesion: 0.06
Nodes (49): Location, CONFIRM_WAIT_H, deadline(), fingerprint(), HEADS_TO_SETTLE, issue_json(), kind_name(), location_json() (+41 more)

### Community 2 - "Sync and Checkpoint Fuzzing"
Cohesion: 0.06
Nodes (42): new_key(), ANSWERERS, DELIVERY_CFG, heads_msg(), lost_node(), MAX_RESEND, resend(), send_queue() (+34 more)

### Community 3 - "Decode Errors"
Cohesion: 0.08
Nodes (17): decode(), DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong (+9 more)

### Community 4 - "Checkpoint Logic"
Cohesion: 0.09
Nodes (22): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+14 more)

### Community 5 - "Agent Skills and Docs"
Cohesion: 0.06
Nodes (40): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+32 more)

### Community 6 - "Test Helpers"
Cohesion: 0.09
Nodes (20): genesis_from_json(), twice(), canonical(), genesis(), gossip(), profile(), second_device(), Site (+12 more)

### Community 7 - "Super Admin Tests"
Cohesion: 0.15
Nodes (35): a_backdated_act_after_the_seal_is_rejected(), a_cancelled_notice_changes_nothing(), a_notice_keeps_admin_powers_until_the_deadline(), a_past_notice_deadline_means_now(), a_remove_outcome_revokes_the_admin(), a_residents_seal_is_rejected(), a_seal_before_the_deadline_is_rejected(), a_super_admin_adds_one_more_named_member_and_no_third() (+27 more)

### Community 8 - "Lifecycle Tests"
Cohesion: 0.10
Nodes (29): a_removal_shows_as_removed_only_once_its_cooldown_is_over(), a_revoke_is_kept_with_who_when_and_why_until_a_new_grant(), an_event_over_the_size_cap_is_refused(), an_oversize_genesis_is_refused(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline() (+21 more)

### Community 10 - "Location Tests"
Cohesion: 0.13
Nodes (25): a_code_that_was_ever_used_cannot_be_added_again(), a_location_pending_removal_cannot_be_edited(), a_location_with_any_report_cannot_be_removed_even_when_resolved(), a_never_used_location_enters_pending_removal(), a_new_location_is_added_and_listed(), a_report_on_a_location_pending_removal_is_rejected_and_kept(), a_report_racing_a_remove_converges_on_every_replica(), a_retired_never_used_location_can_be_removed_and_undo_makes_it_active() (+17 more)

### Community 11 - "Event Body Variants"
Cohesion: 0.07
Nodes (27): Body, Acknowledge, AdminNotice, AdminNoticeCancel, AdminNoticeEndNow, Checkpoint, ClaimResolved, CloseWontfix (+19 more)

### Community 12 - "Core Module Metadata"
Cohesion: 0.08
Nodes (23): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+15 more)

### Community 13 - "Signing Tests"
Cohesion: 0.18
Nodes (15): sign(), an_event_serialized_before_location_retire_round_trips_unchanged(), edit(), GOLDEN_REPORT, location_edit_and_remove_round_trip(), location_edit_and_remove_texts_over_the_cap_are_dropped(), location_retire_reason_over_the_text_cap_is_dropped(), location_retire_round_trips() (+7 more)

### Community 14 - "Retire Tests"
Cohesion: 0.17
Nodes (17): a_reopen_cannot_bring_an_issue_back_onto_a_retired_location(), a_report_on_a_retired_location_is_rejected_and_kept(), a_report_racing_a_retire_converges_on_every_replica(), act(), an_admin_retires_a_location_with_no_open_issues(), claim(), confirm(), is_retired() (+9 more)

### Community 15 - "UI Module Metadata"
Cohesion: 0.09
Nodes (21): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+13 more)

### Community 16 - "Identity Sync Tests"
Cohesion: 0.18
Nodes (14): copy_all(), a_peer_naming_a_later_seq_keeps_us_waiting(), a_restored_node_refuses_to_publish_until_its_history_is_back(), heads(), note(), once_caught_up_the_next_event_continues_the_old_chain(), PW, restored() (+6 more)

### Community 17 - "Identity Encryption"
Cohesion: 0.13
Nodes (8): cipher(), export_identity(), MIN_PASSWORD, NONCE, PREFIX, SALT, VERSION, short_password_is_rejected()

### Community 18 - "Persistence"
Cohesion: 0.24
Nodes (12): join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), remove_if_present(), RESTORING (+4 more)

### Community 19 - "History Screenshot"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 20 - "People and Places Screens"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 21 - "Identity Import Tests"
Cohesion: 0.22
Nodes (13): import_identity(), import_into(), a_failed_import_writes_nothing(), an_existing_v1_backup_still_imports(), blob(), export_then_import_restores_key_and_site(), import_fits_the_ui_call_budget(), import_into_a_fresh_device_gives_an_empty_node_with_the_same_identity() (+5 more)

### Community 22 - "Registry Anchor Program"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 23 - "Event Encoding"
Cohesion: 0.17
Nodes (7): DOMAIN, location_texts(), MAX_EVENT_BYTES, MAX_TEXT, OTHER_LOCATION, VERSION, ZERO

### Community 24 - "Board Screenshot Features"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 25 - "Place Change Kinds"
Cohesion: 0.22
Nodes (9): PlaceChangeKind, Added, Edited, RemovalStarted, RemovalUndone, Removed, Renamed, Restored (+1 more)

### Community 26 - "CI Workflow"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 27 - "Roles and Notice Outcome"
Cohesion: 0.25
Nodes (7): NoticeOutcome, Remove, Steward, Role, Admin, Resident, Steward

### Community 29 - "Brand Logo and Icon"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 31 - "Core CMake Build"
Cohesion: 0.67
Nodes (3): pukaar_core CMakeLists.txt, LogosModule.cmake helper, metadata.json

### Community 32 - "Workspace Crates"
Cohesion: 0.67
Nodes (3): pukaar_core, pukaar_logic, pukaar_logic-fuzz

## Knowledge Gaps
- **188 isolated node(s):** `CheckpointRecord`, `CheckpointRecord`, `T`, `MIN_PASSWORD`, `NONCE` (+183 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 300 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **19 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Decode Errors` to `Logos Runtime Bindings`, `Node Logic`, `Sync and Checkpoint Fuzzing`, `Test Helpers`, `Super Admin Tests`, `Lifecycle Tests`, `Location Tests`, `Retire Tests`, `Identity Sync Tests`, `Identity Encryption`, `Persistence`, `Identity Import Tests`, `Persist Tests`?**
  _High betweenness centrality (0.282) - this node is a cross-community bridge._
- **What connects `CheckpointRecord`, `CheckpointRecord`, `T` to the rest of the system?**
  _188 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Logos Runtime Bindings` be split into smaller, more focused modules?**
  _Cohesion score 0.05249569707401033 - nodes in this community are weakly interconnected._
- **Why does `Event` connect `Decode Errors` to `Logos Runtime Bindings`, `Node Logic`, `Super Admin Tests`, `Event Body Variants`, `Signing Tests`, `Event Encoding`?**
  _High betweenness centrality (0.093) - this node is a cross-community bridge._
- **Should `Node Logic` be split into smaller, more focused modules?**
  _Cohesion score 0.05939629990262902 - nodes in this community are weakly interconnected._
- **Why does `Body` connect `Event Body Variants` to `Logos Runtime Bindings`, `Node Logic`, `Sync and Checkpoint Fuzzing`, `Decode Errors`, `Test Helpers`, `Super Admin Tests`, `Signing Tests`, `Retire Tests`, `Event Encoding`, `Roles and Notice Outcome`?**
  _High betweenness centrality (0.086) - this node is a cross-community bridge._
- **Should `Sync and Checkpoint Fuzzing` be split into smaller, more focused modules?**
  _Cohesion score 0.06116700201207243 - nodes in this community are weakly interconnected._