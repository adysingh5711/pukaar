# Graph Report - field-study  (2026-10-28)

## Corpus Check
- 109 files · ~179,174 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 20 file(s) not represented in the graph (top: (none) 6, .mdc 4, .qml 3)

## Summary
- 2329 nodes · 5998 edges · 110 communities (84 shown, 26 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 457 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Rust Core Tests
- Meet Scheduling Logic
- Lezkey Sealing Crypto
- Instruction Member Status Tests
- Write Operations Review
- Fetch Account Reads
- Chain Clock Program Tests
- Discovery Sync Tests
- Storage Sync Primitives
- Core Module Lifecycle
- Agent Skills Docs
- Join Request Records
- Rules Error Formatting
- Roles Approval Logic
- Ops Queue Checkpoints
- Wire Instruction Layer
- Event Body Types
- Identity Accept Flow
- Node Issue Reducer
- Rules Member Tests
- Rules Account Tests
- Store Branch Acceptance
- Rules Program Core
- Core Lifecycle Entry
- RPC Client Agent
- Node Lifecycle Staging
- Wallet Config
- Rules Effects Apply
- Core Plan Clock
- Main Runner Entry
- Metadata Build Config
- Node Sync Announce
- Lezkey Derivation
- Node Answer Replies
- Metadata Module Config
- Delivery Status Flush
- Module Test Helpers
- Sync Announce Limits
- Queue Intent Build
- Queue Intent Anchor
- Identity Encryption
- Checkpoint Signing
- Chain Writer Queue
- Local Test Script
- Image Board Screens
- Image People Screens
- Queue Body Claims
- Queue Entry Live
- Chain Writes Queue
- Rules Queue Approval
- Queue Anchor Profile
- Link Listener Core
- Icons Metadata Assets
- Store Fork Tests
- RPC Chain Cache
- Chain Roster Admins
- Chain Membership Sync
- Sync Heads Resend
- Chain Roles Tests
- Convergence Fork Tests
- Build LGX Script
- Gate Config Records
- Roles Action Anchor
- Action Body Helpers
- Lifecycle Stage Tests
- Storage Direct Script
- Ops Known Place
- Superkind Ops
- Queue Blob Build
- Wire Pointer Records
- Localnet Script
- Testnet Script
- Sites Fastbuild Tool
- Rules Lib Core
- Registry Build Script
- Sites Build Script
- Basecamp Script
- Lgx Check Script
- Demo Script
- UI Shots Script
- UI Shots Chain Script
- Window Script
- Example Deployment Crate
- Example Programs Crate
- Fastbuild Crate
- Lez Client Gen Crate
- Registry Crate
- Registry Methods Lib
- Board Architecture Images
- Checkpoint Storage Diagram
- Logos Module CMake
- Sites Rules Core Libs
- Logo Brand Assets
- Pukaar Core Crate
- Storage Direct Helpers
- Sites Program Anchor
- Logos Module Lib
- Pukaar Core Assets
- Pukaar Core Cmake
- Core Logic Lib
- Registry Methods Build
- Sites Guest Build
- Basecamp Launch Helper
- Core LGX Check Script
- Community 107
- Community 108

## God Nodes (most connected - your core abstractions)
1. `Node` - 114 edges
2. `RuleError` - 78 edges
3. `Queue` - 66 edges
4. `Pukaar` - 51 edges
5. `PukaarCoreModule` - 49 edges
6. `Meetings` - 44 edges
7. `Rank` - 41 edges
8. `locked()` - 35 edges
9. `Instruction` - 35 edges
10. `Actor` - 35 edges

## Surprising Connections (you probably didn't know these)
- `parse()` --calls--> `request_id()`  [INFERRED]
  modules/pukaar_core/rust-lib/logic/src/joinreq.rs → programs/pukaar_sites/rules/src/records.rs
- `join_request_passes_the_rules_and_both_signatures_verify()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `main()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `run()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `user()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Rust logic crate quality checks** — github_workflows_ci_logic_job, github_workflows_ci_fuzz_smoke_job, github_workflows_ci_rust_logic_crate [EXTRACTED 1.00]
- **Admin role change via notice and seal** — readme_super_admin, readme_admin_notice, readme_notice_seal [INFERRED 0.75]
- **Checkpoint anchoring chain** — readme_pukaar_core, readme_checkpoint_merkle_root, readme_pukaar_registry, readme_lez [INFERRED 0.75]
- **Tamper-evidence via signed events and fork proofs** — readme_signed_event_envelope, readme_fork_proof, readme_fork_resolution_lowest_id [INFERRED 0.75]
- **Pukaar bell logo variants** — modules_pukaar_ui_src_icons_pukaar_app_icon, public_brand_pukaar_logo_color_logo, public_brand_pukaar_logo_color_svg_logo, public_brand_pukaar_logo_logo, public_brand_pukaar_logo_svg_logo [INFERRED 0.85]

## Communities (110 total, 26 thin omitted)

### Community 0 - "Rust Core Tests"
Cohesion: 0.08
Nodes (63): sha256(), Poll, Included, Pending, Rejected, a_backup_overwrites_the_pointer_and_is_done_when_the_chain_shows_the_new_cid(), a_backup_that_the_chain_holds_already_is_not_sent_and_waits_for_funds_like_any_write(), a_balance_that_drops_after_prepare_is_caught_at_the_send() (+55 more)

### Community 1 - "Meet Scheduling Logic"
Cohesion: 0.06
Nodes (50): a_clash_is_named_and_does_not_block_the_proposal(), a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side(), a_newer_summary_wins_and_an_old_one_is_ignored(), a_stranger_a_forgery_and_the_wrong_reader_change_nothing(), Act, Accept, Cancel, Decline (+42 more)

### Community 2 - "Lezkey Sealing Crypto"
Cohesion: 0.08
Nodes (64): seal(), sealed_box_round_trip_wrong_key_and_libsodium(), x25519_public(), a_full_profile_fits_the_inline_cap(), a_v2_file_still_reads_and_a_new_file_is_v3(), About, add_seats(), add_site_wrap() (+56 more)

### Community 3 - "Instruction Member Status Tests"
Cohesion: 0.08
Nodes (57): a_forged_or_unsolicited_decline_is_ignored(), a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download(), a_join_request_is_signed_twice_and_kept(), a_member_page_has_the_profile_the_role_the_joining_time_and_the_pointer_state(), a_member_page_says_loading_none_or_removed(), a_member_queues_a_backup_pointer_and_a_stranger_does_not(), a_new_site_has_its_files_and_the_first_profile_behind_it(), a_private_profile_shows_other_members_the_name_and_role_only() (+49 more)

### Community 4 - "Write Operations Review"
Cohesion: 0.07
Nodes (58): Location, accounts_follow_the_runner_order(), anchor(), apply_place_op(), approve_batch(), approve_batch_names_the_requester_and_the_claims(), auth(), BATCH (+50 more)

### Community 5 - "Fetch Account Reads"
Cohesion: 0.06
Nodes (29): about_of(), account(), ANNOUNCE_EVERY, clean(), clean_about(), cycle(), Details, DETAILS_SCHEMA (+21 more)

### Community 6 - "Chain Clock Program Tests"
Cohesion: 0.12
Nodes (43): A, a_backup_has_negatives(), a_backup_overwrites_the_same_record_of_the_member(), a_call_from_another_program_is_refused(), a_stale_or_future_clock_claim_fails(), a_wrong_member_pda_is_refused(), anchor_needs_an_active_member(), approve_batch_negatives() (+35 more)

### Community 7 - "Discovery Sync Tests"
Cohesion: 0.12
Nodes (38): a_blob_that_does_not_match_the_chain_hash_is_not_used_and_is_fetched_again(), a_file_of_100_kb_comes_in_chunks_and_reassembles(), a_flood_of_requests_gets_few_answers(), a_join_request_and_an_announce_name_accounts_to_read(), a_keyring_entry_that_lies_about_its_key_is_ignored(), a_lying_author_account_is_rejected(), a_node_that_was_offline_gets_the_announces_it_missed_and_only_valid_newer_ones(), a_removal_reason_comes_from_details_and_must_match_its_hash() (+30 more)

### Community 8 - "Storage Sync Primitives"
Cohesion: 0.08
Nodes (28): base(), CALL, CHUNK, Client, config_gets_own_data_dir_and_free_ports(), env_override_wins_and_bad_json_is_an_error(), Events, keep_file() (+20 more)

### Community 10 - "Agent Skills Docs"
Cohesion: 0.05
Nodes (41): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+33 more)

### Community 11 - "Join Request Records"
Cohesion: 0.07
Nodes (32): parse(), sha(), ACCOUNT_ID_PREFIX, Attestation, BACKUP, BackupPointer, check_join_request(), CID_MAX (+24 more)

### Community 12 - "Rules Error Formatting"
Cohesion: 0.05
Nodes (40): RuleError, AlreadyAnchored, AlreadyMember, AlreadySuperAdmin, BadBalance, BadCid, BadJoinRequest, BadMerkle (+32 more)

### Community 13 - "Roles Approval Logic"
Cohesion: 0.19
Nodes (34): check_cid(), Approve, active_rank(), Actor, allow(), apply_approve(), apply_approve_site(), apply_create() (+26 more)

### Community 14 - "Ops Queue Checkpoints"
Cohesion: 0.11
Nodes (24): about_text(), anchor(), anchor_plan(), anchor_push(), anchor_read(), AnchorPlan, BackupSource, create_site() (+16 more)

### Community 15 - "Wire Instruction Layer"
Cohesion: 0.08
Nodes (33): by_hand(), Instruction, Anchor, ApproveBatch, CreateSite, Probe, Reserved, RevokeBatch (+25 more)

### Community 16 - "Event Body Types"
Cohesion: 0.07
Nodes (30): Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm, MarkDuplicate (+22 more)

### Community 17 - "Identity Accept Flow"
Cohesion: 0.12
Nodes (25): import_identity(), import_into(), a_failed_import_writes_nothing(), a_peer_naming_a_later_seq_keeps_us_waiting(), a_restored_node_refuses_to_publish_until_its_history_is_back(), an_existing_v1_backup_still_imports(), blob(), export_then_import_restores_key_and_site() (+17 more)

### Community 18 - "Node Issue Reducer"
Cohesion: 0.11
Nodes (20): issue_json(), apply(), CheckpointRecord, Config, Issue, issue_of(), not_permitted(), Progress (+12 more)

### Community 19 - "Rules Member Tests"
Cohesion: 0.17
Nodes (25): A, a_removed_member_cannot_act_until_approved_again(), a_super_admin_adds_one_more_member_and_no_third(), adds_over_the_limit_keep_the_first(), ASHA, claim(), D, MEERA (+17 more)

### Community 20 - "Rules Account Tests"
Cohesion: 0.16
Nodes (17): acct(), approve_item_table(), check_join_request_table(), create_args(), create_makes_the_creator_super_admin_and_first_member(), create_table(), effective_status_table(), h() (+9 more)

### Community 21 - "Store Branch Acceptance"
Cohesion: 0.14
Nodes (11): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+3 more)

### Community 22 - "Rules Program Core"
Cohesion: 0.09
Nodes (27): check_ed25519(), check_probe(), checkpoint_seed(), CLOCK_TOLERANCE_MS, CP_SEED_PREFIX, ED_PK, ED_SIG, member_record() (+19 more)

### Community 23 - "Core Lifecycle Entry"
Cohesion: 0.09
Nodes (23): advance(), ANNOUNCE_REQ_EVERY, ANSWER_EVERY, chain_tick(), create_and_listen(), delivered(), delivered_once(), DELIVERY_TIMEOUT (+15 more)

### Community 24 - "RPC Client Agent"
Cohesion: 0.10
Nodes (23): a_node_reads_roles_lists_and_identity_from_the_cache(), balance(), CLAIM_MARGIN_MS, claim_ms(), clock_ms(), FLOOR_LEPTA, get_account(), HttpRpc (+15 more)

### Community 25 - "Node Lifecycle Staging"
Cohesion: 0.12
Nodes (22): stage(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline(), claim_reopen_claim_confirm(), finished_cards_carry_no_pending_next_step(), forbidden_transitions_are_inert_and_visible() (+14 more)

### Community 26 - "Wallet Config"
Cohesion: 0.12
Nodes (8): an_unreachable_node_is_an_error_not_a_hang(), GAS_LIMIT, imported_key_gives_the_account_of_lezkey(), POLL, seed_key(), SEND, temp(), WalletWriter

### Community 27 - "Rules Effects Apply"
Cohesion: 0.07
Nodes (27): apply(), apply_effect(), check_balance(), check_clock(), Effect, Anchor, AnchorSigner, Approve (+19 more)

### Community 28 - "Core Plan Clock"
Cohesion: 0.17
Nodes (8): Ctx, Ctx<'a>, nonempty(), pda(), plan(), probe(), set_pointer(), super_admin()

### Community 29 - "Main Runner Entry"
Cohesion: 0.17
Nodes (20): backup_pda(), check_version(), checkpoint_pda(), decode_backup(), decode_checkpoint(), decode_member(), decode_site(), decodes_the_real_site_account() (+12 more)

### Community 30 - "Metadata Build Config"
Cohesion: 0.17
Nodes (4): Chain, now_ms(), people_lists_have_the_shape_of_main_qml(), refresh_fills_the_roster()

### Community 31 - "Node Sync Announce"
Cohesion: 0.08
Nodes (26): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+18 more)

### Community 32 - "Lezkey Derivation"
Cohesion: 0.18
Nodes (16): clock_ms(), hex32(), id_env(), join_request(), join_request_passes_the_rules_and_both_signatures_verify(), Keys, main(), opt() (+8 more)

### Community 33 - "Node Answer Replies"
Cohesion: 0.13
Nodes (12): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), fingerprint(), HEADS_TO_SETTLE, kind_name(), past() (+4 more)

### Community 34 - "Metadata Module Config"
Cohesion: 0.08
Nodes (21): Wire, AnnounceReq, BlobChunk, BlobReq, Busy, ChainPending, Event, JoinDecline (+13 more)

### Community 35 - "Delivery Status Flush"
Cohesion: 0.13
Nodes (5): later(), meet_answer(), place(), Pukaar, read()

### Community 36 - "Module Test Helpers"
Cohesion: 0.11
Nodes (8): checkpoint_signs_and_reproduces(), a_blob_is_kept_only_when_hash_and_signature_hold(), an_inline_profile_has_a_cap_and_a_bigger_file_is_not_a_blob(), announce(), blob_of(), every_message_kind_round_trips(), garbage_is_rejected(), size_limits_hold()

### Community 37 - "Sync Announce Limits"
Cohesion: 0.13
Nodes (15): ACCOUNT_B58, ACCOUNT_ID_PREFIX, h(), INFO, lez_key_and_account_id_match_the_wallet(), lez_public_key(), LEZ_SK, open() (+7 more)

### Community 39 - "Queue Intent Anchor"
Cohesion: 0.19
Nodes (18): parse_id(), backup_push(), drop_held(), grant(), push_staff_name(), reject(), revoke(), send_batch() (+10 more)

### Community 40 - "Identity Encryption"
Cohesion: 0.14
Nodes (9): err(), now(), on_wire(), publish(), publish_with(), save(), start_site(), wire_chain() (+1 more)

### Community 41 - "Checkpoint Signing"
Cohesion: 0.23
Nodes (8): delivery_status(), flush(), id_reply(), now_ms(), queued(), queued_reply(), read_with_queue(), write_tick()

### Community 42 - "Chain Writer Queue"
Cohesion: 0.10
Nodes (20): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+12 more)

### Community 43 - "Local Test Script"
Cohesion: 0.23
Nodes (12): canonical(), comment(), copy_all(), equip(), gossip(), node(), roles_key(), second_device() (+4 more)

### Community 44 - "Image Board Screens"
Cohesion: 0.13
Nodes (15): ANSWERERS, BLOB_CHUNK, DELIVERY_CFG, lost_node(), MAX_CHUNKS, MAX_FALLBACK, MAX_RESEND, MAX_WIRE_BYTES (+7 more)

### Community 45 - "Image People Screens"
Cohesion: 0.25
Nodes (11): build(), Env, fetch_blob(), need_site(), open_ring(), prepare(), prepare_approve(), prepare_publish() (+3 more)

### Community 46 - "Queue Body Claims"
Cohesion: 0.16
Nodes (7): key_arg(), listen(), meet(), meet_job(), meet_q(), meet_send(), ms_arg()

### Community 47 - "Queue Entry Live"
Cohesion: 0.22
Nodes (13): account_b58(), join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), remove_if_present() (+5 more)

### Community 48 - "Chain Writes Queue"
Cohesion: 0.11
Nodes (15): Intent, Anchor, Approve, Backup, CreateSite, Details, Places, Publish (+7 more)

### Community 49 - "Rules Queue Approval"
Cohesion: 0.13
Nodes (8): cipher(), export_identity(), MIN_PASSWORD, NONCE, PREFIX, SALT, VERSION, short_password_is_rejected()

### Community 50 - "Queue Anchor Profile"
Cohesion: 0.22
Nodes (9): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+1 more)

### Community 51 - "Link Listener Core"
Cohesion: 0.20
Nodes (10): ChainWriter, confirm(), effect_on_chain(), KeyState, Failed, Ready, Unknown, send() (+2 more)

### Community 52 - "Icons Metadata Assets"
Cohesion: 0.24
Nodes (16): deploy(), expect(), fail(), LEE_WALLET_HOME_DIR, mem(), neg(), new_account(), NOW() (+8 more)

### Community 53 - "Store Fork Tests"
Cohesion: 0.22
Nodes (12): at_most_one_refresh_per_ten_seconds(), role_is_the_newest_known_at_any_block(), an_event_for_another_site_is_rejected(), equivocation_is_caught(), lamport_must_increase_along_a_chain(), out_of_order_events_wait_then_apply(), resend_is_capped(), branch() (+4 more)

### Community 54 - "RPC Chain Cache"
Cohesion: 0.23
Nodes (10): build(), built_request_parses_and_passes_the_rules(), check_decline(), DECLINE_DOMAIN, decline_message(), keys(), sign_decline(), sizes_are_162_and_290() (+2 more)

### Community 55 - "Chain Roster Admins"
Cohesion: 0.30
Nodes (12): new_key(), every_text_is_capped_on_receipt(), GOLDEN_REPORT, only_issue_events_and_checkpoints_are_bodies(), report(), round_trip(), size_limits(), tampered_byte_fails_signature() (+4 more)

### Community 56 - "Chain Membership Sync"
Cohesion: 0.21
Nodes (6): body_of(), claimed(), Reads, RevokeReq, SHORT_OF_FUNDS, request_id()

### Community 59 - "Convergence Fork Tests"
Cohesion: 0.29
Nodes (13): bal(), check(), die(), fin(), LEE_WALLET_HOME_DIR, log(), mem(), PUKAAR_PAYER (+5 more)

### Community 60 - "Build LGX Script"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 61 - "Gate Config Records"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 62 - "Roles Action Anchor"
Cohesion: 0.17
Nodes (11): ApproveReq, KEEP_DONE, LAND_WINDOW_MS, MAX_REJECTED, MAX_REQUESTS, PREP_WINDOW_MS, Prof, REQUEST_VALID_MS (+3 more)

### Community 64 - "Lifecycle Stage Tests"
Cohesion: 0.15
Nodes (7): Link, Down, Listening, Open, Started, Starting, Shared

### Community 65 - "Storage Direct Script"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 66 - "Ops Known Place"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

### Community 67 - "Superkind Ops"
Cohesion: 0.33
Nodes (6): a_displaced_branch_and_its_descendants_become_evidence(), held_ids(), store_with(), the_alternatives_cap_keeps_the_lowest_ids_in_any_order(), the_lower_id_wins_whichever_arrives_first(), twins()

### Community 68 - "Queue Blob Build"
Cohesion: 0.36
Nodes (10): b58(), program(), a_failed_read_keeps_the_old_cache(), a_promoted_or_revoked_author_is_judged_by_the_newest_role(), a_site_that_is_not_on_the_chain_is_an_error(), acct(), edit(), Fake (+2 more)

### Community 70 - "Localnet Script"
Cohesion: 0.35
Nodes (9): heads_msg(), resend(), to_resend(), to_resend_own(), Heads, a_behind_store_catches_up_from_heads(), a_join_published_while_delivery_was_down_still_reaches_the_admin(), a_peer_on_the_losing_branch_gets_the_winner_even_when_ahead() (+1 more)

### Community 71 - "Testnet Script"
Cohesion: 0.33
Nodes (8): a_checkpoint_from_a_non_member_is_rejected_too(), a_resident_who_is_not_the_reporter_votes_and_staff_do_not(), an_author_who_was_not_a_member_at_the_chain_block_is_rejected_and_kept(), as_rank(), only_staff_acknowledge_claim_and_close(), report(), the_chain_block_decides_not_the_arrival_time(), why()

### Community 72 - "Sites Fastbuild Tool"
Cohesion: 0.38
Nodes (5): a_forked_author_converges_to_one_branch_in_any_delivery_order(), any_delivery_order_gives_the_same_state(), random_action(), replay(), Rng

### Community 73 - "Rules Lib Core"
Cohesion: 0.40
Nodes (8): busy(), fail(), heartbeat(), out(), run_module(), say(), build-lgx.sh script, status()

### Community 74 - "Registry Build Script"
Cohesion: 0.22
Nodes (7): GateConfig, GateMode, Attested, OnChain, NftGate, Site, TokenGate

### Community 75 - "Sites Build Script"
Cohesion: 0.20
Nodes (10): Action, Anchor, Revoke, SetProfile, SetRole, SetSite, SuperAdminAdd, SuperAdminRemove (+2 more)

### Community 76 - "Basecamp Script"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 78 - "Demo Script"
Cohesion: 0.22
Nodes (9): St, Done, Held, Landed, Preparing, Queued, Rejected, Sending (+1 more)

### Community 79 - "UI Shots Script"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 80 - "UI Shots Chain Script"
Cohesion: 0.36
Nodes (8): known_place(), Place, Add, Edit, Remove, Restore, Retire, reason_text()

### Community 81 - "Window Script"
Cohesion: 0.39
Nodes (6): anchor_writes_once(), apply_anchor(), check_anchor(), CheckpointRecord, rec(), record_round_trip_is_168_bytes()

### Community 82 - "Example Deployment Crate"
Cohesion: 0.29
Nodes (3): lez_account_id(), new_site(), the_real_testnet_site_has_the_expected_roles()

### Community 83 - "Example Programs Crate"
Cohesion: 0.38
Nodes (6): deploy(), LEE_WALLET_HOME_DIR, new_account(), PUKAAR_PAYER, PUKAAR_PROGRAM_ID, local-anchor.sh script

### Community 84 - "Fastbuild Crate"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 85 - "Lez Client Gen Crate"
Cohesion: 0.33
Nodes (6): pukaar_core, pukaar_logic, pukaar_logic-fuzz, pukaar_sites_guest, pukaar_sites_rules, pukaar_sites_runner

### Community 86 - "Registry Crate"
Cohesion: 0.53
Nodes (5): log_after(), pid_of(), refuse_if_running(), storage-direct.sh script, since()

### Community 87 - "Registry Methods Lib"
Cohesion: 0.50
Nodes (4): LogosModule.cmake, pukaar_core CMake module, metadata.json, Rust crate (rust-lib)

### Community 88 - "Board Architecture Images"
Cohesion: 0.50
Nodes (4): SuperKind, Add, Remove, Transfer

### Community 91 - "Sites Rules Core Libs"
Cohesion: 0.50
Nodes (4): Pointer, Details, Keyring, Places

### Community 92 - "Logo Brand Assets"
Cohesion: 0.50
Nodes (3): LEE_WALLET_HOME_DIR, LEZ_HOMES, lez-local.sh script

## Knowledge Gaps
- **462 isolated node(s):** `name`, `display_name`, `version`, `type`, `interface` (+457 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 728 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **26 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Queue Intent Build` to `Lezkey Sealing Crypto`, `Write Operations Review`, `Discovery Sync Tests`, `Ops Queue Checkpoints`, `Identity Accept Flow`, `Node Issue Reducer`, `Core Lifecycle Entry`, `RPC Client Agent`, `Node Lifecycle Staging`, `Node Answer Replies`, `Metadata Module Config`, `Delivery Status Flush`, `Queue Intent Anchor`, `Identity Encryption`, `Checkpoint Signing`, `Queue Body Claims`, `Queue Entry Live`, `Rules Queue Approval`, `Queue Anchor Profile`, `Lifecycle Stage Tests`, `Testnet Script`, `Sites Fastbuild Tool`, `Lgx Check Script`, `UI Shots Chain Script`?**
  _High betweenness centrality (0.117) - this node is a cross-community bridge._
- **What connects `name`, `display_name`, `version` to the rest of the system?**
  _462 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Rust Core Tests` be split into smaller, more focused modules?**
  _Cohesion score 0.0799543118218161 - nodes in this community are weakly interconnected._
- **Why does `Rank` connect `Queue Intent Anchor` to `Write Operations Review`, `Wire Pointer Records`, `Queue Intent Build`, `Testnet Script`, `Sites Fastbuild Tool`, `Chain Clock Program Tests`, `Local Test Script`, `Sites Build Script`, `Roles Approval Logic`, `Wire Instruction Layer`, `Node Issue Reducer`, `Rules Member Tests`, `Rules Account Tests`, `Chain Membership Sync`, `RPC Client Agent`, `Metadata Build Config`?**
  _High betweenness centrality (0.060) - this node is a cross-community bridge._
- **Should `Meet Scheduling Logic` be split into smaller, more focused modules?**
  _Cohesion score 0.05504950495049505 - nodes in this community are weakly interconnected._
- **Why does `Effect` connect `Rules Effects Apply` to `Write Operations Review`, `Roles Approval Logic`, `Window Script`, `Rules Program Core`, `Sites Rules Core Libs`?**
  _High betweenness centrality (0.037) - this node is a cross-community bridge._
- **Should `Lezkey Sealing Crypto` be split into smaller, more focused modules?**
  _Cohesion score 0.07508771929824562 - nodes in this community are weakly interconnected._