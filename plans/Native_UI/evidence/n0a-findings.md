# Phase N0a findings

Status: done. `cargo test` in `ui/` is green, including `tests/snapshots.rs`.
After the N0b merge the suite is hermetic: fixture media only, no
`scriptcast-project`, no login shell, no `SCRIPTCAST_PROJECT`.

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

## Hermetic follow-up (after N0b)

`dev` merged SoundCraft decoding with these snapshots. Serial `cargo test --test snapshots` then failed three ways.

- `voices` panicked on `get_all_by_label("OmniVoice")`. The chips were still painted.
  The fast-forward dropped the AccessKit labels on the painted name and chip
  rects, so kittest had no node to query. Those labels are back after
  `paint_speaker_row`. The assertion is unchanged.
- `open_project_data` spawned `scriptcast-project` whenever the snapshot was
  empty, using the episode project root or `SCRIPTCAST_PROJECT`. That replaced
  `show.json` with `/private/tmp/scriptcast-k/proj`. The harness sets
  `Browser.harness` and returns before that spawn. `refresh_commands` returns
  immediately too, so the login shell and PATH lookup never start.
  `Episode::load` ignores `SCRIPTCAST_PROJECT` on the harness thread, and the
  loaded episode's project root is cleared.
- `show.json` root is `tests/fixtures`. Clip sources are `audio/tone.mp4`.
  Voice references are `audio/tone.m4a`. Dry-run and audition JSON use the
  same tree. The new-episode sheet keeps the script, voices, and output
  fields on those relative paths and does not walk up to a parent
  `scriptcast.toml`.

Diff images before regeneration:

- `voices`: header text, reference path, and the decode-error block. The error is gone because `tone.m4a` decodes (0.3 s). Fields below it sit higher. Speaker chips match the previous image.
- `clips`: source column and the detail pane. The ffmpeg "no such file"
  message for `/private/tmp/scriptcast-k/...` is gone. The pane shows the tone
  waveform. One `.mp3` source had been left behind; it now points at
  `tone.mp4` too.
- `new_episode_sheet`: the three path fields. They were absolute worktree
  paths. They are now `tests/fixtures/script.md`, `tests/fixtures/voices.yaml`,
  and `tests/fixtures`.
- `audition`: the project header, the reference filenames (`/tmp/aisha_*.wav` to `tone.m4a`), and Play buttons that appear because the file exists.

`UPDATE_SNAPSHOTS=1` rewrote `voices.png`, `clips.png`, `new_episode_sheet.png`, and `audition.png`.

## Snapshot run time

First pass, before N0b: `cargo test --test snapshots -- --test-threads=1` finished in 6.12s (wall 6.42s, warm binary).

After the hermetic fixtures, the same serial snapshot run finished in 5.93s
and 5.86s. Parallel snapshot runs finished in 0.84s and 0.75s. Full
`cargo test --offline` passed twice with `--test-threads=1` and twice with
the default thread pool. Headless wgpu rendered. No snapshot test is
`#[ignore]`.

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
