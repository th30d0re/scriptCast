# Phase D part 2 findings

Model: Composer (finish pass). Codex implemented the UI; this pass verified, fixed
the single over-length line, ran live copy checks, and closed the report.
Started 2026-10-03; finished 2026-10-03 21:16 local.

The operator's no-git-write and worktree-only instructions override the parent
AGENTS.md commit and Obsidian log requirements. This file is the session log
within the authorized write boundary.

## What was built (Codex implementation)

Shared transcript editing in `ui/src/editing.rs` plus wiring across the browser:

1. **Loading.** Worker-thread `load_turns` / `load_changes` after episode load and
   reload; app state holds `ScriptTurns`, `Changes`, `loaded_sha256`, and errors.
2. **Side-panel editor (default).** Turn view / Edit tabs; TRANSCRIPT body or
   multiline draft field (`EDITOR_MIN_HEIGHT` 120); unsaved draft +
   `circle-alert-14-danger`; Save draft / Cancel; synthesis footnote.
3. **Table-row editor.** Extra virtualized height under the selected row; Draft
   badge; Turn NNN · Speaker; field; footnote; Cancel / Save draft.
4. **One `Draft { turn, text, original }`.** Mode switch keeps the draft;
   select-away Keep editing / Discard; Esc cancels only with editor focus; ⌘S /
   ⌘Return save; Space/arrows suppressed while the editor (or any text field /
   dialog) has focus.
5. **Open paths.** E with selection; double-click table row; double-click
   timeline block; `--edit` / `--edit-mode panel|row` capture flags.
6. **Save.** Worker `script::save`; success status + checkpoint version + reload;
   hash conflict keeps the draft with "The script changed on disk. Reload to edit."
7. **Edited markers.** `edited_turns` → Verification "Edited" (accent) + dashed
   timeline outline; count mismatch banner; "Audio is from before this edit"
   while playing an edited turn.
8. **Re-render.** "Re-render edited turns" opens PrecisionInsert confirmation
   with inserted/modified indices and inserted/modified/deleted counts. Not
   confirmed in live checks (synthesis prohibited).
9. **Settings.** Editing → Side panel / Table row; `Preferences.edit_mode` with
   `#[serde(default)]` = Panel.
10. **Checkpoints.** Kind shown (`audio` / `script edit`); restore confirmation
    names absolute script path via `external_files`.
11. **Capture flags.** `--edit` (requires `--select`) and `--edit-mode`.

Icon registered in `ui/design/assets/icons/index.json` and `theme::icon`.

## Brief items 1–11 audit

| Item | Status | Notes |
|---|---|---|
| 1 Loading | Done | Worker receivers; invalidate on reload |
| 2 Side panel | Done | Tabs, field, unsaved, footnote, icon |
| 3 Table row | Done | Virtualized extra-height editor |
| 4 Draft model | Done | Guard, Esc-with-focus, shortcut suppression |
| 5 Opening | Done | E, double-clicks, capture open |
| 6 Save | Done | Worker save, status, conflict message |
| 7 Markers | Done | Table, timeline, banner, play notice |
| 8 Re-render | Done in UI | Confirmation only; live confirm not run |
| 9 Settings | Done | Persisted edit_mode |
| 10 Checkpoints | Done | Kind + absolute replace path |
| 11 Capture flags | Done | Flags parse; screenshots timed out (below) |

No missing or only-partly-done items under the ~30-line fix budget. Finish-pass
code changes were the `jobs.rs` line-length wrap and shasum `eprintln!`s on the
ignored live test.

## What was verified

### Acceptance (clean env: `SCRIPTCAST_PROJECT` unset)

```text
cargo fmt                          ok
cargo build --release              Finished release in ~0.2s
cargo test                         118 passed; 0 failed; 5 ignored
cargo clippy --all-targets -D warnings   Finished ok
awk 'length>120…' src/*.rs tests/*.rs    (no output)
grep colour literals outside theme.rs    (no output)
```

Note: a first `cargo test` with `SCRIPTCAST_PROJECT=/tmp/scriptcast-d2/live`
still exported failed six jobs fixture tests (episode discovery followed the
live project). Cleared the env and re-ran; that contaminated run is not an
implementation failure.

### Unit coverage for D2 behaviour

Draft open/change/cancel/mode keep; select-away guard; marker/`None` banner;
shortcut suppression + Esc-with-focus; both layouts render; preferences default
and round-trip; stale load invalidation / save worker disconnect.

### Live copy (`/tmp/scriptcast-d2/live`)

Project clone + `.venv-d2/bin/scriptcast-export` shim present. App resolved
export as Project virtualenv under that shim when opened with
`SCRIPTCAST_PROJECT=/tmp/scriptcast-d2/live`.

Ignored test `phase_d2_live_copy_save_changes_and_restore` (debug path, not
clicks):

```text
copy_script_before_save=84a74ecb54ab23053355a7b23661582244dbf93a64568e43d063ba629c858ccd
copy_script_after_save=f019541891b7974852d506309c6990825e9ed178dd8e82f65d7d3ea129079d15
copy_script_after_restore=84a74ecb54ab23053355a7b23661582244dbf93a64568e43d063ba629c858ccd
```

`--changes-json` after save listed turn **6** under `inserted` (with prior copy
artifacts 15, 388 and modified 220 from D1):

```json
{
  "inserted": [6, 15, 388],
  "modified": [220],
  "deleted": ["9e5be0c4c949b6bd", "79e9339dd518836b", "4f523505cf9bd2dd"],
  "unchanged": 388
}
```

Checkpoint `v03-…-script-edit`; restore produced `v04-…-pre-restore`; copy
bytes returned exactly. Artifacts:
`/tmp/scriptcast-d2/live-changes-after-save.json`,
`/tmp/scriptcast-d2/live-roundtrip.json`.

### Real EP03 script (read-only)

Path:
`/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/podcasts/ATO_EP03_redefining_racism_five_levels.md`

| When | sha256 |
|---|---|
| Before all live work | `84a74ecb54ab23053355a7b23661582244dbf93a64568e43d063ba629c858ccd` |
| After save/restore on copy | `84a74ecb54ab23053355a7b23661582244dbf93a64568e43d063ba629c858ccd` |

Unchanged. No synthesis; Precision insert never confirmed; no DAW launch; no
other-agent CLI; no `osascript`.

## What is unverified

- **Screenshots.** Both captures timed out after 20 s waiting for the screenshot
  event (display asleep). Recorded verbatim; no workaround.
  - Command: `--mute --select 6 --edit --edit-mode panel|row --size 1440x900`
    against `/tmp/scriptcast-d2/live/outputs/ATO_EP03_five_levels`
  - Expected paths `plans/Native_UI/evidence/phaseD2-panel.png` and
    `phaseD2-row.png` were **not** written.
- Pixel match to design crops (`03c` / `03b`) without captures.
- Live click-path save from the UI (save exercised only via the ignored Rust
  test calling `script::save`).
- Live Keep editing / Discard modal, Esc-in-focus, and ⌘S in the running app
  (covered by unit tests only).
- Hash-conflict UI notice on a live collide (unit-covered in D1/D2 tests).
- Confirming Precision insert / re-render (prohibited).

## Remaining gaps

1. Re-run panel/row captures when the display is awake; drop PNGs into
   `plans/Native_UI/evidence/`.
2. Optional: one muted interactive pass of E → edit → Keep editing / Discard.
3. Count-mismatch banner is unit-tested; not exercised on the live EP03 copy
   (manifest and script both 392 turns).
4. Pre-existing copy relocation artifact: modified `[220]` and inserted
   `[15, 388]` from D1 remain until a real re-render (out of scope).

## Captures

| File | Result |
|---|---|
| `phaseD2-panel.png` | Missing — screenshot event timeout |
| `phaseD2-row.png` | Missing — screenshot event timeout |

Logs: `/tmp/scriptcast-d2/logs/capture-panel.log`, `capture-row.log`.

## Finish-pass changes (this agent)

- `ui/src/jobs.rs`: wrap checkpoint label line (was 122 chars) and Restore enable
  expression for the 120-char rule.
- `ui/tests/script_edit.rs`: print before/after/restore copy script sha256 in the
  ignored live test.
- `cargo fmt` applied.
- This findings file rewritten.

No git writes.
