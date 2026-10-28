# Graph Report - field-study  (2026-10-28)

## Corpus Check
- 116 files · ~191,071 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 20 file(s) not represented in the graph (top: (none) 6, .mdc 4, .qml 3)

## Summary
- 2556 nodes · 6604 edges · 123 communities (96 shown, 27 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 484 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Core Utilities and Types
- Chat and Conversations
- Membership Approvals and Roles
- Clock and Program Tests
- Peer Discovery and Blob Sync
- Pukaar Core Module API
- Checkpoints and Anchor Plans
- Location Writes and Accounts
- Record Parsing and Backups
- Sealed Box Crypto and Profiles
- Profile Fetch and Announcements
- Rule Errors and Formatting
- PDA Seeds and Runner Keys
- Anchor Rule Checks
- Borsh Instructions and Anchors
- Agent Skills Index
- Admin Membership Rules Tests
- Role Authorization Logic
- Logos Event Subscriptions
- Atomic Storage Chunks
- Identity Import and Export
- Chat and Group Lists
- Store Accept and Fork Handling
- Anchor Effects and Balance Checks
- Account ID Encoding
- Duration and Config Types
- Persistence Files and Events
- Issue Reducer and Checkpoint Records
- Place Hub Lifecycle
- Meetings and Backups
- Lifecycle Stage Tests
- Chain Member Lookup
- Approval Rule Tables
- Node Fetch and Answers
- Clock and LEE Program Plans
- Keyring and Effect Confirmation
- Node RPC and Balances
- Module CMake Build Config
- Event Bodies and Acknowledgements
- Chain PDAs and Backups
- Announce Signatures and Profiles
- Node Body Text and Deadlines
- Checkpoint Fuzz and Tests
- Meeting Accept and Cancel
- Announce Lists and Answers
- Meeting Proposal Tests
- Sync Delivery Limits
- Busy Body and Accept Messages
- Role Parsing and Codes
- Delivery Status and Flush
- Publish and Errors
- UI Module Build Config
- Common Test Helpers
- Meeting Job Checks
- Intents and Write Actions
- Logos Client Events
- Chat Calls and Invites
- Roster and Membership Lookup
- Announce and Blob Requests
- Anchor Notes and Backup Writes
- Local Test Deploy Script
- Identity Encryption
- Chain Event Validation Tests
- Link State Listening
- Checkpoint Signing
- Event Format Tests
- Approval Queue
- Chain Writer and Import
- Join Request Checks
- Testnet Scripts
- UI History and Board Features
- UI People and Members Screen
- Checkpoint Decoding and Settle
- Anchor Registry Program
- Basecamp Docs and Deployment
- Install and Module Versions
- Account Read and Cache
- Meeting Ranges and Steps
- Store Branch Tests
- Chain Role Tests
- Convergence Tests
- LGX Build Script
- Write Kind Actions
- UI Board Columns
- Action Body Parsing
- Decode Errors
- Queue Status States
- Review Tests
- CI Workflow
- Place Edit Actions
- Blob Push and Save
- Send Queue Retry
- Local Anchor Script
- App Icons and Logos
- Rust Crate Workspace
- Storage Direct Script
- Meeting Proposal Replies
- Meeting Status States
- CMake Module Setup
- Super Admin Actions
- Path Buffer Client Gen
- LEZ Local Script
- LEZ Testnet Script
- Build Shell Script
- Sealed Identity Backup Docs
- Basecamp Script
- Core LGX Check Script
- Demo Script
- UI Screenshot Script
- Chain UI Screenshot Script
- Window Script
- Example Program Deployment
- Example Program Programs
- Fastbuild Crate
- LEZ Client Gen Crate
- Registry Crate

## God Nodes (most connected - your core abstractions)
1. `Node` - 118 edges
2. `RuleError` - 78 edges
3. `Queue` - 66 edges
4. `Pukaar` - 59 edges
5. `ChatBook` - 57 edges
6. `PukaarCoreModule` - 57 edges
7. `Meetings` - 46 edges
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

## Communities (123 total, 27 thin omitted)

### Community 0 - "Core Utilities and Types"
Cohesion: 0.08
Nodes (63): sha256(), Poll, Included, Pending, Rejected, a_backup_overwrites_the_pointer_and_is_done_when_the_chain_shows_the_new_cid(), a_backup_that_the_chain_holds_already_is_not_sent_and_waits_for_funds_like_any_write(), a_balance_that_drops_after_prepare_is_caught_at_the_send() (+55 more)

### Community 1 - "Chat and Conversations"
Cohesion: 0.06
Nodes (53): a_conversation_belongs_to_a_person_by_its_sender_and_the_file_keeps_everything(), a_group_is_listed_at_once_with_its_name_and_member_count_and_only_when_it_is_ours(), a_group_message_shows_who_sent_it_by_the_address_book_and_a_short_key_when_unknown(), a_group_that_someone_made_is_ours_once_a_member_we_know_is_in_it(), a_linked_conversation_without_a_message_is_not_in_the_chat_list_until_one_is_queued(), a_member_is_added_once_and_goes_in_only_when_their_address_is_known_and_the_group_is_live(), a_new_conversation_takes_no_message_until_the_other_side_has_had_time_to_join(), a_thread_is_unread_until_its_newest_live_message_is_marked_read_and_that_survives_a_restart() (+45 more)

### Community 2 - "Membership Approvals and Roles"
Cohesion: 0.08
Nodes (57): a_forged_or_unsolicited_decline_is_ignored(), a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download(), a_join_request_is_signed_twice_and_kept(), a_member_page_has_the_profile_the_role_the_joining_time_and_the_pointer_state(), a_member_page_says_loading_none_or_removed(), a_member_queues_a_backup_pointer_and_a_stranger_does_not(), a_new_site_has_its_files_and_the_first_profile_behind_it(), a_private_profile_shows_other_members_the_name_and_role_only() (+49 more)

### Community 3 - "Clock and Program Tests"
Cohesion: 0.12
Nodes (43): A, a_backup_has_negatives(), a_backup_overwrites_the_same_record_of_the_member(), a_call_from_another_program_is_refused(), a_stale_or_future_clock_claim_fails(), a_wrong_member_pda_is_refused(), anchor_needs_an_active_member(), approve_batch_negatives() (+35 more)

### Community 4 - "Peer Discovery and Blob Sync"
Cohesion: 0.12
Nodes (38): a_blob_that_does_not_match_the_chain_hash_is_not_used_and_is_fetched_again(), a_file_of_100_kb_comes_in_chunks_and_reassembles(), a_flood_of_requests_gets_few_answers(), a_join_request_and_an_announce_name_accounts_to_read(), a_keyring_entry_that_lies_about_its_key_is_ignored(), a_lying_author_account_is_rejected(), a_node_that_was_offline_gets_the_announces_it_missed_and_only_valid_newer_ones(), a_removal_reason_comes_from_details_and_must_match_its_hash() (+30 more)

### Community 6 - "Checkpoints and Anchor Plans"
Cohesion: 0.10
Nodes (37): parse_id(), about_text(), anchor(), anchor_plan(), anchor_push(), anchor_read(), AnchorPlan, backup_push() (+29 more)

### Community 7 - "Location Writes and Accounts"
Cohesion: 0.10
Nodes (43): Location, accounts_follow_the_runner_order(), anchor(), apply_place_op(), approve_batch(), approve_batch_names_the_requester_and_the_claims(), auth(), BATCH (+35 more)

### Community 8 - "Record Parsing and Backups"
Cohesion: 0.06
Nodes (40): parse(), sha(), ACCOUNT_ID_PREFIX, Attestation, BACKUP, BackupPointer, check_join_request(), CID_MAX (+32 more)

### Community 9 - "Sealed Box Crypto and Profiles"
Cohesion: 0.08
Nodes (41): seal(), sealed_box_round_trip_wrong_key_and_libsodium(), x25519_public(), About, add_seats(), add_site_wrap(), ANNOUNCE_DOMAIN, Body (+33 more)

### Community 10 - "Profile Fetch and Announcements"
Cohesion: 0.08
Nodes (27): about_of(), ANNOUNCE_EVERY, clean(), clean_about(), cycle(), Details, DETAILS_SCHEMA, fetch_cycle() (+19 more)

### Community 11 - "Rule Errors and Formatting"
Cohesion: 0.05
Nodes (40): RuleError, AlreadyAnchored, AlreadyMember, AlreadySuperAdmin, BadBalance, BadCid, BadJoinRequest, BadMerkle (+32 more)

### Community 12 - "PDA Seeds and Runner Keys"
Cohesion: 0.14
Nodes (18): clock_ms(), Cx, hex32(), id_env(), join_request(), join_request_passes_the_rules_and_both_signatures_verify(), Keys, main() (+10 more)

### Community 13 - "Anchor Rule Checks"
Cohesion: 0.08
Nodes (33): anchor_writes_once(), apply_anchor(), check_anchor(), check_ed25519(), check_probe(), checkpoint_seed(), CheckpointRecord, CLOCK_TOLERANCE_MS (+25 more)

### Community 14 - "Borsh Instructions and Anchors"
Cohesion: 0.08
Nodes (33): by_hand(), Instruction, Anchor, ApproveBatch, CreateSite, Probe, Reserved, RevokeBatch (+25 more)

### Community 15 - "Agent Skills Index"
Cohesion: 0.06
Nodes (34): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+26 more)

### Community 16 - "Admin Membership Rules Tests"
Cohesion: 0.19
Nodes (18): a_removed_member_cannot_act_until_approved_again(), a_super_admin_adds_one_more_member_and_no_third(), adds_over_the_limit_keep_the_first(), claim(), mutual_removal_keeps_the_first(), non_admins_cannot_approve(), only_a_super_admin_grants_or_changes_the_admin_role(), profile_version_must_grow() (+10 more)

### Community 17 - "Role Authorization Logic"
Cohesion: 0.22
Nodes (31): check_cid(), active_rank(), Actor, allow(), apply_approve(), apply_approve_site(), apply_create(), apply_revoke() (+23 more)

### Community 18 - "Logos Event Subscriptions"
Cohesion: 0.08
Nodes (23): advance(), ANNOUNCE_REQ_EVERY, ANSWER_EVERY, chain_tick(), create_and_listen(), delivered(), delivered_once(), DELIVERY_TIMEOUT (+15 more)

### Community 19 - "Atomic Storage Chunks"
Cohesion: 0.10
Nodes (25): base(), CALL, CHUNK, config_gets_own_data_dir_and_free_ports(), env_override_wins_and_bad_json_is_an_error(), keep_file(), keep_file_restores_or_removes(), MAX_EVENTS (+17 more)

### Community 20 - "Identity Import and Export"
Cohesion: 0.12
Nodes (25): import_identity(), import_into(), a_failed_import_writes_nothing(), a_peer_naming_a_later_seq_keeps_us_waiting(), a_restored_node_refuses_to_publish_until_its_history_is_back(), an_existing_v1_backup_still_imports(), blob(), export_then_import_restores_key_and_site() (+17 more)

### Community 21 - "Chat and Group Lists"
Cohesion: 0.16
Nodes (29): list_of(), the_module_lists_read_with_float_times_and_either_wrapping(), active(), Chat, Group, chat_of(), Person, conversations() (+21 more)

### Community 22 - "Store Accept and Fork Handling"
Cohesion: 0.14
Nodes (11): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+3 more)

### Community 23 - "Anchor Effects and Balance Checks"
Cohesion: 0.06
Nodes (31): apply(), apply_effect(), check_balance(), check_clock(), Effect, Anchor, AnchorSigner, Approve (+23 more)

### Community 24 - "Account ID Encoding"
Cohesion: 0.11
Nodes (21): build(), tampering_and_wrong_keys_are_refused(), ACCOUNT_B58, ACCOUNT_ID_PREFIX, derive_lez_secret(), h(), INFO, lez_account_id() (+13 more)

### Community 25 - "Duration and Config Types"
Cohesion: 0.11
Nodes (10): an_unreachable_node_is_an_error_not_a_hang(), GAS_LIMIT, imported_key_gives_the_account_of_lezkey(), POLL, seed_key(), SEND, SEND_RETRY_WAIT, SEND_TRIES (+2 more)

### Community 26 - "Persistence Files and Events"
Cohesion: 0.12
Nodes (16): account_b58(), join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), path_of() (+8 more)

### Community 27 - "Issue Reducer and Checkpoint Records"
Cohesion: 0.12
Nodes (20): issue_json(), apply(), CheckpointRecord, Config, Issue, issue_of(), not_permitted(), Progress (+12 more)

### Community 28 - "Place Hub Lifecycle"
Cohesion: 0.10
Nodes (5): later(), meet_answer(), place(), Pukaar, read()

### Community 29 - "Meetings and Backups"
Cohesion: 0.12
Nodes (6): Clashes, hit(), Meetings, .PREFIX, Peer, Slot

### Community 30 - "Lifecycle Stage Tests"
Cohesion: 0.12
Nodes (22): stage(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline(), claim_reopen_claim_confirm(), finished_cards_carry_no_pending_next_step(), forbidden_transitions_are_inert_and_visible() (+14 more)

### Community 31 - "Chain Member Lookup"
Cohesion: 0.15
Nodes (6): Chain, NATIVE_PROGRAM, now_ms(), people_lists_have_the_shape_of_main_qml(), PROGRAM, refresh_fills_the_roster()

### Community 32 - "Approval Rule Tables"
Cohesion: 0.14
Nodes (24): A, acct(), approve_item_table(), ASHA, check_join_request_table(), create_args(), create_makes_the_creator_super_admin_and_first_member(), create_table() (+16 more)

### Community 34 - "Clock and LEE Program Plans"
Cohesion: 0.17
Nodes (8): Ctx, Ctx<'a>, nonempty(), pda(), plan(), probe(), set_pointer(), super_admin()

### Community 35 - "Keyring and Effect Confirmation"
Cohesion: 0.20
Nodes (16): build(), confirm(), effect_on_chain(), Entry, Env, fetch_blob(), need_site(), open_ring() (+8 more)

### Community 36 - "Node RPC and Balances"
Cohesion: 0.11
Nodes (20): a_node_reads_roles_lists_and_identity_from_the_cache(), balance(), CLAIM_MARGIN_MS, claim_ms(), clock_ms(), FLOOR_LEPTA, get_account(), HttpRpc (+12 more)

### Community 37 - "Module CMake Build Config"
Cohesion: 0.08
Nodes (26): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+18 more)

### Community 38 - "Event Bodies and Acknowledgements"
Cohesion: 0.10
Nodes (21): Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm, MarkDuplicate (+13 more)

### Community 39 - "Chain PDAs and Backups"
Cohesion: 0.19
Nodes (19): backup_pda(), check_version(), checkpoint_pda(), decode_backup(), decode_member(), decode_site(), decodes_the_real_site_account(), member() (+11 more)

### Community 40 - "Announce Signatures and Profiles"
Cohesion: 0.21
Nodes (22): a_full_profile_fits_the_inline_cap(), a_v2_file_still_reads_and_a_new_file_is_v3(), announce_signs_verifies_and_rejects_tampering(), build(), Envelope, envelope_round_trip_for_site_key_and_admin_wrap(), first_profile_is_read_by_the_admin_then_gets_a_site_wrap(), fixed_test_vector() (+14 more)

### Community 41 - "Node Body Text and Deadlines"
Cohesion: 0.13
Nodes (12): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), fingerprint(), HEADS_TO_SETTLE, kind_name(), past() (+4 more)

### Community 42 - "Checkpoint Fuzz and Tests"
Cohesion: 0.11
Nodes (8): checkpoint_signs_and_reproduces(), a_blob_is_kept_only_when_hash_and_signature_hold(), an_inline_profile_has_a_cap_and_a_bigger_file_is_not_a_blob(), announce(), blob_of(), every_message_kind_round_trips(), garbage_is_rejected(), size_limits_hold()

### Community 43 - "Meeting Accept and Cancel"
Cohesion: 0.08
Nodes (22): Act, Accept, Cancel, Decline, BACKUP_INFO, BACKUP_SCHEMA, BUSY_DOMAIN, DEFAULT_LEAD_MIN (+14 more)

### Community 44 - "Announce Lists and Answers"
Cohesion: 0.14
Nodes (3): account(), Fetched, Announce

### Community 45 - "Meeting Proposal Tests"
Cohesion: 0.24
Nodes (18): a_clash_is_named_and_does_not_block_the_proposal(), a_custom_reminder_lead_is_kept_zero_means_none_and_too_long_is_refused(), a_custom_time_and_length_is_checked_and_carries_its_own_start_and_end(), a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side(), a_description_is_checked_travels_in_the_proposal_and_shows_on_both_sides(), a_newer_summary_wins_and_an_old_one_is_ignored(), a_stranger_a_forgery_and_the_wrong_reader_change_nothing(), an_old_message_and_an_old_row_still_read_and_show_who_did_what() (+10 more)

### Community 46 - "Sync Delivery Limits"
Cohesion: 0.14
Nodes (18): ANSWERERS, BLOB_CHUNK, DELIVERY_CFG, heads_msg(), MAX_CHUNKS, MAX_FALLBACK, MAX_RESEND, MAX_WIRE_BYTES (+10 more)

### Community 47 - "Busy Body and Accept Messages"
Cohesion: 0.16
Nodes (11): Body, BusyBody, check_desc(), Kind, Accept, Cancel, Decline, Propose (+3 more)

### Community 48 - "Role Parsing and Codes"
Cohesion: 0.09
Nodes (21): parse_role(), role_code(), role_from_code(), TRIO, Role, Admin, Resident, Steward (+13 more)

### Community 49 - "Delivery Status and Flush"
Cohesion: 0.22
Nodes (9): delivery_status(), flush(), id_reply(), now_ms(), queued(), queued_reply(), read_with_queue(), send_queued() (+1 more)

### Community 50 - "Publish and Errors"
Cohesion: 0.14
Nodes (9): err(), now(), on_wire(), publish(), publish_with(), save(), start_site(), wire_chain() (+1 more)

### Community 51 - "UI Module Build Config"
Cohesion: 0.10
Nodes (20): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+12 more)

### Community 52 - "Common Test Helpers"
Cohesion: 0.23
Nodes (12): canonical(), comment(), copy_all(), equip(), gossip(), node(), roles_key(), second_device() (+4 more)

### Community 53 - "Meeting Job Checks"
Cohesion: 0.16
Nodes (9): check_lead_min(), check_len_ms(), key_arg(), meet(), meet_job(), meet_q(), meet_send(), mins_arg() (+1 more)

### Community 54 - "Intents and Write Actions"
Cohesion: 0.11
Nodes (15): Intent, Anchor, Approve, Backup, CreateSite, Details, Places, Publish (+7 more)

### Community 55 - "Logos Client Events"
Cohesion: 0.23
Nodes (3): Client, Events, LogosStore

### Community 56 - "Chat Calls and Invites"
Cohesion: 0.12
Nodes (12): CALL, EVERY, INVITE, MODULE, NO_ADDR, NOT_GROUP, OLD_GROUP, OPENING (+4 more)

### Community 57 - "Roster and Membership Lookup"
Cohesion: 0.20
Nodes (7): Roster, rank_name(), Rank, Admin, Resident, Steward, SuperAdmin

### Community 58 - "Announce and Blob Requests"
Cohesion: 0.12
Nodes (13): Wire, AnnounceReq, BlobChunk, BlobReq, Busy, ChainPending, ChatAddr, Event (+5 more)

### Community 60 - "Local Test Deploy Script"
Cohesion: 0.24
Nodes (16): deploy(), expect(), fail(), LEE_WALLET_HOME_DIR, mem(), neg(), new_account(), NOW() (+8 more)

### Community 61 - "Identity Encryption"
Cohesion: 0.14
Nodes (8): cipher(), export_identity(), MIN_PASSWORD, NONCE, PREFIX, SALT, VERSION, short_password_is_rejected()

### Community 62 - "Chain Event Validation Tests"
Cohesion: 0.22
Nodes (12): at_most_one_refresh_per_ten_seconds(), role_is_the_newest_known_at_any_block(), an_event_for_another_site_is_rejected(), equivocation_is_caught(), lamport_must_increase_along_a_chain(), out_of_order_events_wait_then_apply(), resend_is_capped(), branch() (+4 more)

### Community 63 - "Link State Listening"
Cohesion: 0.12
Nodes (8): keys_arg(), Link, Down, Listening, Open, Started, Starting, Shared

### Community 64 - "Checkpoint Signing"
Cohesion: 0.27
Nodes (9): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+1 more)

### Community 65 - "Event Format Tests"
Cohesion: 0.30
Nodes (12): new_key(), every_text_is_capped_on_receipt(), GOLDEN_REPORT, only_issue_events_and_checkpoints_are_bodies(), report(), round_trip(), size_limits(), tampered_byte_fails_signature() (+4 more)

### Community 66 - "Approval Queue"
Cohesion: 0.15
Nodes (13): ApproveReq, KEEP_DONE, LAND_WINDOW_MS, MAX_REJECTED, MAX_REQUESTS, PREP_WINDOW_MS, Prof, ProfReq (+5 more)

### Community 67 - "Chain Writer and Import"
Cohesion: 0.14
Nodes (6): ChainWriter, KeyState, Failed, Ready, Unknown, tick()

### Community 68 - "Join Request Checks"
Cohesion: 0.20
Nodes (7): built_request_parses_and_passes_the_rules(), check_decline(), DECLINE_DOMAIN, decline_message(), keys(), sign_decline(), sizes_are_162_and_290()

### Community 69 - "Testnet Scripts"
Cohesion: 0.29
Nodes (13): bal(), check(), die(), fin(), LEE_WALLET_HOME_DIR, log(), mem(), PUKAAR_PAYER (+5 more)

### Community 70 - "UI History and Board Features"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 71 - "UI People and Members Screen"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 72 - "Checkpoint Decoding and Settle"
Cohesion: 0.27
Nodes (5): decode_checkpoint(), body_of(), claimed(), Reads, send()

### Community 73 - "Anchor Registry Program"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 74 - "Basecamp Docs and Deployment"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

### Community 75 - "Install and Module Versions"
Cohesion: 0.21
Nodes (13): chat_module 0.3.0, delivery_module 0.4.0, Install Pukaar in Logos Basecamp, Known limits, Logos Basecamp, logos-repo.json (package source), Option 2: Install by hand, Option 1: Add a package source (+5 more)

### Community 76 - "Account Read and Cache"
Cohesion: 0.32
Nodes (10): b58(), program(), a_failed_read_keeps_the_old_cache(), a_promoted_or_revoked_author_is_judged_by_the_newest_role(), a_site_that_is_not_on_the_chain_is_an_error(), acct(), edit(), Fake (+2 more)

### Community 77 - "Meeting Ranges and Steps"
Cohesion: 0.33
Nodes (3): id_hex(), Meeting, Step

### Community 78 - "Store Branch Tests"
Cohesion: 0.33
Nodes (6): a_displaced_branch_and_its_descendants_become_evidence(), held_ids(), store_with(), the_alternatives_cap_keeps_the_lowest_ids_in_any_order(), the_lower_id_wins_whichever_arrives_first(), twins()

### Community 79 - "Chain Role Tests"
Cohesion: 0.33
Nodes (8): a_checkpoint_from_a_non_member_is_rejected_too(), a_resident_who_is_not_the_reporter_votes_and_staff_do_not(), an_author_who_was_not_a_member_at_the_chain_block_is_rejected_and_kept(), as_rank(), only_staff_acknowledge_claim_and_close(), report(), the_chain_block_decides_not_the_arrival_time(), why()

### Community 80 - "Convergence Tests"
Cohesion: 0.38
Nodes (5): a_forked_author_converges_to_one_branch_in_any_delivery_order(), any_delivery_order_gives_the_same_state(), random_action(), replay(), Rng

### Community 81 - "LGX Build Script"
Cohesion: 0.40
Nodes (8): busy(), fail(), heartbeat(), out(), run_module(), say(), build-lgx.sh script, status()

### Community 82 - "Write Kind Actions"
Cohesion: 0.20
Nodes (9): WriteKind, Anchor, Approve, CreateSite, Revoke, SetProfile, SetRole, SetSite (+1 more)

### Community 83 - "UI Board Columns"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 86 - "Decode Errors"
Cohesion: 0.22
Nodes (9): DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong, TooLarge (+1 more)

### Community 87 - "Queue Status States"
Cohesion: 0.22
Nodes (9): St, Done, Held, Landed, Preparing, Queued, Rejected, Sending (+1 more)

### Community 88 - "Review Tests"
Cohesion: 0.56
Nodes (7): acct(), an_admin_cannot_lock_a_member_out_with_a_huge_profile_version(), an_anchor_signer_key_must_be_the_key_of_the_member(), h(), item(), NOW, site()

### Community 89 - "CI Workflow"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 90 - "Place Edit Actions"
Cohesion: 0.36
Nodes (8): known_place(), Place, Add, Edit, Remove, Restore, Retire, reason_text()

### Community 92 - "Send Queue Retry"
Cohesion: 0.38
Nodes (6): lost_node(), send_queue(), SendError, Refused, Retry, a_message_delivery_did_not_answer_stays_queued_with_everything_after_it()

### Community 93 - "Local Anchor Script"
Cohesion: 0.38
Nodes (6): deploy(), LEE_WALLET_HOME_DIR, new_account(), PUKAAR_PAYER, PUKAAR_PROGRAM_ID, local-anchor.sh script

### Community 94 - "App Icons and Logos"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 95 - "Rust Crate Workspace"
Cohesion: 0.33
Nodes (6): pukaar_core, pukaar_logic, pukaar_logic-fuzz, pukaar_sites_guest, pukaar_sites_rules, pukaar_sites_runner

### Community 96 - "Storage Direct Script"
Cohesion: 0.53
Nodes (5): log_after(), pid_of(), refuse_if_running(), storage-direct.sh script, since()

### Community 97 - "Meeting Proposal Replies"
Cohesion: 0.40
Nodes (5): Did, Accepted, Cancelled, Declined, Proposed

### Community 98 - "Meeting Status States"
Cohesion: 0.40
Nodes (5): Status, Cancelled, Confirmed, Declined, Proposed

### Community 99 - "CMake Module Setup"
Cohesion: 0.50
Nodes (4): LogosModule.cmake, pukaar_core CMake module, metadata.json, Rust crate (rust-lib)

### Community 100 - "Super Admin Actions"
Cohesion: 0.50
Nodes (4): SuperKind, Add, Remove, Transfer

### Community 102 - "LEZ Local Script"
Cohesion: 0.50
Nodes (3): LEE_WALLET_HOME_DIR, LEZ_HOMES, lez-local.sh script

## Knowledge Gaps
- **502 isolated node(s):** `name`, `display_name`, `version`, `type`, `interface` (+497 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 802 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **27 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Node Fetch and Answers` to `Peer Discovery and Blob Sync`, `Checkpoints and Anchor Plans`, `Location Writes and Accounts`, `Sealed Box Crypto and Profiles`, `Logos Event Subscriptions`, `Identity Import and Export`, `Chat and Group Lists`, `Persistence Files and Events`, `Issue Reducer and Checkpoint Records`, `Place Hub Lifecycle`, `Lifecycle Stage Tests`, `Node RPC and Balances`, `Node Body Text and Deadlines`, `Announce Lists and Answers`, `Delivery Status and Flush`, `Publish and Errors`, `Meeting Job Checks`, `Chat Calls and Invites`, `Announce and Blob Requests`, `Identity Encryption`, `Link State Listening`, `Checkpoint Signing`, `Chain Role Tests`, `Convergence Tests`, `Action Body Parsing`, `Place Edit Actions`?**
  _High betweenness centrality (0.118) - this node is a cross-community bridge._
- **What connects `name`, `display_name`, `version` to the rest of the system?**
  _502 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Core Utilities and Types` be split into smaller, more focused modules?**
  _Cohesion score 0.0786034353995519 - nodes in this community are weakly interconnected._
- **Why does `ChatBook` connect `Chat and Conversations` to `Sync Delivery Limits`, `Logos Event Subscriptions`, `Chat and Group Lists`, `Chat Calls and Invites`, `Persistence Files and Events`, `Link State Listening`?**
  _High betweenness centrality (0.044) - this node is a cross-community bridge._
- **Should `Chat and Conversations` be split into smaller, more focused modules?**
  _Cohesion score 0.05901766647252961 - nodes in this community are weakly interconnected._
- **Why does `Queue` connect `Checkpoints and Anchor Plans` to `Core Utilities and Types`, `Membership Approvals and Roles`, `Approval Queue`, `Keyring and Effect Confirmation`, `Chain Writer and Import`, `Shared Queue Loader`, `Checkpoint Decoding and Settle`, `Blob Push and Save`, `Delivery Status and Flush`, `Logos Event Subscriptions`, `Meeting Job Checks`, `Place Edit Actions`, `Anchor Notes and Backup Writes`?**
  _High betweenness centrality (0.035) - this node is a cross-community bridge._
- **Should `Membership Approvals and Roles` be split into smaller, more focused modules?**
  _Cohesion score 0.0782472613458529 - nodes in this community are weakly interconnected._