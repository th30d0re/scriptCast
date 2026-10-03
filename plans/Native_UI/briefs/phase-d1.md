# Phase D, part 1: inline-editing engine (Python read-only modes, script save, script checkpoints)

Your working directory is a git worktree of scriptCast on branch `ui/phase-d`, created from `dev`. Phases A, B, B2 and C are merged; B3 (waveforms) is being finished in a separate worktree. **Part 2, the editing UI, is not yours.** Keep `main.rs`, `timeline.rs`, `table.rs` and `inspector.rs` untouched, except a one-line `mod`/`pub mod` addition in `lib.rs`.

## Prohibitions (read first)

- **No git writes.** The orchestrator commits.
- **Never start other agent CLIs** (`codex`, `kimi`, `agy`, `gemini`, `cursor-agent`, `claude`). Do the work yourself.
- **Write only inside this worktree**, plus `~/.cargo`, `ui/target/` and `/tmp/scriptcast-d/`. Put scratch files in `/tmp/scriptcast-d/`, never in the worktree.
- **The real project is read-only:** `/Users/emmanuel/Documents/Theory/TheOriginalPower/`. That includes its scripts.
  - Tests and live checks must copy what they need into `/tmp/scriptcast-d/`: `cp -cR` the EP03 output folder, `cp` the script and `scriptcast.toml`, and copy the voices file the toml points at.
  - The EP03 script is also being edited by the operator outside the app (`git status` there shows it modified). Never write to it.
- **Python changes are limited to `scriptcast/tools/export_session.py` and `tests/test_export_session.py`.** The operator approved this on 2026-10-03.
- No synthesis and no jobs. Never launch Live or Logic.
- Lines under 120 characters, written as readable code. Run `cargo fmt`. Do not delete existing comments.

## Read

1. `plans/Native_UI/plan.md`: Flow 6 and steps D1, D2, D3 and D5.
2. `plans/Native_UI/tech-plan.md`: the Phase D decisions and failure modes.
3. `scriptcast/parser.py`: `parse_transcript`, `Turn.line_span`, `_compute_turn_id`. **Important:** `line_span = (header_line, last_body_line)`, where the body includes blank lines and HTML comments up to the next header.
4. `scriptcast/render_state.py`: `plan_precision_insert` (:259) and `load_render_state_with_migration` (:390).
5. `scriptcast/__main__.py`:881-895, for how the render CLI finds the project and voices (`project.current()`, `project.voices`, `load_voices`).
6. `scriptcast/tools/export_session.py` (the `--fcpxml` mode is a mutually exclusive group) and `tests/test_export_session.py`.
7. `ui/src/checkpoint.rs`: `PlannedFile { path, stored_as }`, `create(episode, preset, argv, turns, files)`, `list`, `restore`, and `safe_existing`, which refuses paths outside the episode.
8. `ui/src/cli.rs`: how `scriptcast-export` is resolved and run.

## Build

- **D1 (Python).** Add two read-only modes to the existing mutually exclusive group: `--turns-json` and `--changes-json`. Both print JSON to stdout, write nothing, and exit 1 with a message on stderr on failure.
  - `--turns-json`:
    - prints `{"script_path", "script_sha256", "line_ending": "lf"|"crlf", "turns": [{"index", "turn_id", "speaker_id", "display_name", "header_line", "body_start", "body_end", "body"}]}`;
    - the script is found the way the UI does it: `render_state.json`'s `source_file`, relative to the project root; you may accept `--script PATH` as an override for tests;
    - `body` is the exact text of that turn's body lines, without leading or trailing blank lines and without the header;
    - `body_start..=body_end` are its 1-based line numbers; a turn with no body has `body_start == body_end + 1`, so it is empty.
  - `--changes-json`:
    - prints `{"inserted": [index], "modified": [index], "deleted": [turn_id], "unchanged": n}` from `plan_precision_insert(parse_transcript(script), load_render_state_with_migration(...), voices)`;
    - voices are loaded as the render CLI does (project's `voices` from `scriptcast.toml`).
  - Tests: both modes on a temp project, including CRLF, a turn with HTML comments, trailing blank lines, and a text edit that shows up as one inserted plus one deleted id.
- **D2 (`ui/src/script.rs`).** Exactly as plan step D2. Also provide `pub fn load_turns(export: &ExportCommand, episode: &Episode) -> Result<ScriptTurns, String>` and `load_changes(...)`, which run the CLI in the background-friendly way `cli.rs` and `daw.rs` already use (stdin null, login PATH).
- **D3 (`checkpoint.rs`).** Add `create_external(episode, preset, files: &[PathBuf], turns)`, which stores absolute files outside the episode under `external/<file name>`, recording `path` and `stored_as` in `checkpoint.json`. `restore` must put external files back to their absolute paths through the same temp-file-and-rename path, after first taking a pre-restore checkpoint. Keep refusing paths outside the episode for the normal kinds. `list` shows the kind.
- **D5 (`ui/tests/script_edit.rs`).** As plan step D5, plus an integration test that runs the real `--turns-json` and `--changes-json` on a temp copy of the fixture project, using the dev-checkout fallback with this worktree, and then:
  1. edits turn 1 with `script::save`;
  2. checks that `--changes-json` reports it;
  3. restores the script-edit checkpoint;
  4. checks the original bytes are back.

## Acceptance

```bash
python3 -m pytest tests/test_export_session.py -q
cd ui && cargo fmt && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs
```

Live check on copies only:

- run `--turns-json` and `--changes-json` on a `/tmp/scriptcast-d/` copy of EP03 plus its script;
- record the turn count, which must be 392 if the script and manifest align (otherwise record both counts), and the change lists;
- the operator has edited EP03's script since the render, so a non-empty change list is expected.

## Findings

Write `plans/Native_UI/evidence/phaseD1-findings.md` early and keep it current. Record:

- decisions;
- test output;
- the live-check JSON summaries;
- anything left undone.

A well-supported "this does not work as planned" is an acceptable result.
