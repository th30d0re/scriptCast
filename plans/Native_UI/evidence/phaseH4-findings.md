# Phase H4 Findings

Status: complete 2026-10-04. Worktree `phase-h4` on `ui/phase-h4`.
No git writes. Theory tree read-only; live work under `/tmp/scriptcast-h4/`.
No Python changes.

## Scope

Full-screen Discrepancy review view (Whisper only / Ensemble toggle),
reusing H5 reviewer decision logic and H3 `stress_ensemble.json`.

## Early discoveries (confirmed)

| Topic | Finding |
|---|---|
| Source labels | Spans store `expected` / `whisper` / `phonon` / `verdict` only. No gap-fill provenance. When Phonon text is present → **"Phonon"** (not "Phonon primary" / "Gap-fill"). Only-Whisper → **"Whisper backfill"**. |
| Disagreement % / coverage warnings | Not in `stress_ensemble.json` or `stress.json`. Omitted; not invented. |
| `feedback_export` | `scriptcast-review add` has no such flag. Opt-in toggle disabled with tooltip **"Not available yet"**. |
| Shared decision logic | `review::choose_outcome` used by Verification inspector and discrepancy view (unit-tested). |

## What landed

| Piece | Location |
|---|---|
| Span / card model | `ui/src/discrepancy.rs` — Whisper-only diffs (`_normalize`-compatible tokens), ensemble mapping, `[`/`]` bounds, seek helper |
| Full-screen view | `ui/src/discrepancy_view.rs` — left cards, right decision, audio strip |
| Shared outcome choice | `ui/src/review.rs` — `choose_outcome`, `Outcome::card_title` / `card_detail` |
| Preferences | `discrepancy_mode` persisted; toggle in the review view |
| Open / close | Verification **"Review discrepancies"**; **D** on failed turn; Esc / **Back to turns** |
| Capture aids | `--review N`, `--review-mode whisper\|ensemble` |
| Tests | Whisper spans, ensemble verdicts/sources, selection bounds, prefs round-trip, `choose_outcome` |

## Acceptance

```text
cd ui && cargo fmt && cargo build --release && cargo test \
  && cargo clippy --all-targets -- -D warnings
# ok (unset SCRIPTCAST_PROJECT — a leftover env from the live copy
#    makes jobs fixture tests look for Theory source_file under /tmp)

awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs   # empty
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src \
  | grep -v theme.rs   # empty
```

## Live run — EP03 under `/tmp/scriptcast-h4`

Copy: episode + `scriptcast.toml`; `Architecting_the_operation` and
`.venv-fermion` / `.venv-voice` symlinked into Theory (read-only).

Injected into the **copy only** (same as H3):

| Script turn | Manifest turn | `turn_id` | Change |
|---|---|---|---|
| 53 | 119 | `424e0d9be600e5ef` | `failed: true`; expected Portugal → **Spain**; score 0.7 |

```bash
SCRIPTCAST_PROJECT=/tmp/scriptcast-h4 \
PYTHONPATH=<worktree> SCRIPTCAST_FERMION=<Theory>/.venv-fermion/bin/fermion \
  <Theory>/.venv-fermion/bin/python -m scriptcast.tools.verify_ensemble \
  /tmp/scriptcast-h4/outputs/ATO_EP03_five_levels \
  --script .../ATO_EP03_redefining_racism_five_levels.md
# real ~21 s · 1 turn · stress.json SHA unchanged
```

| Field | Value |
|---|---|
| Summary | `likely_audio` |
| Scores | Whisper **0.901**, Phonon **0.901** |
| Spans | expected `spain` / whisper `portugal` / phonon `portugal` |

## Evidence

### `phaseH4-whisper.png`

- `--review 119 --review-mode whisper --mute --fresh`
- Whisper only selected; one card: **spain** / **portugal**; verdict
  "Whisper heard differently"; no source label.
- Right column: outcome cards, Save / Cancel, three collapsed sections.
- Audio strip + playback + status shortcuts present.
- Footer: clue line + stress checks (`lives` · ok). Later wording
  tweak to `lives · verb · ok` is in code; re-capture hit a macOS
  screenshot-event flake with another ScriptCast.app open.

### `phaseH4-ensemble.png`

- `--review 119 --review-mode ensemble`
- Ensemble selected; same span with accent **"Phonon"** source label;
  verdict "Engines agree, but not with the script"; hint
  "Likely audio problem · listen to confirm".

## Visual comparison vs target (`discrepancy-review-v2.png`)

| Aspect | Match |
|---|---|
| Full-screen review; header / strip / playback stay | Yes |
| Two columns; card + decision layout | Yes |
| Whisper / Ensemble toggle + "Ensemble · not ground truth" | Yes (selectable labels, not a filled segment control) |
| Danger highlighting on differing tokens | Yes |
| Ensemble verdict + muted hint | Yes |
| Outcome cards + Save / Cancel + disclaimer | Yes |
| Collapsed Provenance / correction / model opt-in | Yes |
| Status shortcuts + "Audio unchanged · Draft" | Yes |
| Source "Phonon primary" / "Gap-fill" | **No** — data cannot distinguish; labelled **Phonon** |
| Surrounding context words in card phrases | **No** — spans only store dirty tokens |
| Disagreement % / coverage warnings in danger colour | **No** — not stored |
| Model-improvement opt-in functional | **No** — disabled ("Not available yet") |
| OpenDyslexic widths | Yes (app font) |

Honest note: the Figma frame shows a richer multi-span EP04 example with
gap-fill provenance and merge diagnostics that H3's JSON does not carry.
The live EP03 Spain/Portugal card exercises the real data path without inventing
fields.

## Notes

- Mute captures; Theory tree untouched.
- Join failed turns by `turn_id` → manifest **119** (stress `turn` index is stale).
