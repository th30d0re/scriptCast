# L2: Settings window (⌘,) with appearance options

Working directory: this git worktree (branch `ui/layout-settings`). L1 (resizable panes) is already in this worktree, uncommitted or committed; build on it.

## Rules

- **No git commands that write.**
- **Never start other agent CLIs** (`codex`, `kimi`, `agy`, `gemini`, `cursor-agent`, `claude`). Do the work yourself.
- Change only files under `ui/src/` and `ui/tests/`.
- Put scratch files under `/tmp/scriptcast-layout/`.
- Lines under 120 characters, written as normal readable Rust like the surrounding code. Run `cargo fmt`. **Do not delete or reword existing comments** unless you remove the code they describe.
- Colour and size **defaults** live only in `ui/src/theme.rs`.
- Mute every playback you start. The real project is read-only.

## Operator request (2026-10-03)

> "I want an actual settings panel that gets invoked via command-comma, as is tradition. And I want to be able to change colors, do stuff like that … change a couple things."

## Current state

- The header's gear button (`t::icon_button(... "settings-16-text" ...)`, `main.rs` around `egui::Popup::menu(&settings)`) opens a small popover with:
  - "Folder with scriptCast commands" (Choose folder…, Clear), which sets `preferences.cli_dir`;
  - "Playback > Gapless auto-advance".
- Colours are compile-time constants in `theme.rs`:
  - `ACCENT` is used about 15 times across `main.rs`, `timeline.rs`, `table.rs`, `inspector.rs`, `jobs.rs` and `theme.rs`;
  - `SPEAKER: [Color32; 4]` is read through `speaker_colour(i)`;
  - `FONT_SCALE` is a `const` 1.0 used by `font()`, `mono()`, `bold()`.
- `t::apply(ctx)` sets the egui `Visuals`, `Style` and fonts once at startup.

## Build

1. **Runtime appearance in `theme.rs`:**
   - `#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)] pub struct Appearance { pub accent: [u8; 3], pub speakers: [[u8; 3]; 4], pub text_scale: f32 }`, with `Default` equal to today's constants (`text_scale` 1.0).
   - Store the live value in a `static APPEARANCE: std::sync::RwLock<Appearance>`.
   - Add `pub fn appearance() -> Appearance` and `pub fn set_appearance(ctx, a)`. The setter stores the new value, re-runs `apply(ctx)` so Visuals pick up the accent, and calls `ctx.request_repaint()`.
   - Add `pub fn accent() -> Color32`. **Replace every use of `ACCENT`/`t::ACCENT` with `accent()`/`t::accent()`**, and keep `ACCENT` only as the default inside `Appearance::default()`.
   - `speaker_colour(i)` reads from the appearance.
   - `font`, `mono` and `bold` multiply by `appearance().text_scale` (keep `FONT_SCALE` as the base). `text_scale` is clamped to 0.85..=1.3.
   - `ON_ACCENT` (text on accent buttons) stays dark. Tell the operator in the window that very dark accents reduce button-text contrast; you don't have to compute contrast.
2. **Settings window in `main.rs`, or a new `settings.rs` if it reads better:**
   - Opened by **⌘,** (`egui::Key::Comma` with `modifiers.command`), and by the gear button, which no longer opens the popover.
   - An `egui::Window` titled "Settings", about 460 wide, centred, collapsible false, closable, with Esc to close. Use the app's existing button and label helpers and tokens so it matches the app.
   - **Appearance:**
     - Accent colour: a colour picker, `egui::widgets::color_picker::color_edit_button_srgb`;
     - Speaker colours 1–4, each labelled with the open episode's speaker name when available, else "Speaker 1…4";
     - Text size slider 85%–130%;
     - "Reset appearance".
   - **Playback:** Auto-advance, Gapless auto-advance (move it here from the popover).
   - **Commands:** "Folder with scriptCast commands" with Choose…/Clear (move it here), plus a read-only list of where each command resolved to (reuse what the Jobs runner already shows, if a function exists).
   - Changes apply live and persist: add `appearance: Appearance` to `Preferences` with `#[serde(default)]`, and call `set_appearance` at startup from the saved value.
3. Small L1 follow-up: when the timeline is taller than its lanes need (lanes cap at `TIMELINE_LANE_MAX`), the space below the last lane is left blank and looks broken. Let lanes grow up to 2× `TIMELINE_LANE_MAX`; beyond that, keep them top-aligned and extend the lane-area background to the bottom of the timeline so it reads as intended.
4. Add a hidden capture flag `--settings` that opens the window for a screenshot.

## Tests (required)

- An `Appearance` serde round trip.
- An old preferences JSON without `appearance` loads with the defaults.
- `text_scale` clamping.
- `speaker_colour` reflects a changed appearance. Restore the default at the end of the test so other tests aren't affected; mind test parallelism, and prefer a pure helper that takes an `Appearance` if a global makes this flaky.

## Check yourself, then report

```bash
cd ui && cargo fmt && cargo build --release 2>&1 | tail -2 && cargo test 2>&1 | grep "test result" && cargo clippy --all-targets -- -D warnings 2>&1 | tail -1
awk 'length>120{print FILENAME":"FNR}' src/*.rs
grep -rn "t::ACCENT\b\|[^_]ACCENT[^_]" src/*.rs | grep -v "theme.rs"     # should print nothing
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs   # nothing
git diff --stat
./target/release/scriptcast-ui --mute --settings --select 6 --screenshot /tmp/scriptcast-layout/l2-settings.png /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
```

Look at the PNG and describe it plainly.

Write `plans/Native_UI/evidence/layout-l2.md` with:

- the diff stat;
- all command output;
- what the PNG shows.

If anything fails, say so plainly.
