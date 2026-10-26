# Phase H3: verification second opinion (ensemble beside Whisper, for failed turns)

Working directory: a git worktree of scriptCast on branch `ui/phase-h3`, from `dev` (app 0.10.0; `scriptcast.asr` and `scriptcast-clip-transcribe` are merged).

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs, and never use `osascript` or drive other apps.**
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** You may run its venv interpreters. Live runs use `/tmp/scriptcast-h3/` copies.
- **Python changes are allowed** (the operator approved H1–H5) only in:
  - new `scriptcast/tools/verify_ensemble.py`;
  - new `tests/test_verify_ensemble.py`;
  - `pyproject.toml` (console script).

  You may **import** from `scriptcast/tools/verify_render.py` (`_normalize`, `_score`, `_diff`) and from `scriptcast.asr`, but **do not modify** them.
- No synthesis. Mute captures.
- Lines under 120 characters, readable code, `cargo fmt`, no deleted comments, no colour or size literals outside `theme.rs`.

## Read

1. `plans/Native_UI/plan.md`, "Phase H", item H3.
2. `scriptcast/tools/verify_render.py`:
   - how a turn's expected text and Whisper "heard" text are built (around lines 173–270);
   - the `stress.json` entry shape: `{turn, manifest_turn, turn_id, speaker, score, worst_run, failed, expected, heard, stress[], repetition}`.
3. `scriptcast/asr/engines.py`: `ensemble()` and the per-engine helpers.
4. `scriptcast/tools/clip_transcribe.py`: segment offsets, atomic write, and the `--turns` pattern.
5. The UI side:
   - `ui/src/inspector.rs` (the Verification inspector);
   - `episode.rs` (the `stress.json` join by `turn_id`);
   - `clip_transcripts.rs` (a loader with a staleness check, the model to copy);
   - `runner.rs` and `jobs.rs` (presets);
   - `cli.rs` (the clip-transcribe resolver, which probes `fermion` and `mlx_whisper`; reuse it for this command).

## Build

1. **`scriptcast-verify-ensemble <episode_dir> [--script PATH] [--turns SPEC] [--out PATH]`.**
   - Default turns: the turns marked `failed: true` in `stress.json`, matched by `turn_id`.
   - For each turn, use its `expected` text from `stress.json`, or rebuild it the way `verify_render` does if needed. Take Whisper's `heard` from `stress.json` (do not re-run the verifier's Whisper), and run Phonon-2 through `scriptcast.asr.engines` on the turn's processed WAV segments, with segment offsets.
   - Align three token streams (expected, Whisper, Phonon) with `difflib` over normalised tokens (`_normalize`). Produce spans where at least one engine differs from expected:
     `{start_token, end_token, expected, whisper, phonon, verdict}`.
   - The verdict for a span is one of:
     - `likely_audio`: Whisper and Phonon agree with each other but not with expected;
     - `likely_recognition`: Whisper and Phonon disagree with each other and at least one matches expected;
     - `uncertain`: they disagree with each other and neither matches expected;
     - `coverage_gap`: one engine has no tokens for that span.
   - Per turn, a summary verdict:
     - `likely_audio` if any span is `likely_audio`;
     - else `likely_recognition` if all spans are `likely_recognition`;
     - else `mixed`.
   - Write `<episode_dir>/stress_ensemble.json` atomically: `{manifest_sha256, stress_sha256, turns: {"<turn_id>": {score_whisper, score_phonon, spans, summary}}}`. Use `_score` for both scores.
   - **Never modify `stress.json`.** The original verdict stands; this file is advisory.
   - Tests with injected engines, covering each verdict, offsets, default turn selection from `stress.json`, and the atomic write.
2. **UI.**
   - A new Jobs preset, **"Second opinion (failed turns)"**: checkpoints `stress_ensemble.json`, synthesizes nothing, and resolves its command like `scriptcast-clip-transcribe`.
   - A loader, `stress_ensemble.rs`, that marks results stale when `manifest_sha256` or `stress_sha256` changes.
   - In the Verification inspector for a failed turn, below the original result (which stays unchanged and first), add a **Second opinion** block:
     - a summary line:
       - "Engines agree the audio differs from the script: likely an audio problem", or
       - "Engines disagree with each other: likely a recognition error, listen first", or
       - "Mixed: listen";
     - then the spans, each showing expected / Whisper / Phonon with the differing words emphasised, plus a small verdict label;
     - the Phonon score beside Whisper's.
   - Missing or stale hint, with a Run… button.
   - In the table, failed turns whose summary is `likely_recognition` get a small muted "maybe misheard" note after "Failed", so they can be reviewed first.

## Acceptance

```bash
python3 -m pytest tests/test_verify_ensemble.py tests/test_asr_engines.py tests/test_asr_merge.py -q
cd ui && cargo fmt && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs
```

**Live, on a `/tmp/scriptcast-h3` copy of EP03.** EP03's `stress.json` currently has no failed turns among its matched IDs. Create a test copy whose `stress.json` marks two turns as failed: one where you replace the copy's `expected` text with a deliberately wrong word (which should come out `likely_audio`, since both engines will hear the real audio), and one left as is.
- Run the tool through `.venv-fermion`.
- Record the time taken, the spans and the verdicts.
- Capture the Verification inspector for the altered turn into `plans/Native_UI/evidence/phaseH3-*.png`, and describe it honestly.

Write `plans/Native_UI/evidence/phaseH3-findings.md` early and keep it current.
