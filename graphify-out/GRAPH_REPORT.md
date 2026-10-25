# Graph Report - field-study  (2026-10-25)

## Corpus Check
- 66 files · ~113,645 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 18 file(s) not represented in the graph (top: (none) 6, .mdc 4, .lock 2)

## Summary
- 893 nodes · 1814 edges · 46 communities (32 shown, 14 thin omitted)
- Extraction: 90% EXTRACTED · 10% INFERRED · 0% AMBIGUOUS · INFERRED: 175 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Checkpoint Signing and Sync
- Logos Runtime and Atomics
- Location and Event Payloads
- Node State Machine
- Registry Agent Docs
- Super Admin Lifecycle Tests
- Member Lifecycle Tests
- Store and Event Outcomes
- Core Module API
- Location Tests
- Event Body Variants
- Identity Crypto and Backup
- Convergence and Fork Tests
- Core Module Metadata
- Identity Import Tests
- Location Retirement Tests
- UI Module Metadata
- Checkpoint Records
- Event Encoding
- Persistence Layer
- History Screen Spec
- People Screen Spec
- Registry Program Guest
- Architecture Diagram
- Board Screen Spec
- Persistence Tests
- Decode Errors
- CI Workflow
- Notice Outcomes and Roles
- Persistence Round Trips
- Test Site Fixtures
- Brand Logo and Icon
- Registry Client Generator
- Core Build Config
- Package Definitions
- Basecamp Script
- Demo Script
- UI Screenshot Script
- Window Script
- Example Deployment
- Example Deployment Programs
- Client Generator Package
- Registry Package

## God Nodes (most connected - your core abstractions)
1. `Node` - 96 edges
2. `Body` - 59 edges
3. `State` - 40 edges
4. `Store` - 34 edges
5. `Pukaar` - 34 edges
6. `new_key()` - 32 edges
7. `PukaarCoreModule` - 32 edges
8. `now()` - 31 edges
9. `deputy_site()` - 25 edges
10. `apply()` - 21 edges

## Surprising Connections (you probably didn't know these)
- `a_pseudonymous_joiner_still_shows_up_for_approval()` --calls--> `new_key()`  [INFERRED]
  modules/pukaar_core/rust-lib/logic/tests/lifecycle.rs → modules/pukaar_core/rust-lib/logic/src/event.rs
- `staff_are_named_and_residents_choose()` --calls--> `new_key()`  [INFERRED]
  modules/pukaar_core/rust-lib/logic/tests/lifecycle.rs → modules/pukaar_core/rust-lib/logic/src/event.rs
- `Generated IDL (idl/pukaar_registry.json)` --implements--> `pukaar_registry`  [INFERRED]
  programs/pukaar_registry/README.md → README.md
- `Pukaar app icon (bell, orange-magenta gradient, 256px)` --conceptually_related_to--> `Pukaar color logo (PNG)`  [INFERRED]
  modules/pukaar_ui/src/icons/pukaar.png → public/brand/pukaar-logo-color.png
- `Pukaar app icon (bell, orange-magenta gradient, 256px)` --references--> `Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)`  [INFERRED]
  modules/pukaar_ui/src/icons/pukaar.png → public/brand/pukaar-logo.svg

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Rust logic crate quality checks** — github_workflows_ci_logic_job, github_workflows_ci_fuzz_smoke_job, github_workflows_ci_rust_logic_crate [EXTRACTED 1.00]
- **Admin role change via notice and seal** — readme_super_admin, readme_admin_notice, readme_notice_seal [INFERRED 0.75]
- **Checkpoint anchoring chain** — readme_pukaar_core, readme_checkpoint_merkle_root, readme_pukaar_registry, readme_lez [INFERRED 0.75]
- **Tamper-evidence via signed events and fork proofs** — readme_signed_event_envelope, readme_fork_proof, readme_fork_resolution_lowest_id [INFERRED 0.75]
- **Pukaar bell logo variants** — modules_pukaar_ui_src_icons_pukaar_app_icon, public_brand_pukaar_logo_color_logo, public_brand_pukaar_logo_color_svg_logo, public_brand_pukaar_logo_logo, public_brand_pukaar_logo_svg_logo [INFERRED 0.85]

## Communities (46 total, 14 thin omitted)

### Community 0 - "Checkpoint Signing and Sync"
Cohesion: 0.05
Nodes (56): new_key(), sign(), ANSWERERS, DELIVERY_CFG, heads_msg(), lost_node(), MAX_RESEND, resend() (+48 more)

### Community 1 - "Logos Runtime and Atomics"
Cohesion: 0.05
Nodes (34): T, advance(), ANSWER_EVERY, create_and_listen(), delivered(), delivered_once(), delivery_status(), DELIVERY_TIMEOUT (+26 more)

### Community 2 - "Location and Event Payloads"
Cohesion: 0.06
Nodes (47): Location, issue_json(), location_json(), notice_json(), place_change_json(), removed_json(), revocation_json(), stage() (+39 more)

### Community 3 - "Node State Machine"
Cohesion: 0.08
Nodes (18): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), fingerprint(), HEADS_TO_SETTLE, kind_name(), merge() (+10 more)

### Community 4 - "Registry Agent Docs"
Cohesion: 0.05
Nodes (41): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+33 more)

### Community 5 - "Super Admin Lifecycle Tests"
Cohesion: 0.15
Nodes (37): profile(), a_backdated_act_after_the_seal_is_rejected(), a_cancelled_notice_changes_nothing(), a_notice_keeps_admin_powers_until_the_deadline(), a_past_notice_deadline_means_now(), a_remove_outcome_revokes_the_admin(), a_residents_seal_is_rejected(), a_seal_before_the_deadline_is_rejected() (+29 more)

### Community 6 - "Member Lifecycle Tests"
Cohesion: 0.09
Nodes (31): a_pseudonymous_joiner_still_shows_up_for_approval(), a_removal_shows_as_removed_only_once_its_cooldown_is_over(), a_revoke_is_kept_with_who_when_and_why_until_a_new_grant(), an_event_over_the_size_cap_is_refused(), an_oversize_genesis_is_refused(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time() (+23 more)

### Community 7 - "Store and Event Outcomes"
Cohesion: 0.13
Nodes (12): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+4 more)

### Community 9 - "Location Tests"
Cohesion: 0.13
Nodes (24): a_code_that_was_ever_used_cannot_be_added_again(), a_location_pending_removal_cannot_be_edited(), a_location_with_any_report_cannot_be_removed_even_when_resolved(), a_never_used_location_enters_pending_removal(), a_new_location_is_added_and_listed(), a_report_on_a_location_pending_removal_is_rejected_and_kept(), a_report_racing_a_remove_converges_on_every_replica(), a_retired_never_used_location_can_be_removed_and_undo_makes_it_active() (+16 more)

### Community 10 - "Event Body Variants"
Cohesion: 0.08
Nodes (26): Body, Acknowledge, AdminNotice, AdminNoticeCancel, AdminNoticeEndNow, Checkpoint, ClaimResolved, CloseWontfix (+18 more)

### Community 11 - "Identity Crypto and Backup"
Cohesion: 0.11
Nodes (16): cipher(), export_identity(), import_identity(), MIN_PASSWORD, NONCE, PREFIX, SALT, VERSION (+8 more)

### Community 12 - "Convergence and Fork Tests"
Cohesion: 0.15
Nodes (12): genesis_from_json(), gossip(), a_forked_author_converges_to_one_branch_in_any_delivery_order(), after_losing_a_fork_our_next_event_builds_on_the_winner(), any_delivery_order_gives_the_same_state(), random_action(), replay(), Rng (+4 more)

### Community 13 - "Core Module Metadata"
Cohesion: 0.08
Nodes (23): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+15 more)

### Community 14 - "Identity Import Tests"
Cohesion: 0.17
Nodes (16): import_into(), a_failed_import_writes_nothing(), a_peer_naming_a_later_seq_keeps_us_waiting(), a_restored_node_refuses_to_publish_until_its_history_is_back(), heads(), import_into_a_fresh_device_gives_an_empty_node_with_the_same_identity(), import_is_refused_when_a_site_exists(), note() (+8 more)

### Community 15 - "Location Retirement Tests"
Cohesion: 0.17
Nodes (17): a_reopen_cannot_bring_an_issue_back_onto_a_retired_location(), a_report_on_a_retired_location_is_rejected_and_kept(), a_report_racing_a_retire_converges_on_every_replica(), act(), an_admin_retires_a_location_with_no_open_issues(), claim(), confirm(), is_retired() (+9 more)

### Community 16 - "UI Module Metadata"
Cohesion: 0.09
Nodes (21): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+13 more)

### Community 17 - "Checkpoint Records"
Cohesion: 0.21
Nodes (10): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+2 more)

### Community 18 - "Event Encoding"
Cohesion: 0.15
Nodes (10): decode(), DOMAIN, Event, location_texts(), MAX_EVENT_BYTES, MAX_TEXT, OTHER_LOCATION, Unsigned (+2 more)

### Community 19 - "Persistence Layer"
Cohesion: 0.27
Nodes (10): join_site(), leave(), LEFT, left_list(), load_or_create_key(), remove_if_present(), RESTORING, save() (+2 more)

### Community 20 - "History Screen Spec"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 21 - "People Screen Spec"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 22 - "Registry Program Guest"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 23 - "Architecture Diagram"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

### Community 24 - "Board Screen Spec"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 26 - "Decode Errors"
Cohesion: 0.22
Nodes (9): DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong, TooLarge (+1 more)

### Community 27 - "CI Workflow"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 28 - "Notice Outcomes and Roles"
Cohesion: 0.25
Nodes (7): NoticeOutcome, Remove, Steward, Role, Admin, Resident, Steward

### Community 29 - "Persistence Round Trips"
Cohesion: 0.38
Nodes (7): load(), copy_all(), second_device(), the_syncing_flag_survives_a_restart_and_clears_on_disk(), a_name_the_admin_gives_a_resident_shows_on_both_sides_until_they_change_it(), fork_evidence_survives_a_restart_whatever_the_file_order(), save_then_load_gives_the_same_state()

### Community 30 - "Test Site Fixtures"
Cohesion: 0.52
Nodes (3): genesis(), Site, with()

### Community 31 - "Brand Logo and Icon"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 33 - "Core Build Config"
Cohesion: 0.67
Nodes (3): pukaar_core CMakeLists.txt, LogosModule.cmake helper, metadata.json

### Community 34 - "Package Definitions"
Cohesion: 0.67
Nodes (3): pukaar_core, pukaar_logic, pukaar_logic-fuzz

## Knowledge Gaps
- **188 isolated node(s):** `name`, `display_name`, `version`, `type`, `interface` (+183 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 297 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **14 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Node State Machine` to `Logos Runtime and Atomics`, `Super Admin Lifecycle Tests`, `Member Lifecycle Tests`, `Store and Event Outcomes`, `Location Tests`, `Identity Crypto and Backup`, `Convergence and Fork Tests`, `Identity Import Tests`, `Location Retirement Tests`, `Checkpoint Records`, `Persistence Layer`, `Persistence Tests`, `Persistence Round Trips`, `Test Site Fixtures`?**
  _High betweenness centrality (0.333) - this node is a cross-community bridge._
- **What connects `name`, `display_name`, `version` to the rest of the system?**
  _188 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Checkpoint Signing and Sync` be split into smaller, more focused modules?**
  _Cohesion score 0.05158324821246169 - nodes in this community are weakly interconnected._
- **Why does `Body` connect `Event Body Variants` to `Checkpoint Signing and Sync`, `Logos Runtime and Atomics`, `Location and Event Payloads`, `Node State Machine`, `Super Admin Lifecycle Tests`, `Convergence and Fork Tests`, `Location Retirement Tests`, `Event Encoding`, `Notice Outcomes and Roles`, `Test Site Fixtures`?**
  _High betweenness centrality (0.101) - this node is a cross-community bridge._
- **Should `Logos Runtime and Atomics` be split into smaller, more focused modules?**
  _Cohesion score 0.0531883632089333 - nodes in this community are weakly interconnected._
- **Why does `Store` connect `Store and Event Outcomes` to `Checkpoint Signing and Sync`, `Checkpoint Records`, `Node State Machine`?**
  _High betweenness centrality (0.096) - this node is a cross-community bridge._
- **Should `Location and Event Payloads` be split into smaller, more focused modules?**
  _Cohesion score 0.061754385964912284 - nodes in this community are weakly interconnected._