# Phase D part 1 findings

Model: GPT-6 (Codex). Completed 2026-10-03.

## Requested work and scope

Implemented D1, D2, D3 and D5: read-only Python exports, byte-preserving script edits,
external script checkpoints and regression coverage. Part 2 owns the editing UI.
No git writes, synthesis, production jobs or DAW launches were performed. The required
existing job-runner tests use dummy shell/echo/sleep processes. Real project data remained
read-only. This file also serves as the session log within the authorized write boundary.

## Decisions and implementation

- Python remains the sole transcript parser. Raw bodies retain HTML comments, markdown,
  whitespace within nonblank lines, and internal LF/CRLF endings. The final body line's
  terminator is excluded from JSON. Indices are zero-based; line numbers are 1-based.
- Body bounds trim only surrounding blank lines. Empty spans satisfy start == end + 1.
  Unicode paragraph separators remain within physical lines; bare CR files are refused.
- Script discovery uses render_state.source_file, resolved against the episode's project.
  --script overrides discovery for JSON modes. --out is rejected for those modes.
- Changes use the configured voices and the existing in-memory state migration.
  Tokenize parsed turns before precision comparison, matching the render CLI: raw parsing
  leaves zero speech chunks and would falsely mark rendered turns as modified. Fingerprint
  lookups for pronunciation and archive data are anchored at the same project too.
- Hash raw script bytes before saving. Refuse stale hashes and empty drafts before creating
  a checkpoint. Preserve bytes outside the body, LF/CRLF, EOF newline state and file mode.
  Publish through a temporary sibling, sync and rename. Recheck the hash before rename.
- The plan's infallible splice signature conflicts with refusing empty or invalid input.
  `splice` therefore returns `Result<Vec<u8>, String>`; errors are recoverable in the UI.
- `load_turns` and `load_changes` are blocking worker functions with Result return values.
  Part 2 must call them off the UI thread. They use Resolved.base_command for null stdin,
  login PATH, bytecode suppression and dev-checkout module execution.
- Checkpoint.files remains the legacy storage-path list. New kind and
  external_files: [{path, stored_as}] fields preserve compatibility with the existing UI.
  External paths are canonical absolute destinations, stored under external/<filename>.
  Old manifests default to kind audio; external checkpoints use script-edit. A pre-restore
  checkpoint inherits its source checkpoint's kind, while its preset is pre-restore.
- External filename collisions and malformed mappings are refused. Normal checkpoint
  sources and destinations stay inside the episode. Symlink escapes are refused before
  creating backup directories. Restore first checkpoints current files, then uses sibling
  temp files and rename; temporary files are cleaned up on failure.
- The existing Rust fixture transcript uses syntax the Python parser cannot read. The real
  integration test copies the fixture, changes only the temporary script to valid headers,
  and generates matching render fingerprints there. It invokes this worktree's Python module
  through ExportSource::DevCheckout, edits index 1, observes inserted/deleted changes,
  restores its script checkpoint and verifies original bytes and unchanged status.

## Validation

All requested acceptance checks passed. Full command output is retained under
`/tmp/scriptcast-d/`:

| Check | Result | Log |
|---|---|---|
| python3 -m pytest tests/test_export_session.py -q | 40 passed in 0.32s | pytest.log |
| cargo fmt | Passed | Applied formatting |
| cargo build --release | Passed | cargo-build-release.log |
| cargo test | 103 passed, 0 failed, 4 existing ignored tests | cargo-test.log |
| cargo clippy --all-targets -- -D warnings | Passed | cargo-clippy.log |
| awk line-length check over src/*.rs and tests/*.rs | No output | line-lengths.log |
| git diff --check | Passed | No output |

Python ran with PYTHONDONTWRITEBYTECODE=1, TMPDIR=/tmp/scriptcast-d and
--basetemp=/tmp/scriptcast-d/pytest. Rust commands ran from ui/.

Python coverage includes CRLF, comments, trailing blanks, empty bodies, Unicode separators,
text and voice changes, read-only failure paths, invalid YAML, missing dependencies,
mutually exclusive modes and in-memory legacy migration.

Nine new Rust tests cover byte preservation, EOF and empty spans, hash conflicts, empty
save refusal, backup failure, mode preservation, reversible restore, old manifests,
external filename collisions, symlink/traversal refusal, edited-row count gating and
real Python CLI integration.

The first Rust compile exposed sha2 0.11's digest type lacking LowerHex; per-byte hex
formatting fixed it. One initial cargo invocation used the repository root; it was rerun
from ui/. Initial Python coverage passed 34 tests before the extra edge cases were added.

The required full suite originally wrote to /tmp/scriptcast-c. Three test-only scratch
roots now use /tmp/scriptcast-d (common/mod.rs, daw_cli.rs and the capture.rs unit test).
Three pre-existing long test lines in daw_cli.rs and runner_presets.rs were wrapped to
satisfy the required line-length check. No comments were removed.

## Live check on copies

EP03 output was cloned with cp -cR to:
`/tmp/scriptcast-d/live/outputs/ATO_EP03_five_levels`.

The script, scriptcast.toml, configured voices, pronunciation rules and clip registry
were copied. The sole referenced archive source (atwater_1981) was cloned too. Only the
copied render state's absolute source_file was rewritten to the project-relative script
path, so default discovery reads the copied script. The script path under the copied
project is `Architecting_the_operation/podcasts/ATO_EP03_redefining_racism_five_levels.md`.

System python3 lacks misaki, required by this project's pronunciation fingerprints.
That failure now exits 1 with a stderr message. The successful run used the existing
read-only `.venv-voice/bin/python` interpreter from TheOriginalPower, this worktree on
PYTHONPATH, bytecode disabled, and cache/temp paths inside /tmp/scriptcast-d.
No synthesis was invoked.

The read-only commands left all 814 copied files' sizes and modification times unchanged.
Full outputs: live-turns.json, live-changes.json and live-summary.json in /tmp/scriptcast-d.
Summary:

```json
{
  "script_sha256": "84a74ecb54ab23053355a7b23661582244dbf93a64568e43d063ba629c858ccd",
  "line_ending": "lf",
  "turn_count": 392,
  "manifest_turn_count": 392,
  "render_state_turn_count": 392,
  "changes": {
    "inserted": [15, 388],
    "modified": [220],
    "deleted": ["79e9339dd518836b", "4f523505cf9bd2dd"],
    "unchanged": 389
  }
}
```

Index 220 is lee_atwater. Archive fingerprints include the absolute source pathname,
so copying the project changes this fingerprint even when the source audio is identical.
Recomputing with the original path string and the copied file's size produces
`a88de3dcf342b53a`, exactly its stored speech_text_hash. Thus modified [220] is a copy
relocation artifact; the text edits appear as inserted [15, 388] and their two deleted IDs.
No render state fingerprints were altered to suppress this result.

## Plan audit and remaining limitations

| Step | Outcome | Adjustment |
|---|---|---|
| D1 | Complete; both read-only JSON modes and Python tests | Tokenize before fingerprinting |
| D2 | Complete; loaders, splice, save and edited_turns | splice returns Result for refusal |
| D3 | Complete; external checkpoints, kinds, reversible restore | Preserve legacy files field |
| D5 | Complete; nine Rust tests including real CLI round trip | Adapt only copied fixture script |
| D4/D4b/D6 | Outside this assignment | Left for part 2 |

There is no unfinished part 1 acceptance work. main.rs, timeline.rs, table.rs and
inspector.rs are unchanged; lib.rs has only the one module declaration. No commits made.

The final hash check reduces but cannot eliminate the check/rename race with an external
editor. External restore requires its destination parent to remain at the recorded path;
a moved or symlinked parent is refused. Checkpoint kind is available to callers; displaying
it and handling drafts/errors remains part 2. Projects with pronunciation rules need an
interpreter that supplies the render pipeline's pronunciation dependencies.

## Next ideas (handoff)

1. Wire the blocking loaders into UI worker threads.
2. Handle splice/save errors, especially the Reload message on a hash conflict.
3. Preserve drafts when switching editing locations.
4. Show a count-mismatch banner before mapping edited rows.
5. Surface Checkpoint.kind and external destination paths in restore confirmation.
6. Run part 2 UI acceptance on copied project data.
