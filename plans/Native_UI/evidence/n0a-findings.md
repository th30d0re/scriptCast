# Phase N0a findings

Status: done. `cargo test` in `ui/` is green, including `tests/snapshots.rs`.

## What Was Requested

Headless `egui_kittest` snapshot tests so UI visuals can be checked inside `cargo test`, without launching the app. Ten states, layout assertions, and an accept workflow.

## What Shipped

- Dev-dependency only, in `ui/Cargo.toml`: `egui_kittest` 0.36 with `wgpu` and `snapshot`. `waveform.rs` and `audio.rs` were not edited.
- The windowed UI now lives in the library (`ui/src/app_main.rs`, included from `lib.rs`) so `cargo test --test snapshots` can construct it. `ui/src/main.rs` calls `scriptcast_ui::run()`. `extern crate self as scriptcast_ui` keeps the existing `scriptcast_ui::` paths working inside the crate, including `tests/common/mod.rs` when the jobs unit tests compile it.
- `Browser::for_harness` reuses `apply_capture_cli`, the same flag path as `Browser::new` (`init_phase_j`, `apply_phase_m_cli`, selection, context menu, tag toolbar, failed-job log). Audio is left closed. The episode is loaded with `Episode::load` instead of the async opener. Project screens use `tests/fixtures/phase_l/show.json`. Script turns for the editor and speaker menu are the fixture dialogue, so the harness does not spawn `scriptcast`.
- Snapshots at 1440×900, pixels per point 1.0, real fonts and theme, in `ui/tests/snapshots/`:
  - `episode_browser_idle.png`
  - `selected_turn.png`
  - `inline_row_editor.png`
  - `context_menu.png`
  - `new_episode_sheet.png`
  - `pipeline_stopped.png`
  - `voices.png`
  - `clips.png`
  - `audition.png`
  - `failed_job_banner.png`
- Dates go through UTC after `use_fixed_clock`. Theme style sets `animation_time` to 0, disables scroll animation, and keeps the caret from blinking. egui_kittest also disables the caret.
- Named faces are installed on the frame before the first widgets. `set_fonts` is not visible to layout in the same pass.
- Speaker name and engine chip are accesskit labels, so the kittest query API can see the painted row. The inline editor is a `MultilineTextInput`; its height is at least three single-line fields.
- Accept workflow is in `ui/tests/README.md`. A mismatch writes `tests/snapshots/{name}.diff.png` beside `{name}.new.png`. `UPDATE_SNAPSHOTS=1` replaces the PNG and keeps `{name}.old.png`.

## Snapshot run time

`cargo test --test snapshots -- --test-threads=1` finished in 6.12s (wall 6.42s, warm binary). Headless wgpu rendered. No test is `#[ignore]`.

## Broken-layout proof

`paint_speaker_row` temporarily placed the engine chip at `row.bottom() + SPACE_8` instead of the row center. `voices` failed:

`Emmanuel Theodore [[56.0 180.0] - [315.8 224.0]] does not share a row with a chip`

The placement was reverted. `voices` passed again.

## Challenges

1. Integration tests cannot see the binary crate. The UI modules moved into the library. `use scriptcast_ui::audio::{self, ...}` at crate root collided with `pub mod audio`; those imports now name the types only.
2. `FontFamily::Name("OpenDyslexic Bold")` panics if widgets run in the same pass as `set_fonts`. The harness installs fonts, returns, and builds the app on the next pass.
3. AccessKit reports unclipped scroll boxes, so a strict "rect must lie inside 1440×900" check fails on rows that are scrolled out of view. The check ignores boxes that miss the window, and a box that still meets the window must keep its center within 40 points of the edge.
4. The clips pane's ffmpeg error embeds a new pointer (`[in#0 @ 0x…]`). The harness rewrites those addresses to `0x0` before paint so the PNG is stable.
5. `std::env::set_var` is unsafe on this toolchain. The fixed clock is an atomic, set by `use_fixed_clock`.

## Next Ideas

1. Point `--tag-toolbar` at the open editor inside `Browser::new`. The flag currently only sets `--edit` and `--select`; the field itself is unread.
2. Load script turns through the real `scriptcast --turns-json` worker when a fixture CLI is on PATH, and keep the in-memory dialogue as the fallback.
3. Clip accesskit bounds to the egui clip rect so the window assertion can be the full rect, not the center.
4. Share one wgpu device across the ten harnesses. Serial runtime is 6.1s, most of it device startup.
5. Snapshot the running pipeline (`pipeline-running.json`) and the quit alert next to the stopped pipeline.
6. Fail the clips snapshot if a decode is still pending, instead of waiting a fixed 200ms for the worker.
