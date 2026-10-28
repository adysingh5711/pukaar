# Graph Report - field-study  (2026-10-28)

## Corpus Check
- 116 files · ~193,378 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 20 file(s) not represented in the graph (top: (none) 6, .mdc 4, .qml 3)

## Summary
- 2590 nodes · 6735 edges · 122 communities (93 shown, 29 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 485 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Agent Core Logic
- Atomic Concurrency Primitives
- Chat Conversations Tests
- Sealed Box Crypto
- Clock and Lee Core
- Role Rules Tests
- Account Ids and Testnet
- Calendar and Writes
- Clock Program Tests
- Pukaar Core Module
- Atomic Store and Locks
- Accept Actions and Nodes
- Logos SDK Chat Glue
- Effect Apply and Balances
- Lee Private Keys
- Checkpoint Ops
- Meeting Conflicts Tests
- Membership Instructions
- Agent Skills Index
- Role and Actor Checks
- Store BTreeMap and Duplicates
- Identity Import
- Grant and Pending Rows
- Config and Durations
- Serde Deserialize and IO
- Meeting Time Checks
- Sync Join Limits
- Lifecycle and Events
- Registry and Balance Rules
- Location and Writes Fixtures
- Place and Known Places
- Clock and Refresh Limits
- Build and Fixture Config
- Event Blob and Fixes
- Canonical Comment Equip
- Wire Announce and Addr
- Lez Crypto Hashing
- Join Request Checks
- Queue and Approve Requests
- Acct Fetch Cycle
- Announce and Membership Wire
- Anchor Note Users
- Key Args and Meet
- Local Deploy Scripts
- Argon2 AEAD Keys
- Join Request Attestation
- Keyring and Entry Context
- Ident Reply and Anchor
- Checkpoint Records
- Event Format Tests
- Clash and Meeting Steps
- Link Down Listening
- Distinct Invites
- Testnet Script Checks
- Board and Issues UI
- People Screen UI
- Chat Module Install Docs
- Checkpoint Records Decode
- Answer Accepted Declined
- Live and Terminal Status
- Chain Roles Tests
- Convergence Tests
- Build LGX Script
- Action Kind Enum
- Issue Status Enum
- Action Anchor Revoke
- Attestation Body Checks
- Common Test Helpers
- Local Anchor Script
- Cancelled Status Enum
- Meeting Job Queue
- Pukaar App Icons
- Crate Names
- Storage Direct Scripts
- Details Config Reasons
- Module CMake Build
- LEE Client Gen
- Fastbuild Main
- Backup Pointer Codec
- Member Status Enum
- Lez Testnet Script
- Module Context Glue
- Build Script Entry
- Build Shell Script
- LGX Check Script
- Demo Script
- UI Shots Scripts
- Window Script
- Program Deployment Example
- Program Deployment Programs
- Fastbuild Crate
- LEE Client Gen Crate
- Registry Crate
- Pukaar Methods Lib
- Identity Backup UI
- Basecamp Script
- Chain Shots Script
- Chat Event Subscription
- Body Kind Accept Cancel
- Busy Body Delivery
- Member Record Seeds
- Review Actor Checks
- Delivery Status Enum
- Logos Error Advance
- Common Module
- Body Acknowledge Checkpoint
- Board Kanban Columns UI
- Anchored Integrity UI
- Sidebar Navigation UI
- Effect Anchor Types
- History Filters UI
- Common Action Parsing
- Pukaar Icon Assets
- Mini Shell Helpers
- Community 119
- Community 120

## God Nodes (most connected - your core abstractions)
1. `Node` - 118 edges
2. `RuleError` - 78 edges
3. `Queue` - 66 edges
4. `Pukaar` - 59 edges
5. `ChatBook` - 57 edges
6. `PukaarCoreModule` - 57 edges
7. `Meetings` - 47 edges
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

## Communities (122 total, 29 thin omitted)

### Community 0 - "Agent Core Logic"
Cohesion: 0.05
Nodes (58): b58(), backup_pda(), check_version(), checkpoint_pda(), decode_backup(), decode_member(), decode_site(), decodes_the_real_site_account() (+50 more)

### Community 1 - "Atomic Concurrency Primitives"
Cohesion: 0.08
Nodes (63): sha256(), Poll, Included, Pending, Rejected, a_backup_overwrites_the_pointer_and_is_done_when_the_chain_shows_the_new_cid(), a_backup_that_the_chain_holds_already_is_not_sent_and_waits_for_funds_like_any_write(), a_balance_that_drops_after_prepare_is_caught_at_the_send() (+55 more)

### Community 2 - "Chat Conversations Tests"
Cohesion: 0.06
Nodes (55): a_conversation_belongs_to_a_person_by_its_sender_and_the_file_keeps_everything(), a_group_is_listed_at_once_with_its_name_and_member_count_and_only_when_it_is_ours(), a_group_message_shows_who_sent_it_by_the_address_book_and_a_short_key_when_unknown(), a_group_that_someone_made_is_ours_once_a_member_we_know_is_in_it(), a_linked_conversation_without_a_message_is_not_in_the_chat_list_until_one_is_queued(), a_member_is_added_once_and_goes_in_only_when_their_address_is_known_and_the_group_is_live(), a_new_conversation_takes_no_message_until_the_other_side_has_had_time_to_join(), a_thread_is_unread_until_its_newest_live_message_is_marked_read_and_that_survives_a_restart() (+47 more)

### Community 3 - "Sealed Box Crypto"
Cohesion: 0.07
Nodes (64): seal(), sealed_box_round_trip_wrong_key_and_libsodium(), x25519_public(), a_full_profile_fits_the_inline_cap(), a_v2_file_still_reads_and_a_new_file_is_v3(), add_seats(), add_site_wrap(), Announce (+56 more)

### Community 4 - "Clock and Lee Core"
Cohesion: 0.05
Nodes (45): by_hand(), Ctx, Ctx<'a>, nonempty(), pda(), plan(), probe(), set_pointer() (+37 more)

### Community 5 - "Role Rules Tests"
Cohesion: 0.11
Nodes (41): A, a_removed_member_cannot_act_until_approved_again(), a_super_admin_adds_one_more_member_and_no_third(), acct(), adds_over_the_limit_keep_the_first(), approve_item_table(), ASHA, check_join_request_table() (+33 more)

### Community 6 - "Account Ids and Testnet"
Cohesion: 0.11
Nodes (40): derive_lez_secret(), the_real_testnet_site_has_the_expected_roles(), a_blob_that_does_not_match_the_chain_hash_is_not_used_and_is_fetched_again(), a_file_of_100_kb_comes_in_chunks_and_reassembles(), a_flood_of_requests_gets_few_answers(), a_join_request_and_an_announce_name_accounts_to_read(), a_keyring_entry_that_lies_about_its_key_is_ignored(), a_lying_author_account_is_rejected() (+32 more)

### Community 7 - "Calendar and Writes"
Cohesion: 0.08
Nodes (51): Location, accounts_follow_the_runner_order(), anchor(), apply_place_op(), approve_batch(), approve_batch_names_the_requester_and_the_claims(), auth(), BATCH (+43 more)

### Community 8 - "Clock Program Tests"
Cohesion: 0.12
Nodes (43): A, a_backup_has_negatives(), a_backup_overwrites_the_same_record_of_the_member(), a_call_from_another_program_is_refused(), a_stale_or_future_clock_claim_fails(), a_wrong_member_pda_is_refused(), anchor_needs_an_active_member(), approve_batch_negatives() (+35 more)

### Community 10 - "Atomic Store and Locks"
Cohesion: 0.08
Nodes (28): base(), CALL, CHUNK, Client, config_gets_own_data_dir_and_free_ports(), env_override_wins_and_bad_json_is_an_error(), Events, keep_file() (+20 more)

### Community 11 - "Accept Actions and Nodes"
Cohesion: 0.08
Nodes (14): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), fingerprint(), HEADS_TO_SETTLE, kind_name(), Node (+6 more)

### Community 12 - "Logos SDK Chat Glue"
Cohesion: 0.11
Nodes (39): active(), CALL, Chat, Group, chat_of(), Person, conversations(), DRAIN (+31 more)

### Community 13 - "Effect Apply and Balances"
Cohesion: 0.05
Nodes (44): apply(), apply_effect(), check_balance(), check_clock(), RuleError, AlreadyAnchored, AlreadyMember, AlreadySuperAdmin (+36 more)

### Community 14 - "Lee Private Keys"
Cohesion: 0.15
Nodes (18): clock_ms(), Cx, hex32(), id_env(), join_request(), join_request_passes_the_rules_and_both_signatures_verify(), Keys, main() (+10 more)

### Community 15 - "Checkpoint Ops"
Cohesion: 0.10
Nodes (25): about_text(), anchor(), anchor_plan(), anchor_push(), anchor_read(), AnchorPlan, BackupSource, create_site() (+17 more)

### Community 16 - "Meeting Conflicts Tests"
Cohesion: 0.08
Nodes (36): a_bad_group_proposal_is_refused(), Act, Accept, Cancel, Decline, an_invitee_who_accepted_can_leave_and_only_their_part_ends(), BACKUP_INFO, BACKUP_SCHEMA (+28 more)

### Community 17 - "Membership Instructions"
Cohesion: 0.10
Nodes (25): remember_known(), a_member_page_has_the_profile_the_role_the_joining_time_and_the_pointer_state(), a_member_page_says_loading_none_or_removed(), a_private_profile_shows_other_members_the_name_and_role_only(), a_removed_member_stays_in_the_removed_list_after_the_keyring_rotates(), a_restore_takes_the_cid_from_the_chain_and_a_typed_cid_only_as_a_fallback(), a_spent_request_is_not_waiting_so_a_removed_member_sees_the_removal(), a_staff_name_counts_only_with_the_signature_of_an_admin() (+17 more)

### Community 18 - "Agent Skills Index"
Cohesion: 0.06
Nodes (34): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+26 more)

### Community 19 - "Role and Actor Checks"
Cohesion: 0.19
Nodes (34): check_cid(), active_rank(), Actor, allow(), apply_approve(), apply_approve_site(), apply_create(), apply_revoke() (+26 more)

### Community 20 - "Store BTreeMap and Duplicates"
Cohesion: 0.13
Nodes (11): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+3 more)

### Community 21 - "Identity Import"
Cohesion: 0.12
Nodes (26): import_identity(), import_into(), copy_all(), a_failed_import_writes_nothing(), a_peer_naming_a_later_seq_keeps_us_waiting(), a_restored_node_refuses_to_publish_until_its_history_is_back(), an_existing_v1_backup_still_imports(), blob() (+18 more)

### Community 22 - "Grant and Pending Rows"
Cohesion: 0.13
Nodes (32): grant(), last_anchor(), pending_rows(), refresh_items(), send_batch(), status(), a_forged_or_unsolicited_decline_is_ignored(), a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download() (+24 more)

### Community 23 - "Config and Durations"
Cohesion: 0.11
Nodes (10): an_unreachable_node_is_an_error_not_a_hang(), GAS_LIMIT, imported_key_gives_the_account_of_lezkey(), POLL, seed_key(), SEND, SEND_RETRY_WAIT, SEND_TRIES (+2 more)

### Community 24 - "Serde Deserialize and IO"
Cohesion: 0.12
Nodes (16): account_b58(), join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), path_of() (+8 more)

### Community 25 - "Meeting Time Checks"
Cohesion: 0.16
Nodes (5): check_desc(), check_len_ms(), Meetings, .PREFIX, Peer

### Community 26 - "Sync Join Limits"
Cohesion: 0.11
Nodes (24): ANSWERERS, BLOB_CHUNK, DELIVERY_CFG, heads_msg(), lost_node(), MAX_CHUNKS, MAX_FALLBACK, MAX_RESEND (+16 more)

### Community 27 - "Lifecycle and Events"
Cohesion: 0.12
Nodes (22): stage(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline(), claim_reopen_claim_confirm(), finished_cards_carry_no_pending_next_step(), forbidden_transitions_are_inert_and_visible() (+14 more)

### Community 28 - "Registry and Balance Rules"
Cohesion: 0.10
Nodes (24): check_ed25519(), check_probe(), checkpoint_seed(), CLOCK_TOLERANCE_MS, CP_SEED_PREFIX, ED_PK, ED_SIG, merkle_fold() (+16 more)

### Community 29 - "Location and Writes Fixtures"
Cohesion: 0.11
Nodes (24): backup_push(), known_place(), Place, Add, Edit, Remove, Restore, Retire (+16 more)

### Community 30 - "Place and Known Places"
Cohesion: 0.15
Nodes (18): at_most_one_refresh_per_ten_seconds(), role_is_the_newest_known_at_any_block(), a_displaced_branch_and_its_descendants_become_evidence(), an_event_for_another_site_is_rejected(), equivocation_is_caught(), held_ids(), lamport_must_increase_along_a_chain(), out_of_order_events_wait_then_apply() (+10 more)

### Community 31 - "Clock and Refresh Limits"
Cohesion: 0.14
Nodes (15): advance(), create_and_listen(), delivered(), delivered_once(), err(), flush(), now(), on_wire() (+7 more)

### Community 32 - "Build and Fixture Config"
Cohesion: 0.08
Nodes (26): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+18 more)

### Community 33 - "Event Blob and Fixes"
Cohesion: 0.10
Nodes (21): Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm, MarkDuplicate (+13 more)

### Community 34 - "Canonical Comment Equip"
Cohesion: 0.11
Nodes (4): later(), place(), Pukaar, read()

### Community 35 - "Wire Announce and Addr"
Cohesion: 0.13
Nodes (18): build(), tampering_and_wrong_keys_are_refused(), ACCOUNT_B58, ACCOUNT_ID_PREFIX, h(), INFO, lez_account_id(), lez_key_and_account_id_match_the_wallet() (+10 more)

### Community 36 - "Lez Crypto Hashing"
Cohesion: 0.21
Nodes (20): a_clash_is_named_and_does_not_block_the_proposal(), a_custom_reminder_lead_is_kept_zero_means_none_and_too_long_is_refused(), a_custom_time_and_length_is_checked_and_carries_its_own_start_and_end(), a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side(), a_description_is_checked_travels_in_the_proposal_and_shows_on_both_sides(), a_group_meeting_shows_each_answer_and_confirms_by_the_rule(), a_newer_summary_wins_and_an_old_one_is_ignored(), a_row_from_before_groups_reads_as_a_meeting_with_one_invitee() (+12 more)

### Community 37 - "Join Request Checks"
Cohesion: 0.16
Nodes (12): issue_json(), apply(), CheckpointRecord, Config, Issue, issue_of(), not_permitted(), Progress (+4 more)

### Community 38 - "Queue and Approve Requests"
Cohesion: 0.08
Nodes (23): Effect, Anchor, AnchorSigner, Approve, ApproveSite, BackupSigner, BalanceAtLeast, ClockIs (+15 more)

### Community 39 - "Acct Fetch Cycle"
Cohesion: 0.11
Nodes (8): checkpoint_signs_and_reproduces(), a_blob_is_kept_only_when_hash_and_signature_hold(), an_inline_profile_has_a_cap_and_a_bigger_file_is_not_a_blob(), announce(), blob_of(), every_message_kind_round_trips(), garbage_is_rejected(), size_limits_hold()

### Community 40 - "Announce and Membership Wire"
Cohesion: 0.18
Nodes (16): canonical(), comment(), equip(), gossip(), node(), roles_key(), second_device(), Site (+8 more)

### Community 41 - "Anchor Note Users"
Cohesion: 0.10
Nodes (17): ANNOUNCE_REQ_EVERY, ANSWER_EVERY, chain_tick(), DELIVERY_TIMEOUT, env_or(), HEADS_EVERY, LEZ_RPC, LISTENING (+9 more)

### Community 42 - "Key Args and Meet"
Cohesion: 0.10
Nodes (18): about_of(), ANNOUNCE_EVERY, clean(), clean_about(), DETAILS_SCHEMA, MAX_ABOUT, MAX_ANNOUNCE_ANSWERS, MAX_ANNOUNCES (+10 more)

### Community 43 - "Local Deploy Scripts"
Cohesion: 0.16
Nodes (11): ChainWriter, confirm(), effect_on_chain(), Env, KeyState, Failed, Ready, Unknown (+3 more)

### Community 44 - "Argon2 AEAD Keys"
Cohesion: 0.11
Nodes (17): ACCOUNT_ID_PREFIX, CID_MAX, GATE_NATIVE, GATE_NFT, GATE_TOKEN, GateConfig, GateMode, Attested (+9 more)

### Community 46 - "Keyring and Entry Context"
Cohesion: 0.10
Nodes (20): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+12 more)

### Community 47 - "Ident Reply and Anchor"
Cohesion: 0.15
Nodes (11): Attestation, check_join_request(), JoinBody, lez_account_id(), Member, Notice, NoticeOutcome, Remove (+3 more)

### Community 48 - "Checkpoint Records"
Cohesion: 0.14
Nodes (15): ApproveReq, claimed(), KEEP_DONE, LAND_WINDOW_MS, MAX_REJECTED, MAX_REQUESTS, PREP_WINDOW_MS, Prof (+7 more)

### Community 49 - "Event Format Tests"
Cohesion: 0.11
Nodes (15): Intent, Anchor, Approve, Backup, CreateSite, Details, Places, Publish (+7 more)

### Community 50 - "Clash and Meeting Steps"
Cohesion: 0.26
Nodes (8): account(), cycle(), fetch_cycle(), is_staff(), read_profile(), Store, sync_once(), takes_announce()

### Community 51 - "Link Down Listening"
Cohesion: 0.12
Nodes (13): Wire, AnnounceReq, BlobChunk, BlobReq, Busy, ChainPending, ChatAddr, Event (+5 more)

### Community 53 - "Testnet Script Checks"
Cohesion: 0.16
Nodes (7): key_arg(), keys_arg(), meet(), meet_answer(), meet_send(), mins_arg(), ms_arg()

### Community 54 - "Board and Issues UI"
Cohesion: 0.24
Nodes (16): deploy(), expect(), fail(), LEE_WALLET_HOME_DIR, mem(), neg(), new_account(), NOW() (+8 more)

### Community 55 - "People Screen UI"
Cohesion: 0.14
Nodes (8): cipher(), export_identity(), MIN_PASSWORD, NONCE, PREFIX, SALT, VERSION, short_password_is_rejected()

### Community 56 - "Chat Module Install Docs"
Cohesion: 0.17
Nodes (9): built_request_parses_and_passes_the_rules(), check_decline(), DECLINE_DOMAIN, decline_message(), keys(), parse(), sign_decline(), sizes_are_162_and_290() (+1 more)

### Community 57 - "Checkpoint Records Decode"
Cohesion: 0.31
Nodes (10): build(), Entry, fetch_blob(), need_site(), open_ring(), prepare(), prepare_approve(), prepare_publish() (+2 more)

### Community 58 - "Answer Accepted Declined"
Cohesion: 0.29
Nodes (4): id_reply(), now_ms(), queued(), queued_reply()

### Community 59 - "Live and Terminal Status"
Cohesion: 0.27
Nodes (9): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+1 more)

### Community 60 - "Chain Roles Tests"
Cohesion: 0.30
Nodes (12): new_key(), every_text_is_capped_on_receipt(), GOLDEN_REPORT, only_issue_events_and_checkpoints_are_bodies(), report(), round_trip(), size_limits(), tampered_byte_fails_signature() (+4 more)

### Community 61 - "Convergence Tests"
Cohesion: 0.30
Nodes (3): Clashes, Meeting, Step

### Community 62 - "Build LGX Script"
Cohesion: 0.14
Nodes (8): Link, Down, Listening, Open, Started, Starting, save(), Shared

### Community 63 - "Action Kind Enum"
Cohesion: 0.23
Nodes (3): distinct(), Prop, settle()

### Community 64 - "Issue Status Enum"
Cohesion: 0.29
Nodes (13): bal(), check(), die(), fin(), LEE_WALLET_HOME_DIR, log(), mem(), PUKAAR_PAYER (+5 more)

### Community 65 - "Action Anchor Revoke"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 66 - "Attestation Body Checks"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 67 - "Common Test Helpers"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 68 - "Local Anchor Script"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

### Community 69 - "Cancelled Status Enum"
Cohesion: 0.21
Nodes (13): chat_module 0.3.0, delivery_module 0.4.0, Install Pukaar in Logos Basecamp, Known limits, Logos Basecamp, logos-repo.json (package source), Option 2: Install by hand, Option 1: Add a package source (+5 more)

### Community 70 - "Meeting Job Queue"
Cohesion: 0.20
Nodes (4): decode_checkpoint(), Blob, read_site(), Reads

### Community 71 - "Pukaar App Icons"
Cohesion: 0.20
Nodes (10): Answer, Accepted, Declined, Left, Pending, Did, Accepted, Cancelled (+2 more)

### Community 73 - "Storage Direct Scripts"
Cohesion: 0.33
Nodes (8): a_checkpoint_from_a_non_member_is_rejected_too(), a_resident_who_is_not_the_reporter_votes_and_staff_do_not(), an_author_who_was_not_a_member_at_the_chain_block_is_rejected_and_kept(), as_rank(), only_staff_acknowledge_claim_and_close(), report(), the_chain_block_decides_not_the_arrival_time(), why()

### Community 74 - "Details Config Reasons"
Cohesion: 0.38
Nodes (5): a_forked_author_converges_to_one_branch_in_any_delivery_order(), any_delivery_order_gives_the_same_state(), random_action(), replay(), Rng

### Community 75 - "Module CMake Build"
Cohesion: 0.40
Nodes (8): busy(), fail(), heartbeat(), out(), run_module(), say(), build-lgx.sh script, status()

### Community 76 - "LEE Client Gen"
Cohesion: 0.20
Nodes (9): WriteKind, Anchor, Approve, CreateSite, Revoke, SetProfile, SetRole, SetSite (+1 more)

### Community 77 - "Fastbuild Main"
Cohesion: 0.20
Nodes (8): Status, Acknowledged, AwaitingConfirmation, ClosedWontfix, ConfirmedResolved, Duplicate, InProgress, Open

### Community 78 - "Backup Pointer Codec"
Cohesion: 0.20
Nodes (10): Action, Anchor, Approve, Revoke, SetProfile, SetRole, SetSite, SuperAdminAdd (+2 more)

### Community 79 - "Member Status Enum"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 82 - "Module Context Glue"
Cohesion: 0.22
Nodes (4): delivery_status(), listen(), read_with_queue(), write()

### Community 83 - "Build Script Entry"
Cohesion: 0.22
Nodes (9): DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong, TooLarge (+1 more)

### Community 84 - "Build Shell Script"
Cohesion: 0.33
Nodes (3): BusyBody, Signed, Signed<T>

### Community 85 - "LGX Check Script"
Cohesion: 0.22
Nodes (9): St, Done, Held, Landed, Preparing, Queued, Rejected, Sending (+1 more)

### Community 86 - "Demo Script"
Cohesion: 0.25
Nodes (9): member_record(), member_seed(), member_written_once(), sha(), BACKUP, JOIN_BATCH, PLACE_REMOVAL, ROSTER (+1 more)

### Community 87 - "UI Shots Scripts"
Cohesion: 0.56
Nodes (7): acct(), an_admin_cannot_lock_a_member_out_with_a_huge_profile_version(), an_anchor_signer_key_must_be_the_key_of_the_member(), h(), item(), NOW, site()

### Community 88 - "Window Script"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 89 - "Program Deployment Example"
Cohesion: 0.25
Nodes (8): Body, Kind, Accept, Cancel, Decline, Propose, ProposeGroup, ProposeNote

### Community 90 - "Program Deployment Programs"
Cohesion: 0.39
Nodes (6): anchor_writes_once(), apply_anchor(), check_anchor(), CheckpointRecord, rec(), record_round_trip_is_168_bytes()

### Community 91 - "Fastbuild Crate"
Cohesion: 0.38
Nodes (6): deploy(), LEE_WALLET_HOME_DIR, new_account(), PUKAAR_PAYER, PUKAAR_PROGRAM_ID, local-anchor.sh script

### Community 92 - "LEE Client Gen Crate"
Cohesion: 0.33
Nodes (6): Status, Cancelled, Confirmed, Declined, status_of(), Proposed

### Community 94 - "Pukaar Methods Lib"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 95 - "Identity Backup UI"
Cohesion: 0.33
Nodes (6): pukaar_core, pukaar_logic, pukaar_logic-fuzz, pukaar_sites_guest, pukaar_sites_rules, pukaar_sites_runner

### Community 96 - "Basecamp Script"
Cohesion: 0.53
Nodes (5): log_after(), pid_of(), refuse_if_running(), storage-direct.sh script, since()

### Community 98 - "Chat Event Subscription"
Cohesion: 0.50
Nodes (4): LogosModule.cmake, pukaar_core CMake module, metadata.json, Rust crate (rust-lib)

### Community 100 - "Busy Body Delivery"
Cohesion: 0.50
Nodes (3): LEE_WALLET_HOME_DIR, LEZ_HOMES, lez-local.sh script

### Community 103 - "Review Actor Checks"
Cohesion: 0.67
Nodes (3): MemberStatus, Active, Removed

## Knowledge Gaps
- **508 isolated node(s):** `name`, `display_name`, `version`, `type`, `interface` (+503 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 808 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **29 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Accept Actions and Nodes` to `Agent Core Logic`, `Sealed Box Crypto`, `Account Ids and Testnet`, `Calendar and Writes`, `Logos SDK Chat Glue`, `Checkpoint Ops`, `Membership Instructions`, `Identity Import`, `Grant and Pending Rows`, `Serde Deserialize and IO`, `Lifecycle and Events`, `Location and Writes Fixtures`, `Clock and Refresh Limits`, `Canonical Comment Equip`, `Join Request Checks`, `Anchor Note Users`, `Link Down Listening`, `Testnet Script Checks`, `People Screen UI`, `Answer Accepted Declined`, `Live and Terminal Status`, `Build LGX Script`, `Storage Direct Scripts`, `Details Config Reasons`, `Lez Testnet Script`, `Module Context Glue`, `Registry Crate`?**
  _High betweenness centrality (0.100) - this node is a cross-community bridge._
- **What connects `name`, `display_name`, `version` to the rest of the system?**
  _508 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Agent Core Logic` be split into smaller, more focused modules?**
  _Cohesion score 0.05036344755970924 - nodes in this community are weakly interconnected._
- **Why does `Queue` connect `Checkpoint Ops` to `Atomic Concurrency Primitives`, `Crate Names`, `Anchor Note Users`, `Local Deploy Scripts`, `Checkpoint Records`, `Membership Instructions`, `Module Context Glue`, `Distinct Invites`, `Registry Crate`, `Grant and Pending Rows`, `Checkpoint Records Decode`, `Answer Accepted Declined`, `Location and Writes Fixtures`?**
  _High betweenness centrality (0.065) - this node is a cross-community bridge._
- **Should `Atomic Concurrency Primitives` be split into smaller, more focused modules?**
  _Cohesion score 0.0786034353995519 - nodes in this community are weakly interconnected._
- **Why does `PukaarCoreModule` connect `Pukaar Core Module` to `Anchor Note Users`, `Canonical Comment Equip`, `Confirmed Declined Status`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **Should `Chat Conversations Tests` be split into smaller, more focused modules?**
  _Cohesion score 0.057692307692307696 - nodes in this community are weakly interconnected._