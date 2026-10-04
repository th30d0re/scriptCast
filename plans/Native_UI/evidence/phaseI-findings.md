# Phase I findings (final)

Date: 2026-10-04. Branch `ui/phase-i`, this worktree.
Real Theory project was read-only. Live replace/capture used `/tmp/scriptcast-i` only.

## Build audit (items 1–4)

| Item | Status | Evidence |
|------|--------|----------|
| 1. Find words | **done** | `ui/src/find.rs` matcher + `occurrences`; Words mode keeps all rows visible; gold underline + current fill (`FIND_GOLD` / `FIND_CURRENT` in `theme.rs`, painted in `table.rs`); wrap nav; "i of N" / "No matches" / "Invalid pattern"; searches `display_body` |
| 2. Find turns + Options | **done** | Mode Words/Turns; Match case, Whole word, Regular expression; Include audio clips (default on); options on `Preferences.find` with `#[serde(default)]` |
| 3. Replace + `save_many` | **done** | Replace row + planner + clip-tag skips; confirm dialog (≤10 snippets + "+K more"); draft guard; status "Replaced N in M turns · checkpoint vNN"; `script::save_many` one hash check, one `script-edit` checkpoint, bottom-to-top splice, temp+rename |
| 4. Keyboard | **done** | ⌘F, ⌘⌥F, Esc clear then blur, ⌘G / ⇧⌘G, Enter / Shift-Enter in field; find shortcuts suppressed while inline editor focused (Esc still cancels draft) |

## Gap filled during finish

Codex had added an unconditional `request_repaint_after(capture::TICK)` at the start of `capture_logic`. egui's `has_requested_repaint()` treats delayed ticks as pending, so Settling never went quiet (hundreds of frames, 20 s timeout). Fix: keep Visible/Focus while `frames == 0`, and leave keep-alive ticks to the existing settle/loading paths only.

## Tests (required list)

| Requirement | Location | Result |
|-------------|----------|--------|
| Matcher: case, whole word, regex, invalid, clip exclusion | `ui/tests/find_replace.rs` | pass |
| Wrap-around navigation | same + `find::navigate` | pass |
| Turns mode unchanged | `turns_keep_*`, `turns_mode_preserves_legacy_*` | pass |
| Replace planner + clip-tag skips | `planner_counts_skips_*`, regex spanning | pass |
| `save_many` LF/CRLF, one checkpoint, hash conflict, restore | `ui/tests/script_edit.rs` `batch_*` | pass |

Acceptance (clean env, no `SCRIPTCAST_PROJECT`):

- `cargo fmt`
- `cargo build --release` — ok
- `cargo test` — 157 passed, ignored live tests only
- `cargo clippy --all-targets -- -D warnings` — ok
- no lines >120 in `src/*.rs` / `tests/*.rs`
- no colour literals outside `theme.rs`

## Live check (`/tmp/scriptcast-i` only)

Capture (Words mode, muted, export shim on `PATH`):

- `--find racism --screenshot plans/Native_UI/evidence/phaseI-find.png`
- Result: `1 of 29`, gold highlights on matched words, turn 005 selected
- PNG: `plans/Native_UI/evidence/phaseI-find.png` (2996×2252)

Replace-all (test path `phase_i_live_replace_all_changes_and_restore`, whole-word `racism` → `bias`):

- 29 occurrences in 29 turns; skipped in clip tags: 0
- One `script-edit` checkpoint (v05 on this run)
- Replaced SHA-256: `56471afb60f21c58b0d532de4d720a5eca917d2d3dd91fcb0259fe5888b5bab9`
- Restored SHA-256: `84a74ecb54ab23053355a7b23661582244dbf93a64568e43d063ba629c858ccd`

`--changes-json` after applying the replaced bytes (export shim + voice venv, read-only):

```json
"inserted": [5, 7, 13, 15, 18, 29, 56, 73, 75, 83, 88, 93, 96, 98, 101, 104, 105,
  124, 127, 128, 132, 348, 349, 354, 360, 364, 365, 382, 388, 390]
```

(also lists pre-existing `modified: [220]` and deleted turn-ids from the copied episode; not introduced by this replace)

Restore + `shasum -a 256`:

- copy script restored to `84a74ecb54ab23053355a7b23661582244dbf93a64568e43d063ba629c858ccd`
- real `ATO_EP03_redefining_racism_five_levels.md` unchanged: same hash
- real `ATO_EP03_redefining_racism.md` unchanged: `494f9d5abc891a28418c8cb5edf0fec4a67fba388f6e3972ccbd029de13b5abc`

## Notes

- Live export uses `/tmp/scriptcast-i/bin/scriptcast-export` → Theory `.venv-voice` (has `misaki`). System Python without that env fails `--changes-json` with `No module named 'misaki'`; no Python sources were changed.
- No synthesis. Captures were `--mute`. No git writes.
