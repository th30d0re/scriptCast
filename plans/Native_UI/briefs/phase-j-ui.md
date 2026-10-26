# Phase J (UI): new episode, live render, detached jobs, pipeline

Working directory: a git worktree of scriptCast on branch `ui/phase-j`, from `dev` at d14ef08 (app 0.16.4).
A parallel agent is adding the Python side on another branch. Build against the **Contract** in `plans/Native_UI/briefs/phase-j-py.md` (sections 1–4; read only that file's "Contract" section). For tests, use JSON and JSONL fixtures that you write by hand from that contract. Do not run the new flags yet, because they do not exist on this branch.

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs**, never use `osascript`, never launch Live or Logic, and never synthesize.
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** Use fixtures, or copies under `/tmp/scriptcast-j/`.
- **No Python changes.**
- Lines under 120 characters; run `cargo fmt`; delete no comments; no colour or size literals outside `theme.rs`.
- **Tooltips:** only for a reason a control is disabled, or for information that is not visible. No tooltip may repeat a visible label (rule from 0.16.2).
- Redirect cargo output: `cargo test > /tmp/j-ui.log 2>&1; tail -30 /tmp/j-ui.log`.
- **Mute** all captures.

## Design

The Figma screenshots are `ui/design/J1-new-episode.png`, `J2-render-in-progress.png`, `J3-quit-while-rendering.png` and `J4-pipeline.png`. Match them using the existing theme tokens and components.
- **Ignore** the bottom strip of each frame ("J1 Validate…", "Workshop / Core"); it is a Figma annotation.
- **J2:** finished turns on the timeline show **real waveforms** (`waveform.rs`), not the ▶ glyphs in the mock-up. Pending turns are dim placeholders; the turn being rendered has an accent outline.

## Established facts (`file:line` on dev)

- `runner.rs:313` `Job` spawns with `process_group(0)` (:335); `cancel` (:386) kills the group through `cli::kill_group`. Presets are the enum at `runner.rs:21`; argv is built around :150–220.
- `jobs.rs` holds the Jobs panel (`cancel_job` :542) and command resolution. The order is: the Settings folder, then the project `.venv*`, then the login PATH, then the dev checkout.
- `main.rs:2262` `fn ui`: the app entry. Self-capture flags are in `capture.rs` (`--screenshot` :223, `--select` :242).
- The checkpoint before any overwrite is in `checkpoint.rs`; script saves are hash-checked.

## Build

**J1. New episode sheet** (File → New Episode…, ⌘N; also a button on screen 01).
1. Fields:
   - the script `.md`;
   - the episode id, prefilled from the file stem;
   - the voices file, defaulting to the project `voices`;
   - the output folder, defaulting to the project `outputs`.
2. **Dry run** runs `scriptcast --dry-run --json-out <tmp>`. Render, from that JSON:
   - the warnings list (blocking items in red, with "Open voices file" revealing the file in Finder for now);
   - the parsed-turns table;
   - the speaker summary (an unknown speaker's row is red).
3. **Render options** is a collapsed disclosure holding: gap ms, tail ms, speech threshold, overwrite, no trim, skip ALS, FCPXML, max turns.
4. If the output folder exists, ask with a popup: **Resume / Overwrite / Cancel**. Resume passes `--resume`; Overwrite passes `--overwrite`, after a checkpoint of the existing manifest and render state.
5. "Render N turns" stays disabled while any warning has `blocking: true`. The reason is shown as visible helper text, as in the mock-up.

**J2. Live render**
1. The render runs with `--progress-jsonl <episode>/_jobs/<job_id>.progress.jsonl`.
2. A `progress.rs` module tails the file: it reads new lines on each frame, but only when the file length changed, and requests a repaint at most every 250 ms while a job runs. It must not repaint continuously.
3. The timeline and table update per `turn_done`. The status strip reads "Rendering turn n of N · engine · elapsed · about X left"; the estimate is the mean seconds per turn so far. Cancel sends SIGTERM to the group.
4. When `done` arrives, reload the episode, keeping the selection.
5. Turns that are already rendered stay playable during the render.

**J3. Detached jobs and quitting**
1. Every synthesizing or long job writes `<episode>/_jobs/<job_id>.json`, holding `{pid, pgid, preset, argv, log, progress, started}`. The job's stdout and stderr go to `<job_id>.log`, so the job survives the app quitting (it already runs in its own process group; make sure nothing kills it on drop).
2. **Quitting** while a job runs (close the window or ⌘Q) intercepts the close and shows the J3 alert:
   - **Keep rendering:** quit, leaving the job running;
   - **Stop render:** SIGTERM, wait up to 3 s, then quit;
   - **Cancel:** stay open.
3. **On launch**, opening an episode with a `_jobs/*.json` whose pid is alive (`kill(pid, 0)`) reattaches: progress tailing resumes, and so does the status strip. A dead pid with no `done` or `error` line is shown as "Interrupted", with an offer to resume (`--resume`).

**J4. Pipeline sheet** (Episode → Finish Episode…)
1. Six steps, each with a toggle: Render, Verify, Repair (max passes), Relayout (gap slider 0–1000 ms), Stitch MP3, Turn index.
2. **Relayout preview:** "new length mm:ss", computed in Rust from the manifest's speech durations plus N−1 gaps. Unit-test this.
3. Steps run in sequence, each through the existing runner, each with `--progress-jsonl`. Settings lock while the run is active.
4. **Stop at failure:** if repair ends with `failing > 0`, show the stopped state from the mock-up, with **Review failed turns** (closes the sheet and sets Failed-only) and **Continue anyway**. Any `error` event, or a non-zero exit, stops the run with the message.
5. The log disclosure shows the last 4 lines of the current step's log, in mono.
6. A pipeline run is itself a detached job: its `_jobs` record lists the steps and the current step, and reattach works for it too.

**Other J items**
- Add presets and Jobs-panel buttons for:
  - Relayout (`scriptcast-relayout --gap-ms`), followed by a reload;
  - Stitch (`scriptcast-stitch`), with "Reveal in Finder" on done;
  - Turn index (`scriptcast-turn-index`);
  - Re-export (`scriptcast --export-only`, with FCPXML and Logic dry-run toggles).
- Relayout and Re-export overwrite files, so each takes a checkpoint first.
- **Retime** (`scriptcast-retime`) is deferred to Phase K. Leave it out.

## Acceptance (run these)

1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` must all be green. The new tests cover:
   - dry-run JSON parsing, and blocking-warning gating;
   - progress tailing: partial last line, file growth, `done`/`error`, the ETA maths;
   - the job record round-trip, and reattach classification (alive / finished / interrupted) with fake pids;
   - pipeline sequencing: skipped steps, stop on repair failure, Continue anyway, stop on error;
   - the relayout length;
   - the argv for every new preset.
2. Add capture flags so the orchestrator can screenshot each state without clicking:
   - `--new-episode <script.md>`, with an optional `--dry-run-json <fixture>`;
   - `--progress-fixture <jsonl>` (renders J2 from a fixture);
   - `--quit-alert`;
   - `--pipeline-fixture <json>`.

   Capture each state from the fixtures to `plans/Native_UI/evidence/phaseJ-*.png`.
3. Bump the version to 0.17.0.

## Not yours to claim

Real renders, end-to-end runs against the Python flags (they land on another branch), and judgements made by ear.

## Stop conditions

- If intercepting window close is not possible in eframe 0.36 without a fork, stop and report what is possible.
- If a detached child dies when the app exits, even though it runs in its own process group, stop and report what you measured.

## Findings file

Write `plans/Native_UI/evidence/phaseJ-ui-findings.md` **early**. Fill it in as you go: the files changed, the test counts, deviations from the mock-ups or the contract, and what is unverified.
