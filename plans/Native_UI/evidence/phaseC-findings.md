# Phase C — repair and final verification

Model: GPT-6 / Codex. Date: 2026-10-03. Status: repaired and acceptance complete.

## Result

Restored Phase C in `main.rs` from the preserved `ui/main.patch`, reconciled `jobs.rs`, `runner.rs`, `table.rs` and `audio.rs`, and completed the clone-only live checks. Release build, 92 tests, strict clippy, formatting and source audits pass. Detect changes and Verify both ran through the app. Verify completed with seven flagged turns (CLI exit 1), reloaded its report, and captured successfully. Restore passed a byte-for-byte comparison and created a pre-restore checkpoint. All six fresh captures are in this evidence directory.

The real-project hashes and `git status --porcelain` match the saved baselines exactly, checked at entry, mid-session and after the final acceptance run. No git writes, synthesis, Live/Logic launches, Python source changes, protected design/assets changes, or intentional real-project writes were made. No process from these live checks remains running.

The explicit write boundary overrides the ancestor AGENTS.md Obsidian log destination; this report serves as the session log. The explicit no-git-writes instruction overrides that file's commit requirement. Scratch files were preserved.

## What was repaired

- Restored `mod jobs`, `Browser.jobs`, initialization, Cmd+J/Jobs entry, inspector routing, selection, settings integration, polling, confirmation and capture hooks. The saved patch was applied to a scratch copy of commit `359bf6b` with the ordinary `patch` command; no git mutation was used. The patch's malformed workspace strings were replaced with readable, equivalent copy from the current file.
- Repaired checkpoint handling: planning/copy errors and worker disconnection now prevent launch and report the error. Gemini's `.unwrap_or_default()` / `.ok()` paths discarded these errors and could start a child without a successful checkpoint. Confirmed jobs start only after `checkpoint.json` and the saved files exist.
- Restored one-job-at-a-time gating, cancel behavior, bounded 5,000-line logs, elapsed time, command/source display, and episode reload on both success and failure. Cancellation retains its status after the killed child exits.
- Restored Restore completion and error handling. Current files are checkpointed first; the episode reloads after restoration. Checkpoint creation can be canceled before launch; an in-progress Restore copy is not offered a mid-operation Cancel button.
- Deferred automation until episode loading and command discovery complete. `--confirm` opens a dialog without starting any preset. `--run` accepts only Detect changes and Verify. Active-job screenshots have separate event data so they cannot be mistaken for the final screenshot.
- Command discovery now includes all four commands, runs off the UI thread, and uses the startup login PATH. Export remains wired through the existing B2 API.
- Multi-selection uses visible rows for Shift ranges and maps row positions to manifest turn numbers for argv. Keyboard selection and the selected turn remain included in job scope.
- Preserved explicit checkpoint file plans and optional caller-supplied PrecisionInsert affected turns for Phase D. No Phase D implementation was added.
- Reflowed all long Rust lines, including workspace states, confirmations and checkpoint UI. `awk 'length>120' src/*.rs` prints nothing; there are no string-literal exceptions.

## Gemini claims: independent verdict

| Claim | Measured outcome |
|---|---|
| Pipe inheritance can hang reader joins | Reproduced using a shell child with a descendant retaining stdout/stderr. |
| Removing reader joins fixes the runner correctly | Incomplete. It avoids waiting, but can publish exit before final output and leave reader threads alive. Replaced with nonblocking readers that drain already-written output, stop after parent exit even with inherited pipes, then join before `Exited`. Tests cover inherited descriptors, channel disconnection, 2,000 lines, stderr and an unterminated final line. |
| `audio::is_builtin_speakers` and its tests | Held. Retained the case-insensitive requirement that both `macbook` and `speakers` occur. Name and tick tests cover built-ins, headphones, AirPods, DAC and empty name. |
| 250 ms speaker-safe redraw was integrated | Partially held. The constant/helper existed, but integration was broken and tick-selection coverage was missing. Restored the playing-only 250 ms request; other outputs retain 50 ms. |
| Animated table scrolling is disabled on built-in speakers | Held after call-site reconciliation. The table receives `!on_builtin_speakers`. A headless row-reveal test verifies no continuing immediate repaint requests after selection changes. |
| The three supplied PNGs prove the named states | Did not hold as evidence. In particular, inherited `phaseC-06.png` showed the browser with no confirmation modal. All three were moved from `ui/` to this directory, then replaced by inspected fresh captures. Their original bytes remain under `/tmp/scriptcast-c/repair/`. |
| Verify capture necessarily times out headlessly | Not reproduced. This run saved both active and completed Verify screenshots, then exited normally. There was no 20-second/zero-painted-frames timeout. The earlier App Nap/occlusion explanation remains unproven. |
| Previous acceptance and command-resolution report | Superseded by the actual outputs below. Commands are not direct children of the project root. |

The first redraw regression assertion expected egui's output delay to equal the requested 250 ms. The observed delay was 233.333332 ms, with only the cadence call in `repaint_causes`. egui 0.36 subtracts `predicted_dt` from nonzero requests (`context.rs::request_repaint_after`). The isolated test now sets `predicted_dt=0` and checks the requested cadence without that integration adjustment. No acoustic conclusion is claimed; operator listening remains necessary.

## Command resolution on this machine

Project root: `/Users/emmanuel/Documents/Theory/TheOriginalPower`. Startup logged `Login shell PATH loaded`; no noise or timeout occurred. The login-PATH tests also exercise noisy, failed and timed-out fake shells without altering the real PATH.

| Command | Source | Program / module |
|---|---|---|
| `scriptcast` | Project virtualenv | `<project>/.venv-voice/bin/scriptcast` |
| `scriptcast-verify` | Project virtualenv | `<project>/.venv-voice/bin/scriptcast-verify` |
| `scriptcast-repair` | Project virtualenv | `<project>/.venv-voice/bin/scriptcast-repair` |
| `scriptcast-export` | Dev checkout | `<project>/.venv/bin/python -m scriptcast.tools.export_session` |

Export's selected interpreter successfully imports its entry module/dependencies. Its PYTHONPATH is `/Users/emmanuel/Dev/Tools/scriptCast-worktrees/phase-c`. For the fixture confirmation, discovery uses the checkout and `/Library/Frameworks/Python.framework/Versions/3.11/bin/python3`; that displayed synthesis command was never run.

## Clone live checks

Target: `/tmp/scriptcast-c/outputs/ATO_EP03_five_levels` (canonicalized by the loader to `/private/tmp/...`). All EP03 jobs used `SCRIPTCAST_PROJECT=/Users/emmanuel/Documents/Theory/TheOriginalPower`, with clone output paths. Logs are in `/tmp/scriptcast-c/repair/`.

### Checkpoint timing

Executed only `checkpoint::plan_files(PrecisionInsert)` and `checkpoint::create`, through the ignored Rust test. No Precision insert CLI was launched.

```text
Precision scope: 0.115s, 396 files, 365205757 bytes, /tmp/scriptcast-c/outputs/ATO_EP03_five_levels/_backups/ui/v06-20261003_155321-precision-insert
test ep03_checkpoint_timing_restore ... ok
```

This fresh result agrees with the earlier order of magnitude (0.094 s). It is below the 5-second threshold.

### Detect changes

Command: app `--fresh --mute --inspector jobs --run detect-changes --screenshot plans/Native_UI/evidence/phaseC-05-detect.png` against the clone.

```text
Detected 4 changed turn(s):
  Turn 388: [63:47] Emmanuel Theodore: ...
  Turn 4f523505cf9bd2dd: [deleted from transcript]
  Turn 79e9339dd518836b: [deleted from transcript]
  Turn 15: [01:54] Aisha: ...
scriptcast-ui job: Detect changes · Complete · exit Some(0)
screenshot: plans/Native_UI/evidence/phaseC-05-detect.png 2996x1800
```

Full output: `detect.log`. No confirmation or checkpoint was created by this job. Output streamed and the episode reloaded.

### Verify and checkpoint ordering

Command: app `--fresh --mute --inspector jobs --run verify --running-screenshot plans/Native_UI/evidence/phaseC-05-verify.png --screenshot plans/Native_UI/evidence/phaseC-verify-complete.png` against the clone.

`verify.log` establishes the order:

```text
6: scriptcast-ui checkpoint saved: /private/tmp/scriptcast-c/outputs/ATO_EP03_five_levels/_backups/ui/v07-20261003_155418-verify
7: scriptcast-ui starting: .../.venv-voice/bin/scriptcast-verify /private/tmp/scriptcast-c/outputs/ATO_EP03_five_levels --transcript <real transcript> --json-out /private/tmp/scriptcast-c/outputs/ATO_EP03_five_levels/stress.json
8: running screenshot: plans/Native_UI/evidence/phaseC-05-verify.png
9: scriptcast-ui log (Stdout): note: 2 manifest turn(s) are no longer in the script; the manifest is stale. Re-render or relayout to drop them.
441: ... 7 turn(s) flagged (score < 0.9, a mangled run longer than 2 words, a repetition, or a heteronym spoken wrong). Re-check with a larger model before regenerating:
443: scriptcast-ui job: Verify · Failed(Some(1)) · exit Some(1)
448: screenshot: plans/Native_UI/evidence/phaseC-verify-complete.png 2996x1800
```

`v07/checkpoint.json` records `created_at=2026-10-03T15:54:18.413667-04:00`, preset `verify`, the exact argv, only `stress.json`, and 228173 bytes. The saved stress bytes match the pre-run copy. A separate regression child checks for the published checkpoint and compares its saved stress file before printing its first line; the test then checks the log ordering.

Verify processed 390 matching turns and produced 383 Passed / 7 Failed / 2 Not verified / 0 Missing audio across the 392 manifest turns. Flagged manifest turns: 152, 181, 217, 220, 221, 242, 264. Exit 1 reflects flagged results, not a launch/model/capture failure. The app correctly shows that nonzero status and reloads anyway. `phaseC-04.png` selects failed manifest turn 220, showing real expected/heard text and score 0.588.

The inherited clone already had the same 390-result report, so this deterministic re-run produced identical stress bytes and counts. A fixture regression writes a different valid report, waits for the app's asynchronous reload, and verifies the table's model changes to three NotVerified turns; it then restores the original report. This supplies an actual data-change assertion without manufacturing evidence in EP03.

### Restore

Ran `ep03_restore_verify` through the Rust checkpoint API on the clone after Verify. The fresh Verify checkpoint was restored and current files were saved first as v08:

```text
Restored /tmp/scriptcast-c/outputs/ATO_EP03_five_levels/_backups/ui/v07-20261003_155418-verify byte-for-byte; prior state /tmp/scriptcast-c/outputs/ATO_EP03_five_levels/_backups/ui/v08-20261003_155708-pre-restore
test ep03_restore_verify ... ok
```

`cmp /tmp/scriptcast-c/repair/stress-before-verify.json /tmp/scriptcast-c/outputs/ATO_EP03_five_levels/stress.json` returned 0 with no output. Pre-run stress, newly verified stress, restored stress, v07 saved stress and v08 saved stress all hash to:

```text
970ecc8f4d8c8ded4a267ff67996983ae816c6b8224d0f902dc09813a1d3c1be
```

Because the live verifier reproduced the existing bytes, the fixture regression additionally proves restoration across differing bytes and verifies that pre-restore holds the replaced bytes. Live Restore used the same API as the dialog; physical clicking/feel was not assessed.

## Captures

All are fresh, muted, visually inspected, and 2996×1800 pixels as rendered by this Mac. Exact pixel equivalence to the design's illustrative sample data is not claimed.

- [Detect changes complete](phaseC-05-detect.png)
- [Verify running, checkpoint v07 saved](phaseC-05-verify.png)
- [Verify completed with flagged results](phaseC-verify-complete.png)
- [Verification inspector, failed turn 220](phaseC-04.png)
- [Verify confirmation, not confirmed](phaseC-06.png)
- [Precision insert confirmation on in-sync fixture, not confirmed](phaseC-precision-insert.png)

Confirmation logs contain no `scriptcast-ui starting:` line. The fixture capture shows `Whole episode: only changed turns are synthesized`, the exact command, and the untouched idle job state.

## Real-project baseline checks

Final comparisons after acceptance:

```text
diff -u /tmp/scriptcast-c/real-before.sha /tmp/scriptcast-c/repair-final.sha
(no output; exit 0)
diff -u /tmp/scriptcast-c/real-before.status /tmp/scriptcast-c/repair-final.status
(no output; exit 0)
```

Unchanged real EP03 SHA-256 values:

```text
episode_manifest.json b17ae4104871ee0f46dd4b1a2db5b9b3a8cabf990af638d81b06cf7cf6463837
render_state.json     a0f7223409b93f08a89bd2a30fcd572556c2951ec1afbf90d43b29007fbd4dba
stress.json          19d9f6777a29d46c75880df5ae4e2a4639e237f7f0d9729fc317fb246f258c3b
```

Both complete git-status snapshots hash to `396f176b7e7b169c44bca5d10cb3d61ed7c7cc425f02c6e42b554fe5c8f99cea`. The operator's pre-existing modifications remain exactly as recorded. Worktree `git diff --check` passes, and the protected Python/design/assets path diff is empty.

## Acceptance output

Final run from `ui/`: `cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings`, exit 0. The 4 ignored tests are muted audio hardware, real-data loading, and the two clone-only checkpoint checks; the latter two were explicitly run successfully above. DAW tests use fake/no-op openers and never launch Live or Logic.

```text
   Compiling scriptcast-ui v0.1.0 (/Users/emmanuel/Dev/Tools/scriptCast-worktrees/phase-c/ui)
    Finished `release` profile [optimized] target(s) in 24.43s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/lib.rs (target/debug/deps/scriptcast_ui-83ed919a1ddea271)

running 6 tests
test audio::tests::fixture_two_segment_turn_plays_in_order ... ignored, needs an audio output device; plays muted
test theme::tests::fit_text_width_clamps_between_token_and_cap ... ok
test episode::tests::overview_sections_scale_with_episode_length ... ok
test audio::phase_c_tests::test_is_builtin_speakers ... ok
test audio::tests::audio_info_display_formats_layout_and_khz ... ok
test audio::tests::probe_reads_fixture_wav_headers ... ok

test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/scriptcast_ui-e1f45fcb130c51ba)

running 40 tests
test capture::tests::fresh_alone_suppresses_persistence ... ok
test capture::tests::flags_may_follow_the_episode_dir ... ok
test capture::tests::help_is_recognised ... ok
test capture::tests::double_dash_allows_dash_prefixed_paths ... ok
test capture::tests::no_args_keeps_defaults ... ok
test capture::tests::missing_values_and_unknown_options_fail ... ok
test capture::tests::screenshot_alone_implies_fresh_and_never_persists ... ok
test capture::tests::positional_episode_dir_still_works ... ok
test capture::tests::play_flags_parse_and_validate_combinations ... ok
test capture::tests::screenshot_with_episode_is_not_fresh_but_never_persists ... ok
test capture::tests::screenshot_with_play_is_always_muted ... ok
test capture::tests::select_parses_turn_numbers ... ok
test capture::tests::size_parses_points ... ok
test capture::tests::size_rejects_garbage ... ok
test speaker_redraw_tests::only_builtin_speakers_use_the_slow_tick ... ok
test capture::tests::png_round_trip_has_full_dimensions ... ok
test jobs::tests::cancel_preparation_prevents_launch_and_second_job_is_refused ... ok
test jobs::tests::automation_waits_for_discovery_and_confirmation_never_starts ... ok
test jobs::tests::multi_selection_maps_rows_to_manifest_numbers_and_filters_ranges ... ok
test tests::auto_advance_targets_next_visible_turn_and_stops_at_the_end ... ok
test tests::cli_folder_preferences_round_trip_and_old_preferences_still_load ... ok
test tests::canceled_result_cannot_replace_current_episode ... ok
test tests::daw_progress_completion_and_worker_loss_update_status ... ok
test tests::failed_open_keeps_episode_and_recent_history ... ok
test tests::filters_intersect_and_never_manufacture_verification ... ok
test jobs::tests::checkpoint_failure_and_worker_disconnect_never_spawn ... ok
test tests::inspector_labels_transcript_only_with_real_expected_text ... ok
test tests::space_and_esc_drive_the_transport_state_machine ... ok
test tests::rendered_when_formats_local_time_and_passes_garbage_through ... ok
test timeline::tests::zoom_and_reveal_stay_within_episode ... ok
test view::tests::time_formats_mm_ss_mmm ... ok
test table::tests::columns_keep_at_least_the_design_token_width ... ok
test table::tests::wide_content_grows_columns_beyond_the_tokens ... ok
test tests::empty_columns_fit_the_default_window_without_scrolling ... ok
test speaker_redraw_tests::builtin_auto_advance_scroll_settles_without_repaint_bursts ... ok
test jobs::tests::failed_and_canceled_children_reload_and_preserve_status ... ok
test jobs::tests::checkpoint_is_published_before_child_output_and_restore_reloads ... ok
test jobs::tests::phase_c_panels_and_confirmation_render_with_real_fixture_data ... ok
test tests::inspector_renders_headlessly_in_all_transport_states ... ok
test tests::workspace_states_layout_headlessly_without_panics ... ok

test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

     Running tests/checkpoints.rs (target/debug/deps/checkpoints-a5b81226a081c1cf)

running 6 tests
test ep03_checkpoint_timing_restore ... ignored, explicit scratch-clone checkpoint timing and restore; no synthesis
test ep03_restore_verify ... ignored, restore newest Verify checkpoint on scratch clone only
test readonly_backup_fails_without_changing_sources ... ok
test failed_copy_is_not_published_and_traversal_is_refused ... ok
test all_samples_skip_asd_and_incomplete_checkpoints ... ok
test exact_scopes_versions_and_restore ... ok

test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/cli_resolution.rs (target/debug/deps/cli_resolution-c0a6977542bf693a)

running 3 tests
test four_step_resolution_uses_first_venv_with_command ... ok
test dev_interpreter_skips_missing_dependencies ... ok
test login_path_shell_noise_timeout_and_failure ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

     Running tests/daw_cli.rs (target/debug/deps/daw_cli-8fb78f8aa3a360fa)

running 16 tests
test export_argv_cwd_and_python_environment ... ok
test opener_argv_preserves_paths_with_spaces_without_launching ... ok
test checkout_detection_rejects_other_project_and_uses_first_matching_ancestor ... ok
test freshness_missing_older_equal_newer_and_missing_manifest ... ok
test live_without_als_path_uses_episode_set ... ok
test disabled_reasons_allow_fresh_xml_without_exporter ... ok
test resolver_failure_names_all_three_locations ... ok
test live_prefers_existing_render_state_path_then_falls_back ... ok
test resolver_order_folder_path_checkout_without_changing_environment ... ok
test resolver_skips_non_executable_files_and_empty_setting ... ok
test fresh_xml_skips_export_and_opens_even_if_unavailable ... ok
test logic_opener_runs_off_the_calling_thread ... ok
test missing_app_error_is_reported_using_fake_opener ... ok
test successful_export_without_fresh_output_does_not_open ... ok
test failed_export_reports_stderr_tail_and_never_calls_opener ... ok
test real_export_from_dev_checkout_creates_xml_in_copy_and_reports_success ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

     Running tests/episode_load.rs (target/debug/deps/episode_load-0ce5a61ae38b556a)

running 16 tests
test ep03_loads ... ignored, requires real EP03 directory; read-only
test canceled_load_stops_before_opening_metadata ... ok
test csv_handles_escaped_quotes_newlines_and_rejects_malformed_records ... ok
test fixture_loads_and_joins_by_id_in_chunk_order ... ok
test invalid_manifest_preserves_serde_message ... ok
test missing_manifest_names_file ... ok
test malformed_optional_files_warn_and_keep_browsing ... ok
test project_discovery_and_override ... ok
test missing_optional_files_leave_empty_dependent_fields ... ok
test missing_audio_takes_precedence_without_erasing_verification ... ok
test changed_and_unavailable_script_hashes ... ok
test absolute_render_source_overrides_manifest ... ok
test rejects_invalid_time_bounds ... ok
test overview_dominance_handles_chunks_silence_and_zero_slices ... ok
test stale_verification_does_not_fall_back_to_numeric_turn_indices ... ok
test repetition_is_nullable_text_and_missing_fields_remain_optional ... ok

test result: ok. 15 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/fonts.rs (target/debug/deps/fonts-c378583095068591)

running 1 test
test embedded_fonts_have_real_glyphs_and_render_into_atlas ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/runner_presets.rs (target/debug/deps/runner_presets-8aaf5b3789222141)

running 8 tests
test automation_refuses_every_synthesizing_preset ... ok
test exact_presets_and_no_overwrite ... ok
test metadata_and_hash_gates ... ok
test ranges_and_display_round_trip ... ok
test stdin_is_null_environment_is_injected_and_stderr_drains ... ok
test inherited_pipes_cannot_hold_exit_or_leave_readers_running ... ok
test all_buffered_lines_and_unterminated_tail_precede_exit ... ok
test echo_streams_before_exit_and_sleep_cancels ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

   Doc-tests scriptcast_ui

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

    Checking scriptcast-ui v0.1.0 (/Users/emmanuel/Dev/Tools/scriptCast-worktrees/phase-c/ui)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.89s

```

Additional acceptance checks:

```text
$ cargo fmt --check
(no output; exit 0)
$ grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs
(no matches)
$ grep -rn '"--overwrite"' src
(no matches)
$ grep -rn 'remove_file\|remove_dir' src
src/capture.rs:487:        std::fs::remove_file(&path).unwrap();
$ awk 'length>120' src/*.rs
(no output; no exceptions)
```

## Preserved scratch files — exclude from commit

All 33 inherited scratch files below remain in place, unchanged by this repair. The original `ui/main.patch` remains available; `ui/jobs.patch` is still empty. Additional repair backups and logs live only under `/tmp/scriptcast-c/repair/`.

```text
fix_main_ui.py
ui/check_baselines.sh
ui/fix_auto_run.py
ui/fix_auto_run_proper.py
ui/fix_begin_job.py
ui/fix_capture_logic.py
ui/fix_debug.py
ui/fix_jobs.py
ui/fix_jobs2.py
ui/fix_jobs3.py
ui/fix_jobs4.py
ui/fix_jobs5.py
ui/fix_jobs6.py
ui/fix_jobs7.py
ui/fix_jobs8.py
ui/fix_main_repaint.py
ui/fix_main_repaint2.py
ui/fix_main_table_call.py
ui/fix_main_table_call2.py
ui/fix_minified.py
ui/fix_minified2.py
ui/fix_poll_jobs.py
ui/fix_print_log.py
ui/fix_runner_pipes.py
ui/fix_runner_pipes_2.py
ui/fix_table.py
ui/format_jobs.py
ui/format_main.py
ui/format_main2.py
ui/jobs.patch
ui/main.patch
ui/revert_auto_run.py
ui/test_fmt.rs
```

## Remaining operator checks / next validation ideas

1. Listen for the built-in-speaker whine during auto-advance; software timing checks cannot assess the acoustic effect.
2. Open the bundled app from Finder/Dock to validate login-PATH discovery in that launch context.
3. Assess keyboard, modal and Restore interaction feel in the native window.
4. Run Precision insert only when the operator intends synthesis.
5. Run Regenerate selected only when the operator intends synthesis and the transcript is in sync.
6. Run Repair only when the operator intends synthesis and has reviewed its scope.

No requested implementation or authorized clone check is left pending. These six operator legs were not performed or claimed.
