# Phase M Python findings

## Scope and status

- Implemented on `py/phase-m` (2026-10-08).
- Approved files only: `scriptcast/tools/make_clip.py`, `make_reference.py`, `audition_voice.py`, `calibrate_pronunciation.py`, `stress_check.py`, `check_render_outliers.py`; new `scriptcast/tool_json.py`; `tests/test_phase_m.py`; README command rows.
- No git writes, no other agent CLIs, no AppleScript. `/Users/emmanuel/Documents/Theory/TheOriginalPower/` was only read. The live Whisper run used a copy under `/tmp/scriptcast-m/`.
- Unit tests monkeypatch `mlx_whisper.transcribe` and the synthesis engines. No real Whisper or synthesis in pytest.
- Status: complete. After the candidate clamp, `379 passed`, 4 warnings. Log: `/tmp/m-py.log`. Interpreter: `/tmp/scriptcast-l/venv/bin/python` with `PYTHONPATH=.`.

## Shared rules

- Each JSON flag is opt-in. Without it, stdout matches the previous text (covered per tool).
- JSON mode prints one object on stdout. Progress and the old summary lines go to stderr.
- Success exits 0. Failure exits 1 and prints `{"error": "..."}`. Argparse usage errors do that only when a JSON flag is already on the command line; otherwise they stay on stderr and exit 2.
- Clip registry and pronunciation writes in JSON mode go through `scriptcast.config_edit.apply_ops`. The legacy path still uses `yaml.safe_dump`.

## Per-tool notes

### `scriptcast-clip`

- `--words-json` transcribes with word timestamps and does not copy the source or touch the registry. Tokens whose normalized form is empty (punctuation-only) are dropped, so `i` matches the phrase search. `duration` is the file duration from `soundfile`, then `ffprobe`, then the last word’s end. Times are rounded to 2 decimals.
- `--json` registers the clip. `--start` / `--end` are an alternative to `--from` / `--to`. Those seconds are stored as given (rounded to 2 decimals). `--pad` is not applied on an explicit span; the phrase path still subtracts and adds `--pad`.
- An existing id with `--json` exits 1 unless `--replace`. Without `--json`, an existing id is still overwritten, as before.
- `entry` uses the registry keys (`source`, `start`, `end`, `citation`, `origin_url`, `content_note`). `turn` is `Speaker (00:00)\n[clip:id] transcript`. The default speaker label remains `Archive`.

### `scriptcast-reference`

- `--candidates-json` lists every 3–12s sentence run and writes nothing. Each `start` and `end` is clamped to `[0, duration]` of the decoded file. `score` is recomputed from that clamped span: `1 / (1 + abs((end - start) - seconds))`, rounded to 4 decimals. Ranks are sorted again on the clamped span (discovery order breaks ties). The legacy auto-window still cuts Whisper’s raw ends, so a run without `--candidates-json` is unchanged. A 0.1s miss is about 0.91.
- `--json` requires `--start` and `--end` and cuts that sample span (`int(seconds * rate)`). `start < 0` or `end` past the decoded duration exits 1 with `{"error": "span extends outside the recording"}` and writes nothing. The same bounds apply to a non-JSON `--start`/`--end` write, as a `SystemExit` message and exit 1. `text` is the Whisper words that overlap the span. `peak` is the pre-normalization peak, rounded to 3 decimals. `duration` is the cut length in seconds.

### `scriptcast-audition`

- `--json` requires `--keep`. Takes stay in that directory, and `scores.json` is still written.
- The tool does not compute word error rate. `verify_render._score` returns a similarity in `[0, 1]` (higher is better) and the longest mangled-word run. The JSON mirrors that:
  - candidate: `reference` (absolute path), `temperature`, `mean`, `min`, `clean`, `of`, `takes`
  - take: `wav`, `text`, `heard`, `score`, `run`
- `clean` counts passages with similarity ≥ 0.85 and run ≤ 2. Candidates are ordered like the printed table (`clean` descending, then `mean`). One candidate object per reference and temperature.
- A missing reference is `{"error": "missing: ..."}` in JSON mode. Without `--json` the tool still prints `missing:` and continues.
- The contract example’s `score` / `wer` keys are not emitted. `mean` is the candidate score. `wer` is absent on purpose.

### `scriptcast-calibrate`

- `--json` requires `--reading` and `--keep`. Takes are written under `--keep` instead of `outputs/pronunciation_calibration/`.
- `score` is `ok / total` from the existing tally. `takes[].ok` is true when the stress verdict is `"ok"`. `best` is the same winner as the text mode (higher ok-rate; the plain spelling wins a tie). `written` is true only when `--write` actually stores a spelling that beats the plain word.
- `--write` with `--json` updates pronunciations through `config_edit`, so the header and the inline `# the noun` comment survive. Without `--json`, `--write` still uses `yaml.safe_dump` and keeps only the leading comment block.

### `scriptcast-stress`

- Without `--json`, stdout is still one `json.dumps` line per heteronym.
- With `--json`, stdout is `{"heteronyms": [{"word", "expected", "heard", "ok"}]}`. `ok` is `verdict == "ok"`. `expected` is the misaki phoneme string. `heard` is the recognizer’s phoneme string, or `""` when the judge never aligned the word. `reading`, `verdict`, and the distance fields stay on the legacy line only.

### `scriptcast-outliers`

- `--json` prints `{"outliers": [...]}` for turns at or above `--ratio` (default 1.5) and exits 0 even when the list is non-empty. A missing manifest or transcript is `{"error": "..."}` and exit 1.
- Without `--json`, a flagged turn still exits 1 and prints the regenerate hint.
- Fields: `turn_index`, `turn_id`, `speaker_id`, `predicted_s`, `actual_s`, `ratio`, `reason`. Predicted seconds are `(words * ms_per_word + marks * ms_per_mark) / 1000`. Actual seconds are the manifest span minus scripted silence, in seconds. `reason` is `ratio X.XX for N words: <first 60 characters of the turn>`.

## Real Whisper run

Copy only. Source file left in place:

`/Users/emmanuel/Documents/Theory/TheOriginalPower/thoghts/voice_render_DYh5spFJWkP/Samples/Processed/aisha/4c4ac70cf90866fa_chunk_0000.wav`

copied to `/tmp/scriptcast-m/aisha.wav`.

Interpreter: `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-fermion/bin/python` with `PYTHONPATH` set to this worktree. Model: `mlx-community/whisper-small.en-mlx` (`--model small` for the clip tool; the same repo passed to `--candidates-json`).

`--words-json` (stdout `/tmp/scriptcast-m/words.json`):

- source: `/private/tmp/scriptcast-m/aisha.wav` (`Path.resolve()` on this Mac)
- duration: **15.41** seconds
- word count: **45**
- first word: `i=0` `The` **0.00–0.18**
- last word: `i=44` `it.` **15.12–15.38**

`--candidates-json --seconds 9` (stdout `/tmp/scriptcast-m/candidates.json`):

- **10** sentence runs
- rank 1: **6.88–15.60**, score **0.7812**, text begins `It has to be structurally anti-extraction.`
- Whisper’s segment end (15.60) sat past the 15.41s file. That run was before the clamp. `--candidates-json` now clamps that end to the decoded duration (15.41) and rescores the clamped span. The legacy cutter is unchanged.

Nothing was written beside those two JSON files and the copied wav. mlx progress bars went to stderr.
