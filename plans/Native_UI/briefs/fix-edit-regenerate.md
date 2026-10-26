# Fix: editing → regenerate loop (operator report, 2026-10-11)

Working directory: a git worktree on branch `ui/fix-regen`, from `dev` (app 0.20.0).

## Rules

- **No git writes.**
- **Never start other agent CLIs**, and never use `osascript`.
- **No Python changes.**
- **The real project is read-only.** Use `/tmp/scriptcast-k/proj`, a copy that has `.venv-l`.
- Lines under 120 characters; `cargo fmt`; delete no comments; no literals outside `theme.rs`.
- Redirect cargo output to a file.

## Operator report and measured evidence

1. **The inline editor is tiny.** In the inline (row) layout, editing a turn shows the tag toolbar, then a body editor collapsed to **one line**, with the text running off to the right. Save draft and Cancel sit under it. This is a regression from Phase K's tag toolbar (`tag_editor.rs:105`). Before Phase K, the row editor showed the full text at its natural height.
2. **There is no visible way to regenerate after an edit.** The operator found "Re-render edited turns" (`inspector.rs:344`) and "Regenerate selected" (`main.rs:2112`) only through an "Interrupted job" popup (`phase_j.rs:1133`).
3. **Regenerating did nothing, and showed no indication.** Measured in the real project (`outputs/ATO_EP05/_jobs/*.log`): **every** detect-changes and regenerate job failed instantly with:

   ```text
   scriptcast: error: unrecognized arguments: --progress-jsonl …
   ```

   Cause: `cli.rs:294` prefers a project venv that has an installed `bin/scriptcast`. Here that is `.venv-voice/bin/scriptcast`, which is editable-installed from an **older second clone** (`/Users/emmanuel/Documents/Theory/scriptCast`, branch `v0.2-video-tooling`) that predates the Phase J flags. The job's non-zero exit was never surfaced. The only sign was the "Interrupted job" popup with "ended without a done or error event".
4. **Retime should happen automatically** when the episode is opened, if the script's timestamps are stale.

## Build

**A. Capability-checked command resolution** (`cli.rs`)
1. For each candidate (folder, venv console script, PATH, then dev checkout with each venv interpreter), check that it supports the flags the app is about to pass. Run `<cmd> --help` once per candidate, cache the result per program path and mtime, and require each flag the preset uses (`--progress-jsonl`, `--json`, `--edit-json`, `--resume`, `--export-only`…).
2. Skip a candidate that lacks a required flag, and record why: "`.venv-voice/bin/scriptcast` is an older scriptCast (from `/Users/…/Theory/scriptCast`) without `--progress-jsonl`".
3. The dev checkout the app was built from (`find_dev_checkout`) is a valid fallback with a venv interpreter that imports its dependencies, as today.
4. Show the chosen source and any skipped stale installs in Settings → Command sources (it exists), with a visible one-line warning in the Jobs panel when a stale install was skipped: "Skipped an older scriptCast install in .venv-voice. Using the dev checkout."

**B. Job failure is always visible**
1. When a detached job exits non-zero, or exits without a `done`/`error` line, show a **danger banner** in the status strip and the Jobs panel: "<Preset> failed: <last stderr line>", with "Show log" (opens the log tail in a mono panel) and "Dismiss".
2. Never show "Interrupted job" for a process that has **exited**. That label is only for jobs whose pid vanished while the app was closed. A failed job is "Failed".
3. While any job runs, the status strip shows "<Preset> · <progress or elapsed>" and Cancel. This is already true for renders; make it true for every preset.

**C. Regenerate is discoverable**
1. After a body edit is saved, the edited row's Verification cell shows "Edited · audio stale", and the row gets a **"Regenerate"** button. Show it in the row in inline mode, and under the editor in panel mode.
2. Also add "Regenerate turn" to the Phase K context menu.
3. A toolbar-level "Regenerate N edited turns" button appears in the filter bar whenever any turn has stale audio. It runs Detect changes and then Precision insert, the existing flow, as one job sequence, with a checkpoint taken first.
4. When regeneration finishes, the episode reloads, the waveforms and audio update (invalidate the waveform cache and any decoded audio for those turns), and the row shows "Regenerated · not verified".

**D. Inline editor height**
1. In row mode, the body editor is at least the height of the text at the row's width, with a minimum of 3 lines. It wraps at the transcript column width, with no horizontal overflow.
2. The tag toolbar stays one compact row above it.
3. Save draft and Cancel sit right-aligned below it, with no overlap with the next row. Re-check that the row height measurement includes the toolbar, the editor and the buttons; earlier, a too-short row let the next row steal the Save click.

**E. Auto-retime on open**
1. When an episode opens, run `scriptcast-retime` in its preview mode, as Phase K does on a temporary copy, as a background job.
2. If any timestamps differ, apply them automatically: take a checkpoint, do a hash-checked write, and show a dismissible notice: "Retimed N timestamps to match the audio · Undo". Undo restores the checkpoint.
3. If none differ, do nothing visible.
4. Never retime while a render job is running, or while the episode has unsaved edits. In those cases, retry after the job finishes or the edit is saved.
5. Add a Settings toggle, "Retime script on open", defaulting to **on**.

## Acceptance

1. `cargo fmt --check`, clippy with `-D warnings` and `cargo test` must all be green. New tests cover:
   - capability resolution, using fake candidate scripts whose `--help` lacks or has the flag: the stale one is skipped, with the reason recorded;
   - the failed-job banner for a non-zero exit with no terminal line, using a shell fixture;
   - "Interrupted" versus "Failed" classification;
   - the stale-audio marker after an edit, and the Regenerate argv;
   - the row editor height for long text;
   - the auto-retime decision table (no change, change, running job, unsaved edits, toggle off).
2. Add capture flags `--edit-turn <n> --edit-mode row` (on the real-copy episode) and `--failed-job-fixture <log>`.
3. Bump the version to 0.20.1.

## Findings

Write `plans/Native_UI/evidence/fix-regen-findings.md` early.
