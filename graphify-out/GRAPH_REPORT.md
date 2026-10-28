# Graph Report - field-study  (2026-10-28)

## Corpus Check
- 109 files · ~174,152 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 20 file(s) not represented in the graph (top: (none) 6, .mdc 4, .qml 3)

## Summary
- 2254 nodes · 5741 edges · 110 communities (87 shown, 23 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 439 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Meetings and Scheduling
- Request Queue
- Site Writes and Approvals
- Clock and Plan Core
- Sealed Profiles and Crypto
- Discovery and Blob Fetch
- Clock Claim Checks
- Join Requests and Announce
- Storage Chunk Transfer
- Core Module API
- Agent Skills and Anchor Docs
- Join Request Validation
- Records and Attestations
- Rule Error Types
- Runner and Key Derivation
- Member Role Tests
- Role Authorization
- Event Body Types
- Event Store Branches
- Effect Rule Engine
- Node State and Fingerprints
- Logos Event Delivery
- Agent RPC and Tracking
- Anchor Operations
- Report Lifecycle Tests
- Approval Rule Tables
- Wallet and Config
- Anchor Notes and Queue
- Keyring and Confirmation
- CMake Build Config
- Identity Import and Sync
- Chain Account Decoding
- Identity Encryption
- Profile and Fetch
- Chain Roster Access
- Checkpoint and Fuzz Tests
- Action Bodies and Deadlines
- Effect Types
- Fetched Announce Cache
- Test Harness Gossip
- Publish and Act
- Delivery Status and Flush
- UI Build Metadata
- UI Session Actions
- Local Persistence
- Issue Reducer
- Checkpoint Signing
- Sync Delivery Config
- Grants and Revocations
- Local Deploy Scripts
- Fetch Cycle
- Intents and Site Creation
- Event Round-Trip Tests
- Approval Queue
- Chain Refresh Tests
- Store Equivocation Tests
- Meet Commands
- Testnet Scripts
- Product Feature Docs
- People and Places Screens
- Read Cache Tests
- Wire Messages
- Link State
- Registry Program
- Checkpoint Architecture Docs
- Roster Queries
- Account Reads
- Issue Status
- Heads Resend
- Chain Role Tests
- Convergence Tests
- LGX Build Scripts
- Persistence Tests
- Chain Writer
- Write Kinds
- Action Types
- Board and Report Docs
- Send Status States
- CI Workflow
- Anchor Record Tests
- Member Account Seeds
- Shared Blob Queue
- Local Anchor Script
- Brand Logo Assets
- Workspace Crates
- Storage Direct Script
- Offline Chain Sharing
- CMake Module Setup
- Super Admin Kinds
- Meet Backup Jobs
- Client Codegen CLI
- Local LEZ Script
- Member Status
- Testnet LEZ Script
- Build Shell Script
- Basecamp Script
- Core LGX Check
- Demo Script
- UI Screenshot Script
- Chain Screenshot Script
- Window Script
- Deployment Example Docs
- Deployment Program Example
- Fastbuild Crate
- Client Gen Crate
- Registry Crate

## God Nodes (most connected - your core abstractions)
1. `Node` - 107 edges
2. `RuleError` - 76 edges
3. `Queue` - 62 edges
4. `Pukaar` - 50 edges
5. `PukaarCoreModule` - 48 edges
6. `Meetings` - 43 edges
7. `Rank` - 42 edges
8. `Actor` - 35 edges
9. `locked()` - 33 edges
10. `new_key()` - 32 edges

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
- **Checkpoint anchoring chain** — readme_pukaar_core, readme_checkpoint_merkle_root, readme_pukaar_registry, readme_lez [INFERRED 0.75]
- **Admin role change via notice and seal** — readme_super_admin, readme_admin_notice, readme_notice_seal [INFERRED 0.75]
- **Tamper-evidence via signed events and fork proofs** — readme_signed_event_envelope, readme_fork_proof, readme_fork_resolution_lowest_id [INFERRED 0.75]
- **Pukaar bell logo variants** — modules_pukaar_ui_src_icons_pukaar_app_icon, public_brand_pukaar_logo_color_logo, public_brand_pukaar_logo_color_svg_logo, public_brand_pukaar_logo_logo, public_brand_pukaar_logo_svg_logo [INFERRED 0.85]

## Communities (110 total, 23 thin omitted)

### Community 0 - "Meetings and Scheduling"
Cohesion: 0.06
Nodes (50): a_clash_is_named_and_does_not_block_the_proposal(), a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side(), a_newer_summary_wins_and_an_old_one_is_ignored(), a_stranger_a_forgery_and_the_wrong_reader_change_nothing(), Act, Accept, Cancel, Decline (+42 more)

### Community 1 - "Request Queue"
Cohesion: 0.08
Nodes (59): sha256(), Poll, Included, Pending, Rejected, a_balance_that_drops_after_prepare_is_caught_at_the_send(), a_balance_under_the_floor_holds_the_write_with_the_reason(), a_big_profile_goes_by_cid_only() (+51 more)

### Community 2 - "Site Writes and Approvals"
Cohesion: 0.07
Nodes (56): Location, accounts_follow_the_runner_order(), anchor(), apply_place_op(), approve_batch(), approve_batch_names_the_requester_and_the_claims(), auth(), BATCH (+48 more)

### Community 3 - "Clock and Plan Core"
Cohesion: 0.06
Nodes (43): by_hand(), Ctx, Ctx<'a>, nonempty(), pda(), plan(), probe(), set_pointer() (+35 more)

### Community 4 - "Sealed Profiles and Crypto"
Cohesion: 0.11
Nodes (51): seal(), sealed_box_round_trip_wrong_key_and_libsodium(), x25519_public(), a_full_profile_fits_the_inline_cap(), add_seats(), add_site_wrap(), ANNOUNCE_DOMAIN, announce_signs_verifies_and_rejects_tampering() (+43 more)

### Community 5 - "Discovery and Blob Fetch"
Cohesion: 0.12
Nodes (37): a_blob_that_does_not_match_the_chain_hash_is_not_used_and_is_fetched_again(), a_file_of_100_kb_comes_in_chunks_and_reassembles(), a_flood_of_requests_gets_few_answers(), a_join_request_and_an_announce_name_accounts_to_read(), a_keyring_entry_that_lies_about_its_key_is_ignored(), a_lying_author_account_is_rejected(), a_removal_reason_comes_from_details_and_must_match_its_hash(), acct() (+29 more)

### Community 6 - "Clock Claim Checks"
Cohesion: 0.12
Nodes (39): A, a_call_from_another_program_is_refused(), a_stale_or_future_clock_claim_fails(), a_wrong_member_pda_is_refused(), anchor_needs_an_active_member(), approve_batch_negatives(), approve_batch_writes_members_and_site(), auth() (+31 more)

### Community 7 - "Join Requests and Announce"
Cohesion: 0.10
Nodes (45): a_forged_or_unsolicited_decline_is_ignored(), a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download(), a_join_request_is_signed_twice_and_kept(), a_new_site_has_its_files_and_the_first_profile_behind_it(), a_promotion_queues_the_signed_name_with_the_role_change(), a_record_that_is_anchored_already_is_refused_before_anything_is_queued(), a_removed_member_gets_a_reason_and_no_storage_error(), a_removed_member_stays_in_the_removed_list_after_the_keyring_rotates() (+37 more)

### Community 8 - "Storage Chunk Transfer"
Cohesion: 0.08
Nodes (28): base(), CALL, CHUNK, Client, config_gets_own_data_dir_and_free_ports(), env_override_wins_and_bad_json_is_an_error(), Events, keep_file() (+20 more)

### Community 10 - "Agent Skills and Anchor Docs"
Cohesion: 0.05
Nodes (41): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+33 more)

### Community 11 - "Join Request Validation"
Cohesion: 0.09
Nodes (27): build(), built_request_parses_and_passes_the_rules(), check_decline(), DECLINE_DOMAIN, decline_message(), keys(), sign_decline(), sizes_are_162_and_290() (+19 more)

### Community 12 - "Records and Attestations"
Cohesion: 0.08
Nodes (29): parse(), ACCOUNT_ID_PREFIX, Attestation, check_join_request(), CID_MAX, GATE_NATIVE, GATE_NFT, GATE_TOKEN (+21 more)

### Community 13 - "Rule Error Types"
Cohesion: 0.05
Nodes (40): RuleError, AlreadyAnchored, AlreadyMember, AlreadySuperAdmin, BadBalance, BadCid, BadJoinRequest, BadMerkle (+32 more)

### Community 14 - "Runner and Key Derivation"
Cohesion: 0.14
Nodes (18): clock_ms(), Cx, hex32(), id_env(), join_request(), join_request_passes_the_rules_and_both_signatures_verify(), Keys, main() (+10 more)

### Community 15 - "Member Role Tests"
Cohesion: 0.21
Nodes (19): a_removed_member_cannot_act_until_approved_again(), a_super_admin_adds_one_more_member_and_no_third(), acct(), adds_over_the_limit_keep_the_first(), claim(), mutual_removal_keeps_the_first(), non_admins_cannot_approve(), only_a_super_admin_grants_or_changes_the_admin_role() (+11 more)

### Community 16 - "Role Authorization"
Cohesion: 0.20
Nodes (32): check_cid(), active_rank(), Actor, allow(), apply_approve(), apply_approve_site(), apply_create(), apply_revoke() (+24 more)

### Community 17 - "Event Body Types"
Cohesion: 0.07
Nodes (30): Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm, MarkDuplicate (+22 more)

### Community 18 - "Event Store Branches"
Cohesion: 0.13
Nodes (11): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+3 more)

### Community 19 - "Effect Rule Engine"
Cohesion: 0.09
Nodes (28): apply(), apply_effect(), check_balance(), check_clock(), check_ed25519(), check_probe(), checkpoint_seed(), CLOCK_TOLERANCE_MS (+20 more)

### Community 20 - "Node State and Fingerprints"
Cohesion: 0.11
Nodes (8): fingerprint(), Node, last_anchor(), on_decline(), pending_rows(), refresh_items(), remember_known(), status()

### Community 21 - "Logos Event Delivery"
Cohesion: 0.09
Nodes (23): advance(), ANSWER_EVERY, chain_tick(), create_and_listen(), delivered(), delivered_once(), DELIVERY_TIMEOUT, env_or() (+15 more)

### Community 22 - "Agent RPC and Tracking"
Cohesion: 0.11
Nodes (21): a_node_reads_roles_lists_and_identity_from_the_cache(), balance(), CLAIM_MARGIN_MS, claim_ms(), clock_ms(), FLOOR_LEPTA, get_account(), HttpRpc (+13 more)

### Community 23 - "Anchor Operations"
Cohesion: 0.12
Nodes (24): about_text(), anchor(), anchor_plan(), anchor_push(), anchor_read(), AnchorPlan, create_site(), join() (+16 more)

### Community 24 - "Report Lifecycle Tests"
Cohesion: 0.12
Nodes (22): stage(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline(), claim_reopen_claim_confirm(), finished_cards_carry_no_pending_next_step(), forbidden_transitions_are_inert_and_visible() (+14 more)

### Community 25 - "Approval Rule Tables"
Cohesion: 0.12
Nodes (23): A, approve_item_table(), ASHA, check_join_request_table(), create_args(), create_makes_the_creator_super_admin_and_first_member(), create_table(), D (+15 more)

### Community 26 - "Wallet and Config"
Cohesion: 0.12
Nodes (8): an_unreachable_node_is_an_error_not_a_hang(), GAS_LIMIT, imported_key_gives_the_account_of_lezkey(), POLL, seed_key(), SEND, temp(), WalletWriter

### Community 27 - "Anchor Notes and Queue"
Cohesion: 0.12
Nodes (5): AnchorNote, Queue, RevokeReq, tick(), request_id()

### Community 28 - "Keyring and Confirmation"
Cohesion: 0.20
Nodes (15): build(), confirm(), effect_on_chain(), Entry, Env, fetch_blob(), need_site(), open_ring() (+7 more)

### Community 29 - "CMake Build Config"
Cohesion: 0.08
Nodes (26): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+18 more)

### Community 30 - "Identity Import and Sync"
Cohesion: 0.15
Nodes (18): import_into(), copy_all(), a_failed_import_writes_nothing(), a_peer_naming_a_later_seq_keeps_us_waiting(), a_restored_node_refuses_to_publish_until_its_history_is_back(), heads(), import_into_a_fresh_device_gives_an_empty_node_with_the_same_identity(), import_is_refused_when_a_site_exists() (+10 more)

### Community 31 - "Chain Account Decoding"
Cohesion: 0.19
Nodes (19): check_version(), checkpoint_pda(), decode_checkpoint(), decode_member(), decode_site(), decodes_the_real_site_account(), member(), member_pda() (+11 more)

### Community 32 - "Identity Encryption"
Cohesion: 0.11
Nodes (16): cipher(), export_identity(), import_identity(), MIN_PASSWORD, NONCE, PREFIX, SALT, VERSION (+8 more)

### Community 33 - "Profile and Fetch"
Cohesion: 0.10
Nodes (18): about_of(), account(), clean_about(), DETAILS_SCHEMA, is_staff(), MAX_ABOUT, MAX_ANNOUNCES, MAX_ASKED (+10 more)

### Community 34 - "Chain Roster Access"
Cohesion: 0.17
Nodes (6): Chain, key(), now_ms(), people_lists_have_the_shape_of_main_qml(), rank_name(), refresh_fills_the_roster()

### Community 35 - "Checkpoint and Fuzz Tests"
Cohesion: 0.12
Nodes (7): a_blob_is_kept_only_when_hash_and_signature_hold(), an_inline_profile_has_a_cap_and_a_bigger_file_is_not_a_blob(), announce(), blob_of(), every_message_kind_round_trips(), garbage_is_rejected(), size_limits_hold()

### Community 36 - "Action Bodies and Deadlines"
Cohesion: 0.14
Nodes (11): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), HEADS_TO_SETTLE, kind_name(), past(), Restore (+3 more)

### Community 37 - "Effect Types"
Cohesion: 0.09
Nodes (21): Effect, Anchor, AnchorSigner, Approve, ApproveSite, BalanceAtLeast, ClockIs, CreateMember (+13 more)

### Community 38 - "Fetched Announce Cache"
Cohesion: 0.14
Nodes (3): Fetched, Partial, Announce

### Community 39 - "Test Harness Gossip"
Cohesion: 0.20
Nodes (15): comment(), equip(), gossip(), node(), roles_key(), second_device(), Site, with() (+7 more)

### Community 40 - "Publish and Act"
Cohesion: 0.14
Nodes (9): err(), now(), on_wire(), publish(), publish_with(), save(), start_site(), wire_chain() (+1 more)

### Community 41 - "Delivery Status and Flush"
Cohesion: 0.23
Nodes (8): delivery_status(), flush(), id_reply(), now_ms(), queued(), queued_reply(), read_with_queue(), write_tick()

### Community 42 - "UI Build Metadata"
Cohesion: 0.10
Nodes (20): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+12 more)

### Community 43 - "UI Session Actions"
Cohesion: 0.16
Nodes (4): later(), place(), Pukaar, read()

### Community 44 - "Local Persistence"
Cohesion: 0.20
Nodes (13): account_b58(), join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), remove_if_present() (+5 more)

### Community 45 - "Issue Reducer"
Cohesion: 0.23
Nodes (12): issue_json(), apply(), CheckpointRecord, Config, Issue, issue_of(), not_permitted(), Progress (+4 more)

### Community 46 - "Checkpoint Signing"
Cohesion: 0.20
Nodes (10): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+2 more)

### Community 47 - "Sync Delivery Config"
Cohesion: 0.14
Nodes (15): ANSWERERS, BLOB_CHUNK, DELIVERY_CFG, lost_node(), MAX_CHUNKS, MAX_FALLBACK, MAX_RESEND, MAX_WIRE_BYTES (+7 more)

### Community 48 - "Grants and Revocations"
Cohesion: 0.27
Nodes (12): parse_id(), drop_held(), grant(), push_staff_name(), reject(), revoke(), send_batch(), set_staff_name() (+4 more)

### Community 49 - "Local Deploy Scripts"
Cohesion: 0.24
Nodes (16): deploy(), expect(), fail(), LEE_WALLET_HOME_DIR, mem(), neg(), new_account(), NOW() (+8 more)

### Community 50 - "Fetch Cycle"
Cohesion: 0.22
Nodes (6): clean(), cycle(), Details, fetch_cycle(), Store, sync_once()

### Community 51 - "Intents and Site Creation"
Cohesion: 0.12
Nodes (14): Intent, Anchor, Approve, CreateSite, Details, Places, Publish, Revoke (+6 more)

### Community 52 - "Event Round-Trip Tests"
Cohesion: 0.30
Nodes (12): new_key(), every_text_is_capped_on_receipt(), GOLDEN_REPORT, only_issue_events_and_checkpoints_are_bodies(), report(), round_trip(), size_limits(), tampered_byte_fails_signature() (+4 more)

### Community 53 - "Approval Queue"
Cohesion: 0.15
Nodes (12): ApproveReq, KEEP_DONE, LAND_WINDOW_MS, MAX_REJECTED, MAX_REQUESTS, PREP_WINDOW_MS, Prof, ProfReq (+4 more)

### Community 54 - "Chain Refresh Tests"
Cohesion: 0.26
Nodes (10): at_most_one_refresh_per_ten_seconds(), role_is_the_newest_known_at_any_block(), lamport_must_increase_along_a_chain(), out_of_order_events_wait_then_apply(), resend_is_capped(), branch(), chain(), event() (+2 more)

### Community 55 - "Store Equivocation Tests"
Cohesion: 0.26
Nodes (8): a_displaced_branch_and_its_descendants_become_evidence(), an_event_for_another_site_is_rejected(), equivocation_is_caught(), held_ids(), store_with(), the_alternatives_cap_keeps_the_lowest_ids_in_any_order(), the_lower_id_wins_whichever_arrives_first(), twins()

### Community 56 - "Meet Commands"
Cohesion: 0.21
Nodes (5): key_arg(), meet(), meet_answer(), meet_send(), ms_arg()

### Community 57 - "Testnet Scripts"
Cohesion: 0.29
Nodes (13): bal(), check(), die(), fin(), LEE_WALLET_HOME_DIR, log(), mem(), PUKAAR_PAYER (+5 more)

### Community 58 - "Product Feature Docs"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 59 - "People and Places Screens"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 60 - "Read Cache Tests"
Cohesion: 0.26
Nodes (9): b58(), a_failed_read_keeps_the_old_cache(), a_promoted_or_revoked_author_is_judged_by_the_newest_role(), a_site_that_is_not_on_the_chain_is_an_error(), acct(), edit(), Fake, lez_text_and_balance_read() (+1 more)

### Community 61 - "Wire Messages"
Cohesion: 0.15
Nodes (11): Wire, BlobChunk, BlobReq, Busy, ChainPending, Event, JoinDecline, JoinReq (+3 more)

### Community 62 - "Link State"
Cohesion: 0.15
Nodes (7): Link, Down, Listening, Open, Started, Starting, Shared

### Community 63 - "Registry Program"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 64 - "Checkpoint Architecture Docs"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

### Community 66 - "Account Reads"
Cohesion: 0.27
Nodes (5): body_of(), claimed(), read_site(), Reads, SHORT_OF_FUNDS

### Community 67 - "Issue Status"
Cohesion: 0.18
Nodes (8): Status, Acknowledged, AwaitingConfirmation, ClosedWontfix, ConfirmedResolved, Duplicate, InProgress, Open

### Community 68 - "Heads Resend"
Cohesion: 0.35
Nodes (9): heads_msg(), resend(), to_resend(), to_resend_own(), Heads, a_behind_store_catches_up_from_heads(), a_join_published_while_delivery_was_down_still_reaches_the_admin(), a_peer_on_the_losing_branch_gets_the_winner_even_when_ahead() (+1 more)

### Community 69 - "Chain Role Tests"
Cohesion: 0.33
Nodes (8): a_checkpoint_from_a_non_member_is_rejected_too(), a_resident_who_is_not_the_reporter_votes_and_staff_do_not(), an_author_who_was_not_a_member_at_the_chain_block_is_rejected_and_kept(), as_rank(), only_staff_acknowledge_claim_and_close(), report(), the_chain_block_decides_not_the_arrival_time(), why()

### Community 70 - "Convergence Tests"
Cohesion: 0.38
Nodes (5): a_forked_author_converges_to_one_branch_in_any_delivery_order(), any_delivery_order_gives_the_same_state(), random_action(), replay(), Rng

### Community 71 - "LGX Build Scripts"
Cohesion: 0.40
Nodes (8): busy(), fail(), heartbeat(), out(), run_module(), say(), build-lgx.sh script, status()

### Community 72 - "Persistence Tests"
Cohesion: 0.20
Nodes (3): canonical(), save_then_load_gives_the_same_state(), view()

### Community 73 - "Chain Writer"
Cohesion: 0.24
Nodes (6): ChainWriter, KeyState, Failed, Ready, Unknown, send()

### Community 74 - "Write Kinds"
Cohesion: 0.20
Nodes (9): WriteKind, Anchor, Approve, CreateSite, Revoke, SetProfile, SetRole, SetSite (+1 more)

### Community 75 - "Action Types"
Cohesion: 0.20
Nodes (10): Action, Anchor, Approve, Revoke, SetProfile, SetRole, SetSite, SuperAdminAdd (+2 more)

### Community 76 - "Board and Report Docs"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 77 - "Send Status States"
Cohesion: 0.22
Nodes (9): St, Done, Held, Landed, Preparing, Queued, Rejected, Sending (+1 more)

### Community 78 - "CI Workflow"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 79 - "Anchor Record Tests"
Cohesion: 0.39
Nodes (6): anchor_writes_once(), apply_anchor(), check_anchor(), CheckpointRecord, rec(), record_round_trip_is_168_bytes()

### Community 80 - "Member Account Seeds"
Cohesion: 0.29
Nodes (8): member_record(), member_seed(), member_written_once(), sha(), JOIN_BATCH, PLACE_REMOVAL, ROSTER, SITE_ID

### Community 82 - "Local Anchor Script"
Cohesion: 0.38
Nodes (6): deploy(), LEE_WALLET_HOME_DIR, new_account(), PUKAAR_PAYER, PUKAAR_PROGRAM_ID, local-anchor.sh script

### Community 83 - "Brand Logo Assets"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 84 - "Workspace Crates"
Cohesion: 0.33
Nodes (6): pukaar_core, pukaar_logic, pukaar_logic-fuzz, pukaar_sites_guest, pukaar_sites_rules, pukaar_sites_runner

### Community 85 - "Storage Direct Script"
Cohesion: 0.53
Nodes (5): log_after(), pid_of(), refuse_if_running(), storage-direct.sh script, since()

### Community 87 - "CMake Module Setup"
Cohesion: 0.50
Nodes (4): LogosModule.cmake, pukaar_core CMake module, metadata.json, Rust crate (rust-lib)

### Community 88 - "Super Admin Kinds"
Cohesion: 0.50
Nodes (4): SuperKind, Add, Remove, Transfer

### Community 91 - "Local LEZ Script"
Cohesion: 0.50
Nodes (3): LEE_WALLET_HOME_DIR, LEZ_HOMES, lez-local.sh script

### Community 93 - "Member Status"
Cohesion: 0.67
Nodes (3): MemberStatus, Active, Removed

## Knowledge Gaps
- **449 isolated node(s):** `name`, `display_name`, `version`, `type`, `interface` (+444 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 707 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **23 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Node State and Fingerprints` to `Site Writes and Approvals`, `Discovery and Blob Fetch`, `Logos Event Delivery`, `Agent RPC and Tracking`, `Anchor Operations`, `Report Lifecycle Tests`, `Identity Import and Sync`, `Identity Encryption`, `Action Bodies and Deadlines`, `Fetched Announce Cache`, `Publish and Act`, `Delivery Status and Flush`, `UI Session Actions`, `Local Persistence`, `Issue Reducer`, `Checkpoint Signing`, `Grants and Revocations`, `Meet Commands`, `Link State`, `Chain Role Tests`, `Convergence Tests`, `Persistence Tests`, `Meet Backup Jobs`?**
  _High betweenness centrality (0.144) - this node is a cross-community bridge._
- **What connects `name`, `display_name`, `version` to the rest of the system?**
  _449 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Meetings and Scheduling` be split into smaller, more focused modules?**
  _Cohesion score 0.05504950495049505 - nodes in this community are weakly interconnected._
- **Why does `Queue` connect `Anchor Notes and Queue` to `Request Queue`, `Account Reads`, `Join Requests and Announce`, `Chain Writer`, `Delivery Status and Flush`, `Grants and Revocations`, `Shared Blob Queue`, `Node State and Fingerprints`, `Approval Queue`, `Logos Event Delivery`, `Anchor Operations`, `Keyring and Confirmation`?**
  _High betweenness centrality (0.047) - this node is a cross-community bridge._
- **Should `Request Queue` be split into smaller, more focused modules?**
  _Cohesion score 0.08122036693465265 - nodes in this community are weakly interconnected._
- **Why does `Rank` connect `Test Harness Gossip` to `Roster Queries`, `Account Reads`, `Chain Roster Access`, `Action Bodies and Deadlines`, `Site Writes and Approvals`, `Chain Role Tests`, `Convergence Tests`, `Clock Claim Checks`, `Clock and Plan Core`, `Issue Reducer`, `Member Role Tests`, `Grants and Revocations`, `Role Authorization`, `Node State and Fingerprints`?**
  _High betweenness centrality (0.043) - this node is a cross-community bridge._
- **Should `Site Writes and Approvals` be split into smaller, more focused modules?**
  _Cohesion score 0.07080745341614907 - nodes in this community are weakly interconnected._