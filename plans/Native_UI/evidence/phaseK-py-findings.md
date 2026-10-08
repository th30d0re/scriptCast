# Phase K Python findings

## Scope and status

- Started 2026-10-08 on `py/phase-k`.
- Approved implementation scope: `scriptcast/tools/export_session.py`, new `scriptcast/script_edit.py`, tests, and the README `scriptcast-export` command row.
- No synthesis, project-data writes, git writes, agent CLIs, or AppleScript are used.
- Initial inspection found no pre-existing worktree changes.
- Implementation is complete in the approved files. No synthesis or project-data access was performed.

## Contract observations

- `parse_transcript` recognizes turn boundaries only at speaker header lines, and its `line_span` includes all physical lines after a header through the line before the next header. That gives an unambiguous exact byte block for every non-final turn.
- The final turn likewise extends to EOF. Therefore trailing text after the final header is part of that final turn's body under the existing parser, not an ambiguous out-of-turn suffix. The byte-preserving implementation will follow that established extent.
- Existing JSON modes read and hash the raw script before parsing; compatibility tests will snapshot their serialized bytes before adding the new modes.
- Structural edits split the raw UTF-8 file at parser-recognized headers. The preamble and existing turn blocks remain raw bytes; moves reorder those exact blocks. A former EOF block without a terminating newline receives only the necessary inter-block line-ending separator if it is moved before another block or appended after.
- Operations are evaluated in order against an in-memory block list. All per-operation errors are collected as `op[i]: ...`; the script is replaced only after the entire list validates and the on-disk bytes still match the initially hashed bytes.
- New blocks normalize JSON body newlines to the script's detected LF/CRLF style and use the required header/body/blank-line layout.

## Deviations

- No contract deviations in the implementation.
- Environment-only acceptance issue: the required full `pytest -q` invocation reaches 341 passing tests but has 11 failures in `tests/test_pronunciation.py` because the active Python 3.11 environment does not have the declared `misaki` dependency installed (`ModuleNotFoundError: No module named 'misaki'`). No pronunciation code was changed. The exact redirected log is `/tmp/k-py.log`.

## Verification

- `PYTHONPATH=. pytest -q tests/test_phase_k.py tests/test_export_session.py`: **80 passed**. This covers LF and CRLF edits, each operation, sequential indices, all timestamp positions, archive success/failure, unknown speakers, index errors, stale hash exit 3, transaction rollback, byte restoration, tags, speakers/clips, parser round-trips, and the pre-existing export tests.
- `PYTHONPATH=. pytest -q --ignore=tests/test_pronunciation.py`: **337 passed**.
- Required full command was run exactly with redirected output: `PYTHONPATH=. pytest -q > /tmp/k-py.log 2>&1; tail -30 /tmp/k-py.log`; result: **341 passed, 11 failed**, solely due to missing `misaki` as described above.
- `git diff --check`: clean.
- `python3 -m compileall -q scriptcast tests/test_phase_k.py`: clean.
- Existing `--fcpxml`, `--turns-json`, and `--changes-json` implementations remain unchanged apart from extracting their existing script-path resolution into a helper; all 40 pre-existing `tests/test_export_session.py` cases pass, including no-write inventories and exact JSON assertions.
