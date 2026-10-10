# Graph Report - field-study  (2026-10-10)

## Corpus Check
- 63 files · ~82,484 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 18 file(s) not represented in the graph (top: (none) 6, .mdc 4, .lock 2)

## Summary
- 766 nodes · 1499 edges · 43 communities (28 shown, 15 thin omitted)
- Extraction: 90% EXTRACTED · 10% INFERRED · 0% AMBIGUOUS · INFERRED: 157 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Logos Runtime Bindings
- Node and Report Logic
- Signing and Checkpoint Fuzzing
- Identity Encryption
- CI, Build and Agent Skills
- Lifecycle Tests
- Event Store Dedup
- Node API and Sites
- Test Helpers
- Location Tests
- Sync and Resend
- Core Module API
- Core Module Metadata
- UI Module Metadata
- Event Body Variants
- Retire Tests
- Persistence and Membership
- Event Encoding
- Genesis Setup Tests
- Checkpoint Signing
- Board Screenshot Features
- Registry Anchor Program
- Decode Errors
- Place and Anchor Screens
- Brand Logo and Icon
- Roles
- Core CMake Build
- JSON Test Helpers
- Workspace Crates
- Invite and Members Flow
- Identity Backup Screens
- Basecamp Script
- Demo Script
- UI Screenshot Script
- Window Script
- Deploy Methods Example
- Deploy Programs Example
- LEZ Client Gen
- Pukaar Registry
- Sidebar and Proof Link

## God Nodes (most connected - your core abstractions)
1. `Node` - 79 edges
2. `Body` - 48 edges
3. `Store` - 34 edges
4. `new_key()` - 32 edges
5. `State` - 28 edges
6. `Pukaar` - 28 edges
7. `PukaarCoreModule` - 26 edges
8. `now()` - 23 edges
9. `genesis_event()` - 19 edges
10. `sign()` - 15 edges

## Surprising Connections (you probably didn't know these)
- `a_pseudonymous_joiner_still_shows_up_for_approval()` --calls--> `new_key()`  [INFERRED]
  modules/pukaar_core/rust-lib/logic/tests/lifecycle.rs → modules/pukaar_core/rust-lib/logic/src/event.rs
- `staff_are_named_and_residents_choose()` --calls--> `new_key()`  [INFERRED]
  modules/pukaar_core/rust-lib/logic/tests/lifecycle.rs → modules/pukaar_core/rust-lib/logic/src/event.rs
- `logic job (cargo test + clippy)` --conceptually_related_to--> `pukaar_logic (rules engine)`  [EXTRACTED]
  .github/workflows/ci.yml → README.md
- `fuzz-smoke job (cargo-fuzz decode, 60 s)` --conceptually_related_to--> `pukaar_logic (rules engine)`  [EXTRACTED]
  .github/workflows/ci.yml → README.md
- `Generated IDL (idl/pukaar_registry.json)` --implements--> `pukaar_registry (SPEL program)`  [INFERRED]
  programs/pukaar_registry/README.md → README.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Tamper-evident history** — readme_signed_events, readme_hash_chains_forks, readme_checkpoints, readme_pukaar_registry [EXTRACTED 1.00]
- **Pukaar module stack** — readme_pukaar_ui, readme_pukaar_core, readme_pukaar_logic, readme_logos_delivery [EXTRACTED 1.00]
- **Pukaar bell logo variants** — modules_pukaar_ui_src_icons_pukaar_app_icon, public_brand_pukaar_logo_color_logo, public_brand_pukaar_logo_color_svg_logo, public_brand_pukaar_logo_logo, public_brand_pukaar_logo_svg_logo [INFERRED 0.85]
- **Issue closure accountability flow (acknowledge, claim, confirm)** — public_images_02_why_not_closed_yet_why_panel, public_images_02_why_not_closed_yet_steward_actions, public_images_03_confirm_the_fix_confirm_panel, public_images_03_confirm_the_fix_reopen [INFERRED 0.85]
- **Board and History keep issues visible, never deleted** — public_images_01_board_board, public_images_04_history_history, public_images_04_history_nothing_deleted [INFERRED 0.75]
- **Admin trust and audit screens** — public_images_07_members_members_admin, public_images_08_site_change_log_change_log, public_images_09_proof_proof_of_history [INFERRED 0.75]
- **Identity, invite and approval flow** — public_images_06_profile_identity_backup, public_images_06_profile_invite_site_id, public_images_07_members_fingerprint_approval [INFERRED 0.75]

## Communities (43 total, 15 thin omitted)

### Community 0 - "Logos Runtime Bindings"
Cohesion: 0.05
Nodes (36): action_body(), parse_id(), T, advance(), ANSWER_EVERY, create_and_listen(), delivered(), delivered_once() (+28 more)

### Community 1 - "Node and Report Logic"
Cohesion: 0.06
Nodes (50): Location, body_text(), CONFIRM_WAIT_H, deadline(), HEADS_TO_SETTLE, issue_json(), kind_name(), location_json() (+42 more)

### Community 2 - "Signing and Checkpoint Fuzzing"
Cohesion: 0.07
Nodes (37): new_key(), sign(), checkpoint_signs_and_reproduces(), an_event_serialized_before_location_retire_round_trips_unchanged(), edit(), GOLDEN_REPORT, location_edit_and_remove_round_trip(), location_edit_and_remove_texts_over_the_cap_are_dropped() (+29 more)

### Community 3 - "Identity Encryption"
Cohesion: 0.07
Nodes (37): cipher(), export_identity(), import_identity(), import_into(), MIN_PASSWORD, NONCE, PREFIX, SALT (+29 more)

### Community 4 - "CI, Build and Agent Skills"
Cohesion: 0.06
Nodes (43): CI workflow, fuzz-smoke job (cargo-fuzz decode, 60 s), logic job (cargo test + clippy), AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill (+35 more)

### Community 5 - "Lifecycle Tests"
Cohesion: 0.09
Nodes (31): a_pseudonymous_joiner_still_shows_up_for_approval(), a_removal_shows_as_removed_only_once_its_cooldown_is_over(), a_revoke_is_kept_with_who_when_and_why_until_a_new_grant(), an_event_over_the_size_cap_is_refused(), an_oversize_genesis_is_refused(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time() (+23 more)

### Community 6 - "Event Store Dedup"
Cohesion: 0.13
Nodes (12): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+4 more)

### Community 7 - "Node API and Sites"
Cohesion: 0.14
Nodes (4): fingerprint(), Node, Restore, sign_within_limits()

### Community 8 - "Test Helpers"
Cohesion: 0.13
Nodes (15): canonical(), genesis(), gossip(), profile(), second_device(), Site, with(), a_forked_author_converges_to_one_branch_in_any_delivery_order() (+7 more)

### Community 9 - "Location Tests"
Cohesion: 0.13
Nodes (24): a_code_that_was_ever_used_cannot_be_added_again(), a_location_pending_removal_cannot_be_edited(), a_location_with_any_report_cannot_be_removed_even_when_resolved(), a_never_used_location_enters_pending_removal(), a_new_location_is_added_and_listed(), a_report_on_a_location_pending_removal_is_rejected_and_kept(), a_report_racing_a_remove_converges_on_every_replica(), a_retired_never_used_location_can_be_removed_and_undo_makes_it_active() (+16 more)

### Community 10 - "Sync and Resend"
Cohesion: 0.13
Nodes (19): ANSWERERS, DELIVERY_CFG, heads_msg(), lost_node(), MAX_RESEND, resend(), send_queue(), SendError (+11 more)

### Community 12 - "Core Module Metadata"
Cohesion: 0.08
Nodes (23): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+15 more)

### Community 13 - "UI Module Metadata"
Cohesion: 0.09
Nodes (21): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+13 more)

### Community 14 - "Event Body Variants"
Cohesion: 0.11
Nodes (19): Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm, Genesis (+11 more)

### Community 15 - "Retire Tests"
Cohesion: 0.21
Nodes (15): a_reopen_cannot_bring_an_issue_back_onto_a_retired_location(), a_report_on_a_retired_location_is_rejected_and_kept(), a_report_racing_a_retire_converges_on_every_replica(), act(), claim(), confirm(), is_retired(), old_issues_stay_readable_at_a_retired_location() (+7 more)

### Community 16 - "Persistence and Membership"
Cohesion: 0.22
Nodes (12): join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), remove_if_present(), RESTORING (+4 more)

### Community 17 - "Event Encoding"
Cohesion: 0.15
Nodes (10): decode(), DOMAIN, Event, location_texts(), MAX_EVENT_BYTES, MAX_TEXT, OTHER_LOCATION, Unsigned (+2 more)

### Community 18 - "Genesis Setup Tests"
Cohesion: 0.17
Nodes (6): genesis_from_json(), twice(), a_site_map_over_the_event_limit_is_refused_as_too_long(), check(), draft(), with()

### Community 19 - "Checkpoint Signing"
Cohesion: 0.26
Nodes (10): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+2 more)

### Community 20 - "Board Screenshot Features"
Cohesion: 0.15
Nodes (14): Pukaar Board (4-column kanban), Proof - no conflicts and delivery online indicator, Sidebar nav (Board, Report, History, Admin: Members, Site), Red SLA warnings (24h ack, 72h fix target, overdue), Status columns: Reported, In progress, Resolved, Closed without fix, Issue detail drawer with role-based actions and timeline, Steward actions: Acknowledge, Post update, Claim fixed, with disabled-reason notice, Why it isn't closed yet 3-step panel (+6 more)

### Community 21 - "Registry Anchor Program"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 22 - "Decode Errors"
Cohesion: 0.22
Nodes (9): DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong, TooLarge (+1 more)

### Community 23 - "Place and Anchor Screens"
Cohesion: 0.33
Nodes (7): Where? place picker grouped by category with search, group filter, retired toggle, Report a Fault form (What's wrong?), Site change log (21 place events), Retire/remove/restore lifecycle with reasons, hidden not deleted, 3-step anchor flow: compute checkpoint, copy spel command, record reference, Logos blockchain (LEZ) anchor of history fingerprint, Proof of history page

### Community 24 - "Brand Logo and Icon"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 25 - "Roles"
Cohesion: 0.50
Nodes (4): Role, Admin, Resident, Steward

### Community 26 - "Core CMake Build"
Cohesion: 0.67
Nodes (3): pukaar_core CMakeLists.txt, LogosModule.cmake helper, metadata.json

### Community 28 - "Workspace Crates"
Cohesion: 0.67
Nodes (3): pukaar_core, pukaar_logic, pukaar_logic-fuzz

### Community 29 - "Invite and Members Flow"
Cohesion: 0.67
Nodes (3): Invite someone via site id, Grant role only after person reads fingerprint aloud, Members admin page (approve and manage roles)

## Knowledge Gaps
- **169 isolated node(s):** `name`, `display_name`, `version`, `type`, `interface` (+164 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 270 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **15 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Node API and Sites` to `Logos Runtime Bindings`, `Node and Report Logic`, `Identity Encryption`, `Lifecycle Tests`, `Event Store Dedup`, `Test Helpers`, `Location Tests`, `Retire Tests`, `Persistence and Membership`, `Checkpoint Signing`, `JSON Test Helpers`?**
  _High betweenness centrality (0.330) - this node is a cross-community bridge._
- **Are the 30 inferred relationships involving `new_key()` (e.g. with `checkpoint_signs_and_reproduces()` and `.new()`) actually correct?**
  _`new_key()` has 30 INFERRED edges - model-reasoned connections that need verification._
- **What connects `name`, `display_name`, `version` to the rest of the system?**
  _169 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Logos Runtime Bindings` be split into smaller, more focused modules?**
  _Cohesion score 0.05427905427905428 - nodes in this community are weakly interconnected._
- **Why does `Store` connect `Event Store Dedup` to `Node and Report Logic`, `Signing and Checkpoint Fuzzing`, `Node API and Sites`, `Sync and Resend`, `Checkpoint Signing`?**
  _High betweenness centrality (0.111) - this node is a cross-community bridge._
- **Should `Node and Report Logic` be split into smaller, more focused modules?**
  _Cohesion score 0.05754527162977867 - nodes in this community are weakly interconnected._
- **Why does `Body` connect `Event Body Variants` to `Logos Runtime Bindings`, `Node and Report Logic`, `Signing and Checkpoint Fuzzing`, `Node API and Sites`, `Test Helpers`, `Retire Tests`, `Event Encoding`, `Genesis Setup Tests`, `Roles`?**
  _High betweenness centrality (0.093) - this node is a cross-community bridge._