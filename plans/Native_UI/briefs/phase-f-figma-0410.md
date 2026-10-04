# Phase F: align screen 03 with the 2026-10-04 Figma update

Your working directory is a git worktree of scriptCast on branch `ui/phase-f`, created from `dev`, which has everything through Phase E merged (app 0.7.0).

## Prohibitions (read first)

- **No git writes.**
- **Never start other agent CLIs, and never drive other apps** (no `osascript`).
- **Write only inside this worktree**, plus `~/.cargo`, `ui/target/` and `/tmp/scriptcast-f/`. Put scratch files in `/tmp/scriptcast-f/`.
- **The real project is read-only:** `/Users/emmanuel/Documents/Theory/TheOriginalPower/`. Any save test runs on a `/tmp` copy, as in `briefs/phase-d2.md`.
- **No Python changes. No synthesis.** Mute captures.
- Lines under 120 characters, readable code, `cargo fmt`, no deleted comments, no colour or size literals outside `theme.rs`.

## The target

Read `ui/design/DESIGN.md`, section "Update 2026-10-04: screen 03 A/B behaviour", and look at:

- `ui/design/screens/03e-update-2026-10-04/03-selected-turn.png`: **A**, direct inline editing;
- `ui/design/screens/03e-update-2026-10-04/03-alt-b.png`: **B**, side-panel editing with in-row read-along.

Where these differ from `03b` and `03c`, these win.

## Build

1. **Full-text Transcript column.**
   - Rename the "First words" header to "Transcript".
   - Each row shows the turn's full text from `ScriptTurns`. Fall back to first words when the script text is unavailable.
   - Wrap to at most 2 lines, with an ellipsis on the last line.
   - Row height comes from the text: 1 line uses `TABLE_ROW_HEIGHT`, 2 lines a new token of about 52 (`TABLE_ROW_HEIGHT_2`).
   - The table stays virtualised. Precompute each visible-filter row's height, using measured text galleys cached per (turn, column width, text scale) and invalidated when those change. Build a prefix sum for scrolling, `scroll_to` and the "Showing a–b of n" footer.
   - Keep the other columns top-aligned with the first text line, as in the design.
2. **A: edit in the row cell.**
   - When the editing location is Inline and a draft is open, the selected row's Transcript cell becomes the multi-line `TextEdit` itself, using the same `Draft` model from `editing.rs`.
   - Under the text, in the same row: **Save draft** (primary, small), **Cancel**, and at the right "Unsaved draft" with the alert icon (danger), shown when the text differs from the original.
   - The row grows to fit.
   - Remove the separate editor block under the row that the earlier table-row mode drew.
   - Keyboard behaviour, save, conflict and guard logic stay as they are.
3. **Mode toggle in the filter bar.** After "Failed only", add an "Inline / Side panel" segmented control styled as in the design: an accent fill, with a small pill naming the active mode. It reads and writes the same persisted setting as Settings → Editing, renamed to "Inline" and "Side panel".
4. **B: read-along inside the playing row.**
   - When read-along is on, timings exist, and a turn is playing, highlight the current word **inside that row's Transcript text**: an accent pill, radius 3, padding 4×2, bold face (no semibold is bundled), in `ON_ACCENT` text.
   - Remove the separate read-along row under the playing row.
   - Word positions: lay the text out with egui `LayoutJob` sections so the highlighted word's rect is known. Draw the pill behind it, then the text.
   - Map timing words to text words in order, using normalised comparison like `word_timings.py`. If the two counts disagree, highlight nothing for that turn rather than the wrong word.
   - Keep the 50 ms playback repaint; add no other continuous repaint.
5. **Playback bar.** The current-audio line reads "Turn NNN · Speaker · Paused" when paused.
6. **Inspector copy.** The Turn view's playback hint follows the design. Paused: "Paused on turn NNN. Transcript edits are local until saved; rendered audio remains unchanged." Playing: "Listening to turn NNN. Transcript edits are local until saved; rendered audio remains unchanged."

## Tests (required)

- Row-height prefix sums with mixed 1-line and 2-line rows, including `scroll_to` landing on the right row.
- Word mapping: equal counts, unequal counts give no highlight, punctuation and case.
- The mode toggle writes the persisted setting.

## Acceptance

```bash
cd ui && cargo fmt && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs
```

Then capture on the real EP03 (read-only), muted:

- `--select 12 --screenshot plans/Native_UI/evidence/phaseF-rows.png`;
- `--select 12 --edit --edit-mode row --screenshot plans/Native_UI/evidence/phaseF-inline-edit.png`;
- a read-along capture on a `/tmp` copy that has `word_timings.json` for turns 5–12. Create it with `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m scriptcast.tools.word_timings --turns 5-12`, with `PYTHONPATH` set to this worktree. Capture `--select 7 --play --at 3000 --screenshot plans/Native_UI/evidence/phaseF-readalong.png`.

If a capture times out because the display is asleep, record it.

Write `plans/Native_UI/evidence/phaseF-findings.md` early and keep it current.
