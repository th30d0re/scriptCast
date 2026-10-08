# Phase J Python findings

## Status

Implementation complete on branch `py/phase-j`.

## Files changed

- `plans/Native_UI/evidence/phaseJ-py-findings.md` (this running record)
- `README.md` (only the Useful render flags line)
- `scriptcast/__main__.py`
- `scriptcast/progress.py` (new)
- `scriptcast/tools/repair_render.py`
- `scriptcast/tools/verify_render.py`
- `scriptcast/tools/relayout_episode.py`
- `scriptcast/tools/stitch_episode.py`
- `scriptcast/tools/turn_index.py`
- `tests/test_phase_j.py` (new)

## Tests

- Phase J contract tests: 9 passed.
- Existing CLI and verify regression selection: 12 passed.
- Suite excluding the unavailable pronunciation dependency: 297 passed.
- Full suite: 301 passed, 11 failed. Every failure is in
  `tests/test_pronunciation.py` and has the same environmental cause:
  `ModuleNotFoundError: No module named 'misaki'`.
- `python3 -m compileall -q scriptcast tests/test_phase_j.py`: passed.
- `git diff --check`: passed.
- Main CLI help and all five tool help screens were checked; every new flag is
  present.

## Contract deviations

- None.

## Surprises

- The system `pytest` launcher does not put the worktree on `sys.path`, so test
  runs used `PYTHONPATH=.` while preserving the requested redirected-output
  form (`pytest -q > /tmp/j-py.log 2>&1; tail -30 /tmp/j-py.log`).
- The `scriptcast` console executable is not installed in this worktree's
  environment, so help was verified with the equivalent
  `PYTHONPATH=. python3 -m scriptcast --help`.
- No real synthesis, Logic, Live, or real project data was used; render and
  repair coverage used test doubles and temporary fixtures only.
