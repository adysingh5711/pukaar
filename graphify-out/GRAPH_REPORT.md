# Graph Report - field-study  (2026-10-28)

## Corpus Check
- 5 files · ~176,543 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2345 nodes · 5795 edges · 125 communities (79 shown, 46 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 387 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Community 0
- Community 1
- Community 2
- Community 3
- Community 4
- Community 5
- Community 6
- Community 7
- Community 8
- Community 9
- Community 10
- Community 11
- Community 12
- Community 13
- Community 14
- Community 15
- Community 16
- Community 17
- Community 18
- Community 19
- Community 20
- Community 21
- Community 22
- Community 23
- Community 24
- Community 25
- Community 26
- Community 27
- Community 28
- Community 29
- Community 30
- Community 31
- Community 32
- Community 33
- Community 34
- Community 35
- Community 36
- Community 37
- Community 38
- Community 39
- Community 40
- Community 41
- Community 42
- Community 43
- Community 44
- Community 45
- Community 46
- Community 47
- Community 48
- Community 49
- Community 50
- Community 51
- Community 52
- Community 53
- Community 54
- Community 55
- Community 56
- Community 57
- Community 58
- Community 59
- Community 60
- Community 61
- Community 62
- Community 63
- Community 64
- Community 65
- Community 66
- Community 67
- Community 68
- Community 69
- Community 70
- Community 71
- Community 72
- Community 73
- Community 74
- Community 75
- Community 76
- Community 77
- Community 78
- Community 79
- Community 80
- Community 81
- Community 82
- Community 83
- Community 84
- Community 86
- Community 89
- Community 90
- Community 91
- Community 92
- Community 93
- Community 94
- Community 95
- Community 118
- Community 119
- Community 120
- Community 121
- Community 122

## God Nodes (most connected - your core abstractions)
1. `Node` - 112 edges
2. `RuleError` - 76 edges
3. `Pukaar` - 51 edges
4. `PukaarCoreModule` - 49 edges
5. `Meetings` - 39 edges
6. `Actor` - 35 edges
7. `Rank` - 34 edges
8. `locked()` - 34 edges
9. `node()` - 34 edges
10. `Instruction` - 32 edges

## Surprising Connections (you probably didn't know these)
- `the_margin_claim_passes_the_rules_and_the_old_claim_does_not()` --calls--> `apply_effect()`  [INFERRED]
  modules/pukaar_core/rust-lib/logic/src/queue/tests.rs → programs/pukaar_sites/rules/src/lib.rs
- `approve_item_table()` --calls--> `edit()`  [INFERRED]
  programs/pukaar_sites/rules/tests/rules.rs → modules/pukaar_core/rust-lib/logic/src/rpc.rs
- `create_table()` --calls--> `edit()`  [INFERRED]
  programs/pukaar_sites/rules/tests/rules.rs → modules/pukaar_core/rust-lib/logic/src/rpc.rs
- `join_request_passes_the_rules_and_both_signatures_verify()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs
- `main()` --calls--> `keys()`  [INFERRED]
  programs/pukaar_sites/runner/src/main.rs → modules/pukaar_core/rust-lib/logic/src/joinreq.rs

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Rust logic crate quality checks** — github_workflows_ci_logic_job, github_workflows_ci_fuzz_smoke_job, github_workflows_ci_rust_logic_crate [EXTRACTED 1.00]
- **Admin role change via notice and seal** — readme_super_admin, readme_admin_notice, readme_notice_seal [INFERRED 0.75]
- **Checkpoint anchoring chain** — readme_pukaar_core, readme_checkpoint_merkle_root, readme_pukaar_registry, readme_lez [INFERRED 0.75]
- **Tamper-evidence via signed events and fork proofs** — readme_signed_event_envelope, readme_fork_proof, readme_fork_resolution_lowest_id [INFERRED 0.75]
- **Pukaar bell logo variants** — modules_pukaar_ui_src_icons_pukaar_app_icon, public_brand_pukaar_logo_color_logo, public_brand_pukaar_logo_color_svg_logo, public_brand_pukaar_logo_logo, public_brand_pukaar_logo_svg_logo [INFERRED 0.85]

## Communities (125 total, 46 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.08
Nodes (61): b58(), sha256(), derive_lez_secret(), Poll, Included, Pending, Rejected, a_balance_that_drops_after_prepare_is_caught_at_the_send() (+53 more)

### Community 1 - "Community 1"
Cohesion: 0.06
Nodes (50): a_clash_is_named_and_does_not_block_the_proposal(), a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side(), a_newer_summary_wins_and_an_old_one_is_ignored(), a_stranger_a_forgery_and_the_wrong_reader_change_nothing(), Act, Accept, Cancel, Decline (+42 more)

### Community 2 - "Community 2"
Cohesion: 0.08
Nodes (64): seal(), sealed_box_round_trip_wrong_key_and_libsodium(), x25519_public(), a_full_profile_fits_the_inline_cap(), a_v2_file_still_reads_and_a_new_file_is_v3(), About, add_seats(), add_site_wrap() (+56 more)

### Community 3 - "Community 3"
Cohesion: 0.07
Nodes (40): check_version(), checkpoint_pda(), decode_checkpoint(), decode_member(), decode_site(), decodes_the_real_site_account(), member(), member_pda() (+32 more)

### Community 4 - "Community 4"
Cohesion: 0.06
Nodes (29): about_of(), account(), ANNOUNCE_EVERY, clean(), clean_about(), cycle(), Details, DETAILS_SCHEMA (+21 more)

### Community 5 - "Community 5"
Cohesion: 0.05
Nodes (42): cipher(), export_identity(), import_identity(), import_into(), MIN_PASSWORD, NONCE, PREFIX, SALT (+34 more)

### Community 6 - "Community 6"
Cohesion: 0.11
Nodes (39): a_blob_that_does_not_match_the_chain_hash_is_not_used_and_is_fetched_again(), a_file_of_100_kb_comes_in_chunks_and_reassembles(), a_flood_of_requests_gets_few_answers(), a_join_request_and_an_announce_name_accounts_to_read(), a_keyring_entry_that_lies_about_its_key_is_ignored(), a_lying_author_account_is_rejected(), a_node_that_was_offline_gets_the_announces_it_missed_and_only_valid_newer_ones(), a_removal_reason_comes_from_details_and_must_match_its_hash() (+31 more)

### Community 7 - "Community 7"
Cohesion: 0.12
Nodes (40): A, a_call_from_another_program_is_refused(), a_stale_or_future_clock_claim_fails(), a_wrong_member_pda_is_refused(), anchor_needs_an_active_member(), approve_batch_negatives(), approve_batch_writes_members_and_site(), auth() (+32 more)

### Community 8 - "Community 8"
Cohesion: 0.11
Nodes (43): a_forged_or_unsolicited_decline_is_ignored(), a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download(), a_join_request_is_signed_twice_and_kept(), a_member_page_has_the_profile_the_role_the_joining_time_and_the_pointer_state(), a_member_page_says_loading_none_or_removed(), a_new_site_has_its_files_and_the_first_profile_behind_it(), a_promotion_queues_the_signed_name_with_the_role_change(), a_record_that_is_anchored_already_is_refused_before_anything_is_queued() (+35 more)

### Community 9 - "Community 9"
Cohesion: 0.08
Nodes (28): base(), CALL, CHUNK, Client, config_gets_own_data_dir_and_free_ports(), env_override_wins_and_bad_json_is_an_error(), Events, keep_file() (+20 more)

### Community 10 - "Community 10"
Cohesion: 0.08
Nodes (34): anchor_read(), a_staff_name_is_uploaded_and_sent_as_one_profile_pointer(), a_failed_read_keeps_the_old_cache(), a_node_reads_roles_lists_and_identity_from_the_cache(), a_promoted_or_revoked_author_is_judged_by_the_newest_role(), a_site_that_is_not_on_the_chain_is_an_error(), acct(), balance() (+26 more)

### Community 11 - "Community 11"
Cohesion: 0.11
Nodes (41): Location, accounts_follow_the_runner_order(), anchor(), apply_place_op(), approve_batch(), approve_batch_names_the_requester_and_the_claims(), auth(), BATCH (+33 more)

### Community 12 - "Community 12"
Cohesion: 0.07
Nodes (39): built_request_parses_and_passes_the_rules(), parse(), sha(), ACCOUNT_ID_PREFIX, Attestation, check_join_request(), CID_MAX, GATE_NATIVE (+31 more)

### Community 14 - "Community 14"
Cohesion: 0.05
Nodes (41): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+33 more)

### Community 15 - "Community 15"
Cohesion: 0.07
Nodes (37): apply(), anchor_writes_once(), apply_anchor(), apply_effect(), check_anchor(), check_balance(), check_clock(), check_ed25519() (+29 more)

### Community 16 - "Community 16"
Cohesion: 0.05
Nodes (40): RuleError, AlreadyAnchored, AlreadyMember, AlreadySuperAdmin, BadBalance, BadCid, BadJoinRequest, BadMerkle (+32 more)

### Community 17 - "Community 17"
Cohesion: 0.14
Nodes (18): clock_ms(), Cx, hex32(), id_env(), join_request(), join_request_passes_the_rules_and_both_signatures_verify(), Keys, main() (+10 more)

### Community 18 - "Community 18"
Cohesion: 0.21
Nodes (19): a_removed_member_cannot_act_until_approved_again(), a_super_admin_adds_one_more_member_and_no_third(), acct(), adds_over_the_limit_keep_the_first(), claim(), mutual_removal_keeps_the_first(), non_admins_cannot_approve(), only_a_super_admin_grants_or_changes_the_admin_role() (+11 more)

### Community 19 - "Community 19"
Cohesion: 0.08
Nodes (30): by_hand(), Instruction, Anchor, ApproveBatch, CreateSite, Probe, Reserved, RevokeBatch (+22 more)

### Community 20 - "Community 20"
Cohesion: 0.12
Nodes (28): about_text(), anchor(), anchor_plan(), anchor_push(), AnchorPlan, create_site(), join(), join_again() (+20 more)

### Community 22 - "Community 22"
Cohesion: 0.23
Nodes (29): check_cid(), active_rank(), Actor, allow(), apply_approve(), apply_approve_site(), apply_create(), apply_revoke() (+21 more)

### Community 23 - "Community 23"
Cohesion: 0.14
Nodes (11): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+3 more)

### Community 24 - "Community 24"
Cohesion: 0.09
Nodes (24): Event, advance(), ANNOUNCE_REQ_EVERY, ANSWER_EVERY, chain_tick(), create_and_listen(), delivered(), delivered_once() (+16 more)

### Community 25 - "Community 25"
Cohesion: 0.10
Nodes (24): ANSWERERS, BLOB_CHUNK, DELIVERY_CFG, heads_msg(), lost_node(), MAX_CHUNKS, MAX_FALLBACK, MAX_RESEND (+16 more)

### Community 26 - "Community 26"
Cohesion: 0.12
Nodes (22): stage(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline(), claim_reopen_claim_confirm(), finished_cards_carry_no_pending_next_step(), forbidden_transitions_are_inert_and_visible() (+14 more)

### Community 27 - "Community 27"
Cohesion: 0.12
Nodes (23): A, approve_item_table(), ASHA, check_join_request_table(), create_args(), create_makes_the_creator_super_admin_and_first_member(), create_table(), D (+15 more)

### Community 28 - "Community 28"
Cohesion: 0.13
Nodes (19): apply(), CheckpointRecord, Config, Issue, issue_of(), not_permitted(), Progress, progress_by() (+11 more)

### Community 29 - "Community 29"
Cohesion: 0.12
Nodes (8): an_unreachable_node_is_an_error_not_a_hang(), GAS_LIMIT, imported_key_gives_the_account_of_lezkey(), POLL, seed_key(), SEND, temp(), WalletWriter

### Community 30 - "Community 30"
Cohesion: 0.17
Nodes (8): Ctx, Ctx<'a>, nonempty(), pda(), plan(), probe(), set_pointer(), super_admin()

### Community 31 - "Community 31"
Cohesion: 0.20
Nodes (15): build(), confirm(), effect_on_chain(), Entry, Env, fetch_blob(), need_site(), open_ring() (+7 more)

### Community 32 - "Community 32"
Cohesion: 0.10
Nodes (21): Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm, MarkDuplicate (+13 more)

### Community 33 - "Community 33"
Cohesion: 0.13
Nodes (4): AnchorNote, Queue, RevokeReq, tick()

### Community 34 - "Community 34"
Cohesion: 0.08
Nodes (26): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+18 more)

### Community 35 - "Community 35"
Cohesion: 0.13
Nodes (19): build(), tampering_and_wrong_keys_are_refused(), ACCOUNT_B58, ACCOUNT_ID_PREFIX, h(), INFO, lez_account_id(), lez_key_and_account_id_match_the_wallet() (+11 more)

### Community 36 - "Community 36"
Cohesion: 0.07
Nodes (25): Effect, Anchor, AnchorSigner, Approve, ApproveSite, BalanceAtLeast, ClockIs, CreateMember (+17 more)

### Community 37 - "Community 37"
Cohesion: 0.11
Nodes (9): checkpoint_signs_and_reproduces(), a_blob_is_kept_only_when_hash_and_signature_hold(), an_inline_profile_has_a_cap_and_a_bigger_file_is_not_a_blob(), announce(), blob_of(), every_message_kind_round_trips(), garbage_is_rejected(), resend_is_capped() (+1 more)

### Community 38 - "Community 38"
Cohesion: 0.19
Nodes (4): Chain, now_ms(), people_lists_have_the_shape_of_main_qml(), rank_name()

### Community 39 - "Community 39"
Cohesion: 0.17
Nodes (15): a_displaced_branch_and_its_descendants_become_evidence(), an_event_for_another_site_is_rejected(), equivocation_is_caught(), held_ids(), lamport_must_increase_along_a_chain(), out_of_order_events_wait_then_apply(), store_with(), the_alternatives_cap_keeps_the_lowest_ids_in_any_order() (+7 more)

### Community 40 - "Community 40"
Cohesion: 0.20
Nodes (11): parse_id(), drop_held(), grant(), push_staff_name(), reject(), revoke(), set_staff_name(), spent() (+3 more)

### Community 41 - "Community 41"
Cohesion: 0.13
Nodes (11): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), HEADS_TO_SETTLE, kind_name(), past(), Restore (+3 more)

### Community 42 - "Community 42"
Cohesion: 0.09
Nodes (20): parse_role(), role_code(), role_from_code(), Role, Admin, Resident, Steward, Action (+12 more)

### Community 43 - "Community 43"
Cohesion: 0.09
Nodes (21): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+13 more)

### Community 44 - "Community 44"
Cohesion: 0.16
Nodes (4): later(), place(), Pukaar, read()

### Community 45 - "Community 45"
Cohesion: 0.11
Nodes (8): Link, Down, Listening, Open, Started, Starting, meet_job(), Shared

### Community 46 - "Community 46"
Cohesion: 0.27
Nodes (7): flush(), id_reply(), now_ms(), queued(), queued_reply(), send_queued(), write_tick()

### Community 47 - "Community 47"
Cohesion: 0.12
Nodes (14): Intent, Anchor, Approve, CreateSite, Details, Places, Publish, Revoke (+6 more)

### Community 48 - "Community 48"
Cohesion: 0.24
Nodes (16): deploy(), expect(), fail(), LEE_WALLET_HOME_DIR, mem(), neg(), new_account(), NOW() (+8 more)

### Community 49 - "Community 49"
Cohesion: 0.12
Nodes (11): Wire, AnnounceReq, BlobChunk, BlobReq, Busy, ChainPending, JoinDecline, JoinReq (+3 more)

### Community 50 - "Community 50"
Cohesion: 0.23
Nodes (11): account_b58(), join_site(), leave(), LEFT, left_list(), load(), load_or_create_key(), remove_if_present() (+3 more)

### Community 51 - "Community 51"
Cohesion: 0.18
Nodes (5): key_arg(), meet(), meet_answer(), meet_send(), ms_arg()

### Community 52 - "Community 52"
Cohesion: 0.30
Nodes (12): new_key(), every_text_is_capped_on_receipt(), GOLDEN_REPORT, only_issue_events_and_checkpoints_are_bodies(), report(), round_trip(), size_limits(), tampered_byte_fails_signature() (+4 more)

### Community 53 - "Community 53"
Cohesion: 0.15
Nodes (12): ApproveReq, KEEP_DONE, LAND_WINDOW_MS, MAX_REJECTED, MAX_REQUESTS, PREP_WINDOW_MS, Prof, ProfReq (+4 more)

### Community 54 - "Community 54"
Cohesion: 0.30
Nodes (9): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+1 more)

### Community 55 - "Community 55"
Cohesion: 0.29
Nodes (13): bal(), check(), die(), fin(), LEE_WALLET_HOME_DIR, log(), mem(), PUKAAR_PAYER (+5 more)

### Community 56 - "Community 56"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 57 - "Community 57"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 58 - "Community 58"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 59 - "Community 59"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

### Community 60 - "Community 60"
Cohesion: 0.23
Nodes (6): check_decline(), DECLINE_DOMAIN, decline_message(), keys(), sign_decline(), sizes_are_162_and_290()

### Community 61 - "Community 61"
Cohesion: 0.18
Nodes (8): a_removed_member_gets_a_reason_and_no_storage_error(), a_removed_member_stays_in_the_removed_list_after_the_keyring_rotates(), a_spent_request_is_not_waiting_so_a_removed_member_sees_the_removal(), Offline, removed(), removed_keys(), rotate_keyring(), the_queue_brings_back_a_removed_member_that_no_keyring_lists()

### Community 62 - "Community 62"
Cohesion: 0.27
Nodes (5): body_of(), claimed(), read_site(), Reads, SHORT_OF_FUNDS

### Community 63 - "Community 63"
Cohesion: 0.21
Nodes (5): now(), publish(), publish_with(), wire_chain(), write()

### Community 64 - "Community 64"
Cohesion: 0.40
Nodes (8): busy(), fail(), heartbeat(), out(), run_module(), say(), build-lgx.sh script, status()

### Community 66 - "Community 66"
Cohesion: 0.24
Nodes (6): ChainWriter, KeyState, Failed, Ready, Unknown, send()

### Community 67 - "Community 67"
Cohesion: 0.20
Nodes (9): WriteKind, Anchor, Approve, CreateSite, Revoke, SetProfile, SetRole, SetSite (+1 more)

### Community 68 - "Community 68"
Cohesion: 0.27
Nodes (6): fork_evidence_survives_a_restart_whatever_the_file_order(), leaving_drops_the_replica_but_keeps_the_key(), save_then_load_gives_the_same_state(), err(), save(), start_site()

### Community 69 - "Community 69"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 70 - "Community 70"
Cohesion: 0.22
Nodes (9): DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong, TooLarge (+1 more)

### Community 71 - "Community 71"
Cohesion: 0.22
Nodes (9): St, Done, Held, Landed, Preparing, Queued, Rejected, Sending (+1 more)

### Community 72 - "Community 72"
Cohesion: 0.56
Nodes (7): acct(), an_admin_cannot_lock_a_member_out_with_a_huge_profile_version(), an_anchor_signer_key_must_be_the_key_of_the_member(), h(), item(), NOW, site()

### Community 73 - "Community 73"
Cohesion: 0.25
Nodes (3): delivery_status(), listen(), read_with_queue()

### Community 74 - "Community 74"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 76 - "Community 76"
Cohesion: 0.38
Nodes (6): deploy(), LEE_WALLET_HOME_DIR, new_account(), PUKAAR_PAYER, PUKAAR_PROGRAM_ID, local-anchor.sh script

### Community 77 - "Community 77"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 78 - "Community 78"
Cohesion: 0.33
Nodes (6): pukaar_core, pukaar_logic, pukaar_logic-fuzz, pukaar_sites_guest, pukaar_sites_rules, pukaar_sites_runner

### Community 79 - "Community 79"
Cohesion: 0.53
Nodes (5): log_after(), pid_of(), refuse_if_running(), storage-direct.sh script, since()

### Community 81 - "Community 81"
Cohesion: 0.50
Nodes (4): LogosModule.cmake, pukaar_core CMake module, metadata.json, Rust crate (rust-lib)

### Community 82 - "Community 82"
Cohesion: 0.50
Nodes (4): SuperKind, Add, Remove, Transfer

### Community 84 - "Community 84"
Cohesion: 0.50
Nodes (3): LEE_WALLET_HOME_DIR, LEZ_HOMES, lez-local.sh script

## Knowledge Gaps
- **458 isolated node(s):** `CheckpointRecord`, `CheckpointRecord`, `modules metadata.json version consistency`, `Included`, `Pending` (+453 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 764 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **46 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Community 21` to `Community 3`, `Community 4`, `Community 5`, `Community 6`, `Community 10`, `Community 20`, `Community 24`, `Community 26`, `Community 40`, `Community 41`, `Community 44`, `Community 45`, `Community 46`, `Community 49`, `Community 50`, `Community 51`, `Community 63`, `Community 65`, `Community 68`, `Community 73`?**
  _High betweenness centrality (0.127) - this node is a cross-community bridge._
- **What connects `CheckpointRecord`, `CheckpointRecord`, `modules metadata.json version consistency` to the rest of the system?**
  _458 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.07940594059405941 - nodes in this community are weakly interconnected._
- **Why does `PukaarCoreModule` connect `Community 13` to `Community 24`, `Community 44`?**
  _High betweenness centrality (0.060) - this node is a cross-community bridge._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.05504950495049505 - nodes in this community are weakly interconnected._
- **Why does `Rank` connect `Community 3` to `Community 5`, `Community 7`, `Community 42`, `Community 11`, `Community 18`, `Community 19`, `Community 22`, `Community 28`, `Community 62`?**
  _High betweenness centrality (0.040) - this node is a cross-community bridge._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.07848944835246205 - nodes in this community are weakly interconnected._