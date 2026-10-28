# Graph Report - field-study  (2026-10-28)

## Corpus Check
- 9 files · ~174,831 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2314 nodes · 5686 edges · 124 communities (83 shown, 41 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 354 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Meetings and Scheduling
- Polling and Sync State
- Places and Write Ops
- Discovery Tests
- Announce and Membership
- Program Access Control
- Threading and Storage
- Checkpoints and Anchoring
- Core Module API
- Records and Attestations
- Agent Skills Docs
- Rule Errors
- RPC and Fingerprints
- About Text Cleaning
- Anchor Rules
- Keys and Crypto
- Borsh Instructions
- Chain Decoding
- Roles and Actions
- Issue Events
- Approval Table Tests
- Store Branches
- Membership Rules Tests
- Event Signing
- Balance and Clock Effects
- Identity Import
- Report Lifecycle
- Profile and Keyring
- Chain Access
- Plan and Guest Programs
- Announce Fetching
- Issue Reducer
- Store Convergence Tests
- Build Metadata
- Logos Store Delivery
- LEZ Key Handling
- Pukaar Lifecycle
- Checkpoint Fuzz Tests
- Profile Announce Tests
- Gossip and Equip
- Action Body Parsing
- Intent and Approve
- UI Build Metadata
- Action Queue
- Identity Encryption
- Event Timing
- Duration and Accounts
- Test Context
- Sync and Blobs
- Persistence
- Link State
- Meetings UI
- Deploy Scripts
- Join Request Checks
- Admin Actions
- Wallet Writer
- Event Format Tests
- Anchor Notes UI
- Settlement Flow
- Item States
- Queue Landing
- Testnet Scripts
- History UI Spec
- People UI Spec
- Checkpoint Records
- Blob Wire Protocol
- Registry Anchor
- Architecture Notes
- Heads Resend
- Chain Role Tests
- Convergence Tests
- LGX Build Scripts
- Keyring Encryption
- Action Kinds
- Board UI Spec
- Decode Errors
- Blob Pointers
- Queue States
- Write Kinds
- Action Persistence Tests
- Event Subscription
- CI Workflow
- Local Anchor Script
- Offline Chain
- Logo Assets
- Crate Layout
- Storage Direct Script
- Chain Writer
- CMake Module
- Client Gen
- Local Lez Script
- Testnet Script
- Build Script
- Basecamp Script
- Core LGX Check
- Demo Script
- UI Shots Script
- Chain UI Shots Script
- Window Script
- Community 118
- Community 119
- Community 120
- Community 121
- Community 122

## God Nodes (most connected - your core abstractions)
1. `Node` - 108 edges
2. `RuleError` - 76 edges
3. `Queue` - 62 edges
4. `Pukaar` - 50 edges
5. `PukaarCoreModule` - 48 edges
6. `Meetings` - 39 edges
7. `Rank` - 36 edges
8. `Actor` - 35 edges
9. `locked()` - 33 edges
10. `new_key()` - 32 edges

## Surprising Connections (you probably didn't know these)
- `join_request_passes_the_rules_and_both_signatures_verify()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `main()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `run()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `user()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `parse()` --calls--> `request_id()`  [INFERRED]
  modules/pukaar_core/rust-lib/logic/src/joinreq.rs → programs/pukaar_sites/rules/src/records.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Rust logic crate quality checks** — github_workflows_ci_logic_job, github_workflows_ci_fuzz_smoke_job, github_workflows_ci_rust_logic_crate [EXTRACTED 1.00]
- **Admin role change via notice and seal** — readme_super_admin, readme_admin_notice, readme_notice_seal [INFERRED 0.75]
- **Checkpoint anchoring chain** — readme_pukaar_core, readme_checkpoint_merkle_root, readme_pukaar_registry, readme_lez [INFERRED 0.75]
- **Tamper-evidence via signed events and fork proofs** — readme_signed_event_envelope, readme_fork_proof, readme_fork_resolution_lowest_id [INFERRED 0.75]
- **Pukaar bell logo variants** — modules_pukaar_ui_src_icons_pukaar_app_icon, public_brand_pukaar_logo_color_logo, public_brand_pukaar_logo_color_svg_logo, public_brand_pukaar_logo_logo, public_brand_pukaar_logo_svg_logo [INFERRED 0.85]

## Communities (124 total, 41 thin omitted)

### Community 0 - "Meetings and Scheduling"
Cohesion: 0.06
Nodes (50): a_clash_is_named_and_does_not_block_the_proposal(), a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side(), a_newer_summary_wins_and_an_old_one_is_ignored(), a_stranger_a_forgery_and_the_wrong_reader_change_nothing(), Act, Accept, Cancel, Decline (+42 more)

### Community 1 - "Polling and Sync State"
Cohesion: 0.08
Nodes (57): a_staff_name_is_uploaded_and_sent_as_one_profile_pointer(), Poll, Included, Pending, Rejected, a_balance_that_drops_after_prepare_is_caught_at_the_send(), a_balance_under_the_floor_holds_the_write_with_the_reason(), a_big_profile_goes_by_cid_only() (+49 more)

### Community 2 - "Places and Write Ops"
Cohesion: 0.07
Nodes (54): Location, accounts_follow_the_runner_order(), anchor(), apply_place_op(), approve_batch(), approve_batch_names_the_requester_and_the_claims(), auth(), BATCH (+46 more)

### Community 3 - "Discovery Tests"
Cohesion: 0.12
Nodes (38): a_blob_that_does_not_match_the_chain_hash_is_not_used_and_is_fetched_again(), a_file_of_100_kb_comes_in_chunks_and_reassembles(), a_flood_of_requests_gets_few_answers(), a_join_request_and_an_announce_name_accounts_to_read(), a_keyring_entry_that_lies_about_its_key_is_ignored(), a_lying_author_account_is_rejected(), a_removal_reason_comes_from_details_and_must_match_its_hash(), acct() (+30 more)

### Community 4 - "Announce and Membership"
Cohesion: 0.10
Nodes (44): a_forged_or_unsolicited_decline_is_ignored(), a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download(), a_join_request_is_signed_twice_and_kept(), a_new_site_has_its_files_and_the_first_profile_behind_it(), a_promotion_queues_the_signed_name_with_the_role_change(), a_record_that_is_anchored_already_is_refused_before_anything_is_queued(), a_removed_member_gets_a_reason_and_no_storage_error(), a_removed_member_stays_in_the_removed_list_after_the_keyring_rotates() (+36 more)

### Community 5 - "Program Access Control"
Cohesion: 0.12
Nodes (39): A, a_call_from_another_program_is_refused(), a_stale_or_future_clock_claim_fails(), a_wrong_member_pda_is_refused(), anchor_needs_an_active_member(), approve_batch_negatives(), approve_batch_writes_members_and_site(), auth() (+31 more)

### Community 6 - "Threading and Storage"
Cohesion: 0.08
Nodes (28): base(), CALL, CHUNK, Client, config_gets_own_data_dir_and_free_ports(), env_override_wins_and_bad_json_is_an_error(), Events, keep_file() (+20 more)

### Community 7 - "Checkpoints and Anchoring"
Cohesion: 0.08
Nodes (33): about_text(), anchor(), anchor_plan(), anchor_push(), anchor_read(), AnchorPlan, create_site(), join() (+25 more)

### Community 9 - "Records and Attestations"
Cohesion: 0.07
Nodes (37): sha(), ACCOUNT_ID_PREFIX, Attestation, check_join_request(), CID_MAX, GATE_NATIVE, GATE_NFT, GATE_TOKEN (+29 more)

### Community 10 - "Agent Skills Docs"
Cohesion: 0.05
Nodes (41): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+33 more)

### Community 11 - "Rule Errors"
Cohesion: 0.05
Nodes (40): RuleError, AlreadyAnchored, AlreadyMember, AlreadySuperAdmin, BadBalance, BadCid, BadJoinRequest, BadMerkle (+32 more)

### Community 12 - "RPC and Fingerprints"
Cohesion: 0.10
Nodes (32): b58(), program(), a_failed_read_keeps_the_old_cache(), a_node_reads_roles_lists_and_identity_from_the_cache(), a_promoted_or_revoked_author_is_judged_by_the_newest_role(), a_site_that_is_not_on_the_chain_is_an_error(), acct(), balance() (+24 more)

### Community 13 - "About Text Cleaning"
Cohesion: 0.08
Nodes (24): about_of(), clean(), clean_about(), cycle(), Details, DETAILS_SCHEMA, fetch_cycle(), is_staff() (+16 more)

### Community 14 - "Anchor Rules"
Cohesion: 0.08
Nodes (33): anchor_writes_once(), apply_anchor(), check_anchor(), check_ed25519(), check_probe(), checkpoint_seed(), CheckpointRecord, CLOCK_TOLERANCE_MS (+25 more)

### Community 15 - "Keys and Crypto"
Cohesion: 0.14
Nodes (18): clock_ms(), Cx, hex32(), id_env(), join_request(), join_request_passes_the_rules_and_both_signatures_verify(), Keys, main() (+10 more)

### Community 16 - "Borsh Instructions"
Cohesion: 0.08
Nodes (31): by_hand(), Instruction, Anchor, ApproveBatch, CreateSite, Probe, Reserved, RevokeBatch (+23 more)

### Community 17 - "Chain Decoding"
Cohesion: 0.14
Nodes (19): check_version(), checkpoint_pda(), decode_checkpoint(), decode_member(), decode_site(), decodes_the_real_site_account(), member(), member_pda() (+11 more)

### Community 18 - "Roles and Actions"
Cohesion: 0.20
Nodes (32): check_cid(), active_rank(), Actor, allow(), apply_approve(), apply_approve_site(), apply_create(), apply_revoke() (+24 more)

### Community 19 - "Issue Events"
Cohesion: 0.10
Nodes (3): fingerprint(), issue_json(), Node

### Community 20 - "Approval Table Tests"
Cohesion: 0.15
Nodes (18): acct(), approve_item_table(), body(), check_join_request_table(), create_args(), create_makes_the_creator_super_admin_and_first_member(), create_table(), effective_status_table() (+10 more)

### Community 21 - "Store Branches"
Cohesion: 0.14
Nodes (11): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+3 more)

### Community 22 - "Membership Rules Tests"
Cohesion: 0.17
Nodes (25): A, a_removed_member_cannot_act_until_approved_again(), a_super_admin_adds_one_more_member_and_no_third(), adds_over_the_limit_keep_the_first(), ASHA, claim(), D, MEERA (+17 more)

### Community 23 - "Event Signing"
Cohesion: 0.09
Nodes (24): merkle_root(), Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm (+16 more)

### Community 24 - "Balance and Clock Effects"
Cohesion: 0.07
Nodes (29): apply(), apply_effect(), check_balance(), check_clock(), Effect, Anchor, AnchorSigner, Approve (+21 more)

### Community 25 - "Identity Import"
Cohesion: 0.14
Nodes (22): import_into(), copy_all(), a_failed_import_writes_nothing(), a_peer_naming_a_later_seq_keeps_us_waiting(), a_restored_node_refuses_to_publish_until_its_history_is_back(), blob(), export_then_import_restores_key_and_site(), heads() (+14 more)

### Community 26 - "Report Lifecycle"
Cohesion: 0.12
Nodes (22): stage(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline(), claim_reopen_claim_confirm(), finished_cards_carry_no_pending_next_step(), forbidden_transitions_are_inert_and_visible() (+14 more)

### Community 27 - "Profile and Keyring"
Cohesion: 0.11
Nodes (28): add_seats(), ANNOUNCE_DOMAIN, entries(), Keyring, keyring_from_bytes(), keyring_round_trip_and_epochs(), KEYRING_SCHEMA, keyring_to_bytes() (+20 more)

### Community 28 - "Chain Access"
Cohesion: 0.14
Nodes (6): Chain, key(), now_ms(), people_lists_have_the_shape_of_main_qml(), rank_name(), refresh_fills_the_roster()

### Community 29 - "Plan and Guest Programs"
Cohesion: 0.17
Nodes (8): Ctx, Ctx<'a>, nonempty(), pda(), plan(), probe(), set_pointer(), super_admin()

### Community 30 - "Announce Fetching"
Cohesion: 0.11
Nodes (5): account(), Fetched, Partial, Places, Announce

### Community 31 - "Issue Reducer"
Cohesion: 0.14
Nodes (19): apply(), CheckpointRecord, Config, Issue, issue_of(), not_permitted(), Progress, progress_by() (+11 more)

### Community 32 - "Store Convergence Tests"
Cohesion: 0.15
Nodes (18): at_most_one_refresh_per_ten_seconds(), role_is_the_newest_known_at_any_block(), a_displaced_branch_and_its_descendants_become_evidence(), an_event_for_another_site_is_rejected(), equivocation_is_caught(), held_ids(), lamport_must_increase_along_a_chain(), out_of_order_events_wait_then_apply() (+10 more)

### Community 33 - "Build Metadata"
Cohesion: 0.08
Nodes (26): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+18 more)

### Community 34 - "Logos Store Delivery"
Cohesion: 0.10
Nodes (19): ANSWER_EVERY, chain_tick(), delivery_status(), DELIVERY_TIMEOUT, env_or(), HEADS_EVERY, LEZ_RPC, LISTENING (+11 more)

### Community 35 - "LEZ Key Handling"
Cohesion: 0.13
Nodes (18): ACCOUNT_B58, ACCOUNT_ID_PREFIX, h(), INFO, lez_key_and_account_id_match_the_wallet(), lez_public_key(), LEZ_SK, open() (+10 more)

### Community 36 - "Pukaar Lifecycle"
Cohesion: 0.13
Nodes (5): later(), meet_answer(), place(), Pukaar, read()

### Community 37 - "Checkpoint Fuzz Tests"
Cohesion: 0.11
Nodes (8): checkpoint_signs_and_reproduces(), a_blob_is_kept_only_when_hash_and_signature_hold(), an_inline_profile_has_a_cap_and_a_bigger_file_is_not_a_blob(), announce(), blob_of(), every_message_kind_round_trips(), garbage_is_rejected(), size_limits_hold()

### Community 38 - "Profile Announce Tests"
Cohesion: 0.22
Nodes (21): a_full_profile_fits_the_inline_cap(), a_v2_file_still_reads_and_a_new_file_is_v3(), announce_signs_verifies_and_rejects_tampering(), build(), Envelope, envelope_round_trip_for_site_key_and_admin_wrap(), first_profile_is_read_by_the_admin_then_gets_a_site_wrap(), fixed_test_vector() (+13 more)

### Community 39 - "Gossip and Equip"
Cohesion: 0.18
Nodes (16): canonical(), comment(), equip(), gossip(), node(), roles_key(), second_device(), Site (+8 more)

### Community 40 - "Action Body Parsing"
Cohesion: 0.13
Nodes (11): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), HEADS_TO_SETTLE, kind_name(), past(), Restore (+3 more)

### Community 41 - "Intent and Approve"
Cohesion: 0.10
Nodes (16): Intent, Anchor, Approve, CreateSite, Details, Places, Publish, Revoke (+8 more)

### Community 42 - "UI Build Metadata"
Cohesion: 0.09
Nodes (21): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+13 more)

### Community 43 - "Action Queue"
Cohesion: 0.20
Nodes (13): parse_id(), drop_held(), grant(), push_staff_name(), reject(), revoke(), set_staff_name(), super_admin() (+5 more)

### Community 44 - "Identity Encryption"
Cohesion: 0.13
Nodes (12): cipher(), export_identity(), import_identity(), MIN_PASSWORD, NONCE, PREFIX, SALT, VERSION (+4 more)

### Community 45 - "Event Timing"
Cohesion: 0.19
Nodes (11): advance(), create_and_listen(), err(), flush(), now(), on_wire(), publish(), publish_with() (+3 more)

### Community 46 - "Duration and Accounts"
Cohesion: 0.15
Nodes (8): lez_account_id(), the_real_testnet_site_has_the_expected_roles(), an_unreachable_node_is_an_error_not_a_hang(), GAS_LIMIT, imported_key_gives_the_account_of_lezkey(), seed_key(), SEND, temp()

### Community 47 - "Test Context"
Cohesion: 0.25
Nodes (11): build(), Env, fetch_blob(), need_site(), open_ring(), prepare(), prepare_approve(), prepare_publish() (+3 more)

### Community 48 - "Sync and Blobs"
Cohesion: 0.13
Nodes (15): ANSWERERS, BLOB_CHUNK, DELIVERY_CFG, lost_node(), MAX_CHUNKS, MAX_FALLBACK, MAX_RESEND, MAX_WIRE_BYTES (+7 more)

### Community 49 - "Persistence"
Cohesion: 0.20
Nodes (13): account_b58(), join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), remove_if_present() (+5 more)

### Community 50 - "Link State"
Cohesion: 0.12
Nodes (9): Link, Down, Listening, Open, Started, Starting, save(), Shared (+1 more)

### Community 51 - "Meetings UI"
Cohesion: 0.17
Nodes (5): key_arg(), meet(), meet_job(), meet_send(), ms_arg()

### Community 52 - "Deploy Scripts"
Cohesion: 0.24
Nodes (16): deploy(), expect(), fail(), LEE_WALLET_HOME_DIR, mem(), neg(), new_account(), NOW() (+8 more)

### Community 53 - "Join Request Checks"
Cohesion: 0.22
Nodes (11): build(), built_request_parses_and_passes_the_rules(), check_decline(), DECLINE_DOMAIN, decline_message(), keys(), parse(), sign_decline() (+3 more)

### Community 54 - "Admin Actions"
Cohesion: 0.29
Nodes (4): id_reply(), now_ms(), queued(), queued_reply()

### Community 56 - "Event Format Tests"
Cohesion: 0.30
Nodes (12): new_key(), every_text_is_capped_on_receipt(), GOLDEN_REPORT, only_issue_events_and_checkpoints_are_bodies(), report(), round_trip(), size_limits(), tampered_byte_fails_signature() (+4 more)

### Community 58 - "Settlement Flow"
Cohesion: 0.22
Nodes (9): confirm(), effect_on_chain(), KeyState, Failed, Ready, Unknown, send(), step() (+1 more)

### Community 60 - "Queue Landing"
Cohesion: 0.16
Nodes (12): ApproveReq, KEEP_DONE, LAND_WINDOW_MS, MAX_REJECTED, MAX_REQUESTS, PREP_WINDOW_MS, Prof, ProfReq (+4 more)

### Community 61 - "Testnet Scripts"
Cohesion: 0.29
Nodes (13): bal(), check(), die(), fin(), LEE_WALLET_HOME_DIR, log(), mem(), PUKAAR_PAYER (+5 more)

### Community 62 - "History UI Spec"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 63 - "People UI Spec"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 64 - "Checkpoint Records"
Cohesion: 0.33
Nodes (8): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), n_events(), root_for(), verify_anchor()

### Community 65 - "Blob Wire Protocol"
Cohesion: 0.15
Nodes (11): Wire, BlobChunk, BlobReq, Busy, ChainPending, Event, JoinDecline, JoinReq (+3 more)

### Community 66 - "Registry Anchor"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 67 - "Architecture Notes"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

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

### Community 72 - "Keyring Encryption"
Cohesion: 0.40
Nodes (8): add_site_wrap(), content_key(), decrypt(), encrypt(), open(), open_keyring(), SITE_ID, USER

### Community 73 - "Action Kinds"
Cohesion: 0.20
Nodes (10): Action, Anchor, Approve, Revoke, SetProfile, SetRole, SetSite, SuperAdminAdd (+2 more)

### Community 74 - "Board UI Spec"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 75 - "Decode Errors"
Cohesion: 0.22
Nodes (9): DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong, TooLarge (+1 more)

### Community 76 - "Blob Pointers"
Cohesion: 0.22
Nodes (3): Blob, claimed(), Reads

### Community 77 - "Queue States"
Cohesion: 0.22
Nodes (9): St, Done, Held, Landed, Preparing, Queued, Rejected, Sending (+1 more)

### Community 78 - "Write Kinds"
Cohesion: 0.22
Nodes (9): WriteKind, Anchor, Approve, CreateSite, Revoke, SetProfile, SetRole, SetSite (+1 more)

### Community 80 - "Event Subscription"
Cohesion: 0.32
Nodes (4): delivered(), delivered_once(), listen(), open_channel()

### Community 81 - "CI Workflow"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 82 - "Local Anchor Script"
Cohesion: 0.38
Nodes (6): deploy(), LEE_WALLET_HOME_DIR, new_account(), PUKAAR_PAYER, PUKAAR_PROGRAM_ID, local-anchor.sh script

### Community 84 - "Logo Assets"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 85 - "Crate Layout"
Cohesion: 0.33
Nodes (6): pukaar_core, pukaar_logic, pukaar_logic-fuzz, pukaar_sites_guest, pukaar_sites_rules, pukaar_sites_runner

### Community 86 - "Storage Direct Script"
Cohesion: 0.53
Nodes (5): log_after(), pid_of(), refuse_if_running(), storage-direct.sh script, since()

### Community 88 - "CMake Module"
Cohesion: 0.50
Nodes (4): LogosModule.cmake, pukaar_core CMake module, metadata.json, Rust crate (rust-lib)

### Community 90 - "Local Lez Script"
Cohesion: 0.50
Nodes (3): LEE_WALLET_HOME_DIR, LEZ_HOMES, lez-local.sh script

## Knowledge Gaps
- **455 isolated node(s):** `CheckpointRecord`, `CheckpointRecord`, `Accept`, `Cancel`, `Decline` (+450 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 745 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **41 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Issue Events` to `Discovery Tests`, `Checkpoints and Anchoring`, `RPC and Fingerprints`, `About Text Cleaning`, `Identity Import`, `Report Lifecycle`, `Announce Fetching`, `Logos Store Delivery`, `Pukaar Lifecycle`, `Action Body Parsing`, `Action Queue`, `Identity Encryption`, `Event Timing`, `Persistence`, `Link State`, `Meetings UI`, `Admin Actions`, `Chain Role Tests`, `Convergence Tests`, `Action Persistence Tests`?**
  _High betweenness centrality (0.085) - this node is a cross-community bridge._
- **What connects `CheckpointRecord`, `CheckpointRecord`, `Accept` to the rest of the system?**
  _455 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Meetings and Scheduling` be split into smaller, more focused modules?**
  _Cohesion score 0.05504950495049505 - nodes in this community are weakly interconnected._
- **Why does `Rank` connect `Gossip and Equip` to `Places and Write Ops`, `Chain Role Tests`, `Convergence Tests`, `Program Access Control`, `Borsh Instructions`, `Chain Decoding`, `Roles and Actions`, `Approval Table Tests`, `Membership Rules Tests`, `Chain Access`, `Issue Reducer`?**
  _High betweenness centrality (0.050) - this node is a cross-community bridge._
- **Should `Polling and Sync State` be split into smaller, more focused modules?**
  _Cohesion score 0.07998350855493713 - nodes in this community are weakly interconnected._
- **Why does `RuleError` connect `Rule Errors` to `Records and Attestations`, `Anchor Rules`, `Borsh Instructions`, `Roles and Actions`, `Approval Table Tests`, `Balance and Clock Effects`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **Should `Places and Write Ops` be split into smaller, more focused modules?**
  _Cohesion score 0.07418788410886742 - nodes in this community are weakly interconnected._