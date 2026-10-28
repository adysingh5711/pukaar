# Graph Report - field-study  (2026-10-28)

## Corpus Check
- 116 files · ~189,983 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 20 file(s) not represented in the graph (top: (none) 6, .mdc 4, .qml 3)

## Summary
- 2537 nodes · 6542 edges · 116 communities (90 shown, 26 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 482 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Core Types and Utils
- Chat Conversations and Groups
- Admin Approval Flow
- Profile Announce and Sealing
- Action Staging and Storage
- Chain Discovery and Accounts
- Collection Fetch and Cache
- Place Writes and Anchors
- Backup Tests
- Pukaar Core Module API
- Atomic Storage Sync
- Backup Records and Attestation
- Call Handling
- Rule Errors
- Anchor Rules
- Runner and LEZ Keys
- Instructions and Auth
- Agent Skills Docs
- Role Checks
- Checkpoint Anchoring Ops
- Membership Rule Tests
- Agent Node RPC
- Identity Import
- Event Store Branches
- Effect Apply Logic
- Approve Item Table
- Meeting Clashes
- Config Durations
- Issue Reducer
- Pukaar Location Meetings
- Node Approval Sync
- Clock and LEZ Core
- Chain Backup PDAs
- Chain Role Store Tests
- Keyring Context
- CMake Module Config
- Event Body Types
- Delivery Status and Timing
- Meeting Backup Schema
- Node Action Bodies
- Chain Member Queries
- Checkpoint Fuzz Tests
- LEZ Key Derivation
- Sync Blob Chunks
- Backup Push and Grants
- Publish Wire Helpers
- Pukaar UI Module Config
- Common Test Helpers
- Join Request Checks
- Blob Storage
- Custom Meeting Tests
- Intent Types
- Checkpoint Records
- Act and Answer
- Announce Wire Messages
- Local Deploy Scripts
- Identity Export Crypto
- Chain Status Helpers
- Meet Jobs
- Link Status Events
- Event Format Tests
- Anchor Notes
- Testnet Scripts
- UI Board and History Docs
- UI People Screen Docs
- Event Subscription
- Queue Constants
- Settle Claims
- Pukaar Registry Anchors
- Checkpoint and Chain Docs
- Install and Modules Docs
- Chain Role Tests
- Convergence Tests
- LGX Build Script
- Account Cache Tests
- Write Kinds
- Action Types
- UI Board Docs
- Decode Errors
- Chain Writer Keys
- Review Tests
- CI Workflow Docs
- Roster Members
- Rank and Roles
- Busy Signals
- Known Place Edits
- Live Terminal Prune
- Send Queue Retry
- Local Anchor Script
- Logo Assets
- Rust Crates
- Storage Direct Script
- Status Enum
- CMake Core Module
- LEZ Client Gen
- Local LEZ Script
- Testnet LEZ Script
- Build Sh Script
- Identity Backup Docs
- Basecamp Script
- Core LGX Check
- Demo Script
- UI Screenshots Script
- Chain UI Screenshots
- Window Script
- Example Deployment Methods
- Example Deployment Programs
- Fastbuild Crate
- LEZ Client Crate
- Pukaar Registry Crate

## God Nodes (most connected - your core abstractions)
1. `Node` - 118 edges
2. `RuleError` - 78 edges
3. `Queue` - 66 edges
4. `Pukaar` - 59 edges
5. `ChatBook` - 57 edges
6. `PukaarCoreModule` - 57 edges
7. `Meetings` - 45 edges
8. `Rank` - 40 edges
9. `locked()` - 35 edges
10. `Instruction` - 35 edges

## Surprising Connections (you probably didn't know these)
- `join_request_passes_the_rules_and_both_signatures_verify()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `main()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `run()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `user()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `revoke_batch_rotates_the_key()` --calls--> `revoke()`  [INFERRED]
  programs/pukaar_sites/guest/src/tests.rs → modules/pukaar_core/rust-lib/logic/src/ops.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Rust logic crate quality checks** — github_workflows_ci_logic_job, github_workflows_ci_fuzz_smoke_job, github_workflows_ci_rust_logic_crate [EXTRACTED 1.00]
- **Two-key closure flow** — readme_report_lifecycle, readme_two_key_closure, readme_threat_model [EXTRACTED 1.00]
- **Pukaar bell logo variants** — modules_pukaar_ui_src_icons_pukaar_app_icon, public_brand_pukaar_logo_color_logo, public_brand_pukaar_logo_color_svg_logo, public_brand_pukaar_logo_logo, public_brand_pukaar_logo_svg_logo [INFERRED 0.85]

## Communities (116 total, 26 thin omitted)

### Community 0 - "Core Types and Utils"
Cohesion: 0.08
Nodes (64): sha256(), ApproveReq, Poll, Included, Pending, Rejected, a_backup_overwrites_the_pointer_and_is_done_when_the_chain_shows_the_new_cid(), a_backup_that_the_chain_holds_already_is_not_sent_and_waits_for_funds_like_any_write() (+56 more)

### Community 1 - "Chat Conversations and Groups"
Cohesion: 0.06
Nodes (54): a_conversation_belongs_to_a_person_by_its_sender_and_the_file_keeps_everything(), a_group_is_listed_at_once_with_its_name_and_member_count_and_only_when_it_is_ours(), a_group_message_shows_who_sent_it_by_the_address_book_and_a_short_key_when_unknown(), a_group_that_someone_made_is_ours_once_a_member_we_know_is_in_it(), a_linked_conversation_without_a_message_is_not_in_the_chat_list_until_one_is_queued(), a_member_is_added_once_and_goes_in_only_when_their_address_is_known_and_the_group_is_live(), a_new_conversation_takes_no_message_until_the_other_side_has_had_time_to_join(), a_thread_is_unread_until_its_newest_live_message_is_marked_read_and_that_survives_a_restart() (+46 more)

### Community 2 - "Admin Approval Flow"
Cohesion: 0.07
Nodes (58): a_forged_or_unsolicited_decline_is_ignored(), a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download(), a_join_request_is_signed_twice_and_kept(), a_member_page_has_the_profile_the_role_the_joining_time_and_the_pointer_state(), a_member_page_says_loading_none_or_removed(), a_member_queues_a_backup_pointer_and_a_stranger_does_not(), a_new_site_has_its_files_and_the_first_profile_behind_it(), a_private_profile_shows_other_members_the_name_and_role_only() (+50 more)

### Community 3 - "Profile Announce and Sealing"
Cohesion: 0.08
Nodes (61): seal(), x25519_public(), a_full_profile_fits_the_inline_cap(), a_v2_file_still_reads_and_a_new_file_is_v3(), add_seats(), add_site_wrap(), Announce, ANNOUNCE_DOMAIN (+53 more)

### Community 4 - "Action Staging and Storage"
Cohesion: 0.05
Nodes (40): stage(), account_b58(), join_site(), leave(), LEFT, left_list(), load(), load_or_create_key() (+32 more)

### Community 5 - "Chain Discovery and Accounts"
Cohesion: 0.11
Nodes (40): derive_lez_secret(), the_real_testnet_site_has_the_expected_roles(), a_blob_that_does_not_match_the_chain_hash_is_not_used_and_is_fetched_again(), a_file_of_100_kb_comes_in_chunks_and_reassembles(), a_flood_of_requests_gets_few_answers(), a_join_request_and_an_announce_name_accounts_to_read(), a_keyring_entry_that_lies_about_its_key_is_ignored(), a_lying_author_account_is_rejected() (+32 more)

### Community 6 - "Collection Fetch and Cache"
Cohesion: 0.06
Nodes (29): about_of(), account(), ANNOUNCE_EVERY, clean(), clean_about(), cycle(), Details, DETAILS_SCHEMA (+21 more)

### Community 7 - "Place Writes and Anchors"
Cohesion: 0.08
Nodes (51): Location, accounts_follow_the_runner_order(), anchor(), apply_place_op(), approve_batch(), approve_batch_names_the_requester_and_the_claims(), auth(), BATCH (+43 more)

### Community 8 - "Backup Tests"
Cohesion: 0.12
Nodes (43): A, a_backup_has_negatives(), a_backup_overwrites_the_same_record_of_the_member(), a_call_from_another_program_is_refused(), a_stale_or_future_clock_claim_fails(), a_wrong_member_pda_is_refused(), anchor_needs_an_active_member(), approve_batch_negatives() (+35 more)

### Community 10 - "Atomic Storage Sync"
Cohesion: 0.08
Nodes (28): base(), CALL, CHUNK, Client, config_gets_own_data_dir_and_free_ports(), env_override_wins_and_bad_json_is_an_error(), Events, keep_file() (+20 more)

### Community 11 - "Backup Records and Attestation"
Cohesion: 0.07
Nodes (38): sha(), ACCOUNT_ID_PREFIX, Attestation, BACKUP, BackupPointer, check_join_request(), CID_MAX, GATE_NATIVE (+30 more)

### Community 12 - "Call Handling"
Cohesion: 0.12
Nodes (40): list_of(), active(), CALL, Chat, Group, chat_of(), Person, conversations() (+32 more)

### Community 13 - "Rule Errors"
Cohesion: 0.05
Nodes (40): RuleError, AlreadyAnchored, AlreadyMember, AlreadySuperAdmin, BadBalance, BadCid, BadJoinRequest, BadMerkle (+32 more)

### Community 14 - "Anchor Rules"
Cohesion: 0.08
Nodes (33): anchor_writes_once(), apply_anchor(), check_anchor(), check_ed25519(), check_probe(), checkpoint_seed(), CheckpointRecord, CLOCK_TOLERANCE_MS (+25 more)

### Community 15 - "Runner and LEZ Keys"
Cohesion: 0.15
Nodes (18): clock_ms(), Cx, hex32(), id_env(), join_request(), join_request_passes_the_rules_and_both_signatures_verify(), Keys, main() (+10 more)

### Community 16 - "Instructions and Auth"
Cohesion: 0.08
Nodes (33): by_hand(), Instruction, Anchor, ApproveBatch, CreateSite, Probe, Reserved, RevokeBatch (+25 more)

### Community 17 - "Agent Skills Docs"
Cohesion: 0.06
Nodes (34): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+26 more)

### Community 18 - "Role Checks"
Cohesion: 0.19
Nodes (34): check_cid(), active_rank(), Actor, allow(), apply_approve(), apply_approve_site(), apply_create(), apply_revoke() (+26 more)

### Community 19 - "Checkpoint Anchoring Ops"
Cohesion: 0.11
Nodes (26): about_text(), anchor(), anchor_plan(), anchor_push(), anchor_read(), AnchorPlan, BackupSource, create_site() (+18 more)

### Community 20 - "Membership Rule Tests"
Cohesion: 0.20
Nodes (18): a_removed_member_cannot_act_until_approved_again(), a_super_admin_adds_one_more_member_and_no_third(), adds_over_the_limit_keep_the_first(), claim(), mutual_removal_keeps_the_first(), non_admins_cannot_approve(), only_a_super_admin_grants_or_changes_the_admin_role(), profile_version_must_grow() (+10 more)

### Community 21 - "Agent Node RPC"
Cohesion: 0.09
Nodes (24): a_node_reads_roles_lists_and_identity_from_the_cache(), balance(), CLAIM_MARGIN_MS, claim_ms(), clock_ms(), FLOOR_LEPTA, get_account(), HttpRpc (+16 more)

### Community 22 - "Identity Import"
Cohesion: 0.12
Nodes (25): import_identity(), import_into(), a_failed_import_writes_nothing(), a_peer_naming_a_later_seq_keeps_us_waiting(), a_restored_node_refuses_to_publish_until_its_history_is_back(), an_existing_v1_backup_still_imports(), blob(), export_then_import_restores_key_and_site() (+17 more)

### Community 23 - "Event Store Branches"
Cohesion: 0.14
Nodes (11): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+3 more)

### Community 24 - "Effect Apply Logic"
Cohesion: 0.06
Nodes (31): apply(), apply_effect(), check_balance(), check_clock(), Effect, Anchor, AnchorSigner, Approve (+23 more)

### Community 25 - "Approve Item Table"
Cohesion: 0.13
Nodes (25): A, acct(), approve_item_table(), ASHA, body(), check_join_request_table(), create_args(), create_makes_the_creator_super_admin_and_first_member() (+17 more)

### Community 26 - "Meeting Clashes"
Cohesion: 0.16
Nodes (6): Clashes, id_hex(), Meeting, Meetings, .PREFIX, Peer

### Community 27 - "Config Durations"
Cohesion: 0.11
Nodes (10): an_unreachable_node_is_an_error_not_a_hang(), GAS_LIMIT, imported_key_gives_the_account_of_lezkey(), POLL, seed_key(), SEND, SEND_RETRY_WAIT, SEND_TRIES (+2 more)

### Community 28 - "Issue Reducer"
Cohesion: 0.12
Nodes (20): issue_json(), apply(), CheckpointRecord, Config, Issue, issue_of(), not_permitted(), Progress (+12 more)

### Community 29 - "Pukaar Location Meetings"
Cohesion: 0.10
Nodes (5): later(), meet_answer(), place(), Pukaar, read()

### Community 31 - "Clock and LEZ Core"
Cohesion: 0.17
Nodes (8): Ctx, Ctx<'a>, nonempty(), pda(), plan(), probe(), set_pointer(), super_admin()

### Community 32 - "Chain Backup PDAs"
Cohesion: 0.17
Nodes (20): backup_pda(), check_version(), checkpoint_pda(), decode_backup(), decode_checkpoint(), decode_member(), decode_site(), decodes_the_real_site_account() (+12 more)

### Community 33 - "Chain Role Store Tests"
Cohesion: 0.15
Nodes (18): at_most_one_refresh_per_ten_seconds(), role_is_the_newest_known_at_any_block(), a_displaced_branch_and_its_descendants_become_evidence(), an_event_for_another_site_is_rejected(), equivocation_is_caught(), held_ids(), lamport_must_increase_along_a_chain(), out_of_order_events_wait_then_apply() (+10 more)

### Community 34 - "Keyring Context"
Cohesion: 0.21
Nodes (15): build(), confirm(), effect_on_chain(), Entry, Env, fetch_blob(), open_ring(), prepare() (+7 more)

### Community 35 - "CMake Module Config"
Cohesion: 0.08
Nodes (26): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+18 more)

### Community 36 - "Event Body Types"
Cohesion: 0.10
Nodes (21): Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm, MarkDuplicate (+13 more)

### Community 37 - "Delivery Status and Timing"
Cohesion: 0.09
Nodes (19): ANNOUNCE_REQ_EVERY, ANSWER_EVERY, chain_tick(), delivery_status(), DELIVERY_TIMEOUT, env_or(), HEADS_EVERY, LEZ_RPC (+11 more)

### Community 38 - "Meeting Backup Schema"
Cohesion: 0.08
Nodes (25): BACKUP_INFO, BACKUP_SCHEMA, Body, BUSY_DOMAIN, check_lead_min(), DEFAULT_LEAD_MIN, hit(), HOUR (+17 more)

### Community 39 - "Node Action Bodies"
Cohesion: 0.13
Nodes (12): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), fingerprint(), HEADS_TO_SETTLE, kind_name(), past() (+4 more)

### Community 40 - "Chain Member Queries"
Cohesion: 0.18
Nodes (3): Chain, now_ms(), people_lists_have_the_shape_of_main_qml()

### Community 41 - "Checkpoint Fuzz Tests"
Cohesion: 0.11
Nodes (8): checkpoint_signs_and_reproduces(), a_blob_is_kept_only_when_hash_and_signature_hold(), an_inline_profile_has_a_cap_and_a_bigger_file_is_not_a_blob(), announce(), blob_of(), every_message_kind_round_trips(), garbage_is_rejected(), size_limits_hold()

### Community 42 - "LEZ Key Derivation"
Cohesion: 0.13
Nodes (15): ACCOUNT_B58, ACCOUNT_ID_PREFIX, h(), INFO, lez_key_and_account_id_match_the_wallet(), LEZ_SK, open(), schnorr_key() (+7 more)

### Community 43 - "Sync Blob Chunks"
Cohesion: 0.14
Nodes (18): ANSWERERS, BLOB_CHUNK, DELIVERY_CFG, heads_msg(), MAX_CHUNKS, MAX_FALLBACK, MAX_RESEND, MAX_WIRE_BYTES (+10 more)

### Community 44 - "Backup Push and Grants"
Cohesion: 0.19
Nodes (17): parse_id(), backup_push(), drop_held(), grant(), push_staff_name(), reject(), revoke(), send_batch() (+9 more)

### Community 45 - "Publish Wire Helpers"
Cohesion: 0.14
Nodes (9): err(), now(), on_wire(), publish(), publish_with(), save(), start_site(), wire_chain() (+1 more)

### Community 46 - "Pukaar UI Module Config"
Cohesion: 0.10
Nodes (20): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+12 more)

### Community 47 - "Common Test Helpers"
Cohesion: 0.23
Nodes (12): canonical(), comment(), copy_all(), equip(), gossip(), node(), roles_key(), second_device() (+4 more)

### Community 48 - "Join Request Checks"
Cohesion: 0.18
Nodes (13): build(), built_request_parses_and_passes_the_rules(), check_decline(), DECLINE_DOMAIN, decline_message(), keys(), parse(), sign_decline() (+5 more)

### Community 49 - "Blob Storage"
Cohesion: 0.11
Nodes (10): Blob, St, Done, Held, Landed, Preparing, Queued, Rejected (+2 more)

### Community 50 - "Custom Meeting Tests"
Cohesion: 0.29
Nodes (14): a_clash_is_named_and_does_not_block_the_proposal(), a_custom_reminder_lead_is_kept_zero_means_none_and_too_long_is_refused(), a_custom_time_and_length_is_checked_and_carries_its_own_start_and_end(), a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side(), a_newer_summary_wins_and_an_old_one_is_ignored(), a_stranger_a_forgery_and_the_wrong_reader_change_nothing(), nobody_accepts_after_the_end_and_an_old_proposal_shows_unaccepted(), pair() (+6 more)

### Community 51 - "Intent Types"
Cohesion: 0.11
Nodes (15): Intent, Anchor, Approve, Backup, CreateSite, Details, Places, Publish (+7 more)

### Community 52 - "Checkpoint Records"
Cohesion: 0.22
Nodes (9): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+1 more)

### Community 53 - "Act and Answer"
Cohesion: 0.18
Nodes (5): Act, Accept, Cancel, Decline, check_len_ms()

### Community 54 - "Announce Wire Messages"
Cohesion: 0.12
Nodes (13): Wire, AnnounceReq, BlobChunk, BlobReq, Busy, ChainPending, ChatAddr, Event (+5 more)

### Community 55 - "Local Deploy Scripts"
Cohesion: 0.24
Nodes (16): deploy(), expect(), fail(), LEE_WALLET_HOME_DIR, mem(), neg(), new_account(), NOW() (+8 more)

### Community 56 - "Identity Export Crypto"
Cohesion: 0.14
Nodes (8): cipher(), export_identity(), MIN_PASSWORD, NONCE, PREFIX, SALT, VERSION, short_password_is_rejected()

### Community 57 - "Chain Status Helpers"
Cohesion: 0.29
Nodes (4): id_reply(), now_ms(), queued(), queued_reply()

### Community 58 - "Meet Jobs"
Cohesion: 0.19
Nodes (7): key_arg(), meet(), meet_job(), meet_q(), meet_send(), mins_arg(), ms_arg()

### Community 59 - "Link Status Events"
Cohesion: 0.12
Nodes (8): keys_arg(), Link, Down, Listening, Open, Started, Starting, Shared

### Community 60 - "Event Format Tests"
Cohesion: 0.30
Nodes (12): new_key(), every_text_is_capped_on_receipt(), GOLDEN_REPORT, only_issue_events_and_checkpoints_are_bodies(), report(), round_trip(), size_limits(), tampered_byte_fails_signature() (+4 more)

### Community 62 - "Testnet Scripts"
Cohesion: 0.29
Nodes (13): bal(), check(), die(), fin(), LEE_WALLET_HOME_DIR, log(), mem(), PUKAAR_PAYER (+5 more)

### Community 63 - "UI Board and History Docs"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 64 - "UI People Screen Docs"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 65 - "Event Subscription"
Cohesion: 0.24
Nodes (9): advance(), create_and_listen(), delivered(), delivered_once(), flush(), listen(), open_channel(), send_queued() (+1 more)

### Community 66 - "Queue Constants"
Cohesion: 0.17
Nodes (11): KEEP_DONE, LAND_WINDOW_MS, MAX_REJECTED, MAX_REQUESTS, PREP_WINDOW_MS, Prof, ProfReq, REQUEST_VALID_MS (+3 more)

### Community 67 - "Settle Claims"
Cohesion: 0.26
Nodes (6): body_of(), claimed(), need_site(), read_site(), Reads, SHORT_OF_FUNDS

### Community 68 - "Pukaar Registry Anchors"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 69 - "Checkpoint and Chain Docs"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

### Community 70 - "Install and Modules Docs"
Cohesion: 0.21
Nodes (13): chat_module 0.3.0, delivery_module 0.4.0, Install Pukaar in Logos Basecamp, Known limits, Logos Basecamp, logos-repo.json (package source), Option 2: Install by hand, Option 1: Add a package source (+5 more)

### Community 71 - "Chain Role Tests"
Cohesion: 0.33
Nodes (8): a_checkpoint_from_a_non_member_is_rejected_too(), a_resident_who_is_not_the_reporter_votes_and_staff_do_not(), an_author_who_was_not_a_member_at_the_chain_block_is_rejected_and_kept(), as_rank(), only_staff_acknowledge_claim_and_close(), report(), the_chain_block_decides_not_the_arrival_time(), why()

### Community 72 - "Convergence Tests"
Cohesion: 0.38
Nodes (5): a_forked_author_converges_to_one_branch_in_any_delivery_order(), any_delivery_order_gives_the_same_state(), random_action(), replay(), Rng

### Community 73 - "LGX Build Script"
Cohesion: 0.40
Nodes (8): busy(), fail(), heartbeat(), out(), run_module(), say(), build-lgx.sh script, status()

### Community 74 - "Account Cache Tests"
Cohesion: 0.40
Nodes (9): b58(), program(), a_failed_read_keeps_the_old_cache(), a_promoted_or_revoked_author_is_judged_by_the_newest_role(), acct(), edit(), Fake, lez_text_and_balance_read() (+1 more)

### Community 75 - "Write Kinds"
Cohesion: 0.20
Nodes (9): WriteKind, Anchor, Approve, CreateSite, Revoke, SetProfile, SetRole, SetSite (+1 more)

### Community 76 - "Action Types"
Cohesion: 0.20
Nodes (10): Action, Anchor, Approve, Revoke, SetProfile, SetRole, SetSite, SuperAdminAdd (+2 more)

### Community 77 - "UI Board Docs"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 79 - "Decode Errors"
Cohesion: 0.22
Nodes (9): DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong, TooLarge (+1 more)

### Community 80 - "Chain Writer Keys"
Cohesion: 0.25
Nodes (5): ChainWriter, KeyState, Failed, Ready, Unknown

### Community 81 - "Review Tests"
Cohesion: 0.56
Nodes (7): acct(), an_admin_cannot_lock_a_member_out_with_a_huge_profile_version(), an_anchor_signer_key_must_be_the_key_of_the_member(), h(), item(), NOW, site()

### Community 82 - "CI Workflow Docs"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 84 - "Rank and Roles"
Cohesion: 0.36
Nodes (5): Rank, Admin, Resident, Steward, SuperAdmin

### Community 85 - "Busy Signals"
Cohesion: 0.39
Nodes (3): BusyBody, Signed, Signed<T>

### Community 86 - "Known Place Edits"
Cohesion: 0.36
Nodes (8): known_place(), Place, Add, Edit, Remove, Restore, Retire, reason_text()

### Community 88 - "Send Queue Retry"
Cohesion: 0.38
Nodes (6): lost_node(), send_queue(), SendError, Refused, Retry, a_message_delivery_did_not_answer_stays_queued_with_everything_after_it()

### Community 89 - "Local Anchor Script"
Cohesion: 0.38
Nodes (6): deploy(), LEE_WALLET_HOME_DIR, new_account(), PUKAAR_PAYER, PUKAAR_PROGRAM_ID, local-anchor.sh script

### Community 90 - "Logo Assets"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 91 - "Rust Crates"
Cohesion: 0.33
Nodes (6): pukaar_core, pukaar_logic, pukaar_logic-fuzz, pukaar_sites_guest, pukaar_sites_rules, pukaar_sites_runner

### Community 92 - "Storage Direct Script"
Cohesion: 0.53
Nodes (5): log_after(), pid_of(), refuse_if_running(), storage-direct.sh script, since()

### Community 93 - "Status Enum"
Cohesion: 0.40
Nodes (5): Status, Cancelled, Confirmed, Declined, Proposed

### Community 94 - "CMake Core Module"
Cohesion: 0.50
Nodes (4): LogosModule.cmake, pukaar_core CMake module, metadata.json, Rust crate (rust-lib)

### Community 96 - "Local LEZ Script"
Cohesion: 0.50
Nodes (3): LEE_WALLET_HOME_DIR, LEZ_HOMES, lez-local.sh script

## Knowledge Gaps
- **496 isolated node(s):** `name`, `display_name`, `version`, `type`, `interface` (+491 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 795 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **26 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Node Approval Sync` to `Profile Announce and Sealing`, `Action Staging and Storage`, `Chain Discovery and Accounts`, `Place Writes and Anchors`, `Call Handling`, `Checkpoint Anchoring Ops`, `Agent Node RPC`, `Identity Import`, `Issue Reducer`, `Pukaar Location Meetings`, `Delivery Status and Timing`, `Node Action Bodies`, `Backup Push and Grants`, `Publish Wire Helpers`, `Checkpoint Records`, `Announce Wire Messages`, `Identity Export Crypto`, `Chain Status Helpers`, `Meet Jobs`, `Link Status Events`, `Chain Role Tests`, `Convergence Tests`, `Known Place Edits`?**
  _High betweenness centrality (0.103) - this node is a cross-community bridge._
- **What connects `name`, `display_name`, `version` to the rest of the system?**
  _496 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Core Types and Utils` be split into smaller, more focused modules?**
  _Cohesion score 0.07747252747252747 - nodes in this community are weakly interconnected._
- **Why does `PukaarCoreModule` connect `Pukaar Core Module API` to `Pukaar Location Meetings`, `Context Ready Hook`, `Delivery Status and Timing`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **Should `Chat Conversations and Groups` be split into smaller, more focused modules?**
  _Cohesion score 0.05806205977536646 - nodes in this community are weakly interconnected._
- **Why does `ChatBook` connect `Chat Conversations and Groups` to `Action Staging and Storage`, `Delivery Status and Timing`, `Sync Blob Chunks`, `Call Handling`, `Link Status Events`?**
  _High betweenness centrality (0.043) - this node is a cross-community bridge._
- **Should `Admin Approval Flow` be split into smaller, more focused modules?**
  _Cohesion score 0.0747871158830063 - nodes in this community are weakly interconnected._