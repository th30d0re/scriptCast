# Phase M (Python): JSON modes for the voice and clip tools

Working directory: a git worktree of scriptCast on branch `py/phase-m`, from `dev` (app 0.19.0).
A UI agent builds against the **Contract** below, so follow it exactly. Report any deviation in the findings file.

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs**, and never use `osascript`.
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** Use fixtures, or copies under `/tmp/scriptcast-m/`.
- **No real synthesis or Whisper in unit tests.** Use the existing fakes, or monkeypatch the transcribe and synthesize calls.
- **Python changes are approved** (operator, 2026-10-07) in these files only:
  - `scriptcast/tools/make_clip.py`, `make_reference.py`, `audition_voice.py`, `calibrate_pronunciation.py`, `stress_check.py`, `check_render_outliers.py`;
  - a new `scriptcast/tool_json.py` for shared helpers;
  - tests;
  - README rows.
- **Every new flag is opt-in.** Without it, output and behaviour are **byte-identical** to today. Add one test per tool that proves this.
- **Every JSON mode follows the same rules:**
  - it prints exactly one JSON object to stdout;
  - logs go to stderr;
  - exit 0 on success;
  - on failure, exit 1 and print `{"error": "..."}`.
- **Writing YAML:** `make_clip.py:89` and `calibrate_pronunciation.py:163` rewrite YAML with `yaml.safe_dump`, which drops comments. When `--json` mode writes (clip registry, pronunciations), route the write through `scriptcast/config_edit.py` (Phase L, merged; it preserves comments and quoting). The legacy non-JSON path stays as it is.
- Redirect pytest output: `PYTHONPATH=. pytest -q > /tmp/m-py.log 2>&1; tail -30 /tmp/m-py.log`. `ruamel.yaml` is in `/tmp/scriptcast-l/venv`; use that venv.

## Contract

### 1. Clip maker: `scriptcast-clip`

1. `scriptcast-clip SOURCE --words-json [--model small|medium]` transcribes the whole source with word timings and does not touch the registry. It prints:

   ```json
   {"source": "/abs", "duration": 123.4, "words": [{"i": 0, "word": "Hello", "start": 1.20, "end": 1.48}]}
   ```

   Cache the result next to nothing; the UI caches it.
2. `scriptcast-clip SOURCE --id ID (--from P --to P | --start S --end E) [--speaker] [--citation] [--url] [--content-note] [--pad] --json` registers the clip, as the CLI does today, with `--start`/`--end` in seconds as a new alternative to the phrases. It prints:

   ```json
   {"id": "...", "registry": "/abs/clips.yaml", "entry": {...}, "turn": "Speaker (00:00)\n[clip:id] transcript…"}
   ```

   It refuses an id that already exists unless `--replace` is given (exit 1).

### 2. Reference maker: `scriptcast-reference`

1. `scriptcast-reference SOURCE --candidates-json [--seconds S]` lists the sentence runs it would choose from, without writing anything:

   ```json
   {"source": "...", "candidates": [{"rank": 1, "start": 3.1, "end": 14.8, "text": "...", "score": 0.91}]}
   ```

2. `scriptcast-reference SOURCE --name NAME --start S --end E --json` writes the WAV and the transcript the same way as today, using exactly that span. It prints:

   ```json
   {"wav": "/abs", "text": "...", "duration": 11.7, "peak": 0.89}
   ```

### 3. Audition: `scriptcast-audition`

`scriptcast-audition ... --json --keep DIR` prints:

```json
{"speaker": "...", "engine": "...", "candidates": [{"reference": "/abs.wav", "score": 0.93, "wer": 0.04,
 "takes": [{"wav": "/abs/in/DIR.wav", "text": "...", "heard": "...", "wer": 0.0}]}]}
```

Look at the tool's real score fields; if they differ, mirror them and document that in the findings file. `--json` requires `--keep`, so that the takes stay playable.

### 4. Calibrate: `scriptcast-calibrate`

`scriptcast-calibrate WORD --reading R --json --keep DIR [--write]` prints:

```json
{"word": "...", "reading": "...", "candidates": [{"spelling": "...", "score": 0.8, "takes": [{"wav": "...", "sentence": "...", "ok": true}]}],
 "best": "spelling", "written": false}
```

With `--write`, it writes the best spelling to the pronunciations file through `config_edit`, so comments survive.

### 5. Stress check: `scriptcast-stress`

`scriptcast-stress WAV --text T --json` prints:

```json
{"heteronyms": [{"word": "...", "expected": "...", "heard": "...", "ok": true}]}
```

### 6. Outliers: `scriptcast-outliers`

`scriptcast-outliers EPISODE_DIR --transcript T --json [--min-words] [--ratio]` prints:

```json
{"outliers": [{"turn_index": 12, "turn_id": "...", "speaker_id": "...", "predicted_s": 4.1, "actual_s": 9.8, "ratio": 2.39, "reason": "..."}]}
```

## Acceptance

1. `pytest -q` must be green. The new tests cover:
   - the JSON shape for each tool, using monkeypatched Whisper and synthesis;
   - byte-identical output without the flag;
   - the clip id collision, and `--replace`;
   - that `--start`/`--end` spans are exact;
   - that writes through `config_edit` keep comments (use a fixture with comments).
2. Run `--words-json` and `--candidates-json` once **for real** on one short source in a `/tmp` copy, using the `.venv-fermion` interpreter with `PYTHONPATH=<worktree>`. That interpreter has `mlx_whisper`. Paste the timing and the word count into the findings file.

## Findings file

Write `plans/Native_UI/evidence/phaseM-py-findings.md` early, and update it as you go.
