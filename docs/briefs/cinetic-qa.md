# Brief: cinetic QA wrapper for scriptCast renders

You are in a git worktree of scriptCast on branch `cinetic-qa`. Only touch the files listed under "Owned files". Commit when done.

## Goal
Run the QA scripts from the cinetic skill (cloned read-only at `/Users/emmanuel/Dev/Tools/cinetic/skills/cinetic/scripts/`) against a scriptCast video render, and produce one combined report. Do NOT copy cinetic's scripts into this repo; call them by path.

## Owned files
- `scriptcast/tools/cinetic_qa.py` (new)
- `tests/test_cinetic_qa.py` (new)
- `pyproject.toml`: add an optional extra `cinetic` with opencv-python, librosa, pyloudnorm, soundfile, scipy, numpy. Nothing else in this file.
- `docs/briefs/FINDINGS-cinetic-qa.md` (new): the baseline report.

## Behaviour of `cinetic_qa.py`
- CLI: `python -m scriptcast.tools.cinetic_qa VIDEO [--cinetic-root PATH] [--out DIR] [--json]`.
  - `--cinetic-root` defaults to env `CINETIC_ROOT`, else `../cinetic/skills/cinetic` relative to the repo root. Fail loudly with a clear message if it does not exist (no silent fallback).
- Runs, each as a subprocess, capturing exit code, stdout and stderr to `OUT/<check>.log`:
  - `probe.py VIDEO` (with `--json` if it supports it; check `--help`). scriptCast renders at 30 fps; pass flags so the check does not assume cinetic's 60 fps/1080p deliverable spec, or record that it does.
  - `forensics.py` and `sheet.py` (need cv2): if cv2 is missing, mark the check `skipped: missing dependency cv2` instead of crashing.
  - `check-sync.py VIDEO [source audio]` if the run directory has the source audio.
  - `lint-film.mjs` against `video/src/` (theme.ts, cards/) via `node`, if it accepts a path; read its `--help` first.
  - Skip `av-audit.py` (it needs a cinetic cues file that scriptCast does not produce); list it as `not applicable` in the report.
- Writes `OUT/report.json` with `{check: {status: pass|fail|skipped|n/a, exit_code, summary}}` and prints a short table. Exit non-zero only if a run check fails.
- Match the style of existing tools in `scriptcast/tools/` (e.g. `check_render_outliers.py`): argparse, `from __future__ import annotations`, no third-party imports at module top level.

## Tests
`tests/test_cinetic_qa.py`, hermetic: fake cinetic root in `tmp_path` with stub scripts that print/exit, assert report statuses, the missing-root error, and the cv2-skip path. No real video, no network. Run `pytest tests/test_cinetic_qa.py > /tmp/cqa-pytest.txt 2>&1` and read only the tail.

## Baseline run
Run the tool for real on `/Users/emmanuel/Dev/Tools/scriptCast/video/public/episode/run-uw1w370f/clip001.mp4` (audio for the run is in the same folder). Do not install packages globally; if cv2 is missing, let those checks skip. Run `lint-film.mjs` on `video/src`.
Write `docs/briefs/FINDINGS-cinetic-qa.md`: what ran, what each check reported (quote the key numbers), which cinetic checks are irrelevant to podcast episodes and why, and the top 5 concrete issues in the render or cards. Separate measured facts from guesses.

## Stop conditions
- Do not modify anything under `/Users/emmanuel/Dev/Tools/cinetic`.
- Do not change existing video/ or scriptcast/ code beyond the owned files.
- If a cinetic script's interface makes a check impossible, record it in the findings and move on.
