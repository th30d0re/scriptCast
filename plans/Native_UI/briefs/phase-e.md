# Phase E: read-along word highlighting

Your working directory is a git worktree of scriptCast on branch `ui/phase-e`, created from `dev`, which has all phases through D merged (app 0.6.0).

## Prohibitions (read first)

- **No git writes.** The orchestrator commits.
- **Never start other agent CLIs** (`codex`, `kimi`, `agy`, `gemini`, `cursor-agent`, `claude`), and never drive other apps (no `osascript`). Do the work yourself.
- **Write only inside this worktree**, plus `~/.cargo`, `ui/target/` and `/tmp/scriptcast-e/`. Put scratch files in `/tmp/scriptcast-e/`.
- **The real project is read-only:** `/Users/emmanuel/Documents/Theory/TheOriginalPower/`. Live runs use a `cp -cR` copy of EP03's output folder plus copies of its script, `scriptcast.toml` and voices file, under `/tmp/scriptcast-e/`.
- **Python changes allowed** (operator-approved on 2026-10-03 for Phase E): a new `scriptcast/tools/word_timings.py`, its console-script entry in `pyproject.toml`, and `tests/test_word_timings.py`. You may **import** from `scriptcast/tools/captions.py`, but do not change it.
- No synthesis. Never launch Live or Logic. Mute captures.
- Lines under 120 characters, readable code, `cargo fmt`, no deleted comments, no colour or size literals outside `theme.rs`.

## Goal

While a turn plays, a row under the playing row in the table shows that turn's words, and highlights the word being spoken. This is the "Read-along transcript" in `ui/design/screens/03c-alt-b-side-panel-edit/screenshot.png` and `read-along.png`; Figma nodes `23:150`–`23:166` are in `ui/design/page-metadata.xml`:

- words in `text/primary`, 13 px, 4 px gaps, wrapping;
- the current word on an `accent` pill (radius 3, padding 4×2), semibold, in `on-accent` text.

## Established facts

- `scriptcast/tools/captions.py` already transcribes with mlx-whisper word timestamps (`transcribe`, lines 126-134). It aligns recognised words to the script's own words with `difflib` (`script_words`, `_assign_times`, `_fill_gaps`).
- **But** it transcribes the stitched episode `.mp3`, whose timing may not match the current manifest. Phase E must transcribe the **per-turn processed WAVs** listed in `episode_manifest.json` instead, so word times are relative to each turn's own audio.
- mlx-whisper is installed only in the project venv `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/`. The UI's command resolver (`cli.rs`, step C0) already finds commands in project venvs, or falls back to the dev checkout with `python -m`.
- Archive clips (turns whose text holds `[clip:…]`) have no script words to align. Skip them; the UI shows no read-along for them.

## Build

1. **`scriptcast/tools/word_timings.py`**, as `scriptcast-word-timings <episode_dir> [--script PATH] [--model ID] [--turns 5,10-12] [--out PATH]`.
   - For each turn (skipping clips), transcribe each of its segment WAVs with word timestamps, offsetting each segment's times by that segment's `start_ms − turn start` from the manifest.
   - Align the result to that turn's script words, reusing `captions.py`'s helpers where they fit, or a small local version of the same `difflib` method.
   - Write `<episode_dir>/word_timings.json` (or `--out`) atomically:
     `{"manifest_sha256", "model", "turns": {"<turn_id>": [{"text", "start_ms", "end_ms"}]}}`.
     Times are milliseconds relative to the turn's start; every script word is present, in order, with unmatched words placed by gap-filling.
   - Writing this file is a derived output, like `.fcpxml`. It replaces audio nothing.
   - The default model is `captions.py`'s `DEFAULT_MODEL`.
   - Test with an injected fake `transcribe`, so tests need no model: offsets across two segments, alignment with a dropped word and an extra word, clip turns skipped, atomic write, and `--turns`.
2. **`pyproject.toml`**: register `scriptcast-word-timings`.
3. **UI.**
   - `word_timings.rs` loads `word_timings.json` at episode open, if it is present. It is stale when its `manifest_sha256` differs from the current manifest's; then show nothing and note "Read-along timings are out of date" in the inspector.
   - Add a Jobs preset **"Read-along timings"**: no synthesis, so no confirmation is needed beyond the existing rule for files it overwrites. It overwrites `word_timings.json` when that exists, so it takes a checkpoint like Verify. Resolve the command like the others.
   - **Table:** while a turn is playing, and when timings exist for it, draw a read-along row directly under the playing row, about one line tall, wrapping to two lines at most, then ellipsis. It shows the turn's words, with the word whose `start_ms ≤ position < end_ms` highlighted.
     - Keep repaints at the existing 50 ms playback tick; add no extra continuous repaint.
     - Virtualised rows must account for the extra height, the way Phase D's row editor does in `editing.rs` and `table.rs`. Read that code and follow the same approach.
   - **Settings:** add "Playback → Read-along highlighting" (default on).
4. **Tests.**
   - Rust: current-word lookup (before the first word, in a gap, after the last word); staleness detection; row-height accounting.
   - Python: as listed in item 1.

## Acceptance

```bash
python3 -m pytest tests/test_word_timings.py tests/test_export_session.py -q
cd ui && cargo fmt && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs
```

**Live run, on the `/tmp` copy only.** Run the real `scriptcast-word-timings` with `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m scriptcast.tools.word_timings`, with `PYTHONPATH` set to this worktree, on `--turns 5-12` of the copy (not all 392). Record:
- the time taken;
- one turn's words with times, checked against the audio length.

Then capture `--select 7 --play --at 3000 --screenshot plans/Native_UI/evidence/phaseE-readalong.png` with `SCRIPTCAST_PROJECT` pointing at the copy. If the capture times out because the display is asleep, record that and move on.

Write `plans/Native_UI/evidence/phaseE-findings.md` early and keep it current.
