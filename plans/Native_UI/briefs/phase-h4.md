# Phase H4: Discrepancy review view with Whisper only / Ensemble toggle

Working directory: a git worktree of scriptCast on branch `ui/phase-h4`, from `dev` (app 0.12.0; H1, H2, H3 and H5 merged).

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs, and never use `osascript` or drive other apps.**
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** Live checks use `/tmp/scriptcast-h4/` copies.
- **No Python changes:** everything the view needs already exists in `stress.json`, `stress_ensemble.json` (H3) and `review.py` (H5).
- No synthesis. Mute captures.
- Lines under 120 characters, readable code, `cargo fmt`, no deleted comments, no colour or size literals outside `theme.rs`.

## Target

`ui/design/screens/09-audio-clips/discrepancy-review-v2.png` (Figma `48:3588`). Build to it, adapted to OpenDyslexic widths.

- **A full-screen review view** replacing the timeline, table and inspector area. The header, the turn's audio strip and the playback bar stay.
- **How to open and close it:**
  - opens from the Verification inspector of a failed turn via a "Review discrepancies" button, or with the D key on a selected failed turn;
  - closes with Esc or a "Back to turns" button.
- **Left column:**
  - The title "Discrepancy review · Turn NNN", then episode, speaker, start time and duration.
  - The segmented toggle **Whisper only / Ensemble**, persisted in Preferences. Beside it the note "Ensemble example / Not ground truth", worded "Ensemble · not ground truth".
  - An "Edit script" button that opens the existing inline editor for the turn.
  - Two column headings: Expected script, Recognizer heard.
  - **One card per difference span**, in order:
    - left: the expected phrase with the differing words in danger colour;
    - right: the heard phrase with the differing words in danger colour, plus a small accent source label;
    - "Difference i of n";
    - a bold verdict line and a muted hint.
  - The selected card has an accent border. [ and ] move the selection, and clicking a card selects it.
  - The footer: "Agreement is a clue, not a verdict. Listen before deciding.", then the turn's stress checks.
- **Whisper only mode:** spans come from diffing expected against Whisper's `heard` (from `stress.json`, using the same tokenisation as `stress_ensemble`). No source labels, no engine verdicts. Each card shows only "Whisper heard differently".
- **Ensemble mode:** spans come from `stress_ensemble.json` for the turn.
  - Source label per span: "Phonon primary" when Phonon supplied the heard words, "Whisper backfill" when only Whisper did, "Gap-fill" when the span came from a gap-fill pass. If the stored data cannot tell gap-fill apart, label it "Phonon" and record that in the findings. **Do not invent a source.**
  - Verdict lines:
    - `likely_audio`: "Engines agree, but not with the script" / "Likely audio problem · listen to confirm";
    - `likely_recognition`: "Engines disagree · listen first";
    - `uncertain`: "Engines disagree with the script and each other";
    - `coverage_gap`: "One engine skipped · coverage gap".
  - Show per-turn coverage warnings and the disagreement % (the "listen manually" line in danger colour) under the card they belong to, or at the top if they are not span-specific.
  - When `stress_ensemble.json` is missing or stale, Ensemble mode shows the existing missing/stale hint with Run….
- **Right column: "Why did this turn fail?"**
  - "Manual decision · not saved"; three outcome cards with radio dots, Save decision, Cancel; and "Saved decisions can be undone. Saving never replaces audio. Regeneration requires confirmation."
  - **Reuse the H5 Reviewer decision logic** (`review.rs`): one implementation shared by the inspector block and this view. Do not duplicate it.
  - Below that, three collapsible sections, collapsed by default:
    - **Provenance:** score / threshold and "Original audio retained";
    - **Corrected transcription draft:** the H5 correction field;
    - **Save for model improvement:** opt-in, off by default, with "no automatic retraining". Saving with it on only records `"feedback_export": true` in the review record through the existing `scriptcast-review add` (check whether it accepts such a flag; if not, leave the toggle disabled with the tooltip "Not available yet" and record that). **No new Python.**
- **Audio strip** above the playback bar: "Turn NNN · Original rendered audio · Paused/Playing", plus the turn's waveform (the existing `waveform.rs` envelope). Clicking a card seeks to the span's approximate time when word timings exist (`word_timings.json`); otherwise it doesn't seek.
- **Status bar hints:** "[ Previous difference ] Next difference Space Play/pause ⌘↩ Save decision". ⌘↩ saves the decision.

## Tests (required)

- Span building in Whisper-only mode from fixture text.
- Mapping `stress_ensemble` spans to cards (verdict lines and source labels).
- [ / ] selection bounds.
- The toggle persists.
- The shared decision logic is used by both views (one function under test).

## Acceptance

```bash
cd ui && cargo fmt && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs
```

**Live, on a `/tmp/scriptcast-h4` copy of EP03.**
- Reuse the H3 approach: alter the expected text of one turn in the copy's `stress.json`, run `scriptcast-verify-ensemble` on it through `.venv-fermion`, and open the view on that turn.
- Add a capture flag `--review <turn>` for this.
- Capture both modes into `plans/Native_UI/evidence/phaseH4-whisper.png` and `phaseH4-ensemble.png`.
- Look at each and compare it honestly with the target.

Write `plans/Native_UI/evidence/phaseH4-findings.md` early and keep it current.
