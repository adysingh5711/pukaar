# Graph Report - field-study  (2026-10-28)

## Corpus Check
- 8 files · ~176,824 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2365 nodes · 5729 edges · 143 communities (79 shown, 64 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 310 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- meet module 0
- identity module 1
- rpc module 2
- profile module 3
- fetch module 4
- tests module 5
- discovery module 6
- tests module 7
- tests module 8
- storage module 9
- writes module 10
- lib module 11
- records module 12
- ops module 13
- README module 14
- lib module 15
- lib module 16
- rules module 17
- node module 18
- main module 19
- chain module 20
- lib module 21
- roles module 22
- lib module 23
- store module 24
- lezkey module 25
- event module 26
- lib module 27
- lifecycle module 28
- wallet module 29
- main module 30
- reducer module 31
- rules module 32
- sync module 33
- metadata module 34
- lib module 35
- lib module 36
- sync module 37
- roles module 38
- store module 39
- tests module 40
- node module 41
- queue module 42
- metadata module 43
- sync module 44
- queue module 45
- queue module 46
- queue module 47
- ops module 48
- queue module 49
- persist module 50
- lib module 51
- local module 52
- tests module 53
- joinreq module 54
- event module 55
- persist module 56
- lib module 57
- testnet module 58
- 04-history module 59
- 07-people module 60
- checkpoint module 61
- lib module 62
- architecture module 63
- queue module 64
- chain_roles module 65
- lib module 66
- build-lgx module 67
- 01-board module 68
- event module 69
- queue module 70
- sync module 71
- review module 72
- ci module 73
- queue module 74
- local-anchor module 75
- pukaar-logo module 76
- Cargo module 77
- storage-direct module 78
- CMakeLists module 79
- ops module 80
- main module 81
- lez-local module 82
- lez-testnet module 84
- build module 87
- basecamp module 88
- check-core-lgx module 89
- demo module 90
- ui-shots module 91
- ui-shots-chain module 92
- window module 93
- Cargo module 130
- Cargo module 131
- Cargo module 132
- Cargo module 133
- Cargo module 134

## God Nodes (most connected - your core abstractions)
1. `Node` - 112 edges
2. `RuleError` - 76 edges
3. `Queue` - 62 edges
4. `Pukaar` - 51 edges
5. `PukaarCoreModule` - 49 edges
6. `World` - 40 edges
7. `Meetings` - 39 edges
8. `Actor` - 35 edges
9. `node()` - 34 edges
10. `locked()` - 34 edges

## Surprising Connections (you probably didn't know these)
- `approve_item_table()` --calls--> `edit()`  [INFERRED]
  programs/pukaar_sites/rules/tests/rules.rs → modules/pukaar_core/rust-lib/logic/src/rpc.rs
- `create_table()` --calls--> `edit()`  [INFERRED]
  programs/pukaar_sites/rules/tests/rules.rs → modules/pukaar_core/rust-lib/logic/src/rpc.rs
- `join_request_passes_the_rules_and_both_signatures_verify()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `main()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `run()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Rust logic crate quality checks** — github_workflows_ci_logic_job, github_workflows_ci_fuzz_smoke_job, github_workflows_ci_rust_logic_crate [EXTRACTED 1.00]
- **Admin role change via notice and seal** — readme_super_admin, readme_admin_notice, readme_notice_seal [INFERRED 0.75]
- **Checkpoint anchoring chain** — readme_pukaar_core, readme_checkpoint_merkle_root, readme_pukaar_registry, readme_lez [INFERRED 0.75]
- **Tamper-evidence via signed events and fork proofs** — readme_signed_event_envelope, readme_fork_proof, readme_fork_resolution_lowest_id [INFERRED 0.75]
- **Pukaar bell logo variants** — modules_pukaar_ui_src_icons_pukaar_app_icon, public_brand_pukaar_logo_color_logo, public_brand_pukaar_logo_color_svg_logo, public_brand_pukaar_logo_logo, public_brand_pukaar_logo_svg_logo [INFERRED 0.85]

## Communities (143 total, 64 thin omitted)

### Community 0 - "meet module 0"
Cohesion: 0.06
Nodes (50): a_clash_is_named_and_does_not_block_the_proposal(), a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side(), a_newer_summary_wins_and_an_old_one_is_ignored(), a_stranger_a_forgery_and_the_wrong_reader_change_nothing(), Act, Accept, Cancel, Decline (+42 more)

### Community 1 - "identity module 1"
Cohesion: 0.05
Nodes (49): cipher(), export_identity(), import_identity(), import_into(), MIN_PASSWORD, NONCE, PREFIX, SALT (+41 more)

### Community 2 - "rpc module 2"
Cohesion: 0.07
Nodes (37): a_failed_read_keeps_the_old_cache(), a_node_reads_roles_lists_and_identity_from_the_cache(), a_promoted_or_revoked_author_is_judged_by_the_newest_role(), a_site_that_is_not_on_the_chain_is_an_error(), acct(), balance(), Chain, CLAIM_MARGIN_MS (+29 more)

### Community 3 - "profile module 3"
Cohesion: 0.08
Nodes (60): a_full_profile_fits_the_inline_cap(), a_v2_file_still_reads_and_a_new_file_is_v3(), About, add_seats(), add_site_wrap(), Announce, ANNOUNCE_DOMAIN, announce_signs_verifies_and_rejects_tampering() (+52 more)

### Community 4 - "fetch module 4"
Cohesion: 0.05
Nodes (29): about_of(), account(), ANNOUNCE_EVERY, clean(), clean_about(), cycle(), Details, DETAILS_SCHEMA (+21 more)

### Community 5 - "tests module 5"
Cohesion: 0.09
Nodes (53): a_forged_or_unsolicited_decline_is_ignored(), a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download(), a_join_request_is_signed_twice_and_kept(), a_member_page_has_the_profile_the_role_the_joining_time_and_the_pointer_state(), a_member_page_says_loading_none_or_removed(), a_new_site_has_its_files_and_the_first_profile_behind_it(), a_private_profile_shows_other_members_the_name_and_role_only(), a_promotion_queues_the_signed_name_with_the_role_change() (+45 more)

### Community 6 - "discovery module 6"
Cohesion: 0.12
Nodes (39): a_blob_that_does_not_match_the_chain_hash_is_not_used_and_is_fetched_again(), a_file_of_100_kb_comes_in_chunks_and_reassembles(), a_flood_of_requests_gets_few_answers(), a_join_request_and_an_announce_name_accounts_to_read(), a_keyring_entry_that_lies_about_its_key_is_ignored(), a_lying_author_account_is_rejected(), a_node_that_was_offline_gets_the_announces_it_missed_and_only_valid_newer_ones(), a_removal_reason_comes_from_details_and_must_match_its_hash() (+31 more)

### Community 7 - "tests module 7"
Cohesion: 0.15
Nodes (41): a_balance_that_drops_after_prepare_is_caught_at_the_send(), a_balance_under_the_floor_holds_the_write_with_the_reason(), a_big_profile_goes_by_cid_only(), a_clock_that_goes_backwards_gives_a_lower_claim_and_the_wall_clock_does_not_count(), a_dead_storage_gives_up_after_five_tries(), a_failed_download_keeps_the_entry_preparing_with_its_reason_and_wait(), a_profile_is_sealed_uploaded_and_announced(), a_rejected_write_keeps_its_reason_and_the_next_one_goes() (+33 more)

### Community 8 - "tests module 8"
Cohesion: 0.12
Nodes (40): A, a_call_from_another_program_is_refused(), a_stale_or_future_clock_claim_fails(), a_wrong_member_pda_is_refused(), anchor_needs_an_active_member(), approve_batch_negatives(), approve_batch_writes_members_and_site(), auth() (+32 more)

### Community 9 - "storage module 9"
Cohesion: 0.08
Nodes (28): base(), CALL, CHUNK, Client, config_gets_own_data_dir_and_free_ports(), env_override_wins_and_bad_json_is_an_error(), Events, keep_file() (+20 more)

### Community 10 - "writes module 10"
Cohesion: 0.11
Nodes (40): Location, accounts_follow_the_runner_order(), anchor(), apply_place_op(), approve_batch(), approve_batch_names_the_requester_and_the_claims(), auth(), BATCH (+32 more)

### Community 12 - "records module 12"
Cohesion: 0.07
Nodes (36): sha(), ACCOUNT_ID_PREFIX, Attestation, check_join_request(), CID_MAX, GATE_NATIVE, GATE_NFT, GATE_TOKEN (+28 more)

### Community 13 - "ops module 13"
Cohesion: 0.09
Nodes (33): about_text(), anchor(), anchor_plan(), anchor_push(), anchor_read(), AnchorPlan, create_site(), join() (+25 more)

### Community 14 - "README module 14"
Cohesion: 0.05
Nodes (41): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+33 more)

### Community 15 - "lib module 15"
Cohesion: 0.05
Nodes (40): RuleError, AlreadyAnchored, AlreadyMember, AlreadySuperAdmin, BadBalance, BadCid, BadJoinRequest, BadMerkle (+32 more)

### Community 16 - "lib module 16"
Cohesion: 0.08
Nodes (33): anchor_writes_once(), apply_anchor(), check_anchor(), check_ed25519(), check_probe(), checkpoint_seed(), CheckpointRecord, CLOCK_TOLERANCE_MS (+25 more)

### Community 17 - "rules module 17"
Cohesion: 0.14
Nodes (33): A, a_removed_member_cannot_act_until_approved_again(), a_super_admin_adds_one_more_member_and_no_third(), adds_over_the_limit_keep_the_first(), approve_item_table(), ASHA, body(), check_join_request_table() (+25 more)

### Community 18 - "node module 18"
Cohesion: 0.09
Nodes (3): fingerprint(), issue_json(), Node

### Community 19 - "main module 19"
Cohesion: 0.14
Nodes (18): clock_ms(), Cx, hex32(), id_env(), join_request(), join_request_passes_the_rules_and_both_signatures_verify(), Keys, main() (+10 more)

### Community 20 - "chain module 20"
Cohesion: 0.14
Nodes (21): b58(), check_version(), checkpoint_pda(), decode_checkpoint(), decode_member(), decode_site(), decodes_the_real_site_account(), member() (+13 more)

### Community 21 - "lib module 21"
Cohesion: 0.08
Nodes (30): by_hand(), Instruction, Anchor, ApproveBatch, CreateSite, Probe, Reserved, RevokeBatch (+22 more)

### Community 22 - "roles module 22"
Cohesion: 0.21
Nodes (31): check_cid(), active_rank(), Actor, allow(), apply_approve(), apply_approve_site(), apply_create(), apply_revoke() (+23 more)

### Community 23 - "lib module 23"
Cohesion: 0.08
Nodes (24): advance(), ANNOUNCE_REQ_EVERY, ANSWER_EVERY, chain_tick(), create_and_listen(), delivered(), delivered_once(), DELIVERY_TIMEOUT (+16 more)

### Community 24 - "store module 24"
Cohesion: 0.13
Nodes (11): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+3 more)

### Community 25 - "lezkey module 25"
Cohesion: 0.11
Nodes (23): build(), tampering_and_wrong_keys_are_refused(), ACCOUNT_B58, ACCOUNT_ID_PREFIX, derive_lez_secret(), h(), INFO, lez_account_id() (+15 more)

### Community 26 - "event module 26"
Cohesion: 0.09
Nodes (24): merkle_root(), Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm (+16 more)

### Community 27 - "lib module 27"
Cohesion: 0.07
Nodes (29): apply(), apply_effect(), check_balance(), check_clock(), Effect, Anchor, AnchorSigner, Approve (+21 more)

### Community 28 - "lifecycle module 28"
Cohesion: 0.12
Nodes (22): stage(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline(), claim_reopen_claim_confirm(), finished_cards_carry_no_pending_next_step(), forbidden_transitions_are_inert_and_visible() (+14 more)

### Community 29 - "wallet module 29"
Cohesion: 0.12
Nodes (8): an_unreachable_node_is_an_error_not_a_hang(), GAS_LIMIT, imported_key_gives_the_account_of_lezkey(), POLL, seed_key(), SEND, temp(), WalletWriter

### Community 30 - "main module 30"
Cohesion: 0.17
Nodes (8): Ctx, Ctx<'a>, nonempty(), pda(), plan(), probe(), set_pointer(), super_admin()

### Community 31 - "reducer module 31"
Cohesion: 0.14
Nodes (19): apply(), CheckpointRecord, Config, Issue, issue_of(), not_permitted(), Progress, progress_by() (+11 more)

### Community 32 - "rules module 32"
Cohesion: 0.19
Nodes (10): acct(), create_args(), create_makes_the_creator_super_admin_and_first_member(), create_table(), max_cid(), max_member(), max_site(), records_fit_the_shard_limit() (+2 more)

### Community 33 - "sync module 33"
Cohesion: 0.10
Nodes (21): heads_msg(), resend(), to_resend(), to_resend_own(), Wire, AnnounceReq, BlobChunk, BlobReq (+13 more)

### Community 34 - "metadata module 34"
Cohesion: 0.08
Nodes (26): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+18 more)

### Community 35 - "lib module 35"
Cohesion: 0.19
Nodes (11): delivery_status(), err(), flush(), id_reply(), now_ms(), queued(), queued_reply(), read_with_queue() (+3 more)

### Community 36 - "lib module 36"
Cohesion: 0.12
Nodes (5): later(), meet_answer(), place(), Pukaar, read()

### Community 37 - "sync module 37"
Cohesion: 0.11
Nodes (9): checkpoint_signs_and_reproduces(), a_blob_is_kept_only_when_hash_and_signature_hold(), an_inline_profile_has_a_cap_and_a_bigger_file_is_not_a_blob(), announce(), blob_of(), every_message_kind_round_trips(), garbage_is_rejected(), resend_is_capped() (+1 more)

### Community 38 - "roles module 38"
Cohesion: 0.09
Nodes (23): parse_role(), role_code(), role_from_code(), Role, Admin, Resident, Steward, Action (+15 more)

### Community 39 - "store module 39"
Cohesion: 0.17
Nodes (15): a_displaced_branch_and_its_descendants_become_evidence(), an_event_for_another_site_is_rejected(), equivocation_is_caught(), held_ids(), lamport_must_increase_along_a_chain(), out_of_order_events_wait_then_apply(), store_with(), the_alternatives_cap_keeps_the_lowest_ids_in_any_order() (+7 more)

### Community 40 - "tests module 40"
Cohesion: 0.11
Nodes (8): ApproveReq, DIRS, FakeRpc, FakeStore, joiner(), joiner_with(), PROGRAM, World

### Community 41 - "node module 41"
Cohesion: 0.14
Nodes (11): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), HEADS_TO_SETTLE, kind_name(), past(), Restore (+3 more)

### Community 42 - "queue module 42"
Cohesion: 0.16
Nodes (11): ChainWriter, confirm(), effect_on_chain(), Env, KeyState, Failed, Ready, Unknown (+3 more)

### Community 43 - "metadata module 43"
Cohesion: 0.09
Nodes (21): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+13 more)

### Community 44 - "sync module 44"
Cohesion: 0.12
Nodes (15): ANSWERERS, BLOB_CHUNK, DELIVERY_CFG, lost_node(), MAX_CHUNKS, MAX_FALLBACK, MAX_RESEND, MAX_WIRE_BYTES (+7 more)

### Community 46 - "queue module 46"
Cohesion: 0.10
Nodes (14): Intent, Anchor, Approve, CreateSite, Details, Places, Publish, Revoke (+6 more)

### Community 47 - "queue module 47"
Cohesion: 0.12
Nodes (12): KEEP_DONE, LAND_WINDOW_MS, MAX_REJECTED, MAX_REQUESTS, PREP_WINDOW_MS, Prof, ProfReq, REQUEST_VALID_MS (+4 more)

### Community 48 - "ops module 48"
Cohesion: 0.27
Nodes (9): parse_id(), drop_held(), grant(), push_staff_name(), reject(), revoke(), set_staff_name(), super_admin() (+1 more)

### Community 49 - "queue module 49"
Cohesion: 0.29
Nodes (10): build(), Entry, fetch_blob(), need_site(), open_ring(), prepare(), prepare_approve(), prepare_publish() (+2 more)

### Community 50 - "persist module 50"
Cohesion: 0.21
Nodes (11): account_b58(), join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), remove_if_present() (+3 more)

### Community 51 - "lib module 51"
Cohesion: 0.17
Nodes (5): key_arg(), meet(), meet_job(), meet_send(), ms_arg()

### Community 52 - "local module 52"
Cohesion: 0.24
Nodes (16): deploy(), expect(), fail(), LEE_WALLET_HOME_DIR, mem(), neg(), new_account(), NOW() (+8 more)

### Community 53 - "tests module 53"
Cohesion: 0.16
Nodes (9): a_staff_name_is_uploaded_and_sent_as_one_profile_pointer(), Poll, Included, Pending, Rejected, an_approval_adds_the_seat_and_the_site_wrap(), claim_in(), decode() (+1 more)

### Community 54 - "joinreq module 54"
Cohesion: 0.19
Nodes (9): built_request_parses_and_passes_the_rules(), check_decline(), DECLINE_DOMAIN, decline_message(), keys(), parse(), sign_decline(), sizes_are_162_and_290() (+1 more)

### Community 55 - "event module 55"
Cohesion: 0.30
Nodes (12): new_key(), every_text_is_capped_on_receipt(), GOLDEN_REPORT, only_issue_events_and_checkpoints_are_bodies(), report(), round_trip(), size_limits(), tampered_byte_fails_signature() (+4 more)

### Community 56 - "persist module 56"
Cohesion: 0.16
Nodes (5): fork_evidence_survives_a_restart_whatever_the_file_order(), leaving_drops_the_replica_but_keeps_the_key(), save_then_load_gives_the_same_state(), view(), save()

### Community 57 - "lib module 57"
Cohesion: 0.15
Nodes (7): Link, Down, Listening, Open, Started, Starting, Shared

### Community 58 - "testnet module 58"
Cohesion: 0.29
Nodes (13): bal(), check(), die(), fin(), LEE_WALLET_HOME_DIR, log(), mem(), PUKAAR_PAYER (+5 more)

### Community 59 - "04-history module 59"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 60 - "07-people module 60"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 61 - "checkpoint module 61"
Cohesion: 0.33
Nodes (8): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), n_events(), root_for(), verify_anchor()

### Community 62 - "lib module 62"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 63 - "architecture module 63"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

### Community 64 - "queue module 64"
Cohesion: 0.23
Nodes (5): claimed(), read_site(), Reads, RevokeReq, SHORT_OF_FUNDS

### Community 65 - "chain_roles module 65"
Cohesion: 0.33
Nodes (8): a_checkpoint_from_a_non_member_is_rejected_too(), a_resident_who_is_not_the_reporter_votes_and_staff_do_not(), an_author_who_was_not_a_member_at_the_chain_block_is_rejected_and_kept(), as_rank(), only_staff_acknowledge_claim_and_close(), report(), the_chain_block_decides_not_the_arrival_time(), why()

### Community 66 - "lib module 66"
Cohesion: 0.24
Nodes (5): now(), publish(), publish_with(), wire_chain(), write()

### Community 67 - "build-lgx module 67"
Cohesion: 0.40
Nodes (8): busy(), fail(), heartbeat(), out(), run_module(), say(), build-lgx.sh script, status()

### Community 68 - "01-board module 68"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 69 - "event module 69"
Cohesion: 0.22
Nodes (9): DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong, TooLarge (+1 more)

### Community 70 - "queue module 70"
Cohesion: 0.22
Nodes (9): St, Done, Held, Landed, Preparing, Queued, Rejected, Sending (+1 more)

### Community 71 - "sync module 71"
Cohesion: 0.22
Nodes (9): WriteKind, Anchor, Approve, CreateSite, Revoke, SetProfile, SetRole, SetSite (+1 more)

### Community 72 - "review module 72"
Cohesion: 0.56
Nodes (7): acct(), an_admin_cannot_lock_a_member_out_with_a_huge_profile_version(), an_anchor_signer_key_must_be_the_key_of_the_member(), h(), item(), NOW, site()

### Community 73 - "ci module 73"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 75 - "local-anchor module 75"
Cohesion: 0.38
Nodes (6): deploy(), LEE_WALLET_HOME_DIR, new_account(), PUKAAR_PAYER, PUKAAR_PROGRAM_ID, local-anchor.sh script

### Community 76 - "pukaar-logo module 76"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 77 - "Cargo module 77"
Cohesion: 0.33
Nodes (6): pukaar_core, pukaar_logic, pukaar_logic-fuzz, pukaar_sites_guest, pukaar_sites_rules, pukaar_sites_runner

### Community 78 - "storage-direct module 78"
Cohesion: 0.53
Nodes (5): log_after(), pid_of(), refuse_if_running(), storage-direct.sh script, since()

### Community 79 - "CMakeLists module 79"
Cohesion: 0.50
Nodes (4): LogosModule.cmake, pukaar_core CMake module, metadata.json, Rust crate (rust-lib)

### Community 80 - "ops module 80"
Cohesion: 0.50
Nodes (4): SuperKind, Add, Remove, Transfer

### Community 82 - "lez-local module 82"
Cohesion: 0.50
Nodes (3): LEE_WALLET_HOME_DIR, LEZ_HOMES, lez-local.sh script

## Knowledge Gaps
- **459 isolated node(s):** `CheckpointRecord`, `CheckpointRecord`, `modules metadata.json version consistency`, `Accept`, `Cancel` (+454 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 780 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **64 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `node module 18` to `identity module 1`, `rpc module 2`, `profile module 3`, `fetch module 4`, `chain_roles module 65`, `discovery module 6`, `lib module 66`, `lib module 35`, `node module 41`, `lib module 36`, `ops module 13`, `ops module 48`, `persist module 50`, `lib module 51`, `lib module 23`, `persist module 56`, `lib module 57`, `lifecycle module 28`?**
  _High betweenness centrality (0.103) - this node is a cross-community bridge._
- **What connects `CheckpointRecord`, `CheckpointRecord`, `modules metadata.json version consistency` to the rest of the system?**
  _459 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `meet module 0` be split into smaller, more focused modules?**
  _Cohesion score 0.05504950495049505 - nodes in this community are weakly interconnected._
- **Why does `RuleError` connect `lib module 15` to `rules module 32`, `records module 12`, `lib module 16`, `lib module 21`, `roles module 22`, `lib module 27`?**
  _High betweenness centrality (0.044) - this node is a cross-community bridge._
- **Should `identity module 1` be split into smaller, more focused modules?**
  _Cohesion score 0.05258033106134372 - nodes in this community are weakly interconnected._
- **Why does `Effect` connect `lib module 27` to `lib module 16`, `writes module 10`, `roles module 38`, `roles module 22`?**
  _High betweenness centrality (0.042) - this node is a cross-community bridge._
- **Should `rpc module 2` be split into smaller, more focused modules?**
  _Cohesion score 0.06516105146242132 - nodes in this community are weakly interconnected._