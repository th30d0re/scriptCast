# Phase J (Python): machine-readable render, progress and export-only

Working directory: a git worktree of scriptCast on branch `py/phase-j`, from `dev` at d14ef08.
A parallel agent is building the UI against the **Contract** below at the same time, so follow it exactly. If it cannot be followed, say so in the findings file; do not invent a different shape.

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs**, never use `osascript`, and never launch Logic or Live.
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** Use fixtures under `tests/` or copies in `/tmp/scriptcast-j/`.
- **No real synthesis in tests.** Use the existing test doubles for engines (see how `tests/` already fakes engines).
- **Python changes are approved** (operator, 2026-10-07) for exactly these files:
  - `scriptcast/__main__.py`;
  - `scriptcast/tools/repair_render.py`, `verify_render.py`, `relayout_episode.py`, `stitch_episode.py`, `turn_index.py`, **only** to add `--progress-jsonl`;
  - a new `scriptcast/progress.py`;
  - tests.
- **Every new flag is opt-in.** Without the new flags, output and behaviour must be **byte-identical** to today. Add a test that proves this for the dry run.
- Redirect pytest output to a file and read the tail: `pytest -q > /tmp/j-py.log 2>&1; tail -30 /tmp/j-py.log`.

## Established facts

- `scriptcast/__main__.py:290` asks interactively when output exists ("Type 'resume', 'overwrite', or 'abort'"); it exits on EOF. A GUI cannot answer it.
- The dry run prints a text summary only (`_print_dry_run_summary`, :773).
- Exports are gated in `__main__.py` at :1034 (ALS, `--skip-als`), :1040 (`--fcpxml`) and :1045 (`--logic` / `--logic-dry-run`). They only run after synthesis.
- There is no per-turn progress output today.

## Contract (the UI depends on this exactly)

### 1. `scriptcast --dry-run --json-out PATH`

Writes the following, also when there are warnings. An unknown speaker becomes a warning, not an exception, **in this mode only**.

```json
{"schema": 1, "episode_id": "...", "transcript": "/abs/path.md", "voices": "/abs/voices.yaml",
 "turns": [{"index": 0, "turn_id": "...", "speaker_id": "alex", "speaker_name": "Alex",
            "timestamp_ms": 0, "first_words": "Welcome to ...", "clip_id": null}],
 "speakers": [{"id": "alex", "name": "Alex", "engine": "omnivoice", "turns": 68}],
 "warnings": [{"kind": "preamble_skipped" | "unknown_speaker" | "other",
               "line": 1, "speaker_id": null, "blocking": false, "message": "..."}]}
```

- `engine` is `null` for an unknown speaker; `unknown_speaker` is `blocking: true`.
- `first_words`: the first 12 words of the display text.

### 2. `--progress-jsonl PATH`

Applies to `scriptcast`, `scriptcast-verify`, `scriptcast-repair`, `scriptcast-relayout`, `scriptcast-stitch` and `scriptcast-turn-index`. The tool appends one JSON object per line and flushes after each, through `scriptcast/progress.py`:

```text
{"event":"start","tool":"render","total":342,"ts":"ISO8601"}
{"event":"turn_start","turn_index":127,"turn_id":"...","speaker_id":"alex","engine":"omnivoice","n":128,"total":342}
{"event":"turn_done","turn_index":127,"turn_id":"...","n":128,"total":342,"seconds":4.2}
{"event":"pass","pass":2,"max_passes":3,"failing":6}               (repair only)
{"event":"result","passed":328,"failed":14}                         (verify; repair after each pass)
{"event":"stage","name":"layout|als|fcpxml|logic|stitch|index"}
{"event":"done","ok":true,"outputs":{"manifest":"...","als":"...","mp3":"..."}}
{"event":"error","message":"..."}
```

- `verify` emits `turn_start`/`turn_done` per checked turn.
- Every run ends with exactly one `done` or `error`, including on exceptions and SIGTERM (catch it, write `error` with "cancelled", exit 130).

### 3. `--resume`

`scriptcast --resume` takes the "resume" branch of the prompt at :290 without asking. `--resume` together with `--overwrite` is an argparse error.

### 4. `--export-only`

`scriptcast --export-only --episode-id X [--out-dir] [--fcpxml] [--logic-dry-run] [--skip-als] [--ableton-project-als P]`:
- read the existing `episode_manifest.json` and `render_state.json`;
- regenerate the requested exports with the same generators and paths as a normal render;
- do no parsing-driven synthesis and write no WAVs.
- If the manifest is missing, exit 2 with a clear message.
- Emits progress `stage` events when `--progress-jsonl` is given.

## Acceptance (run these)

1. Run `pytest -q`; every existing test and every new one must pass. The new tests cover:
   - the dry-run JSON on a fixture script, including the preamble and unknown-speaker warnings;
   - the dry-run text output, byte-identical without `--json-out`;
   - a progress JSONL from a render with a faked engine (`start`, a `turn_start`/`turn_done` per turn, the stages, `done`);
   - an `error` line on an exception;
   - `--resume` versus `--overwrite`;
   - `--export-only` writing an ALS (and FCPXML when asked) and no WAVs;
   - `--export-only` failing on a missing manifest;
   - `pass`/`result` events from repair, with faked verify results.
2. Run `scriptcast --help`; the new flags appear with clear help text.
3. Update the `README.md` "Useful render flags" line with the four new flags. Change nothing else in the README.

## Not yours to claim

Real synthesis on the real project, and UI integration.

## Stop conditions

- If faking the engine for the progress test needs changes outside the approved files, stop and report.
- Stop and report if making unknown speakers non-fatal in the dry run would change the non-JSON path.

## Findings file

Write `plans/Native_UI/evidence/phaseJ-py-findings.md` **early**. Fill it in as you go: the files changed, the test counts, any deviation from the Contract (with the reason), and anything surprising.

## Later work (2026-10-18) — not part of the contract above

This brief is shipped. The section below is new scope for a later loop. Do not reopen the flags, the JSON shapes, or the approved file list while doing it.

Operator decision: anything taken from Voicebox is rewritten in Rust. Do not copy `backend/utils/chunked_tts.py` into this package.

**J9. Long-speech chunking.** `_speech_jobs` in `scriptcast/__main__.py` sends each speech markup chunk to the engine as one call. A Rust library on that path, called by the renderer for J2, Precision insert, and Regenerate, splits text that is longer than the engine's chunk limit:

1. At the last real sentence end (`.!?`, and CJK `。！？`). Skip a period after an abbreviation (`Mr`, `Dr`, `U.S.`) and a period in a decimal.
2. Never cut inside a `[...]` tag.
3. Otherwise at the last clause boundary (`;:,—`).
4. Otherwise at the last whitespace.
5. Otherwise a hard cut that still lands outside a tag.

If the chunk's audio is unstable, split that text in half and retry, at most twice. Concatenate the pieces with a short crossfade inside the turn. Gaps between turns stay `scriptcast-relayout` (J3).

J2 progress stays one `turn_start` / `turn_done` per turn. The chunker does not add progress events.

The Python renderer shells out to the Rust binary. It does not link it, and the egui app does not either.
