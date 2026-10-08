# Phase K (Python): structural script edits via scriptcast-export

Working directory: a git worktree of scriptCast on branch `py/phase-k`, from `dev` (app 0.17.0).
A UI agent will build against the **Contract** below, so follow it exactly. Report any deviation in the findings file.

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs**, and never use `osascript`.
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** Use fixtures, or copies under `/tmp/scriptcast-k/`.
- **No synthesis.**
- **Python changes are approved** (operator, 2026-10-07) only in:
  - `scriptcast/tools/export_session.py`;
  - a new `scriptcast/script_edit.py`;
  - `scriptcast/parser.py`, **read-only use only**: import its regexes and helpers, do not change behaviour;
  - tests;
  - the README "Commands" row for `scriptcast-export`.
- Existing `--fcpxml`, `--turns-json` and `--changes-json` output must stay **byte-identical**. Prove it with a test.
- Redirect pytest output: `pytest -q > /tmp/k-py.log 2>&1; tail -30 /tmp/k-py.log`. Run with `PYTHONPATH=.`.

## Established facts

- `export_session.py:201` `main` has a mutually exclusive format group: `--fcpxml`, `--turns-json`, `--changes-json`, plus `--script` (overrides the transcript path).
- `script_json` returns `{"script_sha256", "line_ending": "lf"|"crlf", "turns": [... {"body_start": line, ...}]}` (:189–197). The UI already saves body edits by splicing bytes with this hash check (`ui/src/script.rs::splice`).
- `parser.py:13` `_SPEAKER_RE` matches the header `Display Name (MM:SS)`; `_speaker_id` (:21) lowercases and joins with underscores; `_TAG_RE` is at :30.
- Speakers' display names come from the voices file (`speakers.<id>.name`). The voices path comes from `scriptcast.toml` (`scriptcast/project.py`).

## Contract

### 1. `scriptcast-export EPISODE_DIR --edit-json OPS_FILE --expect-sha256 HEX [--script PATH] [--voices PATH]`

- `OPS_FILE` is a JSON list of operations, applied **in order** and **atomically**: a temp file, then a rename. Either all apply or the script is unchanged.
- Indices refer to the turn list as it stands **before each op** (that is, after the previous ops).
- Operations:
  - `{"op": "insert", "position": N, "speaker_id": "aisha", "timestamp": "03:21" | null, "body": "text"}`: the new turn becomes index N, and `position` may equal the turn count (append). A `null` timestamp means the midpoint of its neighbours' timestamps, rounded down to whole seconds; when inserting at the end, use the previous timestamp + 5 s; at the start, 00:00. `body` must be non-empty.
  - `{"op": "delete", "index": N}`
  - `{"op": "move", "index": N, "to": M}`: the turn ends up at index M. Its timestamp is kept as written.
  - `{"op": "set_speaker", "index": N, "speaker_id": "aisha"}`: rewrites only the header display name, keeping the timestamp.
- A `speaker_id` must exist in the voices file. The header uses its `name`, or the id title-cased if it has no `name`.
- An `archive` speaker's turn body must start with a `[clip:id]` tag that exists in the clips registry; otherwise reject it.
- **Byte preservation:** bytes outside the touched turns stay identical, including the preamble, comments, blank-line layout and line endings (lf/crlf). Each turn moves as its exact byte block (header through the line before the next header). A new turn is written as `Name (MM:SS)\n<body>\n` followed by one blank line, using the file's line ending.
- **Exit codes:**
  - 3: the hash differs from `--expect-sha256` (`stale script`); nothing is written;
  - 1: validation errors, listing every failed op as `op[i]: message`; nothing is written.
- **On success**, print the same JSON shape as `--turns-json` for the new file (new `script_sha256` included), and exit 0.

### 2. `scriptcast-export EPISODE_DIR --tags-json [--script PATH]` (read-only)

Prints `{"script_sha256", "problems": [{"index": N, "line": L, "tag": "[pause:abc]", "message": "..."}]}`.
- Validation rules:
  - `[pause:<int>ms]` or `[pause:<int>]` with 0 < ms ≤ 10000;
  - `[beat]`;
  - `[emphasis]`;
  - `[tone:<non-empty>]`;
  - `[clip:<id>]`, where the id exists in the registry and the clip appears at most once per turn.
- Any other bracket is reported with the message "unrecognised tag; it will be spoken aloud" (a warning, but in the same list).
- Also report an archive speaker's turn whose body lacks a clip tag.

### 3. `scriptcast-export EPISODE_DIR --speakers-json [--voices PATH]` (read-only)

Prints `{"speakers": [{"id", "name", "engine", "has_expressive": bool}], "clips": [{"id", "source", "start", "end", "citation"}]}`. The UI's speaker picker and clip picker read this.

## Acceptance

1. Run `pytest -q`; everything must be green. The new tests cover, on fixture scripts in **both lf and crlf**:
   - each op alone, and several ops in one file in sequence;
   - the timestamp interpolation, including at the start and end;
   - an archive speaker with and without a clip;
   - an unknown speaker; out-of-range indices;
   - a stale hash (exit 3, file unchanged);
   - atomicity (op 2 fails, so op 1 is not applied);
   - that a delete followed by an insert of the same block restores the original bytes;
   - every tag rule;
   - `--speakers-json`;
   - byte-identical output for the existing modes.
2. Run the turn parser (`parse_transcript`) on every edited fixture; it must yield the expected turn count, speakers and order.

## Stop conditions

Stop and report if byte-preserving moves cannot be done because a turn block's extent is ambiguous (for example, trailing text after the last turn).

## Findings file

Write `plans/Native_UI/evidence/phaseK-py-findings.md` early, and update it as you go.
