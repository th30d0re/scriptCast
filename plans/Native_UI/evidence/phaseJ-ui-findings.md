# Phase J UI findings

Status: Rust implementation and automated checks complete; native screenshot acceptance is blocked.
Version: 0.17.0. No git writes, Python edits, synthesis, DAW launches, or writes to the real project.
Only the Contract section (1–4) of the Python brief was read. New Python flags were not executed.

## Files changed

- `ui/src/new_episode.rs`: schema-1 preview, blocking-warning gate, render/dry-run argv, planned episode model.
- `ui/src/progress.rs`: incremental JSONL tail, partial-line buffering, terminal events, turn state, ETA.
- `ui/src/detached.rs`: durable records, file-backed stdout/stderr, process-group supervisor, reattachment.
- `ui/src/pipeline.rs`: six-step sequencing, repair pause/Continue anyway, failure handling, relayout maths.
- `ui/src/phase_j.rs`: New Episode, pipeline, quit alert, progress polling, live reload and interrupted-job UI.
- `main.rs`, `jobs.rs`, `runner.rs`: menus/shortcuts, screen-01 button, detached job execution, new presets.
- `timeline.rs`, `table.rs`: accent outline for the rendering turn, dim pending clips, live table states.
- `checkpoint.rs`: export checkpoint coverage includes FCPXML and Logic export paths.
- `capture.rs`: all requested fixture flags; all screenshot runs are muted automatically.
- `theme.rs`: sheet dimensions and path-control width tokens.
- `ui/Cargo.toml`, `ui/Cargo.lock`: 0.17.0.
- `ui/tests/phase_j.rs`, Phase J unit tests, handwritten JSON/JSONL fixtures: acceptance coverage.
- `ui/tests/common/mod.rs`: shared test copies now use `/tmp/scriptcast-j/`.
- `capture-phaseJ.sh`: reproducible, muted fixture-only captures for J1, J2, J3, J4 running and stopped.

## Validation

Final checks (from `ui/`, with cargo output redirected):

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: 197 passed, 0 failed, 6 pre-existing ignored.
- `git diff --check`: passed; new Rust source and added lines are under 120 characters.

The 20 added tests cover preview parsing and blocking warnings; partial JSONL lines, growth, truncation,
terminal events and ETA; durable record round-trip and minimal contract records; fake-pid classification;
skipped pipeline steps, repair failure, Continue anyway, error/nonzero stops; relayout length; new argv;
planned/completed audio and speaker/chunk discovery; fixture flags, headless sheets and close interception.

Two process-level checks use harmless shell fixtures, never Python synthesis:

1. A separate parent launches the actual Rust job worker and exits. The detached job subsequently completes,
   with both `survived` stdout and `stderr-survived` stderr preserved in its log. This passed.
2. The worker runs Verify and Repair, pauses with subsequent steps unstarted, then runs Relayout and Turn index
   after a Continue decision file. Disabled steps never execute. This passed.

The installed eframe 0.36.2 source supports `ViewportCommand::CancelClose`; the headless close-event test
confirms interception. No eframe fork is needed. Neither requested stop condition was observed.

## Capture blocker

No `phaseJ-*.png` files were produced. Do not treat headless layout tests as screenshot evidence.

The native `--screenshot` attempt failed to reach a window from the restricted shell, reporting macOS
LaunchServices/XPC connection errors. The purpose-built macOS app-launch tool then returned:
`MCP tool call requires approval, but approval policy is never`.
The stalled shell capture was terminated through its execution session (exit 130).
No permission workaround, osascript, Live, or Logic was used.

After building the UI, an orchestrator with native GUI launch access can run, from the worktree root:

```sh
sh plans/Native_UI/evidence/capture-phaseJ.sh
```

The script copies existing WAV fixtures under `/tmp/scriptcast-j/`, removes only pending-turn WAVs in that
copy, and passes `--fresh --mute` for every capture. It does not execute any rendering command.

## Design / contract notes and remaining verification

- The bottom Figma annotation strips are intentionally omitted. Finished clips use `waveform.rs` WAV envelopes.
- The previews use a small three-turn handwritten fixture rather than the mock-up's illustrative 342 turns.
- The preview gives approximately two thirds of its width to parsed turns, following the mock-up.
- The repair-stopped explanation and its two actions are inside the pipeline sheet rather than a separate side card.
- Pixel fidelity, native window-close/Command-Q interaction, and actual screenshot settling remain unverified.
- The record `pid`/`pgid` identify the detached Rust supervisor. Step processes inherit its process group.
  Records retain the required fields plus serialized `steps`, pipeline state, current step and log offset.
- Each pipeline step has its own JSONL file; the pipeline also has an overall terminal JSONL file.
  Settings and queued steps stay locked while Repair awaits a persisted Continue decision.
- Until a final manifest exists, timing is explicitly planned. The last placeholder estimates 400 ms per
  displayed word. Completed WAVs are discovered using the existing speaker/turn-id chunk naming convention;
  the final manifest replaces those estimates. This fallback is covered by fixtures, not a real render.
- Python progress flags, real rendering/resume/export/pipeline integration, and judgments by ear are unverified
  and outside this branch's authorized acceptance work.

## Visual round 1

Compared `ui/design/J1-new-episode.png`, `J2-render-in-progress.png`, `J3-quit-while-rendering.png`, and
`J4-pipeline.png` with the sheet code. This session cannot launch the GUI, so `capture-phaseJ.sh` was not
re-run and no new `phaseJ-*.png` files were written. Pixel comparison is still the orchestrator's capture.

Layout changes, and nothing else:

- J1: header is title left and `N turns · N speakers · 48 kHz` right (48 kHz is the planned rate; the dry-run
  schema has no sample rate). Script / Episode id and Voices / Output folder are equal columns with each label
  directly above its field. `Choose…` is painted inside the right end of the path field. Each warning is a
  full-width bordered row: blocking uses the danger fill and border with `Open voices editor` on the right;
  other warnings use a neutral border and `Review line N`. Parsed turns and Speaker summary are separate
  bordered panels. Turn scrolling is clipped to a whole number of row heights. The speaker panel keeps a
  Total row, and an unknown speaker row uses the danger tint. The footer puts the blocking reason on the left
  and Cancel plus Render on the right.
- J2: while render progress is present, the status strip grows by `PROGRESS_HEIGHT` and paints an accent bar
  across the window. Cancel stays at the right, including fixture progress that has no durable job.
- J3: shield icon before the title, one body paragraph, `Stop render` as a danger outline on the left, Cancel
  and Keep rendering on the right. Content width is 392 so the framed alert is 440 with dialog padding.
- J4: each step is a full-width row separated by a divider, with a switch instead of a checkbox and status
  text on the right (green complete, red failing counts, accent `Running · …`, muted Queued). The header shows
  a dot plus Running or Stopped. The relayout slider fills the row beside `new length mm:ss`, with `0 ms` and
  `1000 ms` under the track. The footer keeps the checkpoint sentence on the left, `Cancel run` while a run is
  active, and `Run pipeline` plus Close when the pipeline is idle.

Checks from `ui/`, cargo output in `/tmp/phasej-cargo.txt`:

- `cargo fmt --all`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: 197 passed, 0 failed, 6 ignored.
- `git diff --check`: passed. New and edited Rust lines stay within 120 characters.

Screenshot settling is still for the orchestrator.

## Visual round 2

Verify status is one right-aligned LayoutJob, `Complete · N failed`, with the count in red. `pipeline-running.json` failing count is 2, matching its verify log line. The relayout slider width is `slider_width` set to the row width minus the `new length` label, with `0 ms` and `1000 ms` under the track ends. Speaker rows scroll inside the panel (`min_height` of every speaker) and Total stays pinned under that list. `cargo fmt`, `clippy -D warnings`, and `cargo test` passed (197 passed, 0 failed, 6 ignored); output in `/tmp/phasej-cargo.txt`.
