# Phase H5: review feedback loop that improves accuracy over time

Working directory: a git worktree of scriptCast on branch `ui/phase-h5`, from `dev` (app 0.11.1; H1–H3 merged).

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs, and never use `osascript` or drive other apps.**
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** Live checks use `/tmp/scriptcast-h5/` copies of the project files and an episode.
- **Python changes are allowed** (the operator approved H1–H5 on 2026-10-04) in:
  - new `scriptcast/review.py` and new `scriptcast/tools/review_stats.py`, with their tests and the `pyproject.toml` console script;
  - **`scriptcast/tools/verify_render.py`, minimally.** Both changes below must be no-ops when the new files are absent. With no `accepted_hearings.yaml` and no hint list, output must be **byte-identical** to today. Prove it with a test that runs verify scoring on fixture text with and without the feature.
- No synthesis. Mute captures.
- Lines under 120 characters, readable code, `cargo fmt`, no deleted comments, no colour or size literals outside `theme.rs`.

## Read

1. `plans/Native_UI/plan.md`, "Phase H", item **H5** (the spec).
2. `scriptcast/tools/verify_render.py`: `_normalize` :70, `_sounds_alike` :76, `_score` :124, and the Whisper call at :236.
3. `scriptcast/tools/word_timings.py`: its Whisper call.
4. `scriptcast/tools/verify_ensemble.py` and `ui/src/stress_ensemble.rs` (H3).
5. `scriptcast/project.py`: how the project root and `scriptcast.toml` are found.
6. The project layout: per-project files live under `<project>/Architecting_the_operation/scriptcast/` in TheOriginalPower. Use the same folder as `voices`/`pronunciations` from `scriptcast.toml`, i.e. the folder of the configured `voices` file. Name it `project_dir` in code and document the rule.
7. The UI's Verification inspector (`inspector.rs`) and the Phase H3 Second opinion block.

## Build

1. **`scriptcast/review.py`.**
   - `append_review(project_dir, record)` appends one JSON line to `review_log.jsonl`, with fields `{turn_id, episode, span: {start_token, end_token}, script_text, heard: {whisper, phonon?}, outcome, reviewer_correction?, timestamp}`, where `outcome` is one of `recognizer_misheard`, `audio_wrong`, `script_needs_correction`. Append-only.
   - `load_accepted_hearings(project_dir) -> dict[str, set[str]]` reads `accepted_hearings.yaml` (`word: [heard, ...]`, normalised). Each entry carries a comment linking to its review timestamp.
   - `add_accepted_hearing(project_dir, word, heard, review_ts)` writes the file atomically.
   - `vocabulary_hints(project_dir, limit=40) -> str`: the distinct script words from `recognizer_misheard` reviews, newest first, joined for Whisper's `initial_prompt`.
2. **`verify_render.py`, minimal.**
   - (a) When `accepted_hearings` is non-empty, `_score` and the run detection treat a heard token as matching the expected token when it is listed for that word.
   - (b) When hints exist, pass `initial_prompt=hints` to `mlx_whisper.transcribe`.
   - Add `--no-feedback` to switch both off.
   - Record in the `stress.json` entry which mechanisms applied (`"feedback": {"accepted": n, "hints": true}`), and add this key **only** when one applied.
3. **`word_timings.py`:** pass the same hints when present. Gate this the same way.
4. **`scriptcast-review-stats <project_dir> [--episode ID]`** prints:
   - the false-failure rate per episode (failures later marked `recognizer_misheard`);
   - repeat-offender words across episodes;
   - for words with an accepted hearing, failures before vs after the hearing was added (from the timestamps in `stress.json` runs, where available).

   Use the plain-text output style of the existing tools.
5. **UI**, in the Verification inspector for a failed turn: below the Second opinion, a **Reviewer decision** block, following Figma "ATO_EP04 · Discrepancy review" where practical (`ui/design/screens/09-audio-clips/discrepancy-review.png`). It has:
   - three radio choices, the outcomes, with the design's one-line explanations;
   - an optional correction text field;
   - **Save decision**, which calls a new `scriptcast-review add …` console script (in `review.py`) through the command resolver, never writing JSON from Rust;
   - for `recognizer_misheard`, a checkbox "Accept this hearing for <word> in future verifications" (default on). It is enabled when exactly one word differs in the selected span; otherwise it lists the word pairs, each with its own checkbox;
   - for `audio_wrong` with a single-word span, a muted hint showing the exact `scriptcast-calibrate record …` command to try a respelling. Never run it.

   After saving, show "Saved · applies from the next Verify".

   Also add "Review stats" to Jobs (read-only output in the log).
6. **The UI must never claim retraining.** Copy: "Saved locally. Used by the next verification; does not retrain any model."

## Tests (required)

- Byte-identical `_score` and output with the feature absent.
- An accepted hearing turns a failing fixture into a pass.
- Hints are passed to the transcribe call (with an injected fake).
- `review.py` append, accepted-hearing round trip, and the hint order and limit.
- The `review_stats` numbers on a fixture log.
- Rust: the decision block enabling rules, and the single-word versus multi-pair accept options.

## Acceptance

```bash
python3 -m pytest tests/test_review.py tests/test_review_stats.py tests/test_verify_render*.py tests/test_verify_ensemble.py -q
python3 -m pytest -q 2>&1 | tail -1   # only the 11 known test_pronunciation failures may remain
cd ui && cargo fmt && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs
```

**Live, on `/tmp` copies only:**
1. Make a copy whose `stress.json` has a failing turn caused by one misheard word. Build it from real EP03 text plus an altered `heard`.
2. Save a `recognizer_misheard` decision with "accept this hearing" through the CLI.
3. Re-score that turn with the copy's verifier scoring path, without re-running Whisper. A scoring-only test helper is fine. Show the failure clears.
4. Run `scriptcast-review-stats`.
5. Capture the inspector's Reviewer decision block to `plans/Native_UI/evidence/phaseH5-*.png`, and describe it honestly.

Write `plans/Native_UI/evidence/phaseH5-findings.md` early and keep it current.
