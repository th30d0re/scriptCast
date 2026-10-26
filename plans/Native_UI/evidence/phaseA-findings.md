# Phase A findings

Model: GPT-6 / Codex. Scope: plan steps 1–8, read-only native browser.

## Constraints and decisions

- The explicit brief overrides ancestor AGENTS.md requirements to commit and write an Obsidian session log. No git writes or out-of-worktree logs were made.
- Step 4's field names are the public contract (the tech-plan sketch uses different names). Additional convenience fields may support later phases.
- Step 5 says three WAVs but requires four segments; the fixture contains four distinct 0.2 s WAVs.
- The spec omits empty-workspace column sizes, minimum window size, and several padding values from tokens.json. Values explicitly given in DESIGN.md / captured TSX are named in theme.rs; this is necessary to implement screen 01 without changing the protected tokens file.
- Python project.find_root falls back to its starting directory; the UI plan explicitly requires missing project root to remain None, so the UI uses None when no config/override is found.

## Progress

Steps 1–8 are implemented. Release build, ordinary tests and strict Clippy pass. The requested EP03 integration assertion fails for the documented stale-report contradiction below. FONT_SCALE is 1.0. No GUI was launched; visual fidelity, live interaction, idle CPU, audio, and jobs are not claimed.

## API substitutions (registry source: 0.36.2)

- eframe::App implements `ui(&mut egui::Ui, &mut Frame)`, with optional `logic`, rather than older `update(&Context, ...)`. Panels in 0.36 also take `&mut Ui`.
- Context styling uses `set_style_of(Theme::Dark, style)` plus `set_theme(Theme::Dark)`; `set_style` does not exist.
- SHA-256 0.11 digest arrays no longer implement LowerHex; format each byte as two hex digits.
- Dependencies use current Cargo-resolved sha2 0.11 and toml 1.1. No eframe downgrade.

## Fonts

- Original CFF OTFs rasterize successfully in egui/epaint **0.36.2**. No conversion and no fallback fonts.
- FONT_SCALE = **1.0**. `Toussaint 00:19.740`: proportional 12 px = 155.5 × 22 px; mono 11 px = 153 × 19 px; bold 12 px = 166.8 × 22 px. These fit 28 px rows / 30 px buttons without reducing font size.
- Initial font test failed with `missing glyph 'T' in FontId { size: 12.0, family: Proportional }`. Investigation of epaint `text/font.rs:719` showed `has_glyph` compares the resolved **face** with the replacement face. In a one-face family it returns false even for supported characters. The final test compares each rasterized non-space glyph's atlas UV rectangle with an unsupported codepoint's replacement rectangle, checks mesh vertices, positive width, line height and generated texture data. This is evidence of actual CFF rendering, not a silent fallback.
- Headless egui 0.36 tests must explicitly clear unapplied TexturesDelta after assertions; dropping one panics in debug builds.

## Browser decisions and remaining validation

- More 0.36 API updates: TextEdit takes `frame(Frame::NONE)`, not a boolean; `Context::text_edit_focused()` gives the exact shortcut suppression rule. `Button::new((image, text)).gap(7)` preserves the required 16 px icon size, whereas `image_and_text` caps images to font size.
- The supplied fonts lack U+2318 (command) and U+2304 (down chevron), confirmed from each font's cmap. Hints use readable `Cmd+O` / `Cmd+F`; the disabled output label omits a fake dropdown chevron. The real speaker ComboBox draws its own arrow. No fallback font is introduced.
- The empty workspace lists actual required/optional filenames rather than the design's generic illustrative directory bullets. No illustrative recent episodes, verification, dates, CLI availability, output devices, or progress fractions are manufactured.
- Empty is the full screen 01 layout. Loading and Error use the component card. Loading is indeterminate because the synchronous loader has no truthful record-progress numerator; it runs off the UI thread and the card appears only after 150 ms.
- Initial timeline shows up to 90 seconds as in screen 02; Fit episode shows the full duration. Overview sections are cached at load, and silent slices are represented by None/control fill.
- `stress_mtime` and `voices_path` are already exposed for later phases. Failed-only footer uses UTC explicitly for the mtime; recent timestamps are persisted and displayed as Today/Yesterday/N days ago.
- Up/Down and shared selection are implemented in Phase A because screen 02 advertises them; playback/jobs/inspectors remain Phase B/C.

## Real-data schema correction

The first EP03 release check loaded in **29.861 ms** but failed the verification assertion: `[0, 0, 392, 0]` instead of `[18, 0, 374, 0]`. The model initially interpreted the plan's “repetition flag” as a boolean. Real `stress.json` has `repetition: null` on all 18 entries; `scriptcast/tools/verify_render.py:92-121` defines `_repetition(...) -> str | None` and returns the repeated phrase. Corrected `VerifyResult.repetition` to `Option<String>`, updated the fixture to include both null and repeated text, and added a regression test. The optional-file warning path prevented invented results during this failure. Final EP03 load timing and the unresolved count assertion are recorded below.

- Cancellation now signals an atomic flag checked between metadata reads and per-turn path resolution, as well as discarding the result receiver. A filesystem read already in progress is allowed to finish; canceled results cannot replace the current episode. Closing the app drops the same cancellation guard.
- Added headless layout smoke coverage for Empty, loaded browser, Loading and Error at both specified window sizes. This runs egui in memory, opens no native window, and writes no screenshot. It verifies absence of layout panics only; it does not claim visual fidelity or live input validation.

## Blocking contradiction: EP03 verification acceptance

The mandated ignored test retains the brief's exact 18 Passed / 0 Failed / 374 NotVerified / 0 MissingAudio assertion. It fails against the supplied read-only episode because **five of the 18 report IDs are absent from its manifest**. There are 18 distinct stress entries, all failed=false, but only 13 join by turn_id. Correct displayed counts are **13 Passed / 0 Failed / 379 NotVerified / 0 MissingAudio**. The other assertions (392 turns, 4 speakers, changed script, 3,861,350 ms) pass.

| Report turn / manifest_turn | Report turn_id (absent) | Current manifest ID at that number | Report speaker → current speaker |
|---|---|---|---|
| 122 / 122 | `e7086c3f14a5ac5a` | `6ab4265612723cf1` | emmanuel_theodore → emmanuel_theodore |
| 175 / 175 | `12b54a024cd23297` | `bb53a1ab25cdbc3a` | aisha → toussaint |
| 180 / 180 | `dfb700aec8e3fa96` | `630d927c4b3bde01` | toussaint → toussaint |
| 181 / 181 | `75de0e539b0aa440` | `9bb5890d1093feb2` | aisha → emmanuel_theodore |
| 184 / 184 | `d9a4e71a5036cac5` | `4a3a7afc39ea5746` | toussaint → toussaint |

Evidence fingerprints (read only):
- `episode_manifest.json`: SHA-256 `b17ae4104871ee0f46dd4b1a2db5b9b3a8cabf990af638d81b06cf7cf6463837`; mtime 2026-09-18T14:27:16.919621+00:00.
- `stress.json`: SHA-256 `19d9f6777a29d46c75880df5ae4e2a4639e237f7f0d9729fc317fb246f258c3b`; mtime 2026-09-17T00:07:42.871386+00:00.
- `render_state.json`: SHA-256 `a0f7223409b93f08a89bd2a30fcd572556c2951ec1afbf90d43b29007fbd4dba`; mtime 2026-09-18T14:27:16.924206+00:00.

The stress report predates the manifest. Falling back to numeric indices would attribute two reports to different speakers, contradicting plan Flow 4, the tech-plan stable-ID rule, and the requirement never to manufacture verification. No fallback was added, no real file was modified, and the acceptance assertion was not weakened. The loader now surfaces unmatched IDs in header warning tooltips, with a fixture regression test. A refreshed verification report or a corrected EP03 expected-count requirement is needed for that single acceptance check.

Measured corrected-schema load before assertions: **10.109 ms** (under 50 ms); the prior boolean-schema attempt measured 29.861 ms. The final acceptance run and its timing are recorded below.

## Step audit

| Step | Outcome | Deviations |
|---|---|---|
| 1 | ui/target/ ignored | None |
| 2 | eframe persistence, egui_extras SVG/image loaders, latest resolved toml/sha2, release profile | No eframe downgrade; rodio retained but unused |
| 3 | Central tokens, original embedded OTFs, all 21 SVG sources, button/field helpers | Missing tokens and font symbol limitations detailed above; scale 1.0 |
| 4 | Read-only loader, absolute paths, cancellation, ID joins, warnings, hash comparison, cached overview | repetition is nullable text; stale stress entries remain unverified |
| 5 | Three turns, two speakers, four tiny WAVs, matching SHA-256 fixture, quoted CSV | Four files required for four distinct segments; second turn deliberately stores chunks reversed |
| 6 | All ordinary loader cases pass, with malformed CSV, cancellation and stale-report regressions | Required ignored EP03 count test retained and fails against stale report |
| 7 | Public episode and theme modules | None |
| 8 | Browser shell, persistence, timeline, virtualized table, filters, keyboard selection, workspace states | Actual egui 0.36 APIs; text hints and progress differences documented above; visual assessment remains with orchestrator |

## Final acceptance output

Run from `ui/`. Full captured command outputs are also in ignored build-output files `ui/target/phaseA-{build,test,clippy,ep03}.log`.

### `cargo build --release` — exit 0

```text
   Compiling scriptcast-ui v0.1.0 (/Users/emmanuel/Dev/Tools/scriptCast-worktrees/phase-a/ui)
    Finished `release` profile [optimized] target(s) in 25.08s
```

### `cargo test` — exit 0

```text
   Compiling scriptcast-ui v0.1.0 (/Users/emmanuel/Dev/Tools/scriptCast-worktrees/phase-a/ui)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.72s
     Running unittests src/lib.rs (target/debug/deps/scriptcast_ui-6d44936532acf55a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/scriptcast_ui-b3a38922d693bcfd)

running 5 tests
test timeline::tests::zoom_and_reveal_stay_within_episode ... ok
test tests::filters_intersect_and_never_manufacture_verification ... ok
test tests::failed_open_keeps_episode_and_recent_history ... ok
test tests::canceled_result_cannot_replace_current_episode ... ok
test tests::workspace_states_layout_headlessly_without_panics ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

     Running tests/episode_load.rs (target/debug/deps/episode_load-38549678953ba9bc)

running 16 tests
test ep03_loads ... ignored, requires real EP03 directory; read-only
test canceled_load_stops_before_opening_metadata ... ok
test csv_handles_escaped_quotes_newlines_and_rejects_malformed_records ... ok
test fixture_loads_and_joins_by_id_in_chunk_order ... ok
test missing_manifest_names_file ... ok
test project_discovery_and_override ... ok
test missing_audio_takes_precedence_without_erasing_verification ... ok
test invalid_manifest_preserves_serde_message ... ok
test missing_optional_files_leave_empty_dependent_fields ... ok
test malformed_optional_files_warn_and_keep_browsing ... ok
test absolute_render_source_overrides_manifest ... ok
test overview_dominance_handles_chunks_silence_and_zero_slices ... ok
test changed_and_unavailable_script_hashes ... ok
test rejects_invalid_time_bounds ... ok
test repetition_is_nullable_text_and_missing_fields_remain_optional ... ok
test stale_verification_does_not_fall_back_to_numeric_turn_indices ... ok

test result: ok. 15 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/fonts.rs (target/debug/deps/fonts-71fd68e3f39de7f4)

running 1 test
test embedded_fonts_have_real_glyphs_and_render_into_atlas ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

   Doc-tests scriptcast_ui

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### `cargo clippy --all-targets -- -D warnings` — exit 0

```text
    Checking scriptcast-ui v0.1.0 (/Users/emmanuel/Dev/Tools/scriptCast-worktrees/phase-a/ui)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.87s
```

### `SCRIPTCAST_EP03=/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels cargo test --release -- --ignored --nocapture` — exit 101

```text
   Compiling scriptcast-ui v0.1.0 (/Users/emmanuel/Dev/Tools/scriptCast-worktrees/phase-a/ui)
    Finished `release` profile [optimized] target(s) in 22.21s
     Running unittests src/lib.rs (target/release/deps/scriptcast_ui-3031b26cc61fe867)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/release/deps/scriptcast_ui-a64292bc56943769)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

     Running tests/episode_load.rs (target/release/deps/episode_load-55ef5ecd4796d5fe)

running 1 test
EP03 load time: 9.049 ms

thread 'ep03_loads' (17806257) panicked at tests/episode_load.rs:224:5:
assertion `left == right` failed
  left: [13, 0, 379, 0]
 right: [18, 0, 374, 0]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test ep03_loads ... FAILED

failures:

failures:
    ep03_loads

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.01s

error: test failed, to rerun pass `--test episode_load`
```


Final EP03 load time: **9.049 ms**, below the 50 ms target. The load succeeds; the exact expected-count assertion fails as explained above. Ordinary suite: **21 passed, 0 failed, 1 ignored** (5 browser/timeline tests, 15 loader tests, 1 embedded-font test).

Additional checks:

- `git diff --check`: no output.
- Exact requested colour-literal scan outside theme.rs: no output.
- `git status --short --untracked-files=all`: only `.gitignore`, `ui/Cargo.toml`, `ui/Cargo.lock`, owned `ui/src` modules, `ui/tests/**`, and this findings file changed.
- All four fixture WAV headers inspected: 9,600 frames at 48,000 Hz, mono, 2-byte/16-bit samples (0.2 s).
- Fixture script SHA-256 equality is asserted by the loader test. Real episode metadata and audio were only read.

## Left undone / review boundary

The single unresolved acceptance item is the incompatible EP03 expected verification count. No report was regenerated and no stale result was assigned to a different turn. All other requested nonvisual checks pass. The orchestrator still needs to assess screenshots, live clicking/scrolling, native window appearance, and idle CPU; none were tested or claimed here. Audio playback, jobs and later inspectors remain Phases B/C. No git commit or other git write was performed.

## Screenshot flags

Follow-up (2026-10-02): self-screenshot flags `--screenshot`, `--size`, `--fresh`, `--select` so the orchestrator can verify visuals without macOS Screen Recording permission. New module `ui/src/capture.rs` (flags + capture state machine), wiring in `ui/src/main.rs`.

### API used (checked against egui/eframe 0.36.2 sources, not memory)

- `ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()))` — `UserData` is a mandatory payload (`egui/src/viewport.rs:1199`); `UserData::default()` carries nothing.
- Reply arrives as `egui::Event::Screenshot { viewport_id, user_data, image: Arc<ColorImage> }` in a later frame's `RawInput.events` (`egui/src/data/input/event.rs:181`), read in `App::logic` via `ctx.input(|i| &i.events)`.
- Default glow backend path: egui-winit turns the command into `ActionRequested::Screenshot` (`egui-winit-0.36.2/src/lib.rs:1940`), and after the next frame is painted `painter.read_screen_rgba(window.inner_size())` pushes the event (`eframe-0.36.2/src/native/glow_integration.rs:806-818`). The image is in **physical pixels**, so a 1440×900 pt window captures at 2880×1800 on a 2× Retina display — full pixel density with no extra work.
- Encoding: `image` 0.25 with `default-features = false, features = ["png"]`, `RgbaImage::from_raw` + `save_with_format(ImageFormat::Png)` from `ColorImage.pixels` via `Color32::to_array()`. Chosen over the `png` crate because `image` 0.25.10 + `png` 0.18.1 were already in the lockfile (via `egui_extras`/`arboard`): the `Cargo.lock` diff is exactly one dependency edge, zero new packages.

### Frame and settle logic

State machine `Loading → Settling → Requested`, driven from `App::logic` (deadline, event pickup, load-finish transition) and the end of `App::ui` (frame counting):

- **Loading**: wait until `self.pending` is none. A load error or an unsatisfiable `--select` fails immediately (exit 1, message on stderr).
- **Settling**: count painted frames; require `frames >= 3` **and** `ctx.has_requested_repaint() == false` at the end of the frame (catches scroll-into-view and selection reveals) **and** a 500 ms quiet window since the last frame that requested a repaint. The quiet window covers asynchronously loading icons: `egui_extras` loaders call `ctx.request_repaint()` on completion (`loaders/image_loader.rs:128`), which resets the quiet clock.
- **Requested**: send the screenshot command, keep ticking; the event is written to PNG, `screenshot: <path> <W>x<H>` is printed to stdout, and the process exits 0 via `std::process::exit`.
- While any capture is pending the app requests a repaint every 50 ms, so the 20 s deadline and the settle condition are always re-evaluated; exceeding the deadline fails (exit 1, includes the phase it was stuck in). Usage errors exit 2 with the help text. If the window is closed before the capture arrives, `main` reports it and exits 1. A capture run therefore cannot hang.

Persistence rules: `--screenshot` implies `--fresh` unless an episode dir is given, and never writes state either way. Implemented as `Cli::effective_fresh()` / `Cli::persist()`; when not persisting, `Browser::new` skips storage reads, `save()` writes nothing, `NativeOptions.persist_window = false` (otherwise restored geometry would override `--size`), and `App::persist_egui_memory()` returns false. With no flags, every code path is identical to before (persist defaults true, argument-then-remembered episode order unchanged).

`--select N` matches the **manifest** turn index (`Turn.index == turn_index`, `episode.rs:361`), applied once in `accept_episode` through the existing `select()` (so it is also scrolled into view and revealed on the timeline). An unknown turn is a hard failure in capture mode, a one-line stderr warning otherwise.

### Measured results (this machine, 2× Retina)

```
./target/release/scriptcast-ui --screenshot /tmp/sc-empty.png      → exit 0, 2880x1800, 352,266 bytes, ~1.9 s
./target/release/scriptcast-ui --screenshot /tmp/sc-ep03.png …/ATO_EP03_five_levels
                                                                  → exit 0, 2880x1800, 952,396 bytes, ~1.0 s
./target/release/scriptcast-ui --screenshot /tmp/sc-ep03-sel.png --select 3 …/ATO_EP03_five_levels
                                                                  → exit 0, 2880x1800, 949,269 bytes, ~1.0 s
```

Failure probes (all fast, none hung): nonexistent episode dir → exit 1; `--select 9999` on EP03 → exit 1 (`episode has no manifest turn 9999 (392 turns loaded)`); unwritable output path → exit 1; unknown option → exit 2 + usage; `--help` → exit 0.

Acceptance suite after the change: `cargo build --release` clean; `cargo test` 18 bin tests (13 new for argument parsing + a PNG round-trip) + 15 loader tests + 1 font test, all pass; `cargo clippy --all-targets -- -D warnings` clean; the colour-literal scan outside `theme.rs` still prints nothing (tests use named `Color32::RED`).

### Limitations

- The capture is the app's own window content only (that is the point); the OS title bar is not included — sizes are the *inner* size.
- Pixel dimensions are points × display backing scale: 2880×1800 here, 1440×900 on a 1× display. `--size` is in points accordingly.
- `--size` below `WINDOW_MIN` (1024×640) is clamped by the min-size constraint.
- Requires a GUI session (a window must be created); behaviour with an occluded or minimized window was not tested.
- The 500 ms quiet window is a heuristic: an image that took longer than that to load *and* had not yet requested a repaint could be missed. All icons are tiny embedded files loading in milliseconds, so this is theoretical here; the 20 s deadline bounds the opposite failure.
- Visual judgement of the captures is the orchestrator's job; only exit codes, dimensions and non-empty bytes were verified here.

## Visual fixes round 1

Root cause addressed throughout: text-bearing boxes are now sized from measured
OpenDyslexic text with the design token as the minimum, via new helpers in
`theme.rs`: `text_width` / `wrapped_height` / `line_height` (all built on
`ui.painter().layout*`/`fonts_mut`; `Ui` has no `fonts()` in egui 0.36) and
`fit_text_width(measured, min)` which clamps to the token floor and a new
`TEXT_WIDTH_CAP = 200.0` ceiling. Font, `FONT_SCALE`, region heights and row
heights unchanged.

1. **Empty workspace clipped list** — `main.rs::empty` no longer uses the fixed
   `(EMPTY_DIVIDER + 4*22)` height. `empty_left_height`/`empty_right_height`
   compute each column's content height from the same strings/spacings the
   renderers use (copy is now in consts `HERO_TITLE_TEXT`, `HERO_BODY_TEXT`,
   `DIR_LIST`, `LOCAL_FILES_NOTE` so the two cannot drift); the block is laid out
   at `max(columns)` capped by the workspace, centred vertically, and each column
   gets a `ScrollArea` only when its own content exceeds the capped height. The
   divider spans the laid-out (taller) column. Gotcha recorded: egui applies
   `item_spacing` **after** each widget using the value in effect when that
   widget was added, so the gap after the field label is `SPACE_20` even though
   the spacing is set to `SPACE_7` right after — the first attempt under-measured
   by that 13 px and still clipped the fourth mono line; confirmed fixed in the
   capture. Regression test: `empty_columns_fit_the_default_window_without_scrolling`.
2. **Speaker names** — `view::speaker_label_width` measures the widest
   `▎<display name>` in the speaker-label font (`▎` U+258E is not in
   OpenDyslexic, so it measures as the replacement glyph, roughly the painted
   bar + gap). Timeline label column = `fit_text_width(measured, TIMELINE_LANE_LABEL_WIDTH)`;
   ruler and lane tracks start at `label_width + TIMELINE_LANES_TO_LABEL_GAP`
   (the fixed `TIMELINE_RULER_LEFT_PAD` is no longer used; the token remains in
   theme.rs). Table Speaker column = `fit_text_width(same measurement, TABLE_COL_SPEAKER)`.
   Longer names still truncate with `…` at the 200 cap.
3. **Table fixed columns** — new `ColumnWidths::measure` (table.rs): Turn, Time,
   Speaker, Duration, Verification each take `max(token, bold header, widest
   value)` capped at 200; First words keeps the remainder. Widest values are
   derived analytically (max turn index, `view::time(total_ms)`, longest
   duration, all four verification labels) so the per-frame cost is ~10 galley
   layouts. Tests: `columns_keep_at_least_the_design_token_width`,
   `wide_content_grows_columns_beyond_the_tokens`.
4. **Search field** — real cause found: egui 0.36 `Ui::put` advances the parent
   cursor (`scope_dyn` → `advance_cursor_after_rect`), so the speaker ComboBox
   started at the edit rect's right edge — 68 px inside the field — and painted
   over "Cmd+F" leaving "C". The TextEdit now lives in a non-cursor-advancing
   child ui (left_to_right/Center for vertical centring, `desired_width` = edit
   width). Field width = `max(SEARCH_WIDTH, 7 + icon 14 + 7 + "Search turns" +
   8 + "Cmd+F" + 8)`, hint right-aligned with 8 px padding. The 10 px bar
   spacing is unchanged and now actually visible between field and ComboBox.
5. **Zoom controls** — `timeline.rs::zoom_button` paints `layout_no_wrap` text
   with `galley_with_override_text_color` (`TEXT_MUTED` → `TEXT_PRIMARY` on
   hover), allocated exactly at galley size, `PointingHand` cursor, no frame or
   fill. Order `−  Zoom  +` (8 px gaps) then `Fit episode` after 14 px, matching
   design screen 02. Behaviour (zoom 1.5× steps, fit) unchanged.
6. **Overview density** — `episode::overview_section_count(total_ms) =
   clamp(round(total_minutes / 3), 12, 32)`; EP03 (64:21) now renders 21
   sections instead of 120. Dominant-speaker colouring, 48% outside-range
   opacity, visible-range box and time-positioned failed markers unchanged.
   Tests in episode.rs cover 20 min → 12, 64 min → 21, 3 h → 32 (and 0 → 12).
7. **Header timestamp** — new `rendered_when` (main.rs): `rendered_at` is parsed
   with `chrono::DateTime::parse_from_rfc3339` and formatted `18 Sep 2026, 10:26`
   in `chrono::Local` (manual English month table, unpadded day); unparseable
   values pass through unchanged, `None` still shows `unknown`. The line was and
   remains one `view::line` in sans 11 px muted — the "different face" was the
   raw ISO string's length/punctuation, not a font mix. **New dependency:
   `chrono` 0.4.45 with `default-features = false, features = ["clock", "std"]`**
   (not previously in the lockfile; chosen as the lightest correct local-time
   option — adds `iana-time-zone` + `iana-time-zone-haiku`, 3 packages total).
   Recent-episodes "when" values are relative strings ("Today", "N days ago"),
   not absolute dates, so the new format does not apply there.
8. **Playback current-audio width** — block width = `max(PLAYBACK_CURRENT_AUDIO_WIDTH,
   measured "Turn NNN · Speaker" line, measured "00:00.000 / —" mono line)`; the
   seek track still fills the remainder.

Acceptance, this machine: `cargo build --release`, `cargo test`
(2 lib + 22 bin + 15 loader + 1 font, all pass; 1 ignored), and
`cargo clippy --all-targets -- -D warnings` all exit 0;
`--screenshot` captures `/tmp/sc-v1-empty.png`, `/tmp/sc-v1-ep03.png` and
`/tmp/sc-v1-ep03-sel.png` all exit 0 at 2880×1800; the colour-literal grep
outside `theme.rs` prints nothing. The four mono list lines are visible in the
empty capture and no text reviewed above is truncated. Visual judgement against
the design remains the orchestrator's.

Not done / notes:
- `EMPTY_DIVIDER` and `TIMELINE_RULER_LEFT_PAD` tokens are now unused by code
  (still exported from theme.rs as design tokens).
- In the shorter-than-content window case the columns scroll; content then wraps
  about a scrollbar-width narrower than the measured 410/386 px, so the measured
  heights are a slight underestimate in that mode only — harmless, since that
  mode scrolls by definition.
