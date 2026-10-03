# Native UI — Implementation Loops

Source plan: [`plan.md`](plan.md)
Source tech plan: [`tech-plan.md`](tech-plan.md)
Product spec: [`ui/SPEC.md`](../../ui/SPEC.md)

This file defines the loops that build `scriptcast-ui` in three phases. Each loop is an implementer prompt followed by a verifier prompt. Run one phase at a time. Do not start Phase B until the Phase A verifier is green and the operator has opened a real episode; do not start Phase C until Phase B playback has been checked by ear.

---

## Active Loops

| Phase | Pattern | Status | Trigger | File/artifact produced |
|---|---|---|---|---|
| A — Episode browser | Implementer + verifier | Not started | Manual | `theme.rs`, `episode.rs`, `lib.rs`, `main.rs` browser (screens 01-02), fixtures, loader tests |
| B — Playback | Implementer + verifier | Blocked on A | Phase A verifier green + operator opened a real episode | `audio.rs`, playback, Selected turn inspector (screen 03) |
| C — Job runner + verify overlay | Implementer + verifier | Blocked on B | Phase B verifier green + listening check | `runner.rs`, `checkpoint.rs`, Verification + Job runner inspectors, Checkpoints list + Restore, confirmation (screens 04-06), preset and checkpoint tests, SPEC.md update |

---

## Prerequisites

- Repo: `file:Dev/Tools/scriptCast/`, branch `dev`. It is its own git root; commits are scoped to `ui/`, `plans/` and `.gitignore`.
- Rust toolchain on PATH (`cargo 1.96.0` verified). `cd ui && cargo build` succeeds on the `cargo new` stub before Phase A.
- Design snapshot present: `file:Dev/Tools/scriptCast/ui/design/` with `DESIGN.md`, `tokens.json`, `screens/01-06`, `components/workspace-state`, `assets/icons`. Every loop reads `DESIGN.md` before writing UI code. **No loop calls the Figma MCP** (Starter plan, ~20 calls a month); a missing value is derived from `tokens.json` or escalated.
- Fonts: OpenDyslexic Regular and Bold plus OpenDyslexicMono are already in `ui/assets/fonts/` with their licences (operator decision: dyslexia-friendly fonts instead of the design's Inter / IBM Plex Mono). Do not substitute other fonts.
- A real episode for live checks: `file:Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels/` (392 turns, 4 speakers, has `stress.json` with 18 entries). Read-only for every phase except Phase C job runs the operator approves.
- For Phase C live checks: a scratch clone of EP03 made with `cp -cR` (APFS clone, near-free) into the loop's scratch directory. Every Phase C live check that runs a job, makes a checkpoint or restores runs against the scratch clone, never the real EP03.
- For Phase C live checks: a Python environment where `scriptcast`, `scriptcast-verify` and `scriptcast-repair` run (the `dev` venv or the Documents install). Its `bin/` directory goes in the UI's bin-dir setting.
- No file under `scriptcast/` (Python) or `tests/` (Python) may change in any phase.

---

## Phase A — Episode Browser

### Implementer prompt

```text
Implement Phase A of the scriptCast Native UI per file:Dev/Tools/scriptCast/plans/Native_UI/plan.md Section 4, steps 1-8. Read, in order: ui/design/DESIGN.md, ui/design/tokens.json, plan.md Observations, tech-plan.md Key Architectural Decisions. Open ui/design/screens/01-no-episode-open and 02-episode-browser-idle (screenshot.png for the target, design-context.tsx for exact values) and components/workspace-state.

Changes:
1. .gitignore — add ui/target/.
2. ui/Cargo.toml — eframe "persistence"; egui_extras 0.36.x with "svg"; toml; sha2; [profile.release] lto = "thin", codegen-units = 1, strip = true. Binary name scriptcast-ui.
3. ui/src/theme.rs — every token from tokens.json as constants; apply(ctx) for dark Visuals and text styles; OpenDyslexic Regular/Bold and OpenDyslexicMono embedded from ui/assets/fonts (400/500 -> Regular, 600 -> Bold) with one FONT_SCALE constant, starting at 1.0 and lowered only if text clips the design's fixed heights; speaker palette; include_image! sources for ui/design/assets/icons; primary/secondary button and field-label helpers. No hex literals outside theme.rs.
4. ui/src/episode.rs — per plan.md step 4, including verification states (Passed, Failed, NotVerified, MissingAudio) and overview_sections.
5. ui/tests/fixtures/ — per plan.md step 5.
6. ui/tests/episode_load.rs — the cases listed in plan.md step 6.
7. ui/src/lib.rs — pub mod episode; pub mod theme.
8. ui/src/main.rs — per plan.md step 8: native window 1440x900, empty workspace with recent episodes (screen 01), header, timeline (overview strip, ruler, speaker lanes, zoom/fit, click-to-select), filter bar, virtualised table with verification column and footer (screen 02), Empty/Loading/Error workspace states, disabled playback bar, status bar. No continuous repaint.

Constraints:
- Read-only: the app must not create, modify or delete any file outside its own eframe storage.
- Real data only: replace every design sample value (DESIGN.md "sample data" section). Turns without a stress.json entry show "Not verified".
- Do not build the Figma "Workshop annotation" bar or draw fake window controls.
- No Python changes. No new dependencies beyond those named.
- cargo build, cargo test, cargo clippy -- -D warnings all pass in ui/.

After implementing, run the tests, then launch `cargo run --release -- <EP03 path>` at 1440x900, take a screenshot of the idle browser and of the empty workspace (start with no argument and no remembered episode), and save them to plans/Native_UI/evidence/phaseA-02.png and phaseA-01.png. Report test output, the measured EP03 load time (log it once with std::time::Instant in debug builds), the FONT_SCALE you settled on, and a list of any visible differences from the design screenshots (glyph shapes and text widths will differ because of OpenDyslexic; layout, colours and sizes must not). Do not proceed to Phase B.
```

### Verifier prompt

```text
Verify Phase A of the scriptCast Native UI.

Checks:
1. Every file in plan.md steps 1-8 exists; .gitignore has ui/target/; OpenDyslexic and OpenDyslexicMono are embedded and actually render (not egui's fallback font).
2. In ui/: cargo build --release, cargo test, cargo clippy -- -D warnings all pass with no warnings.
3. tests/episode_load.rs covers every case in plan.md step 6.
4. grep: no hex colour literals (#xxxxxx or Color32::from_rgb with literals) outside ui/src/theme.rs.
5. Live: open file:Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels. Header shows the real id, engine, 48 kHz, 392 turns, 4 speakers; 4 speaker lanes with the real names; the last segment ends at the last turn's end_ms; most turns read "Not verified" and exactly the failed stress.json entries read "Failed".
6. Live: search narrows the table; the speaker filter dims other segments; Failed only shows only failed turns; clicking a segment selects the matching row.
7. Live: opening a directory without episode_manifest.json shows the Error state; if EP03 was open, it stays open and usable.
8. Design fidelity: compare plans/Native_UI/evidence/phaseA-01.png and phaseA-02.png with ui/design/screens/01-*/screenshot.png and 02-*/screenshot.png. Region heights (header 76, timeline 188, filter bar 42, header row 30, rows 28, playback 64, status 26), colours, icon placement and button styles match; no text clips vertically or overlaps a neighbour. List any mismatch; text content and glyph shapes are expected to differ (real data, OpenDyslexic).
9. EP03 load under 50 ms (release); scrolling the full table stays smooth; CPU ~0% when idle (Activity Monitor).
10. git status shows no changes under scriptcast/ or tests/ (Python) and no new files in the EP03 directory; no Figma MCP calls were made.

Record results in STATE.md. If any check fails, attempt a fix (max 3 attempts per check), then escalate to the operator.
```

---

## Phase B — Playback

### Implementer prompt

```text
Implement Phase B of the scriptCast Native UI per file:Dev/Tools/scriptCast/plans/Native_UI/plan.md Section 4, steps 9-10. Re-read ui/design/DESIGN.md (Playback bar, Inspector "Selected turn", Keyboard) and open ui/design/screens/03-selected-turn-playing plus the Missing audio and Unavailable device states in components/workspace-state.

Changes:
9. ui/src/audio.rs — AudioOut over rodio 0.22 per plan.md step 9: open default or named output (rodio::speakers::available_outputs, SpeakersBuilder::device(..).default_config()?.open_mixer()), outputs(), play, pause, resume, stop, is_paused, is_finished (Player::empty), position (Player::get_pos), playing().
10. ui/src/main.rs — per plan.md step 10: Space play/pause, Esc stop, Up/Down selection with scroll-into-view, immediate switch on selection, Play/Pause + Stop buttons, current-audio time, seek track, auto-advance (persisted), output device menu (persisted), playhead, playing row, Selected turn inspector (376), Missing audio and Unavailable device states. request_repaint_after(50 ms) only while playing.

Constraints:
- No changes to episode.rs beyond what playback strictly needs (none expected).
- Still read-only over episode files. Still no hex literals outside theme.rs.
- cargo build, cargo test, cargo clippy -- -D warnings all pass.

Add unit tests for non-audio logic you factor out (selection movement over a filtered list, auto-advance target, time formatting mm:ss.mmm). Capture plans/Native_UI/evidence/phaseB-03.png with a turn playing. Report results. Do not proceed to Phase C.
```

### Verifier prompt

```text
Verify Phase B of the scriptCast Native UI.

Checks:
1. audio.rs exists; build/test/clippy green.
2. Live (EP03): select a turn, Space plays it; Space pauses and resumes; Esc stops.
3. Live: while playing, click another turn — the first stops, the second starts, no overlap.
4. Live: a multi-segment turn (any turn with more than one entry in segments) plays all chunks in order; the time readout and seek fill advance.
5. Live: auto-advance on — playback continues into the next visible turn, respecting the active filter.
6. Live: typing a space in the search box does not start playback.
7. Live: in a COPY of the fixture episode with one WAV renamed, that row shows Missing audio and other turns still play.
8. Live: switch the output device in the menu; choosing a device that disappears (unplug headphones) pauses playback, keeps position and selection, and shows Unavailable device.
9. Design fidelity: compare plans/Native_UI/evidence/phaseB-03.png with ui/design/screens/03-*/screenshot.png (inspector 376 wide, playhead, playing row, transport, seek track).
10. Idle CPU ~0% when nothing is playing.

The operator must confirm checks 2-5 and 8 by ear. Record results in STATE.md. Max 3 fix attempts per check before escalation.
```

---

## Phase C — Job Runner and Verify Overlay

### Implementer prompt

```text
Implement Phase C of the scriptCast Native UI per file:Dev/Tools/scriptCast/plans/Native_UI/plan.md Section 4, steps 11-17. Re-read ui/design/DESIGN.md (Inspector Verification and Job runner, Confirmation dialog, Checkpoints, sample-data rules) and open ui/design/screens/04, 05 and 06 plus the CLI unavailable state.

Changes:
11. ui/src/runner.rs — Preset enum; JobSpec::from_preset exactly as in plan.md step 11 (DetectChanges, PrecisionInsert, RegenerateTurns with compacted ranges, Verify with --json-out stress.json, Repair with --voices from scriptcast.toml); JobSpec::display() for the exact shell-quoted command. Never emit --overwrite. Turn-targeted presets error unless script_in_sync == Some(true). Job::start: cwd = project root, stdin null, piped stdout/stderr, one reader thread per stream sending JobEvent over mpsc and calling ctx.request_repaint(); cancel() and Drop kill the child; JobState.
12. ui/src/checkpoint.rs — plan_files, create, list, restore exactly as in plan.md step 12: clones with std::fs::copy into <episode>/_backups/ui/vNN-<YYYYmmdd_HHMMSS>-<preset>/, checkpoint.json written last, .asd skipped, restore takes a pre-restore checkpoint first and replaces files via temp name + rename, nothing is ever deleted.
13. ui/src/episode.rs — voices_path from scriptcast.toml (toml crate); stress.json joined by turn_id; stress_mtime.
14. ui/src/main.rs — Verification inspector (436), Job runner inspector (510) opened by a filter-bar "Jobs" button and ⌘J, confirmation dialog (540, scrim) with the exact argv and "A checkpoint is saved first.", checkpoint created after Confirm and before Job::start (job never starts if it fails), active job subtitle "· checkpoint vNN saved", Checkpoints list with Restore and its own confirmation, CLI unavailable state; persisted bin-dir; multi-select; indeterminate progress; bounded log (5,000 lines); reload on exit and after restore. Render real stress[] entries, never the design's sample categories.
15. ui/tests/runner_presets.rs — cases in plan.md step 15.
16. ui/tests/checkpoints.rs — cases in plan.md step 16.
17. ui/SPEC.md — presets replace "render, verify, repair, mix"; mix deferred (needs --config and --asr); bin-dir decision; checkpoints and Restore as the one exception to read-only; OpenDyslexic; link ui/design/DESIGN.md.

Constraints:
- The UI writes only checkpoints under <episode>/_backups/ui/ and, through a confirmed Restore, the files a checkpoint holds. It never deletes anything. All other writes come from launched CLIs.
- No Python changes. No hex literals outside theme.rs.
- cargo build, cargo test, cargo clippy -- -D warnings all pass.

Live checks run against a scratch clone of EP03 (cp -cR into your scratch directory), never the real EP03 directory. Without operator approval, run only Detect changes and Verify on the scratch clone, then Restore the Verify checkpoint and confirm stress.json is back byte for byte. Time the creation of a Precision-insert-scope checkpoint on the scratch clone (call checkpoint::create directly from a test binary or debug command; do not run the CLI) and record the time. Do NOT run Precision insert, Regenerate or Repair — those synthesize audio and need a human gate. Capture evidence/phaseC-04.png, phaseC-05.png (a Verify job running) and phaseC-06.png (the Verify confirmation). Report results.
```

### Verifier prompt

```text
Verify Phase C of the scriptCast Native UI.

Checks:
1. Files from plan.md steps 11-17 exist; build/test/clippy green.
2. runner_presets.rs and checkpoints.rs pass; grep confirms the string "--overwrite" appears nowhere in ui/src/, and no remove_file / remove_dir calls exist in ui/src/ outside tests.
3. Every Job is spawned with Stdio::null() for stdin (read runner.rs).
4. Live (scratch clone of EP03, bin dir set): Detect changes streams output and ends with an exit code; no confirmation and no checkpoint for it.
5. Live: Verify shows a confirmation with the exact command line and the checkpoint note; after Confirm, _backups/ui/vNN-*-verify/ holds stress.json and checkpoint.json before the CLI starts; output streams; reload on exit updates the verification column.
6. Live: Restore that checkpoint — confirmation lists stress.json; afterwards stress.json matches the checkpoint byte for byte and a pre-restore checkpoint exists.
7. Live: make the scratch clone's _backups read-only and press Verify — the job does not start and the error names the path.
8. Live: Cancel during Verify kills the process within 1 s (ps shows no scriptcast-verify left); state reads Canceled.
9. Live: with a script edited after render (scratch copy only), Regenerate selected is disabled with the guard message.
10. Live: wrong bin dir shows the CLI unavailable state; presets are disabled; browsing and playback keep working.
11. Verification inspector shows real score, expected, heard and stress[] entries for a failed EP03 turn; Failed only shows exactly the entries with failed == true.
12. Design fidelity: compare evidence/phaseC-04/05/06.png with ui/design/screens/04, 05, 06 (inspector widths 436/510, dialog 540, scrim, button styles, log well).
13. SPEC.md reflects the presets, the mix deferral, checkpoints, the font and the design link.
14. git status: no changes under scriptcast/ or tests/ (Python); the real EP03 directory has no new files and an unchanged stress.json (shasum before and after).

Record results in STATE.md (including the measured checkpoint time) and fill plan.md Section 5 Plan Audit. Max 3 fix attempts per check before escalation.
```

---

## Human Gates

- No Phase B work until the Phase A verifier reports all checks passed and the operator has opened EP03 in the app.
- No Phase C work until the operator confirms Phase B playback by ear.
- Any job that synthesizes audio (Precision insert, Regenerate selected, Repair) runs against real episodes only with the operator present and approving the confirmation dialog. Loops never click Confirm on these.
- Restore against a real episode (not a scratch clone) only with the operator present. Loops restore only on scratch clones.
- ElevenLabs-voiced speakers spend credits; any synthesizing job on an episode with an `elevenlabs` speaker needs explicit operator approval in chat, not just the in-app dialog.
- No Python file changes in any phase; any diff under `scriptcast/` or `tests/` is an automatic fail.
- No push and no merge to `main` or `v0.2-video-tooling` without the operator.
- No loop calls the Figma MCP. If a needed value is not in `ui/design/`, derive it from `tokens.json` and note it, or escalate; the operator decides whether to spend a Figma call.
- Operator decisions are recorded in STATE.md (fonts: OpenDyslexic; checkpoints: in scope; Jobs entry: plan default).

---

## State Tracking

Track in `file:Dev/Tools/scriptCast/plans/Native_UI/STATE.md`:

```markdown
## Native UI Implementation State

### Phase A — Episode browser
- Last run: YYYY-MM-DD HH:MM UTC
- Status: in_progress | passed | blocked
- Test status: ...

### Phase B — Playback
- Last run: ...
- Status: ...

### Phase C — Job runner + verify overlay
- Last run: ...
- Status: ...

### Blockers
- ...
```

---

## Failure Modes & Circuit Breakers

| Failure | Response |
|---|---|
| Same check fails 3 times across loop runs | Escalate to the operator; do not auto-retry |
| eframe 0.36 API differs from the plan (persistence, run_native, modal) | Adapt to the installed API and note it in Plan Audit; do not downgrade eframe without operator approval |
| rodio cannot open a device in the loop's environment | Mark Phase B live checks as operator-only; unit tests still required |
| A job hangs in a live check | Cancel; if the process survives, kill it and record the command in STATE.md |
| A live check would modify a real episode beyond stress.json | Stop; that is a human-gate violation |
| Clippy or dependency churn from a crate update | Pin the versions that worked in Cargo.lock; never `cargo update` mid-phase |
| Live screenshot differs from the design screenshot | Fix layout and theme values first; record any intentional difference (real data, native chrome) in Plan Audit |
| OpenDyslexic OTF (CFF) does not load in egui | Convert once to TTF with fontTools, check the TTFs in next to the OTFs, note it in Plan Audit; never fall back to egui's default font silently |
| Checkpoint creation is slow (not cloning: non-APFS volume or a different filesystem) | Record the time in STATE.md and escalate before Phase C sign-off; do not drop checkpoints |
| Token budget hits 80% of the daily cap | Switch to report-only for remaining runs |

---

## Budget

- Phase A: ~2 implementer runs; the timeline and table layout dominate.
- Phase B: ~1-2 implementer runs; small, but live checks need the operator.
- Phase C: ~2-3 implementer runs; process handling and the confirmation flow dominate.
- Suggested daily token cap: 400K per phase.

---

## Links

- Plan: [`plan.md`](plan.md)
- Tech plan: [`tech-plan.md`](tech-plan.md)
- Spec: [`ui/SPEC.md`](../../ui/SPEC.md)
- Design: [`ui/design/DESIGN.md`](../../ui/design/DESIGN.md)
- Host project: `file:Dev/Tools/scriptCast/`
- Format exemplar: `file:Dev/Tools/Eocon-Orchestrator-swarm/plans/Embedded_Vendor_TUI/`
