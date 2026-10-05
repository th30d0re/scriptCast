# Failed-marks findings (2026-10-04)

## What changed

- New `scriptcast_ui::not_heard` helper: `not_heard(expected, heard)` and
  `not_heard_in_display(expected, heard, display)` — tokenise/normalise like
  `verify_render._normalize`, diff, map deleted/replaced expected words to
  display byte ranges (empty when mapping fails).
- Failed rows only. WAV newer than `stress.json` → treat as stale → no marks.
- **Mode A** (`EditMode::Row`): not-heard words in danger bold; summary line
  `N words not heard: "…"`.
- **Mode B** (`EditMode::Panel`): danger pills + small “not heard” label inserted
  as its own `LayoutJob` section after each not-heard run (wrap/height include
  it); pill rects from galley job-char ranges covering word + label. Accent
  read-along pill painted on top when both apply.
- Table remeasures transcript at the painted cell width so wrap matches clip.
- Theme: `ON_DANGER`, `DANGER_PILL`, `NOT_HEARD_LABEL`, `NOT_HEARD_SUMMARY_MAX`.

## Tests

- Helper: deletion, replacement, punctuation, edited display, no-diff, summary
  truncation — pass.
- Row height grows with A-mode summary — pass.
- Pill LayoutJob: label glyphs before following words; danger pills cover word
  + label and not “or”/“diversity” — pass (`cargo test --bins -- pill_`).
- `cargo test --lib not_heard` / `cargo clippy --all-targets -- -D warnings` —
  clean.

## Live capture (EP03 copy)

`/tmp/scriptcast-clip/ATO_EP03_failed` — turn **115** (`e1b0c78e35ea0402`)
forced Failed; `heard` drops **training** and **loop.** from `expected`.

| Capture | Path | Mode |
| --- | --- | --- |
| A | `plans/Native_UI/evidence/failed-marks-A.png` | `--edit-mode row` |
| B | `plans/Native_UI/evidence/failed-marks-B.png` | `--edit-mode panel` |

### A (inline) — honest read

Turn 115 shows **training** and **loop.** in danger bold inside the transcript
cell. Under the text: `2 words not heard: "training loop."`. Row is taller than
neighbours. Filter shows Inline selected. Verification column: Failed.
**Unchanged by this follow-up.**

### B (side panel) — honest read (post LayoutJob fix)

Same turn with Side panel selected. Two danger pills:

1. **training not heard** near the end of the first transcript line; following
   words `or diversity education, works on the` continue on the same line after
   a selected-bg gap (~80 px). No pill over “or”.
2. **loop. not heard** at the start of the next line; `It should…` follows after
   a clear gap. Label is fully visible (not “loop. not” clipped).

OCR of the EOL band: `bias training not heard or diversity education, works on
the`. Galley dump matches: labels are layout sections, not overlays.

**Overlap:** none remaining between labels/pills and following words.
**Clipping:** “not heard” labels are complete inside both pills.
