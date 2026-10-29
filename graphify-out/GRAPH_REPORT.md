# Graph Report - field-study  (2026-10-29)

## Corpus Check
- 116 files · ~193,926 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 20 file(s) not represented in the graph (top: (none) 6, .mdc 4, .qml 3)

## Summary
- 2594 nodes · 6762 edges · 127 communities (99 shown, 28 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 486 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Core Concurrency Utilities
- Chat and Group Conversations
- Profile Sealing and Encryption
- Clock and Program Tests
- Discovery and Blob Transfer
- Member Status and Decline Rules
- Core Module Actions
- Records and Account Parsing
- Writes and Anchor Ordering
- Storage Layer
- Module Call Dispatch
- Effect and Balance Checks
- Borsh Serialization and Auth
- Key Derivation and LEE Accounts
- Anchor Rules and Writes
- Meetings and Proposals
- Agent Skills Index
- Role Checks and Actors
- Member Limits and Approval
- Time Duration Types
- Fingerprints and Answers
- Event Store Forks and Duplicates
- RPC and Account Fingerprints
- Meeting Validation and Backup
- Anchor Queue and Backup Writes
- Lifecycle Size Limits
- Intent and Approve Actions
- Chain Balance and Admin Keys
- Role Permission Table
- Account ID Encoding
- Chain Checkpoints and Backup PDAs
- Event Body Types
- Site Creation and Join
- Clock and LEE Core Plans
- Logos Error and Delivery
- Module Build Metadata
- Pukaar Place Lifecycle
- Identity Encryption (AEAD)
- Meeting Scheduling Tests
- Identity Import and Copy
- Issue JSON Export
- Role Parsing and Codes
- Anchor Effects and Approval
- Node Action Bodies
- Checkpoint Fetch and Fuzzing
- Sync Timing Constants
- Sync Limits and Chunks
- Fetch and Announce Collections
- Fetched Announce Lists
- Keyring and Hashing
- UI Module Build Metadata
- Backup Push and Grants
- Common Test Helpers
- Persistence and Site Join
- Fetch Cycle and Staff Check
- Wire Announce Messages
- Meeting Send and Answers
- Local Deploy Scripts
- Checkpoint Creation
- Anchor Planning and Push
- Chain Status and Anchoring
- Chain Event Ordering Tests
- Event Format Tests
- Meeting Clashes and Steps
- Link State Listening
- Meeting Proposals and Invites
- Queue and Approve Requests
- Core Test Store Events
- Testnet Script Helpers
- Board and History Screens
- People and Roles Screens
- Pukaar Registry Program
- Checkpoint Records and Delivery
- Basecamp Install and Limits
- Join Request Validation
- Serialization and SiteFile Paths
- Removed Member Handling
- Roster and Membership
- Meeting Answers Status
- Entry Confirm and Reject
- Chain Role Tests
- Convergence Tests
- Build Script Helpers
- Write Kind Actions
- Issue Status Lifecycle
- Kanban and Sidebar UI
- Action Parsing Tests
- Event Subscription and Delivery
- Signed Decode Errors
- Busy Signed Delivery
- Chain Writer and Keys
- Send State Machine
- Review and Admin Tests
- CI Workflow
- Meeting Body Kinds
- Send Queue Retry
- Local Anchor Deploy
- Meeting Status Values
- Meeting Backup Jobs
- Pukaar Logo Assets
- Workspace Crate Names
- Storage Direct Script
- Details Config Types
- Shared Queue Blobs
- CMake Module Build
- LEZ Local Script
- HTTP RPC Agent
- LEZ Testnet Script
- Build Script
- Identity Backup and Meetings
- Basecamp Script
- Core LGX Check Script
- Demo Script
- UI Screenshot Scripts
- Chain UI Screenshots
- Window Script
- Program Deployment Methods
- Program Deployment Programs
- Fastbuild Crate
- LEZ Client Gen
- Pukaar Registry Tool

## God Nodes (most connected - your core abstractions)
1. `Node` - 118 edges
2. `RuleError` - 78 edges
3. `Queue` - 66 edges
4. `Pukaar` - 59 edges
5. `ChatBook` - 58 edges
6. `PukaarCoreModule` - 57 edges
7. `Meetings` - 47 edges
8. `Rank` - 40 edges
9. `locked()` - 35 edges
10. `Intent` - 35 edges

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

## Communities (127 total, 28 thin omitted)

### Community 0 - "Core Concurrency Utilities"
Cohesion: 0.08
Nodes (63): derive_lez_secret(), Poll, Included, Pending, Rejected, a_backup_overwrites_the_pointer_and_is_done_when_the_chain_shows_the_new_cid(), a_backup_that_the_chain_holds_already_is_not_sent_and_waits_for_funds_like_any_write(), a_balance_that_drops_after_prepare_is_caught_at_the_send() (+55 more)

### Community 1 - "Chat and Group Conversations"
Cohesion: 0.06
Nodes (54): a_conversation_belongs_to_a_person_by_its_sender_and_the_file_keeps_everything(), a_group_is_listed_at_once_with_its_name_and_member_count_and_only_when_it_is_ours(), a_group_message_shows_who_sent_it_by_the_address_book_and_a_short_key_when_unknown(), a_group_that_someone_made_is_ours_once_a_member_we_know_is_in_it(), a_linked_conversation_without_a_message_is_not_in_the_chat_list_until_one_is_queued(), a_member_is_added_once_and_goes_in_only_when_their_address_is_known_and_the_group_is_live(), a_member_who_restarted_is_added_again_by_the_lowest_member_only(), a_new_conversation_takes_no_message_until_the_other_side_has_had_time_to_join() (+46 more)

### Community 2 - "Profile Sealing and Encryption"
Cohesion: 0.08
Nodes (64): seal(), sealed_box_round_trip_wrong_key_and_libsodium(), x25519_public(), a_full_profile_fits_the_inline_cap(), a_v2_file_still_reads_and_a_new_file_is_v3(), About, add_seats(), add_site_wrap() (+56 more)

### Community 3 - "Clock and Program Tests"
Cohesion: 0.12
Nodes (43): A, a_backup_has_negatives(), a_backup_overwrites_the_same_record_of_the_member(), a_call_from_another_program_is_refused(), a_stale_or_future_clock_claim_fails(), a_wrong_member_pda_is_refused(), anchor_needs_an_active_member(), approve_batch_negatives() (+35 more)

### Community 4 - "Discovery and Blob Transfer"
Cohesion: 0.12
Nodes (38): a_blob_that_does_not_match_the_chain_hash_is_not_used_and_is_fetched_again(), a_file_of_100_kb_comes_in_chunks_and_reassembles(), a_flood_of_requests_gets_few_answers(), a_join_request_and_an_announce_name_accounts_to_read(), a_keyring_entry_that_lies_about_its_key_is_ignored(), a_lying_author_account_is_rejected(), a_node_that_was_offline_gets_the_announces_it_missed_and_only_valid_newer_ones(), a_removal_reason_comes_from_details_and_must_match_its_hash() (+30 more)

### Community 5 - "Member Status and Decline Rules"
Cohesion: 0.09
Nodes (49): a_forged_or_unsolicited_decline_is_ignored(), a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download(), a_join_request_is_signed_twice_and_kept(), a_member_page_has_the_profile_the_role_the_joining_time_and_the_pointer_state(), a_member_page_says_loading_none_or_removed(), a_member_queues_a_backup_pointer_and_a_stranger_does_not(), a_new_site_has_its_files_and_the_first_profile_behind_it(), a_private_profile_shows_other_members_the_name_and_role_only() (+41 more)

### Community 7 - "Records and Account Parsing"
Cohesion: 0.06
Nodes (41): built_request_parses_and_passes_the_rules(), parse(), sha(), ACCOUNT_ID_PREFIX, Attestation, BACKUP, BackupPointer, check_join_request() (+33 more)

### Community 8 - "Writes and Anchor Ordering"
Cohesion: 0.10
Nodes (43): Location, accounts_follow_the_runner_order(), anchor(), apply_place_op(), approve_batch(), approve_batch_names_the_requester_and_the_claims(), auth(), BATCH (+35 more)

### Community 9 - "Storage Layer"
Cohesion: 0.08
Nodes (28): base(), CALL, CHUNK, Client, config_gets_own_data_dir_and_free_ports(), env_override_wins_and_bad_json_is_an_error(), Events, keep_file() (+20 more)

### Community 10 - "Module Call Dispatch"
Cohesion: 0.11
Nodes (42): list_of(), the_module_lists_read_with_float_times_and_either_wrapping(), active(), CALL, Chat, Group, chat_of(), Person (+34 more)

### Community 11 - "Effect and Balance Checks"
Cohesion: 0.05
Nodes (44): apply(), apply_effect(), check_balance(), check_clock(), RuleError, AlreadyAnchored, AlreadyMember, AlreadySuperAdmin (+36 more)

### Community 12 - "Borsh Serialization and Auth"
Cohesion: 0.07
Nodes (38): by_hand(), set_pointer(), Instruction, Anchor, ApproveBatch, CreateSite, Probe, Reserved (+30 more)

### Community 13 - "Key Derivation and LEE Accounts"
Cohesion: 0.14
Nodes (18): clock_ms(), Cx, hex32(), id_env(), join_request(), join_request_passes_the_rules_and_both_signatures_verify(), Keys, main() (+10 more)

### Community 14 - "Anchor Rules and Writes"
Cohesion: 0.08
Nodes (33): anchor_writes_once(), apply_anchor(), check_anchor(), check_ed25519(), check_probe(), checkpoint_seed(), CheckpointRecord, CLOCK_TOLERANCE_MS (+25 more)

### Community 15 - "Meetings and Proposals"
Cohesion: 0.08
Nodes (36): a_bad_group_proposal_is_refused(), Act, Accept, Cancel, Decline, an_invitee_who_accepted_can_leave_and_only_their_part_ends(), BACKUP_INFO, BACKUP_SCHEMA (+28 more)

### Community 16 - "Agent Skills Index"
Cohesion: 0.06
Nodes (34): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+26 more)

### Community 17 - "Role Checks and Actors"
Cohesion: 0.19
Nodes (34): check_cid(), active_rank(), Actor, allow(), apply_approve(), apply_approve_site(), apply_create(), apply_revoke() (+26 more)

### Community 18 - "Member Limits and Approval"
Cohesion: 0.21
Nodes (19): a_removed_member_cannot_act_until_approved_again(), a_super_admin_adds_one_more_member_and_no_third(), acct(), adds_over_the_limit_keep_the_first(), claim(), mutual_removal_keeps_the_first(), non_admins_cannot_approve(), only_a_super_admin_grants_or_changes_the_admin_role() (+11 more)

### Community 19 - "Time Duration Types"
Cohesion: 0.09
Nodes (12): an_unreachable_node_is_an_error_not_a_hang(), GAS_LIMIT, imported_key_gives_the_account_of_lezkey(), POLL, seed_key(), SEND, SEND_RETRY_WAIT, SEND_TRIES (+4 more)

### Community 20 - "Fingerprints and Answers"
Cohesion: 0.10
Nodes (7): fingerprint(), Node, last_anchor(), on_decline(), pending_rows(), refresh_items(), status()

### Community 21 - "Event Store Forks and Duplicates"
Cohesion: 0.14
Nodes (11): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+3 more)

### Community 22 - "RPC and Account Fingerprints"
Cohesion: 0.12
Nodes (25): b58(), program(), a_failed_read_keeps_the_old_cache(), a_node_reads_roles_lists_and_identity_from_the_cache(), a_promoted_or_revoked_author_is_judged_by_the_newest_role(), a_site_that_is_not_on_the_chain_is_an_error(), acct(), CLAIM_MARGIN_MS (+17 more)

### Community 23 - "Meeting Validation and Backup"
Cohesion: 0.16
Nodes (5): check_desc(), check_len_ms(), Meetings, .PREFIX, Peer

### Community 24 - "Anchor Queue and Backup Writes"
Cohesion: 0.12
Nodes (3): AnchorNote, Queue, tick()

### Community 25 - "Lifecycle Size Limits"
Cohesion: 0.12
Nodes (22): stage(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline(), claim_reopen_claim_confirm(), finished_cards_carry_no_pending_next_step(), forbidden_transitions_are_inert_and_visible() (+14 more)

### Community 26 - "Intent and Approve Actions"
Cohesion: 0.09
Nodes (20): body_of(), claimed(), Intent, Anchor, Approve, Backup, CreateSite, Details (+12 more)

### Community 27 - "Chain Balance and Admin Keys"
Cohesion: 0.15
Nodes (6): balance(), Chain, NATIVE_PROGRAM, now_ms(), people_lists_have_the_shape_of_main_qml(), PROGRAM

### Community 28 - "Role Permission Table"
Cohesion: 0.12
Nodes (23): A, approve_item_table(), ASHA, check_join_request_table(), create_args(), create_makes_the_creator_super_admin_and_first_member(), create_table(), D (+15 more)

### Community 29 - "Account ID Encoding"
Cohesion: 0.11
Nodes (19): build(), tampering_and_wrong_keys_are_refused(), ACCOUNT_B58, ACCOUNT_ID_PREFIX, h(), INFO, lez_account_id(), lez_key_and_account_id_match_the_wallet() (+11 more)

### Community 30 - "Chain Checkpoints and Backup PDAs"
Cohesion: 0.17
Nodes (20): backup_pda(), check_version(), checkpoint_pda(), decode_backup(), decode_checkpoint(), decode_member(), decode_site(), decodes_the_real_site_account() (+12 more)

### Community 31 - "Event Body Types"
Cohesion: 0.10
Nodes (21): Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm, MarkDuplicate (+13 more)

### Community 32 - "Site Creation and Join"
Cohesion: 0.12
Nodes (23): about_text(), create_site(), join(), join_again(), known_place(), my_pending(), new_site(), NewSite (+15 more)

### Community 33 - "Clock and LEE Core Plans"
Cohesion: 0.18
Nodes (7): Ctx, Ctx<'a>, nonempty(), pda(), plan(), probe(), super_admin()

### Community 34 - "Logos Error and Delivery"
Cohesion: 0.14
Nodes (15): advance(), create_and_listen(), delivered(), delivered_once(), err(), flush(), now(), on_wire() (+7 more)

### Community 35 - "Module Build Metadata"
Cohesion: 0.08
Nodes (26): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+18 more)

### Community 36 - "Pukaar Place Lifecycle"
Cohesion: 0.11
Nodes (4): later(), place(), Pukaar, read()

### Community 37 - "Identity Encryption (AEAD)"
Cohesion: 0.10
Nodes (16): cipher(), export_identity(), import_identity(), MIN_PASSWORD, NONCE, PREFIX, SALT, VERSION (+8 more)

### Community 38 - "Meeting Scheduling Tests"
Cohesion: 0.21
Nodes (20): a_clash_is_named_and_does_not_block_the_proposal(), a_custom_reminder_lead_is_kept_zero_means_none_and_too_long_is_refused(), a_custom_time_and_length_is_checked_and_carries_its_own_start_and_end(), a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side(), a_description_is_checked_travels_in_the_proposal_and_shows_on_both_sides(), a_group_meeting_shows_each_answer_and_confirms_by_the_rule(), a_newer_summary_wins_and_an_old_one_is_ignored(), a_row_from_before_groups_reads_as_a_meeting_with_one_invitee() (+12 more)

### Community 39 - "Identity Import and Copy"
Cohesion: 0.16
Nodes (18): import_into(), copy_all(), a_failed_import_writes_nothing(), a_peer_naming_a_later_seq_keeps_us_waiting(), a_restored_node_refuses_to_publish_until_its_history_is_back(), heads(), import_into_a_fresh_device_gives_an_empty_node_with_the_same_identity(), import_is_refused_when_a_site_exists() (+10 more)

### Community 40 - "Issue JSON Export"
Cohesion: 0.16
Nodes (12): issue_json(), apply(), CheckpointRecord, Config, Issue, issue_of(), not_permitted(), Progress (+4 more)

### Community 41 - "Role Parsing and Codes"
Cohesion: 0.09
Nodes (23): parse_role(), role_code(), role_from_code(), TRIO, Role, Admin, Resident, Steward (+15 more)

### Community 42 - "Anchor Effects and Approval"
Cohesion: 0.08
Nodes (23): Effect, Anchor, AnchorSigner, Approve, ApproveSite, BackupSigner, BalanceAtLeast, ClockIs (+15 more)

### Community 43 - "Node Action Bodies"
Cohesion: 0.13
Nodes (11): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), HEADS_TO_SETTLE, kind_name(), past(), Restore (+3 more)

### Community 44 - "Checkpoint Fetch and Fuzzing"
Cohesion: 0.11
Nodes (8): checkpoint_signs_and_reproduces(), a_blob_is_kept_only_when_hash_and_signature_hold(), an_inline_profile_has_a_cap_and_a_bigger_file_is_not_a_blob(), announce(), blob_of(), every_message_kind_round_trips(), garbage_is_rejected(), size_limits_hold()

### Community 45 - "Sync Timing Constants"
Cohesion: 0.10
Nodes (17): ANNOUNCE_REQ_EVERY, ANSWER_EVERY, chain_tick(), DELIVERY_TIMEOUT, env_or(), HEADS_EVERY, LEZ_RPC, LISTENING (+9 more)

### Community 46 - "Sync Limits and Chunks"
Cohesion: 0.14
Nodes (18): ANSWERERS, BLOB_CHUNK, DELIVERY_CFG, heads_msg(), MAX_CHUNKS, MAX_FALLBACK, MAX_RESEND, MAX_WIRE_BYTES (+10 more)

### Community 47 - "Fetch and Announce Collections"
Cohesion: 0.10
Nodes (18): about_of(), ANNOUNCE_EVERY, clean(), clean_about(), DETAILS_SCHEMA, MAX_ABOUT, MAX_ANNOUNCE_ANSWERS, MAX_ANNOUNCES (+10 more)

### Community 49 - "Keyring and Hashing"
Cohesion: 0.21
Nodes (12): sha256(), build(), Env, fetch_blob(), need_site(), open_ring(), prepare(), prepare_approve() (+4 more)

### Community 50 - "UI Module Build Metadata"
Cohesion: 0.10
Nodes (20): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+12 more)

### Community 51 - "Backup Push and Grants"
Cohesion: 0.26
Nodes (13): parse_id(), backup_push(), drop_held(), grant(), push_staff_name(), reject(), revoke(), send_batch() (+5 more)

### Community 52 - "Common Test Helpers"
Cohesion: 0.23
Nodes (11): canonical(), comment(), equip(), gossip(), node(), roles_key(), second_device(), Site (+3 more)

### Community 53 - "Persistence and Site Join"
Cohesion: 0.22
Nodes (13): account_b58(), join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), remove_if_present() (+5 more)

### Community 54 - "Fetch Cycle and Staff Check"
Cohesion: 0.26
Nodes (8): account(), cycle(), fetch_cycle(), is_staff(), read_profile(), Store, sync_once(), takes_announce()

### Community 55 - "Wire Announce Messages"
Cohesion: 0.12
Nodes (13): Wire, AnnounceReq, BlobChunk, BlobReq, Busy, ChainPending, ChatAddr, Event (+5 more)

### Community 56 - "Meeting Send and Answers"
Cohesion: 0.16
Nodes (7): key_arg(), keys_arg(), meet(), meet_answer(), meet_send(), mins_arg(), ms_arg()

### Community 57 - "Local Deploy Scripts"
Cohesion: 0.24
Nodes (16): deploy(), expect(), fail(), LEE_WALLET_HOME_DIR, mem(), neg(), new_account(), NOW() (+8 more)

### Community 58 - "Checkpoint Creation"
Cohesion: 0.24
Nodes (9): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+1 more)

### Community 59 - "Anchor Planning and Push"
Cohesion: 0.23
Nodes (13): anchor(), anchor_plan(), anchor_push(), anchor_read(), AnchorPlan, BackupSource, restore_source(), claim_ms() (+5 more)

### Community 60 - "Chain Status and Anchoring"
Cohesion: 0.29
Nodes (4): id_reply(), now_ms(), queued(), queued_reply()

### Community 61 - "Chain Event Ordering Tests"
Cohesion: 0.24
Nodes (10): at_most_one_refresh_per_ten_seconds(), role_is_the_newest_known_at_any_block(), lamport_must_increase_along_a_chain(), out_of_order_events_wait_then_apply(), resend_is_capped(), branch(), chain(), event() (+2 more)

### Community 62 - "Event Format Tests"
Cohesion: 0.30
Nodes (12): new_key(), every_text_is_capped_on_receipt(), GOLDEN_REPORT, only_issue_events_and_checkpoints_are_bodies(), report(), round_trip(), size_limits(), tampered_byte_fails_signature() (+4 more)

### Community 63 - "Meeting Clashes and Steps"
Cohesion: 0.30
Nodes (3): Clashes, Meeting, Step

### Community 64 - "Link State Listening"
Cohesion: 0.14
Nodes (8): Link, Down, Listening, Open, Started, Starting, save(), Shared

### Community 65 - "Meeting Proposals and Invites"
Cohesion: 0.23
Nodes (3): distinct(), Prop, settle()

### Community 66 - "Queue and Approve Requests"
Cohesion: 0.16
Nodes (12): ApproveReq, KEEP_DONE, LAND_WINDOW_MS, MAX_REJECTED, MAX_REQUESTS, PREP_WINDOW_MS, Prof, ProfReq (+4 more)

### Community 67 - "Core Test Store Events"
Cohesion: 0.26
Nodes (8): a_displaced_branch_and_its_descendants_become_evidence(), an_event_for_another_site_is_rejected(), equivocation_is_caught(), held_ids(), store_with(), the_alternatives_cap_keeps_the_lowest_ids_in_any_order(), the_lower_id_wins_whichever_arrives_first(), twins()

### Community 68 - "Testnet Script Helpers"
Cohesion: 0.29
Nodes (13): bal(), check(), die(), fin(), LEE_WALLET_HOME_DIR, log(), mem(), PUKAAR_PAYER (+5 more)

### Community 69 - "Board and History Screens"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 70 - "People and Roles Screens"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 71 - "Pukaar Registry Program"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 72 - "Checkpoint Records and Delivery"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

### Community 73 - "Basecamp Install and Limits"
Cohesion: 0.21
Nodes (13): chat_module 0.3.0, delivery_module 0.4.0, Install Pukaar in Logos Basecamp, Known limits, Logos Basecamp, logos-repo.json (package source), Option 2: Install by hand, Option 1: Add a package source (+5 more)

### Community 74 - "Join Request Validation"
Cohesion: 0.23
Nodes (6): check_decline(), DECLINE_DOMAIN, decline_message(), keys(), sign_decline(), sizes_are_162_and_290()

### Community 75 - "Serialization and SiteFile Paths"
Cohesion: 0.27
Nodes (3): path_of(), SiteFile, .PREFIX

### Community 76 - "Removed Member Handling"
Cohesion: 0.18
Nodes (8): remember_known(), a_removed_member_gets_a_reason_and_no_storage_error(), a_removed_member_stays_in_the_removed_list_after_the_keyring_rotates(), Offline, removed(), removed_keys(), rotate_keyring(), the_queue_brings_back_a_removed_member_that_no_keyring_lists()

### Community 78 - "Meeting Answers Status"
Cohesion: 0.20
Nodes (10): Answer, Accepted, Declined, Left, Pending, Did, Accepted, Cancelled (+2 more)

### Community 79 - "Entry Confirm and Reject"
Cohesion: 0.42
Nodes (6): confirm(), effect_on_chain(), Entry, send(), step(), watch()

### Community 80 - "Chain Role Tests"
Cohesion: 0.33
Nodes (8): a_checkpoint_from_a_non_member_is_rejected_too(), a_resident_who_is_not_the_reporter_votes_and_staff_do_not(), an_author_who_was_not_a_member_at_the_chain_block_is_rejected_and_kept(), as_rank(), only_staff_acknowledge_claim_and_close(), report(), the_chain_block_decides_not_the_arrival_time(), why()

### Community 81 - "Convergence Tests"
Cohesion: 0.38
Nodes (5): a_forked_author_converges_to_one_branch_in_any_delivery_order(), any_delivery_order_gives_the_same_state(), random_action(), replay(), Rng

### Community 82 - "Build Script Helpers"
Cohesion: 0.40
Nodes (8): busy(), fail(), heartbeat(), out(), run_module(), say(), build-lgx.sh script, status()

### Community 83 - "Write Kind Actions"
Cohesion: 0.20
Nodes (9): WriteKind, Anchor, Approve, CreateSite, Revoke, SetProfile, SetRole, SetSite (+1 more)

### Community 84 - "Issue Status Lifecycle"
Cohesion: 0.20
Nodes (8): Status, Acknowledged, AwaitingConfirmation, ClosedWontfix, ConfirmedResolved, Duplicate, InProgress, Open

### Community 85 - "Kanban and Sidebar UI"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 88 - "Event Subscription and Delivery"
Cohesion: 0.22
Nodes (4): delivery_status(), listen(), read_with_queue(), write()

### Community 89 - "Signed Decode Errors"
Cohesion: 0.22
Nodes (9): DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong, TooLarge (+1 more)

### Community 90 - "Busy Signed Delivery"
Cohesion: 0.33
Nodes (3): BusyBody, Signed, Signed<T>

### Community 91 - "Chain Writer and Keys"
Cohesion: 0.25
Nodes (5): ChainWriter, KeyState, Failed, Ready, Unknown

### Community 92 - "Send State Machine"
Cohesion: 0.22
Nodes (9): St, Done, Held, Landed, Preparing, Queued, Rejected, Sending (+1 more)

### Community 93 - "Review and Admin Tests"
Cohesion: 0.56
Nodes (7): acct(), an_admin_cannot_lock_a_member_out_with_a_huge_profile_version(), an_anchor_signer_key_must_be_the_key_of_the_member(), h(), item(), NOW, site()

### Community 94 - "CI Workflow"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 95 - "Meeting Body Kinds"
Cohesion: 0.25
Nodes (8): Body, Kind, Accept, Cancel, Decline, Propose, ProposeGroup, ProposeNote

### Community 96 - "Send Queue Retry"
Cohesion: 0.38
Nodes (6): lost_node(), send_queue(), SendError, Refused, Retry, a_message_delivery_did_not_answer_stays_queued_with_everything_after_it()

### Community 97 - "Local Anchor Deploy"
Cohesion: 0.38
Nodes (6): deploy(), LEE_WALLET_HOME_DIR, new_account(), PUKAAR_PAYER, PUKAAR_PROGRAM_ID, local-anchor.sh script

### Community 98 - "Meeting Status Values"
Cohesion: 0.33
Nodes (6): Status, Cancelled, Confirmed, Declined, status_of(), Proposed

### Community 100 - "Pukaar Logo Assets"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 101 - "Workspace Crate Names"
Cohesion: 0.33
Nodes (6): pukaar_core, pukaar_logic, pukaar_logic-fuzz, pukaar_sites_guest, pukaar_sites_rules, pukaar_sites_runner

### Community 102 - "Storage Direct Script"
Cohesion: 0.53
Nodes (5): log_after(), pid_of(), refuse_if_running(), storage-direct.sh script, since()

### Community 105 - "CMake Module Build"
Cohesion: 0.50
Nodes (4): LogosModule.cmake, pukaar_core CMake module, metadata.json, Rust crate (rust-lib)

### Community 106 - "LEZ Local Script"
Cohesion: 0.50
Nodes (3): LEE_WALLET_HOME_DIR, LEZ_HOMES, lez-local.sh script

## Knowledge Gaps
- **508 isolated node(s):** `name`, `display_name`, `version`, `type`, `interface` (+503 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 808 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **28 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Fingerprints and Answers` to `Profile Sealing and Encryption`, `Discovery and Blob Transfer`, `Writes and Anchor Ordering`, `Module Call Dispatch`, `RPC and Account Fingerprints`, `Lifecycle Size Limits`, `Site Creation and Join`, `Logos Error and Delivery`, `Pukaar Place Lifecycle`, `Identity Encryption (AEAD)`, `Identity Import and Copy`, `Issue JSON Export`, `Node Action Bodies`, `Sync Timing Constants`, `Backup Push and Grants`, `Persistence and Site Join`, `Wire Announce Messages`, `Meeting Send and Answers`, `Checkpoint Creation`, `Anchor Planning and Push`, `Chain Status and Anchoring`, `Link State Listening`, `Removed Member Handling`, `Chain Role Tests`, `Convergence Tests`, `Action Parsing Tests`, `Event Subscription and Delivery`, `Meeting Backup Jobs`?**
  _High betweenness centrality (0.098) - this node is a cross-community bridge._
- **What connects `name`, `display_name`, `version` to the rest of the system?**
  _508 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Core Concurrency Utilities` be split into smaller, more focused modules?**
  _Cohesion score 0.07823002240477969 - nodes in this community are weakly interconnected._
- **Why does `Queue` connect `Anchor Queue and Backup Writes` to `Site Creation and Join`, `Core Concurrency Utilities`, `Queue and Approve Requests`, `Meeting Backup Jobs`, `Member Status and Decline Rules`, `Shared Queue Blobs`, `Chain Writer and Keys`, `Removed Member Handling`, `Sync Timing Constants`, `Entry Confirm and Reject`, `Backup Push and Grants`, `Fingerprints and Answers`, `Event Subscription and Delivery`, `Intent and Approve Actions`, `Anchor Planning and Push`, `Chain Status and Anchoring`?**
  _High betweenness centrality (0.051) - this node is a cross-community bridge._
- **Should `Chat and Group Conversations` be split into smaller, more focused modules?**
  _Cohesion score 0.06030619865571322 - nodes in this community are weakly interconnected._
- **Why does `PukaarCoreModule` connect `Core Module Actions` to `Pukaar Place Lifecycle`, `Sync Timing Constants`, `Module Context Hooks`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **Should `Profile Sealing and Encryption` be split into smaller, more focused modules?**
  _Cohesion score 0.07848944835246205 - nodes in this community are weakly interconnected._