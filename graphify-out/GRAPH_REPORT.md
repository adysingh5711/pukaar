# Graph Report - field-study  (2026-10-28)

## Corpus Check
- 92 files · ~175,352 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 20 file(s) not represented in the graph (top: (none) 6, .mdc 4, .qml 3)

## Summary
- 2278 nodes · 5800 edges · 114 communities (91 shown, 23 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 440 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `3ecf3b82`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- meet.rs
- queue/tests.rs
- writes.rs
- discovery.rs
- ops/tests.rs
- src/tests.rs
- storage.rs
- ops.rs
- PukaarCoreModule
- records.rs
- pukaar_logic
- RuleError
- rpc.rs
- fetch.rs
- rules/src/lib.rs
- runner/src/main.rs
- Instruction
- chain.rs
- roles.rs
- Node
- rules.rs
- Store
- acct
- src/event.rs
- Effect
- tests/identity.rs
- lifecycle.rs
- profile.rs
- Chain
- refresh
- Fetched
- reducer.rs
- tests/store.rs
- pukaar_core/metadata.json
- rust-lib/src/lib.rs
- lezkey.rs
- Pukaar
- tests/sync.rs
- AccountId
- common/mod.rs
- node.rs
- Intent
- pukaar_ui/metadata.json
- Rank
- src/identity.rs
- now
- chain_live.rs
- Entry
- src/sync.rs
- src/persist.rs
- Shared
- meet
- local.sh
- joinreq.rs
- now_ms
- wallet.rs
- new_key
- Queue
- Env
- cycle
- queue.rs
- testnet.sh
- History screen (resolved/closed archive)
- People screen (07-people.png)
- src/checkpoint.rs
- Wire
- super
- delivery_module
- Status
- review.rs
- convergence.rs
- build-lgx.sh
- util/mod.rs
- Action
- Board kanban columns (Reported, In progress, Resolved, Closed without fix)
- apply_anchor
- Reads
- St
- WriteKind
- tests/persist.rs
- Edit
- CI Workflow (ci)
- local-anchor.sh
- Offline
- Pukaar color logo (PNG)
- pukaar_logic
- storage-direct.sh
- SendError
- pukaar_core CMake module
- Config
- lez-local.sh
- lez-testnet.sh
- build.sh
- basecamp.sh
- check-core-lgx.sh
- demo.sh
- ui-shots.sh
- ui-shots-chain.sh
- window.sh
- status
- JoinBody
- MemberStatus
- NoticeOutcome
- Blob
- example_program_deployment_methods
- example_program_deployment_programs
- fastbuild
- lez-client-gen
- pukaar-registry

## God Nodes (most connected - your core abstractions)
1. `Node` - 109 edges
2. `RuleError` - 76 edges
3. `Queue` - 62 edges
4. `Pukaar` - 51 edges
5. `PukaarCoreModule` - 49 edges
6. `Meetings` - 43 edges
7. `Rank` - 41 edges
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
- **Admin role change via notice and seal** — readme_super_admin, readme_admin_notice, readme_notice_seal [INFERRED 0.75]
- **Checkpoint anchoring chain** — readme_pukaar_core, readme_checkpoint_merkle_root, readme_pukaar_registry, readme_lez [INFERRED 0.75]
- **Tamper-evidence via signed events and fork proofs** — readme_signed_event_envelope, readme_fork_proof, readme_fork_resolution_lowest_id [INFERRED 0.75]
- **Pukaar bell logo variants** — modules_pukaar_ui_src_icons_pukaar_app_icon, public_brand_pukaar_logo_color_logo, public_brand_pukaar_logo_color_svg_logo, public_brand_pukaar_logo_logo, public_brand_pukaar_logo_svg_logo [INFERRED 0.85]

## Communities (114 total, 23 thin omitted)

### Community 0 - "meet.rs"
Cohesion: 0.06
Nodes (50): a_clash_is_named_and_does_not_block_the_proposal(), a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side(), a_newer_summary_wins_and_an_old_one_is_ignored(), a_stranger_a_forgery_and_the_wrong_reader_change_nothing(), Act, Accept, Cancel, Decline (+42 more)

### Community 1 - "queue/tests.rs"
Cohesion: 0.08
Nodes (61): sha256(), derive_lez_secret(), ApproveReq, Poll, Included, Pending, Rejected, a_balance_that_drops_after_prepare_is_caught_at_the_send() (+53 more)

### Community 2 - "writes.rs"
Cohesion: 0.08
Nodes (49): Location, accounts_follow_the_runner_order(), anchor(), apply_place_op(), approve_batch(), approve_batch_names_the_requester_and_the_claims(), auth(), BATCH (+41 more)

### Community 3 - "discovery.rs"
Cohesion: 0.09
Nodes (45): a_checkpoint_from_a_non_member_is_rejected_too(), a_resident_who_is_not_the_reporter_votes_and_staff_do_not(), an_author_who_was_not_a_member_at_the_chain_block_is_rejected_and_kept(), as_rank(), only_staff_acknowledge_claim_and_close(), report(), the_chain_block_decides_not_the_arrival_time(), why() (+37 more)

### Community 4 - "ops/tests.rs"
Cohesion: 0.09
Nodes (47): a_forged_or_unsolicited_decline_is_ignored(), a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download(), a_join_request_is_signed_twice_and_kept(), a_member_page_has_the_profile_the_role_the_joining_time_and_the_pointer_state(), a_member_page_says_loading_none_or_removed(), a_new_site_has_its_files_and_the_first_profile_behind_it(), a_promotion_queues_the_signed_name_with_the_role_change(), a_record_that_is_anchored_already_is_refused_before_anything_is_queued() (+39 more)

### Community 5 - "src/tests.rs"
Cohesion: 0.12
Nodes (39): A, a_call_from_another_program_is_refused(), a_stale_or_future_clock_claim_fails(), a_wrong_member_pda_is_refused(), anchor_needs_an_active_member(), approve_batch_negatives(), approve_batch_writes_members_and_site(), auth() (+31 more)

### Community 6 - "storage.rs"
Cohesion: 0.08
Nodes (28): base(), CALL, CHUNK, Client, config_gets_own_data_dir_and_free_ports(), env_override_wins_and_bad_json_is_an_error(), Events, keep_file() (+20 more)

### Community 7 - "ops.rs"
Cohesion: 0.12
Nodes (27): parse_id(), anchor(), anchor_plan(), anchor_push(), anchor_read(), AnchorPlan, create_site(), drop_held() (+19 more)

### Community 9 - "records.rs"
Cohesion: 0.11
Nodes (17): sha(), ACCOUNT_ID_PREFIX, CID_MAX, GATE_NATIVE, GATE_NFT, GATE_TOKEN, JOIN_BATCH, JOIN_BODY_LEN (+9 more)

### Community 10 - "pukaar_logic"
Cohesion: 0.05
Nodes (41): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+33 more)

### Community 11 - "RuleError"
Cohesion: 0.05
Nodes (44): apply(), apply_effect(), check_balance(), check_clock(), RuleError, AlreadyAnchored, AlreadyMember, AlreadySuperAdmin (+36 more)

### Community 12 - "rpc.rs"
Cohesion: 0.11
Nodes (21): balance(), CLAIM_MARGIN_MS, claim_ms(), clock_ms(), FLOOR_LEPTA, get_account(), HttpRpc, idj() (+13 more)

### Community 13 - "fetch.rs"
Cohesion: 0.11
Nodes (17): about_of(), clean_about(), DETAILS_SCHEMA, is_staff(), MAX_ABOUT, MAX_ANNOUNCES, MAX_ASKED, MAX_BLOB (+9 more)

### Community 14 - "rules/src/lib.rs"
Cohesion: 0.09
Nodes (27): check_ed25519(), check_probe(), checkpoint_seed(), CLOCK_TOLERANCE_MS, CP_SEED_PREFIX, ED_PK, ED_SIG, member_record() (+19 more)

### Community 15 - "runner/src/main.rs"
Cohesion: 0.15
Nodes (18): clock_ms(), Cx, hex32(), id_env(), join_request(), join_request_passes_the_rules_and_both_signatures_verify(), Keys, main() (+10 more)

### Community 16 - "Instruction"
Cohesion: 0.06
Nodes (43): by_hand(), Ctx, Ctx<'a>, nonempty(), pda(), plan(), probe(), set_pointer() (+35 more)

### Community 17 - "chain.rs"
Cohesion: 0.21
Nodes (17): check_version(), checkpoint_pda(), decode_member(), decode_site(), decodes_the_real_site_account(), member(), member_pda(), pda() (+9 more)

### Community 18 - "roles.rs"
Cohesion: 0.20
Nodes (32): check_cid(), active_rank(), Actor, allow(), apply_approve(), apply_approve_site(), apply_create(), apply_revoke() (+24 more)

### Community 19 - "Node"
Cohesion: 0.10
Nodes (9): fingerprint(), Node, about_text(), join(), join_again(), my_pending(), next_version(), request_waits() (+1 more)

### Community 20 - "rules.rs"
Cohesion: 0.12
Nodes (23): A, approve_item_table(), ASHA, check_join_request_table(), create_args(), create_makes_the_creator_super_admin_and_first_member(), create_table(), D (+15 more)

### Community 21 - "Store"
Cohesion: 0.14
Nodes (11): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+3 more)

### Community 22 - "acct"
Cohesion: 0.21
Nodes (19): a_removed_member_cannot_act_until_approved_again(), a_super_admin_adds_one_more_member_and_no_third(), acct(), adds_over_the_limit_keep_the_first(), claim(), mutual_removal_keeps_the_first(), non_admins_cannot_approve(), only_a_super_admin_grants_or_changes_the_admin_role() (+11 more)

### Community 23 - "src/event.rs"
Cohesion: 0.07
Nodes (30): Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm, MarkDuplicate (+22 more)

### Community 24 - "Effect"
Cohesion: 0.09
Nodes (21): Effect, Anchor, AnchorSigner, Approve, ApproveSite, BalanceAtLeast, ClockIs, CreateMember (+13 more)

### Community 25 - "tests/identity.rs"
Cohesion: 0.12
Nodes (26): import_identity(), import_into(), copy_all(), a_failed_import_writes_nothing(), a_peer_naming_a_later_seq_keeps_us_waiting(), a_restored_node_refuses_to_publish_until_its_history_is_back(), an_existing_v1_backup_still_imports(), blob() (+18 more)

### Community 26 - "lifecycle.rs"
Cohesion: 0.12
Nodes (22): stage(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline(), claim_reopen_claim_confirm(), finished_cards_carry_no_pending_next_step(), forbidden_transitions_are_inert_and_visible() (+14 more)

### Community 27 - "profile.rs"
Cohesion: 0.08
Nodes (64): seal(), sealed_box_round_trip_wrong_key_and_libsodium(), x25519_public(), a_full_profile_fits_the_inline_cap(), a_v2_file_still_reads_and_a_new_file_is_v3(), About, add_seats(), add_site_wrap() (+56 more)

### Community 28 - "Chain"
Cohesion: 0.22
Nodes (3): Chain, now_ms(), people_lists_have_the_shape_of_main_qml()

### Community 29 - "refresh"
Cohesion: 0.20
Nodes (16): b58(), program(), a_failed_read_keeps_the_old_cache(), a_node_reads_roles_lists_and_identity_from_the_cache(), a_promoted_or_revoked_author_is_judged_by_the_newest_role(), a_site_that_is_not_on_the_chain_is_an_error(), acct(), at_most_one_refresh_per_ten_seconds() (+8 more)

### Community 30 - "Fetched"
Cohesion: 0.15
Nodes (3): account(), Fetched, Partial

### Community 31 - "reducer.rs"
Cohesion: 0.18
Nodes (11): issue_json(), apply(), CheckpointRecord, Issue, issue_of(), not_permitted(), Progress, progress_by() (+3 more)

### Community 32 - "tests/store.rs"
Cohesion: 0.22
Nodes (10): a_displaced_branch_and_its_descendants_become_evidence(), an_event_for_another_site_is_rejected(), equivocation_is_caught(), held_ids(), lamport_must_increase_along_a_chain(), out_of_order_events_wait_then_apply(), store_with(), the_alternatives_cap_keeps_the_lowest_ids_in_any_order() (+2 more)

### Community 33 - "pukaar_core/metadata.json"
Cohesion: 0.08
Nodes (26): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+18 more)

### Community 34 - "rust-lib/src/lib.rs"
Cohesion: 0.09
Nodes (24): advance(), ANSWER_EVERY, chain_tick(), create_and_listen(), delivered(), delivered_once(), DELIVERY_TIMEOUT, env_or() (+16 more)

### Community 35 - "lezkey.rs"
Cohesion: 0.13
Nodes (14): ACCOUNT_B58, ACCOUNT_ID_PREFIX, h(), INFO, lez_key_and_account_id_match_the_wallet(), LEZ_SK, open(), schnorr_key() (+6 more)

### Community 36 - "Pukaar"
Cohesion: 0.13
Nodes (5): later(), meet_job(), place(), Pukaar, read()

### Community 37 - "tests/sync.rs"
Cohesion: 0.11
Nodes (9): checkpoint_signs_and_reproduces(), a_blob_is_kept_only_when_hash_and_signature_hold(), an_inline_profile_has_a_cap_and_a_bigger_file_is_not_a_blob(), announce(), blob_of(), every_message_kind_round_trips(), garbage_is_rejected(), resend_is_capped() (+1 more)

### Community 38 - "AccountId"
Cohesion: 0.18
Nodes (11): Attestation, GateConfig, GateMode, Attested, OnChain, Member, NftGate, Notice (+3 more)

### Community 39 - "common/mod.rs"
Cohesion: 0.23
Nodes (11): canonical(), comment(), equip(), gossip(), node(), roles_key(), second_device(), Site (+3 more)

### Community 40 - "node.rs"
Cohesion: 0.13
Nodes (11): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), HEADS_TO_SETTLE, kind_name(), past(), Restore (+3 more)

### Community 41 - "Intent"
Cohesion: 0.11
Nodes (15): Intent, Anchor, Approve, CreateSite, Details, Places, Publish, Revoke (+7 more)

### Community 42 - "pukaar_ui/metadata.json"
Cohesion: 0.10
Nodes (20): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+12 more)

### Community 43 - "Rank"
Cohesion: 0.21
Nodes (7): Roster, rank_name(), Rank, Admin, Resident, Steward, SuperAdmin

### Community 44 - "src/identity.rs"
Cohesion: 0.14
Nodes (8): cipher(), export_identity(), MIN_PASSWORD, NONCE, PREFIX, SALT, VERSION, short_password_is_rejected()

### Community 45 - "now"
Cohesion: 0.21
Nodes (5): now(), publish(), publish_with(), wire_chain(), write()

### Community 46 - "chain_live.rs"
Cohesion: 0.25
Nodes (3): the_real_testnet_site_has_the_expected_roles(), main(), run()

### Community 47 - "Entry"
Cohesion: 0.21
Nodes (11): build(), Entry, fetch_blob(), need_site(), open_ring(), prepare(), prepare_approve(), prepare_publish() (+3 more)

### Community 48 - "src/sync.rs"
Cohesion: 0.15
Nodes (18): ANSWERERS, BLOB_CHUNK, DELIVERY_CFG, heads_msg(), MAX_CHUNKS, MAX_FALLBACK, MAX_RESEND, MAX_WIRE_BYTES (+10 more)

### Community 49 - "src/persist.rs"
Cohesion: 0.18
Nodes (13): account_b58(), join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), remove_if_present() (+5 more)

### Community 50 - "Shared"
Cohesion: 0.13
Nodes (8): Link, Down, Listening, Open, Started, Starting, save(), Shared

### Community 51 - "meet"
Cohesion: 0.21
Nodes (5): key_arg(), meet(), meet_answer(), meet_send(), ms_arg()

### Community 52 - "local.sh"
Cohesion: 0.24
Nodes (16): deploy(), expect(), fail(), LEE_WALLET_HOME_DIR, mem(), neg(), new_account(), NOW() (+8 more)

### Community 53 - "joinreq.rs"
Cohesion: 0.21
Nodes (12): build(), built_request_parses_and_passes_the_rules(), check_decline(), DECLINE_DOMAIN, decline_message(), keys(), parse(), sign_decline() (+4 more)

### Community 54 - "now_ms"
Cohesion: 0.20
Nodes (10): delivery_status(), err(), flush(), id_reply(), now_ms(), queued(), queued_reply(), read_with_queue() (+2 more)

### Community 55 - "wallet.rs"
Cohesion: 0.12
Nodes (8): an_unreachable_node_is_an_error_not_a_hang(), GAS_LIMIT, imported_key_gives_the_account_of_lezkey(), POLL, seed_key(), SEND, temp(), WalletWriter

### Community 56 - "new_key"
Cohesion: 0.30
Nodes (12): new_key(), every_text_is_capped_on_receipt(), GOLDEN_REPORT, only_issue_events_and_checkpoints_are_bodies(), report(), round_trip(), size_limits(), tampered_byte_fails_signature() (+4 more)

### Community 57 - "Queue"
Cohesion: 0.13
Nodes (4): remember_known(), AnchorNote, Queue, tick()

### Community 58 - "Env"
Cohesion: 0.18
Nodes (11): ChainWriter, confirm(), effect_on_chain(), Env, KeyState, Failed, Ready, Unknown (+3 more)

### Community 59 - "cycle"
Cohesion: 0.36
Nodes (5): clean(), cycle(), fetch_cycle(), Store, sync_once()

### Community 60 - "queue.rs"
Cohesion: 0.14
Nodes (12): KEEP_DONE, LAND_WINDOW_MS, MAX_REJECTED, MAX_REQUESTS, PREP_WINDOW_MS, Prof, ProfReq, REQUEST_VALID_MS (+4 more)

### Community 61 - "testnet.sh"
Cohesion: 0.29
Nodes (13): bal(), check(), die(), fin(), LEE_WALLET_HOME_DIR, log(), mem(), PUKAAR_PAYER (+5 more)

### Community 62 - "History screen (resolved/closed archive)"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 63 - "People screen (07-people.png)"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 64 - "src/checkpoint.rs"
Cohesion: 0.27
Nodes (9): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+1 more)

### Community 65 - "Wire"
Cohesion: 0.15
Nodes (11): Wire, BlobChunk, BlobReq, Busy, ChainPending, Event, JoinDecline, JoinReq (+3 more)

### Community 66 - "super"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 67 - "delivery_module"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

### Community 68 - "Status"
Cohesion: 0.20
Nodes (8): Status, Acknowledged, AwaitingConfirmation, ClosedWontfix, ConfirmedResolved, Duplicate, InProgress, Open

### Community 69 - "review.rs"
Cohesion: 0.56
Nodes (7): acct(), an_admin_cannot_lock_a_member_out_with_a_huge_profile_version(), an_anchor_signer_key_must_be_the_key_of_the_member(), h(), item(), NOW, site()

### Community 70 - "convergence.rs"
Cohesion: 0.38
Nodes (5): a_forked_author_converges_to_one_branch_in_any_delivery_order(), any_delivery_order_gives_the_same_state(), random_action(), replay(), Rng

### Community 71 - "build-lgx.sh"
Cohesion: 0.40
Nodes (8): busy(), fail(), heartbeat(), out(), run_module(), say(), build-lgx.sh script, status()

### Community 72 - "util/mod.rs"
Cohesion: 0.50
Nodes (4): branch(), event(), first_event(), SITE

### Community 73 - "Action"
Cohesion: 0.20
Nodes (10): Action, Anchor, Approve, Revoke, SetProfile, SetRole, SetSite, SuperAdminAdd (+2 more)

### Community 74 - "Board kanban columns (Reported, In progress, Resolved, Closed without fix)"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 75 - "apply_anchor"
Cohesion: 0.39
Nodes (6): anchor_writes_once(), apply_anchor(), check_anchor(), CheckpointRecord, rec(), record_round_trip_is_168_bytes()

### Community 76 - "Reads"
Cohesion: 0.29
Nodes (4): decode_checkpoint(), body_of(), claimed(), Reads

### Community 77 - "St"
Cohesion: 0.22
Nodes (9): St, Done, Held, Landed, Preparing, Queued, Rejected, Sending (+1 more)

### Community 78 - "WriteKind"
Cohesion: 0.20
Nodes (9): WriteKind, Anchor, Approve, CreateSite, Revoke, SetProfile, SetRole, SetSite (+1 more)

### Community 80 - "Edit"
Cohesion: 0.43
Nodes (7): known_place(), Place, Add, Edit, Remove, Restore, Retire

### Community 81 - "CI Workflow (ci)"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 82 - "local-anchor.sh"
Cohesion: 0.38
Nodes (6): deploy(), LEE_WALLET_HOME_DIR, new_account(), PUKAAR_PAYER, PUKAAR_PROGRAM_ID, local-anchor.sh script

### Community 84 - "Pukaar color logo (PNG)"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 85 - "pukaar_logic"
Cohesion: 0.33
Nodes (6): pukaar_core, pukaar_logic, pukaar_logic-fuzz, pukaar_sites_guest, pukaar_sites_rules, pukaar_sites_runner

### Community 86 - "storage-direct.sh"
Cohesion: 0.53
Nodes (5): log_after(), pid_of(), refuse_if_running(), storage-direct.sh script, since()

### Community 87 - "SendError"
Cohesion: 0.38
Nodes (6): lost_node(), send_queue(), SendError, Refused, Retry, a_message_delivery_did_not_answer_stays_queued_with_everything_after_it()

### Community 88 - "pukaar_core CMake module"
Cohesion: 0.50
Nodes (4): LogosModule.cmake, pukaar_core CMake module, metadata.json, Rust crate (rust-lib)

### Community 90 - "lez-local.sh"
Cohesion: 0.50
Nodes (3): LEE_WALLET_HOME_DIR, LEZ_HOMES, lez-local.sh script

### Community 102 - "status"
Cohesion: 0.33
Nodes (5): last_anchor(), pending_rows(), refresh_items(), status(), a_removed_member_gets_a_reason_and_no_storage_error()

### Community 103 - "JoinBody"
Cohesion: 0.50
Nodes (3): check_join_request(), JoinBody, lez_account_id()

### Community 104 - "MemberStatus"
Cohesion: 0.67
Nodes (3): MemberStatus, Active, Removed

### Community 105 - "NoticeOutcome"
Cohesion: 0.67
Nodes (3): NoticeOutcome, Remove, Steward

## Knowledge Gaps
- **454 isolated node(s):** `name`, `display_name`, `version`, `type`, `interface` (+449 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 715 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **23 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Node` to `writes.rs`, `discovery.rs`, `ops.rs`, `rpc.rs`, `tests/identity.rs`, `lifecycle.rs`, `profile.rs`, `reducer.rs`, `rust-lib/src/lib.rs`, `Pukaar`, `node.rs`, `src/identity.rs`, `now`, `src/persist.rs`, `Shared`, `meet`, `now_ms`, `Queue`, `src/checkpoint.rs`, `convergence.rs`, `tests/persist.rs`, `Edit`, `Config`, `status`?**
  _High betweenness centrality (0.138) - this node is a cross-community bridge._
- **What connects `name`, `display_name`, `version` to the rest of the system?**
  _454 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `meet.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05504950495049505 - nodes in this community are weakly interconnected._
- **Why does `Rank` connect `Rank` to `writes.rs`, `discovery.rs`, `src/tests.rs`, `convergence.rs`, `ops.rs`, `common/mod.rs`, `Reads`, `Instruction`, `roles.rs`, `Node`, `acct`, `Chain`, `reducer.rs`?**
  _High betweenness centrality (0.047) - this node is a cross-community bridge._
- **Should `queue/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07900990099009901 - nodes in this community are weakly interconnected._
- **Why does `RuleError` connect `RuleError` to `JoinBody`, `apply_anchor`, `rules/src/lib.rs`, `Instruction`, `roles.rs`, `acct`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **Should `writes.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0814207650273224 - nodes in this community are weakly interconnected._