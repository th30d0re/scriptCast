# Fix edit → regenerate findings (2026-10-11)

Working tree: `ui/fix-regen`. Version is 0.20.1. No git writes. The GUI was not launched.

## What was wrong

1. **Inline editor height.** `tag_editor::body` drew the tag toolbar and the
   multiline field inside `editing::inline_editor`'s vertical `ScrollArea`, whose
   height was only the wrapped text. The toolbar consumed that viewport, so the
   body collapsed to about one line and ran off to the right. The row added
   `TAG_TOOLBAR_HEIGHT` outside that viewport.
2. **Regenerate was buried.** After a save, the verification cell said "Edited".
   "Re-render edited turns" lived in the inspector and "Regenerate selected" in
   the Jobs panel.
3. **Stale console script.** `cli::resolve_with` returned the first executable
   `scriptcast` under a project `.venv*`. An older editable install rejects
   `--progress-jsonl`, which `detached::run` appends. The non-zero exit wrote
   no `done`/`error` line, and a watched dead pid was labeled Interrupted.
4. **Retime was manual.** Opening an episode did not retime.

## What changed

- `resolve_with_flags` runs `--help` once per program path and mtime when the
  caller passes flags. A candidate that lacks one is skipped and the reason
  records the install path, the editable origin when the script text contains
  it, and the missing flags. `refresh_commands` asks for the flags each command
  actually receives (`--progress-jsonl`, `--resume`, `--export-only` on
  `scriptcast`; `--edit-json` and `--fcpxml` on `scriptcast-export`;
  `--progress-jsonl` on the Phase J utilities). Empty flags keep the old
  first-executable order. Jobs and Settings → Command sources list every skip.
  When the chosen `scriptcast` is the dev checkout, the Jobs panel shows
  "Skipped an older scriptCast install in .venv-…. Using the dev checkout."
- A process this session watched until exit is Complete, Failed, or Canceled.
  Failed (non-zero, or no successful terminal line) sets a danger banner in the
  status strip and the Jobs panel: "<Preset> failed: <last log line>", with
  Show log and Dismiss. Interrupted remains the reattach case: pid gone, no
  recorded exit, no terminal line. Every running preset shows
  "<Preset> · <progress or elapsed>" and Cancel. Renders keep the existing
  "Rendering turn …" line.
- Saved edits show "Edited · audio stale" and, in inline mode, a Regenerate
  button on the row. Panel mode puts Regenerate under the editor. The row
  menu has "Regenerate turn". The filter bar shows "Regenerate N edited turns"
  when any turn is stale. That runs detect-changes, then precision insert, as
  one detached job after one checkpoint. The worker now continues through a
  plain command list until a step fails or the list ends. On success the
  episode reloads (waveforms and the decoded-audio probe are dropped) and those
  rows show "Regenerated · not verified".
- The inline body is not in a scroll viewport. It wraps at the transcript
  column width, at least three lines, and grows with the text up to
  `EDITOR_MAX_HEIGHT`. Save draft and Cancel are right-aligned. The row height
  includes the toolbar, the body, the raw-script line, and the buttons.
- With "Retime script on open" on (the default), opening an episode runs
  `scriptcast-retime` on a temporary copy. Timestamp changes are applied with
  the existing hash-checked write and checkpoint, and the status strip says
  "Retimed N timestamps to match the audio" with Undo and Dismiss. No visible
  change when nothing differs. A running job or unsaved edit defers the pass
  until the episode is accepted again.
- Capture flags: `--edit-turn <n>` (also selects and opens the editor) and
  `--failed-job-fixture <log>` (Jobs inspector, failure banner from the log).

## Tests

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test --all-targets` are green. New coverage:

- stale `--help` is skipped and the reason names `.venv-voice`, the origin
  path, and `--progress-jsonl`
- shell fixture exit 2 with no terminal line sets the failure banner and does
  not set Interrupted; a record with an exit is Finished, a record with neither
  exit nor terminal line is Interrupted
- detect-changes then precision-insert argv
- "Edited · audio stale" and "Regenerated · not verified"
- row editor height grows for long text and stays at least three lines
- auto-retime table: no change, change, running job, unsaved edits, toggle off
- `--edit-turn` and `--failed-job-fixture` parse

## Live capture

Not run. Flags for the orchestrator: `--edit-turn <n> --edit-mode row` and
`--failed-job-fixture <log>`.
