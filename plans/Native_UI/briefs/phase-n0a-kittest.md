# Phase N0a: headless snapshot tests with egui_kittest

Worktree: branch `ui/n0a-kittest`, from `dev` (app 0.20.1). A parallel agent is working on `ui/n0b-audio-io`, which changes `waveform.rs`, `audio.rs` and `Cargo.toml`. Keep your edits to those files minimal: add the dev-dependency only.

## Rules

- **No git writes.**
- **Never start other agent CLIs**, and never use `osascript`.
- **No Python changes.**
- **The real project is read-only.**
- Lines under 120 characters; `cargo fmt`; delete no comments.
- Redirect cargo output to a file.

## Goal

Agents can verify UI visuals inside `cargo test`, without launching the app. Today the orchestrator runs `--screenshot` captures by hand, and they time out when the machine is loaded.

## Facts

- The app uses egui/eframe 0.36.2.
- SoundCraft (MIT/Apache) uses `egui_kittest = { version = "0.36", features = ["wgpu", "snapshot"] }` with snapshot tests in `crates/ui-egui/tests/`. A read-only clone is at `/private/tmp/claude-501/-Users-emmanuel-Dev-Tools-scriptCast/bf35c9bf-0d0b-4d7b-9bf1-8b66645c0f23/scratchpad/soundcraft`. Read its test harness for the pattern.
- The existing capture flags in `capture.rs` (`--select`, `--context-menu`, `--project-data`, the `*-fixture` flags) already put the app into each state. Reuse that state setup; don't duplicate it.

## Build

1. Add `egui_kittest` as a dev-dependency, with the snapshot feature and the renderer that works headless on macOS.
2. Add a `ui/tests/snapshots.rs` harness:
   - it builds the app state from the fixture episode `ui/tests/fixtures/episode_small`, plus the phase fixtures;
   - it renders at 1440×900 and a scale of 1.0, with the real fonts and theme;
   - it compares against PNGs in `ui/tests/snapshots/`.
3. Snapshots, one per state:
   - the episode browser, idle;
   - a selected turn in the side panel;
   - the inline row editor with the tag toolbar (long text);
   - the context menu with the speaker submenu;
   - the J1 New episode sheet (dry-run fixture);
   - the J4 pipeline, stopped;
   - L2 Voices;
   - L3 Clips;
   - M3 Audition (fixture);
   - the failed-job banner.
4. **Layout assertions,** in addition to pixels:
   - the speaker-row chip and name share a row (the Phase L regression);
   - the row editor height is at least 3 text lines;
   - no widget rect extends past the window.

   Use the kittest node/query API.
5. **The accept workflow:** `UPDATE_SNAPSHOTS=1 cargo test --test snapshots` regenerates the images. Document it in `ui/tests/README.md`, including that a diff image is written on failure.
6. **Determinism:**
   - disable the caret blink and animations;
   - use a fixed time and dates;
   - mute audio;
   - mark tests needing a GPU `#[ignore]` only if headless rendering truly fails. If it does, record why.

## Acceptance

- `cargo test` is green, with the new snapshots generated and committed to the worktree (no git: just leave the files).
- A deliberately broken layout makes the relevant test fail. Prove it once by temporarily breaking a padding value, then revert, and note the result in the findings.
- Report the snapshot run time.

## Findings

Write `plans/Native_UI/evidence/n0a-findings.md` early.
