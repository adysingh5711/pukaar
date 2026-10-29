# Graph Report - field-study  (2026-10-29)

## Corpus Check
- 116 files · ~197,747 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 20 file(s) not represented in the graph (top: (none) 6, .mdc 4, .qml 3)

## Summary
- 2637 nodes · 6916 edges · 125 communities (95 shown, 30 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 492 edges (avg confidence: 0.85)
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
- Community 85
- Community 86
- Community 87
- Community 88
- Community 89
- Community 90
- Community 91
- Community 92
- Community 93
- Community 94
- Community 95
- Community 96
- Community 97
- Community 98
- Community 99
- Community 100
- Community 101
- Community 102
- Community 103
- Community 104
- Community 107
- Community 111
- Community 112
- Community 113
- Community 114
- Community 115
- Community 116
- Community 117
- Community 118
- Community 119
- Community 120
- Community 121
- Community 122
- Community 123

## God Nodes (most connected - your core abstractions)
1. `Node` - 118 edges
2. `RuleError` - 78 edges
3. `ChatBook` - 74 edges
4. `Queue` - 66 edges
5. `Pukaar` - 60 edges
6. `PukaarCoreModule` - 58 edges
7. `Meetings` - 47 edges
8. `Rank` - 40 edges
9. `now_ms()` - 37 edges
10. `Wire` - 36 edges

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
- **Two-key closure flow** — readme_report_lifecycle, readme_two_key_closure, readme_threat_model [EXTRACTED 1.00]
- **Pukaar bell logo variants** — modules_pukaar_ui_src_icons_pukaar_app_icon, public_brand_pukaar_logo_color_logo, public_brand_pukaar_logo_color_svg_logo, public_brand_pukaar_logo_logo, public_brand_pukaar_logo_svg_logo [INFERRED 0.85]

## Communities (125 total, 30 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.05
Nodes (58): b58(), backup_pda(), check_version(), checkpoint_pda(), decode_backup(), decode_member(), decode_site(), decodes_the_real_site_account() (+50 more)

### Community 1 - "Community 1"
Cohesion: 0.08
Nodes (63): sha256(), Poll, Included, Pending, Rejected, a_backup_overwrites_the_pointer_and_is_done_when_the_chain_shows_the_new_cid(), a_backup_that_the_chain_holds_already_is_not_sent_and_waits_for_funds_like_any_write(), a_balance_that_drops_after_prepare_is_caught_at_the_send() (+55 more)

### Community 2 - "Community 2"
Cohesion: 0.08
Nodes (62): seal(), a_full_profile_fits_the_inline_cap(), a_v2_file_still_reads_and_a_new_file_is_v3(), About, add_seats(), add_site_wrap(), Announce, ANNOUNCE_DOMAIN (+54 more)

### Community 3 - "Community 3"
Cohesion: 0.08
Nodes (56): a_forged_or_unsolicited_decline_is_ignored(), a_join_and_an_inline_profile_reach_an_admin_who_approves_with_no_download(), a_join_request_is_signed_twice_and_kept(), a_member_page_has_the_profile_the_role_the_joining_time_and_the_pointer_state(), a_member_page_says_loading_none_or_removed(), a_member_queues_a_backup_pointer_and_a_stranger_does_not(), a_new_site_has_its_files_and_the_first_profile_behind_it(), a_private_profile_shows_other_members_the_name_and_role_only() (+48 more)

### Community 4 - "Community 4"
Cohesion: 0.06
Nodes (29): about_of(), account(), ANNOUNCE_EVERY, clean(), clean_about(), cycle(), Details, DETAILS_SCHEMA (+21 more)

### Community 5 - "Community 5"
Cohesion: 0.12
Nodes (43): A, a_backup_has_negatives(), a_backup_overwrites_the_same_record_of_the_member(), a_call_from_another_program_is_refused(), a_stale_or_future_clock_claim_fails(), a_wrong_member_pda_is_refused(), anchor_needs_an_active_member(), approve_batch_negatives() (+35 more)

### Community 6 - "Community 6"
Cohesion: 0.12
Nodes (38): a_blob_that_does_not_match_the_chain_hash_is_not_used_and_is_fetched_again(), a_file_of_100_kb_comes_in_chunks_and_reassembles(), a_flood_of_requests_gets_few_answers(), a_join_request_and_an_announce_name_accounts_to_read(), a_keyring_entry_that_lies_about_its_key_is_ignored(), a_lying_author_account_is_rejected(), a_node_that_was_offline_gets_the_announces_it_missed_and_only_valid_newer_ones(), a_removal_reason_comes_from_details_and_must_match_its_hash() (+30 more)

### Community 8 - "Community 8"
Cohesion: 0.11
Nodes (46): list_of(), the_module_lists_read_with_float_times_and_either_wrapping(), active(), CALL, Chat, Group, chat_of(), Person (+38 more)

### Community 9 - "Community 9"
Cohesion: 0.10
Nodes (37): about_text(), anchor(), anchor_plan(), anchor_push(), anchor_read(), AnchorPlan, backup_push(), BackupSource (+29 more)

### Community 10 - "Community 10"
Cohesion: 0.10
Nodes (43): Location, accounts_follow_the_runner_order(), anchor(), apply_place_op(), approve_batch(), approve_batch_names_the_requester_and_the_claims(), auth(), BATCH (+35 more)

### Community 11 - "Community 11"
Cohesion: 0.08
Nodes (28): base(), CALL, CHUNK, Client, config_gets_own_data_dir_and_free_ports(), env_override_wins_and_bad_json_is_an_error(), Events, keep_file() (+20 more)

### Community 12 - "Community 12"
Cohesion: 0.09
Nodes (25): Checkpoint, checkpoint_now(), CP_DOMAIN, cp_message(), leaf(), merkle_root(), n_events(), root_for() (+17 more)

### Community 13 - "Community 13"
Cohesion: 0.05
Nodes (40): RuleError, AlreadyAnchored, AlreadyMember, AlreadySuperAdmin, BadBalance, BadCid, BadJoinRequest, BadMerkle (+32 more)

### Community 14 - "Community 14"
Cohesion: 0.13
Nodes (12): AddrBody, after_our_restart_a_live_member_moves_the_group_and_when_all_restarted_the_lowest_with_a_new_address_does(), AskBody, ChatBook, .PREFIX, distinct(), drop_first(), Held (+4 more)

### Community 15 - "Community 15"
Cohesion: 0.06
Nodes (34): AI agent skills index, basecamp skill, lez-framework-template skill, lez-template skill, lgs-cli skill, Anchored checkpoint (checkpoint_json), First anchor record (localnet), Program ID 23365d87...0890 (+26 more)

### Community 16 - "Community 16"
Cohesion: 0.08
Nodes (33): by_hand(), Instruction, Anchor, ApproveBatch, CreateSite, Probe, Reserved, RevokeBatch (+25 more)

### Community 17 - "Community 17"
Cohesion: 0.19
Nodes (18): a_removed_member_cannot_act_until_approved_again(), a_super_admin_adds_one_more_member_and_no_third(), adds_over_the_limit_keep_the_first(), claim(), mutual_removal_keeps_the_first(), non_admins_cannot_approve(), only_a_super_admin_grants_or_changes_the_admin_role(), profile_version_must_grow() (+10 more)

### Community 18 - "Community 18"
Cohesion: 0.08
Nodes (34): a_bad_group_proposal_is_refused(), a_group_meeting_shows_each_answer_and_confirms_by_the_rule(), Act, Accept, Cancel, Decline, an_invitee_who_accepted_can_leave_and_only_their_part_ends(), BACKUP_INFO (+26 more)

### Community 19 - "Community 19"
Cohesion: 0.22
Nodes (31): check_cid(), active_rank(), Actor, allow(), apply_approve(), apply_approve_site(), apply_create(), apply_revoke() (+23 more)

### Community 20 - "Community 20"
Cohesion: 0.08
Nodes (23): advance(), ANNOUNCE_REQ_EVERY, ANSWER_EVERY, chain_tick(), create_and_listen(), delivered(), delivered_once(), DELIVERY_TIMEOUT (+15 more)

### Community 21 - "Community 21"
Cohesion: 0.14
Nodes (11): Accept, Duplicate, Fork, New, Pending, Rejected, has_id(), hold() (+3 more)

### Community 22 - "Community 22"
Cohesion: 0.09
Nodes (27): check_ed25519(), check_probe(), checkpoint_seed(), CLOCK_TOLERANCE_MS, CP_SEED_PREFIX, ED_PK, ED_SIG, member_record() (+19 more)

### Community 23 - "Community 23"
Cohesion: 0.09
Nodes (5): later(), meet_answer(), place(), Pukaar, read()

### Community 24 - "Community 24"
Cohesion: 0.11
Nodes (10): an_unreachable_node_is_an_error_not_a_hang(), GAS_LIMIT, imported_key_gives_the_account_of_lezkey(), POLL, seed_key(), SEND, SEND_RETRY_WAIT, SEND_TRIES (+2 more)

### Community 25 - "Community 25"
Cohesion: 0.11
Nodes (24): ANSWERERS, BLOB_CHUNK, DELIVERY_CFG, heads_msg(), lost_node(), MAX_CHUNKS, MAX_FALLBACK, MAX_RESEND (+16 more)

### Community 26 - "Community 26"
Cohesion: 0.12
Nodes (20): issue_json(), apply(), CheckpointRecord, Config, Issue, issue_of(), not_permitted(), Progress (+12 more)

### Community 27 - "Community 27"
Cohesion: 0.12
Nodes (22): stage(), an_oversize_hindi_report_is_refused_and_nothing_is_inserted(), an_oversize_note_is_refused(), an_unanswered_report_has_no_ack_or_claim_time(), anchored_checkpoint_shows_in_timeline(), claim_reopen_claim_confirm(), finished_cards_carry_no_pending_next_step(), forbidden_transitions_are_inert_and_visible() (+14 more)

### Community 28 - "Community 28"
Cohesion: 0.14
Nodes (24): A, acct(), approve_item_table(), ASHA, check_join_request_table(), create_args(), create_makes_the_creator_super_admin_and_first_member(), create_table() (+16 more)

### Community 29 - "Community 29"
Cohesion: 0.22
Nodes (24): a_conversation_belongs_to_a_person_by_its_sender_and_the_file_keeps_everything(), a_group_is_listed_at_once_with_its_name_and_member_count_and_only_when_it_is_ours(), a_group_message_shows_who_sent_it_by_the_address_book_and_a_short_key_when_unknown(), a_group_that_someone_made_is_ours_once_a_member_we_know_is_in_it(), a_linked_conversation_without_a_message_is_not_in_the_chat_list_until_one_is_queued(), a_member_who_restarted_is_added_again_by_the_lowest_member_only(), a_member_who_restarts_before_their_add_commits_in_a_group_of_two_moves_the_group_at_once(), a_new_group_is_made_once_and_its_fresh_invites_do_not_move_it_again() (+16 more)

### Community 30 - "Community 30"
Cohesion: 0.07
Nodes (27): apply(), apply_effect(), check_balance(), check_clock(), Effect, Anchor, AnchorSigner, Approve (+19 more)

### Community 32 - "Community 32"
Cohesion: 0.10
Nodes (21): sha(), ACCOUNT_ID_PREFIX, BACKUP, check_join_request(), CID_MAX, GATE_NATIVE, GATE_NFT, GATE_TOKEN (+13 more)

### Community 33 - "Community 33"
Cohesion: 0.17
Nodes (8): Ctx, Ctx<'a>, nonempty(), pda(), plan(), probe(), set_pointer(), super_admin()

### Community 34 - "Community 34"
Cohesion: 0.10
Nodes (21): Body, Acknowledge, Checkpoint, ClaimResolved, CloseWontfix, Comment, Confirm, MarkDuplicate (+13 more)

### Community 35 - "Community 35"
Cohesion: 0.15
Nodes (18): at_most_one_refresh_per_ten_seconds(), role_is_the_newest_known_at_any_block(), a_displaced_branch_and_its_descendants_become_evidence(), an_event_for_another_site_is_rejected(), equivocation_is_caught(), held_ids(), lamport_must_increase_along_a_chain(), out_of_order_events_wait_then_apply() (+10 more)

### Community 36 - "Community 36"
Cohesion: 0.12
Nodes (13): action_body(), body_text(), CONFIRM_WAIT_H, deadline(), fingerprint(), HEADS_TO_SETTLE, kind_name(), parse_id() (+5 more)

### Community 37 - "Community 37"
Cohesion: 0.08
Nodes (26): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, codegen, rust, dependencies (+18 more)

### Community 38 - "Community 38"
Cohesion: 0.19
Nodes (11): delivery_status(), err(), flush(), id_reply(), now_ms(), queued(), queued_reply(), read_with_queue() (+3 more)

### Community 39 - "Community 39"
Cohesion: 0.10
Nodes (16): cipher(), export_identity(), import_identity(), MIN_PASSWORD, NONCE, PREFIX, SALT, VERSION (+8 more)

### Community 40 - "Community 40"
Cohesion: 0.18
Nodes (16): clock_ms(), hex32(), id_env(), join_request(), join_request_passes_the_rules_and_both_signatures_verify(), Keys, main(), opt() (+8 more)

### Community 41 - "Community 41"
Cohesion: 0.12
Nodes (17): ACCOUNT_B58, ACCOUNT_ID_PREFIX, h(), INFO, lez_key_and_account_id_match_the_wallet(), lez_public_key(), LEZ_SK, open() (+9 more)

### Community 42 - "Community 42"
Cohesion: 0.10
Nodes (18): Attestation, BackupPointer, GateConfig, GateMode, Attested, OnChain, Member, MemberStatus (+10 more)

### Community 43 - "Community 43"
Cohesion: 0.11
Nodes (9): checkpoint_signs_and_reproduces(), a_blob_is_kept_only_when_hash_and_signature_hold(), an_inline_profile_has_a_cap_and_a_bigger_file_is_not_a_blob(), announce(), blob_of(), every_message_kind_round_trips(), garbage_is_rejected(), only_a_few_peers_answer_each_heads_message() (+1 more)

### Community 44 - "Community 44"
Cohesion: 0.08
Nodes (20): a_new_conversation_takes_no_message_until_the_other_side_has_had_time_to_join(), ADDR_DOMAIN, AFTER, ANSWER_EVERY_MS, ASK_DOMAIN, ASK_EVERY_MS, ASKING_MS, FALLBACK_MS (+12 more)

### Community 45 - "Community 45"
Cohesion: 0.24
Nodes (19): a_clash_is_named_and_does_not_block_the_proposal(), a_custom_reminder_lead_is_kept_zero_means_none_and_too_long_is_refused(), a_custom_time_and_length_is_checked_and_carries_its_own_start_and_end(), a_decline_frees_the_slot_and_a_cancel_reaches_the_other_side(), a_description_is_checked_travels_in_the_proposal_and_shows_on_both_sides(), a_newer_summary_wins_and_an_old_one_is_ignored(), a_row_from_before_groups_reads_as_a_meeting_with_one_invitee(), a_stranger_a_forgery_and_the_wrong_reader_change_nothing() (+11 more)

### Community 46 - "Community 46"
Cohesion: 0.16
Nodes (5): check_desc(), check_len_ms(), Meetings, .PREFIX, Peer

### Community 47 - "Community 47"
Cohesion: 0.16
Nodes (17): import_into(), a_failed_import_writes_nothing(), a_peer_naming_a_later_seq_keeps_us_waiting(), a_restored_node_refuses_to_publish_until_its_history_is_back(), heads(), import_into_a_fresh_device_gives_an_empty_node_with_the_same_identity(), import_is_refused_when_a_site_exists(), note() (+9 more)

### Community 48 - "Community 48"
Cohesion: 0.09
Nodes (21): parse_role(), role_code(), role_from_code(), TRIO, Role, Admin, Resident, Steward (+13 more)

### Community 49 - "Community 49"
Cohesion: 0.20
Nodes (16): comment(), copy_all(), equip(), gossip(), node(), roles_key(), second_device(), Site (+8 more)

### Community 50 - "Community 50"
Cohesion: 0.15
Nodes (14): build(), built_request_parses_and_passes_the_rules(), check_decline(), DECLINE_DOMAIN, decline_message(), keys(), parse(), sign_decline() (+6 more)

### Community 51 - "Community 51"
Cohesion: 0.16
Nodes (11): ChainWriter, confirm(), effect_on_chain(), Env, KeyState, Failed, Ready, Unknown (+3 more)

### Community 52 - "Community 52"
Cohesion: 0.13
Nodes (9): check_lead_min(), key_arg(), keys_arg(), meet(), meet_job(), meet_q(), meet_send(), mins_arg() (+1 more)

### Community 53 - "Community 53"
Cohesion: 0.10
Nodes (20): category, extra_include_dirs, extra_link_libraries, extra_sources, find_packages, dependencies, description, display_name (+12 more)

### Community 54 - "Community 54"
Cohesion: 0.11
Nodes (14): Wire, AnnounceReq, BlobChunk, BlobReq, Busy, ChainPending, ChatAddr, ChatAsk (+6 more)

### Community 55 - "Community 55"
Cohesion: 0.14
Nodes (15): ApproveReq, claimed(), KEEP_DONE, LAND_WINDOW_MS, MAX_REJECTED, MAX_REQUESTS, PREP_WINDOW_MS, Prof (+7 more)

### Community 57 - "Community 57"
Cohesion: 0.11
Nodes (15): Intent, Anchor, Approve, Backup, CreateSite, Details, Places, Publish (+7 more)

### Community 58 - "Community 58"
Cohesion: 0.28
Nodes (3): Clashes, hit(), id_hex()

### Community 59 - "Community 59"
Cohesion: 0.24
Nodes (16): deploy(), expect(), fail(), LEE_WALLET_HOME_DIR, mem(), neg(), new_account(), NOW() (+8 more)

### Community 60 - "Community 60"
Cohesion: 0.31
Nodes (10): build(), Entry, fetch_blob(), need_site(), open_ring(), prepare(), prepare_approve(), prepare_publish() (+2 more)

### Community 61 - "Community 61"
Cohesion: 0.28
Nodes (3): Meeting, settle(), Step

### Community 62 - "Community 62"
Cohesion: 0.25
Nodes (7): a_thread_joins_the_old_and_new_conversations_and_ends_with_the_unsent_messages(), GroupQueued, Msg, ordered(), Queued, row(), thread_messages()

### Community 63 - "Community 63"
Cohesion: 0.30
Nodes (12): new_key(), every_text_is_capped_on_receipt(), GOLDEN_REPORT, only_issue_events_and_checkpoints_are_bodies(), report(), round_trip(), size_limits(), tampered_byte_fails_signature() (+4 more)

### Community 64 - "Community 64"
Cohesion: 0.14
Nodes (8): Link, Down, Listening, Open, Started, Starting, save(), Shared

### Community 66 - "Community 66"
Cohesion: 0.29
Nodes (13): bal(), check(), die(), fin(), LEE_WALLET_HOME_DIR, log(), mem(), PUKAAR_PAYER (+5 more)

### Community 67 - "Community 67"
Cohesion: 0.14
Nodes (14): History filters (search, place, status, when, only my reports), History screen (resolved/closed archive), History table grouped by month with closing notes, Anchored integrity banner with Open Integrity link, Nothing is deleted; issues can be reopened, SLA target breach warnings (acknowledged/fixed late), One-line description, minimal-effort reporting, Place picker list grouped by type with search and group filter (+6 more)

### Community 68 - "Community 68"
Cohesion: 0.15
Nodes (14): People screen (07-people.png), Members list with role chips and actions, Role hierarchy: Resident, Steward, Admin, Super admin, Super admin chip and Transfer super admin, Waiting for approval section with fingerprint, Places and Change log screen (08-places-change-log.png), Change log table (When, Place, Action, By, Reason), Nothing is deleted: retire/hide keeps signed history (+6 more)

### Community 69 - "Community 69"
Cohesion: 0.17
Nodes (4): anchor(), CheckpointRecord, anchor(), CheckpointRecord

### Community 70 - "Community 70"
Cohesion: 0.18
Nodes (13): Checkpoint, CheckpointRecord PDAs, Content topic /pukaar/1/site-<site_hex>/proto, delivery_module, LEZ, Logos Basecamp 0.3.1, Peer device, pukaar_core (+5 more)

### Community 71 - "Community 71"
Cohesion: 0.21
Nodes (13): chat_module 0.3.0, delivery_module 0.4.0, Install Pukaar in Logos Basecamp, Known limits, Logos Basecamp, logos-repo.json (package source), Option 2: Install by hand, Option 1: Add a package source (+5 more)

### Community 72 - "Community 72"
Cohesion: 0.23
Nodes (6): a_member_is_added_once_and_goes_in_only_when_their_address_is_known_and_the_group_is_live(), advance(), Convo, group_convo(), Read, unread()

### Community 73 - "Community 73"
Cohesion: 0.18
Nodes (10): Answer, Accepted, Declined, Left, Pending, Did, Accepted, Cancelled (+2 more)

### Community 74 - "Community 74"
Cohesion: 0.17
Nodes (10): Body, distinct(), Kind, Accept, Cancel, Decline, Propose, ProposeGroup (+2 more)

### Community 75 - "Community 75"
Cohesion: 0.21
Nodes (5): now(), publish(), publish_with(), wire_chain(), write()

### Community 76 - "Community 76"
Cohesion: 0.18
Nodes (3): canonical(), save_then_load_gives_the_same_state(), view()

### Community 77 - "Community 77"
Cohesion: 0.20
Nodes (4): decode_checkpoint(), Blob, read_site(), Reads

### Community 79 - "Community 79"
Cohesion: 0.33
Nodes (8): a_checkpoint_from_a_non_member_is_rejected_too(), a_resident_who_is_not_the_reporter_votes_and_staff_do_not(), an_author_who_was_not_a_member_at_the_chain_block_is_rejected_and_kept(), as_rank(), only_staff_acknowledge_claim_and_close(), report(), the_chain_block_decides_not_the_arrival_time(), why()

### Community 80 - "Community 80"
Cohesion: 0.38
Nodes (5): a_forked_author_converges_to_one_branch_in_any_delivery_order(), any_delivery_order_gives_the_same_state(), random_action(), replay(), Rng

### Community 81 - "Community 81"
Cohesion: 0.40
Nodes (8): busy(), fail(), heartbeat(), out(), run_module(), say(), build-lgx.sh script, status()

### Community 82 - "Community 82"
Cohesion: 0.20
Nodes (9): WriteKind, Anchor, Approve, CreateSite, Revoke, SetProfile, SetRole, SetSite (+1 more)

### Community 83 - "Community 83"
Cohesion: 0.22
Nodes (10): Board kanban columns (Reported, In progress, Resolved, Closed without fix), Older fixed issues, nothing deleted (History link), Overdue and unacknowledged warnings on report cards, Sidebar nav (Board, Report, History, People, Places) with ward selector and delivery status, Report detail side panel with steward actions, Not available to you explainer for disabled actions, Why it isn't closed yet: 3-step closure checklist, Reporter confirmation flow (Confirm it's fixed or Reopen with reason) (+2 more)

### Community 85 - "Community 85"
Cohesion: 0.22
Nodes (9): DecodeError, BadDomain, BadKey, BadSig, BadVersion, Malformed, TextTooLong, TooLarge (+1 more)

### Community 86 - "Community 86"
Cohesion: 0.22
Nodes (9): St, Done, Held, Landed, Preparing, Queued, Rejected, Sending (+1 more)

### Community 87 - "Community 87"
Cohesion: 0.56
Nodes (7): acct(), an_admin_cannot_lock_a_member_out_with_a_huge_profile_version(), an_anchor_signer_key_must_be_the_key_of_the_member(), h(), item(), NOW, site()

### Community 88 - "Community 88"
Cohesion: 0.32
Nodes (8): CI Workflow (ci), cargo-fuzz decode target (60s), cargo test + clippy -D warnings, fuzz-smoke job, logic job, modules metadata.json version consistency, modules/pukaar_core/rust-lib/logic, version job

### Community 89 - "Community 89"
Cohesion: 0.39
Nodes (3): BusyBody, Signed, Signed<T>

### Community 90 - "Community 90"
Cohesion: 0.39
Nodes (6): anchor_writes_once(), apply_anchor(), check_anchor(), CheckpointRecord, rec(), record_round_trip_is_168_bytes()

### Community 92 - "Community 92"
Cohesion: 0.43
Nodes (7): known_place(), Place, Add, Edit, Remove, Restore, Retire

### Community 93 - "Community 93"
Cohesion: 0.38
Nodes (6): deploy(), LEE_WALLET_HOME_DIR, new_account(), PUKAAR_PAYER, PUKAAR_PROGRAM_ID, local-anchor.sh script

### Community 96 - "Community 96"
Cohesion: 0.33
Nodes (6): Status, Cancelled, Confirmed, Declined, status_of(), Proposed

### Community 97 - "Community 97"
Cohesion: 0.47
Nodes (6): Pukaar app icon (bell, orange-magenta gradient, 256px), Pukaar color logo (PNG), Pukaar color logo (SVG), Pukaar mono logo (PNG, dark bell), Pukaar mono logo (SVG, dark bell), Pukaar bell mark (hanging bell on chain link, calling/alert metaphor)

### Community 98 - "Community 98"
Cohesion: 0.33
Nodes (6): pukaar_core, pukaar_logic, pukaar_logic-fuzz, pukaar_sites_guest, pukaar_sites_rules, pukaar_sites_runner

### Community 99 - "Community 99"
Cohesion: 0.53
Nodes (5): log_after(), pid_of(), refuse_if_running(), storage-direct.sh script, since()

### Community 100 - "Community 100"
Cohesion: 0.50
Nodes (4): LogosModule.cmake, pukaar_core CMake module, metadata.json, Rust crate (rust-lib)

### Community 101 - "Community 101"
Cohesion: 0.50
Nodes (4): SuperKind, Add, Remove, Transfer

### Community 103 - "Community 103"
Cohesion: 0.50
Nodes (4): Pointer, Details, Keyring, Places

### Community 104 - "Community 104"
Cohesion: 0.50
Nodes (3): LEE_WALLET_HOME_DIR, LEZ_HOMES, lez-local.sh script

## Knowledge Gaps
- **520 isolated node(s):** `name`, `display_name`, `version`, `type`, `interface` (+515 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 827 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **30 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Community 31` to `Community 0`, `Community 2`, `Community 6`, `Community 8`, `Community 9`, `Community 10`, `Community 12`, `Community 20`, `Community 23`, `Community 26`, `Community 27`, `Community 36`, `Community 38`, `Community 39`, `Community 47`, `Community 52`, `Community 54`, `Community 64`, `Community 75`, `Community 76`, `Community 79`, `Community 80`, `Community 92`?**
  _High betweenness centrality (0.093) - this node is a cross-community bridge._
- **What connects `name`, `display_name`, `version` to the rest of the system?**
  _520 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.05036344755970924 - nodes in this community are weakly interconnected._
- **Why does `Queue` connect `Community 9` to `Community 1`, `Community 3`, `Community 38`, `Community 105`, `Community 60`, `Community 78`, `Community 51`, `Community 20`, `Community 52`, `Community 55`, `Community 56`, `Community 92`?**
  _High betweenness centrality (0.063) - this node is a cross-community bridge._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.0786034353995519 - nodes in this community are weakly interconnected._
- **Why does `ChatBook` connect `Community 14` to `Community 64`, `Community 72`, `Community 8`, `Community 44`, `Community 12`, `Community 20`, `Community 25`, `Community 94`, `Community 91`, `Community 29`, `Community 62`, `Community 95`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.0791476407914764 - nodes in this community are weakly interconnected._