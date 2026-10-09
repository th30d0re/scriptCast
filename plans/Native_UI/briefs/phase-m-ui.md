# Phase M (UI): clip maker, reference maker, audition, calibrate, outliers, stress check

Working directory: a git worktree of scriptCast on branch `ui/phase-m`, from `dev` (app 0.19.0). The Phase M Python JSON modes are merged on this branch.

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs**, never use `osascript`, and never launch Live or Logic.
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** Test against `/tmp/scriptcast-k/proj`; it has `.venv-l`, which carries `ruamel.yaml`.
- **No Python changes.** No synthesis or Whisper in unit tests: use JSON fixtures.
- Lines under 120 characters; `cargo fmt`; delete no comments; no colour or size literals outside `theme.rs`.
- **Tooltips:** only for a disabled reason, or for information that is not visible.
- Redirect cargo output: `cargo test > /tmp/m-ui.log 2>&1; tail -30 /tmp/m-ui.log`. Mute captures.
- **Long jobs** (transcribe, audition, calibrate) run through the existing job runner as detached jobs (Phase J `detached.rs`), with Cancel. They never block the UI thread.

## Design

Match `ui/design/M1-clip-maker.png`, `M2-reference-maker.png`, `M3-audition.png` and `M4-calibrate-outliers.png`, inside the Phase L Project data window.

**Ignore in the mock-ups:**
- the bottom annotation strips;
- the ▶ glyph waveforms (draw real ones, decoded through the Phase L cached ffmpeg path);
- the "illustrative" and "example" labels;
- the M1 "alternate · id collision" box and the "after saving" preview. These are states, not extra panels: show the collision as an inline error on the id field, and show the result card only after a save.

**Use the real metrics** (from `plans/Native_UI/evidence/phaseM-py-findings.md`, which overrides the mock-ups):
- **Audition:** each candidate has `mean`, `min`, `clean`/`of` and `temperature`; each take has `score` (similarity) and `run` (the longest mangled-word stretch). There is **no WER**. The big number is `mean` × 100, with `clean/of` takes beneath it. Heard words that differ from the sentence are underlined in the danger colour, using a simple word diff in Rust.
- **Stress:** `heard` is a phoneme string. Show it in mono.

## Contract

The authority is `plans/Native_UI/briefs/phase-m-py.md`, section "Contract", together with the deviations in the findings file named above.

## Where things live

| Entry point | Opens |
|---|---|
| Clips tab "+ Register clip…" (enable it; it was disabled in Phase L) | **M1** Clip maker sheet |
| Voices tab: "Make reference…" button on the reference audio row | **M2** |
| Voices tab: "Audition…" button for engines with references | **M3** |
| Pronunciations tab: "Calibrate…" | **M4** left panel |
| Episode menu → "Find outliers", or the Jobs panel | **M4** right panel; a row click selects that turn in the episode |
| Turn context menu (Phase K) "Check stress" | a stress result card in the side panel |

## Build

**M1. Clip maker**
1. **Source:** a file picker (any audio or video ffmpeg can read). "Transcribe" runs `scriptcast-clip SOURCE --words-json` as a detached job; show progress text while it runs and cache the result in the app per source path and mtime.
2. **Picking words:** words render as flowing text. Click selects the first word; shift-click selects the last. The range shows start → end · duration, and the waveform shades the same range. Dragging a waveform handle moves the range to the nearest word boundary. "Play range" plays it.
3. **Details:** id, with a live collision check against the registry from `show --json`; speaker, limited to archive speakers; citation; URL; content note; pad.
4. **Register** runs `--start S --end E --json`. The result card shows the `turn` text in mono, with **Copy** and **Insert into script after turn N**. The insert uses the Phase K `insert` op with an archive speaker and a clip body.

**M2. Reference maker**
1. "Find sentence runs" runs `--candidates-json`. Rows show rank, time range, duration, a score bar (score × 100), the sentence, and Play.
2. **Expanded candidate:** a waveform with start/end handles, start/end fields, the editable exact transcript, the reference name, and peak.
3. "Write reference" runs `--name --start --end --json`. The result shows the paths, **Reveal files**, and **Use for <speaker> in Voices**, which sets `reference_audio`/`reference_text` in the Voices draft. The user still presses Save there.

**M3. Audition**
1. Pick the speaker and 2–4 reference chips, chosen from files under the project's `references` folder.
2. "Run audition" runs `--json --keep <tmp dir>`, with progress per candidate.
3. Results go in columns, as in M3:
   - the winner badge goes on the highest `mean`;
   - each take shows its sentence, what was heard with differing words underlined, and Play on its WAV;
   - "Use this reference for <speaker>" sets the Voices draft.

**M4. Calibrate, outliers and the stress check**
1. **Calibrate:**
   - word field, reading options, speaker;
   - "Run calibration" runs `--json --keep`;
   - the results grid shows spelling, score × 100 and up to 3 takes, each with Play and ✓/✗;
   - the best row is highlighted;
   - "Write '<spelling>' to pronunciations.yaml" runs again with `--write`, or uses the Phase L `set_entry` op. Pick whichever round-trips comments; the tool already writes through `config_edit`.
2. **Outliers:**
   - `scriptcast-outliers --json` on the open episode;
   - table columns: turn/speaker, predicted, actual, a ratio bar (danger above 1.5× or below 0.6×), reason;
   - clicking a row selects the turn in the main window, and shows the selected-turn card from the mock-up.
3. **Stress check:** runs `scriptcast-stress <turn wav> --text <turn text> --json` and shows a card with word, expected reading and heard phonemes in mono, plus ✓/✗.

## Acceptance

1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` must all be green. The new tests use JSON fixtures in `ui/tests/fixtures/phase_m/`, built from the Python tests' shapes plus the real `--words-json` run in the findings. They cover:
   - word-range selection, including shift-click and handle snapping to word boundaries;
   - the id collision check;
   - the argv for every tool call;
   - the audition winner choice and the heard-word diff;
   - the outlier ratio classification;
   - parsing the stress card;
   - "Use for <speaker>" changing only the Voices draft.
2. Add capture flags:
   - `--clip-maker [SOURCE] [--words-fixture F]`;
   - `--reference-maker [--candidates-fixture F]`;
   - `--audition-fixture F`;
   - `--calibrate-fixture F`;
   - `--outliers-fixture F`.

   Add `plans/Native_UI/evidence/capture-phaseM.sh`. You cannot launch the GUI, so the orchestrator runs it.
3. Bump the version to 0.20.0.

## Findings file

Write `plans/Native_UI/evidence/phaseM-ui-findings.md` early: the files changed, the tests, deviations, and what is unverified.
