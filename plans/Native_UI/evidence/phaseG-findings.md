# Phase G Findings

Date: 2026-10-04  
Branch: `ui/phase-g`  
App: 0.8.0

## Status

Finished. All five operator fixes shipped; acceptance green; four muted EP03 captures reviewed.

Constraints held: no git writes; no other agent CLIs / `osascript`; real EP03 read-only; no Python / no synthesis; captures muted (`--mute` / unset `DISPLAY` for native Metal); scratch under `/tmp/scriptcast-g/`.

## Fixes

### 1. Unlimited transcript lines

Removed `READ_ALONG_LINES` / ellipsis overflow from `read_along::Transcript::measure`. Rows grow with the wrapped galley; measured-height cache and `RowOffsets` prefix sums kept. Test covers 1 / ≥3 / ≥6-line layouts.

### 2. Collapsible side panel

List icon (`list-15-text-muted`) toggles `preferences.inspector_hidden` with tooltips "Hide side panel" / "Show side panel". Hidden → table full width. Selecting a turn does not reopen; Selected / Verification / Jobs (and Cmd+J) clear the flag and show the panel. Persisted with `#[serde(default)]`. Capture aid: `--inspector-hidden`.

### 3. Read-along says why it is missing

EP03 has no `word_timings.json`. With read-along on and no usable words for the selected turn, Turn view shows:

- missing: "No read-along timings yet. Run Jobs → Read-along timings."
- stale: "Read-along timings are out of date (episode re-rendered). Run Jobs → Read-along timings."

Plus a **Run…** button that opens the Read-along timings confirmation via `request_job(Preset::ReadAlong, …)`.

### 4. Settings window

Replaced the non-resizable centred `Window` with an `egui::Modal` (scrim), title + × always outside the scroll, content in `ScrollArea::vertical()` capped at 80% of `content_rect` height, **Done** at the bottom. Esc still closes (existing shortcut); backdrop click / × / Done set `settings_open = false`. Speaker colour rows: one per speaker in the open episode (EP03’s four names); more than four cycle `appearance.speakers[i % 4]`. Layout helper `settings_layout` is unit-tested (not pixels).

### 5. Audio clips

- Detect `[clip:<id>]` in script body (`script::clip_id` / `display_body` / `preserve_clip_tag` / `RenderScope`).
- Transcript cell: Audio clip badge (`BG_AUDIO_CLIP_BADGE`, `TEXT_AUDIO_CLIP`, `file-audio-13-text`, radius 4, pad 7, gap 6) + mono muted id above the spoken text (tag stripped from display).
- Playback bar: "Original recording · Not synthesized" beside the waveform when the current turn is a clip.
- Editor footnote for clip drafts: transcript-only correction; no regeneration.
- Save refuses if the original `[clip:…]` tag is removed.
- Re-render: synthesis turns vs "Transcript corrections (no synthesis)"; clip-only disables the button with the required tooltip.

**Orchestrator note:** a later Precision insert still sees clip-transcript corrections as changed turns, because turn identity includes the text. Re-cutting a clip from the original recording is not synthesis — raise that engine change with the operator.

**Scroll fix:** after script turns load, full transcript heights replace `first_words`; selection is re-anchored with `scroll_to` so deep selects (e.g. 220) stay in view for captures.

## Acceptance

```text
cargo fmt
cargo build --release          # ok
cargo test                     # all suites green (ignored = live/hardware/scratch only)
cargo clippy --all-targets -- -D warnings   # ok
awk 'length>120…'              # no hits
grep colour literals outside theme.rs       # no hits
```

## Captures (muted, real EP03 read-only)

```sh
unset DISPLAY   # avoid XQuartz; native Metal
EP=…/outputs/ATO_EP03_five_levels
BIN=ui/target/release/scriptcast-ui

caffeinate -i $BIN "$EP" --fresh --mute --select 17 \
  --screenshot plans/Native_UI/evidence/phaseG-turn017.png
# 2996×2252

caffeinate -i $BIN "$EP" --fresh --mute --select 220 \
  --screenshot plans/Native_UI/evidence/phaseG-clip220.png
# 2996×2252

caffeinate -i $BIN "$EP" --fresh --mute --select 6 --settings \
  --screenshot plans/Native_UI/evidence/phaseG-settings.png
# 2996×2252 (~6 MB; modal + scrim)

caffeinate -i $BIN "$EP" --fresh --mute --select 17 --inspector-hidden \
  --screenshot plans/Native_UI/evidence/phaseG-inspector-hidden.png
# 2996×2252
```

### `phaseG-turn017.png`

Selected turn 017 (Emmanuel Theodore) with inspector open. Transcript cell wraps the full line (“Mine too… software speaks it.”) across multiple lines with no ellipsis. Inspector Turn view shows the missing read-along copy and a **Run…** control under it. List icon visible at the right of the filter bar.

### `phaseG-clip220.png`

Turn 220 selected and scrolled into view after script load. Transcript cell shows the Audio clip badge + mono `atwater_1981` above the spoken text (no raw `[clip:…]` in the cell). Inspector still shows the full body including the tag. Playback bar reads **Original recording · Not synthesized**. Read-along missing line + Run… present. Long clip text expands the row to many lines.

### `phaseG-settings.png`

Settings modal centred with scrim. Title + × on screen; Appearance lists Toussaint / Aisha / Emmanuel Theodore / Lee Atwater colour pickers; Playback / Editing / Commands follow; **Done** at the bottom. Background table still shows multi-line transcripts (e.g. turn 006).

### `phaseG-inspector-hidden.png`

Turn 017 selected; no inspector pane. Table uses the full review width. Selected / Verification / Jobs and the list icon remain in the filter bar.

## Tests added / updated

- `read_along::transcripts_grow_to_full_wrapped_text` (1 / 3+ / 6+ lines, no ellipsis)
- `tests::inspector_hidden_persists_and_defaults_false`
- `settings::tall_content_keeps_title_chrome_on_screen`
- `script::clip_id_*`, `save_refuses_removed_clip_tag`, `render_scope_splits_synthesis_and_clip_only_disables`
- Preferences round-trip includes `inspector_hidden`
