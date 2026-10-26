# Phase D, part 2: inline-editing UI

Your working directory is a git worktree of scriptCast on branch `ui/phase-d2`, created from `dev`. Merged so far:
- A, B, B2, C;
- B3 (gapless playback, 48 kHz output, waveforms);
- L1 and L2 (resizable panes; Settings window on ⌘, in `ui/src/settings.rs`; runtime `t::accent()`);
- **Phase D part 1**: `ui/src/script.rs`, `checkpoint::create_external`, and `scriptcast-export --turns-json/--changes-json`.

## Prohibitions (read first)

- **No git writes.** The orchestrator commits.
- **Never start other agent CLIs** (`codex`, `kimi`, `agy`, `gemini`, `cursor-agent`, `claude`), and never drive other apps (no `osascript`). Do the work yourself.
- **Write only inside this worktree**, plus `~/.cargo`, `ui/target/` and `/tmp/scriptcast-d2/`. Put scratch files in `/tmp/scriptcast-d2/`.
- **The real project is read-only:** `/Users/emmanuel/Documents/Theory/TheOriginalPower/`, including its scripts. The operator edits EP03's script outside the app.
  - Every live save test runs on a copy: `cp -cR` EP03's output folder, and copy the script, `scriptcast.toml` and the voices file, all under `/tmp/scriptcast-d2/`.
  - Point the app at the copy with `SCRIPTCAST_PROJECT`.
- **No Python changes. No synthesis:** never confirm Precision insert. Never launch Live or Logic. Mute captures.
- Lines under 120 characters, readable code, `cargo fmt`, no deleted comments, no colour or size literals outside `theme.rs`.

## Read

1. `plans/Native_UI/plan.md`: Flow 6, steps D4, D4b and D6, and the operator decision at the top of Flow 6.
2. `plans/Native_UI/evidence/phaseD1-findings.md`, especially "Next ideas (handoff)".
3. `ui/src/script.rs`: `ScriptTurns`, `ScriptTurn`, `Changes`, `load_turns`, `load_changes`, `save`, `edited_turns`. The loaders **block**: run them on a worker thread.
4. The designs:
   - **Side panel (default):** `ui/design/screens/03c-alt-b-side-panel-edit/screenshot.png` and `inspector-edit.png`. Figma nodes `23:194`–`23:275` are in `ui/design/page-metadata.xml`.
   - **Table row (option):** `ui/design/screens/03b-alt-a-inline-row-edit/screenshot.png` and `editor.png`, nodes `18:10`–`18:27`.
   - Ignore the read-along row in 03c (word highlighting is Phase E).
5. The existing code: `inspector.rs`, `table.rs`, `timeline.rs`, `main.rs`, `jobs.rs` (the confirmation dialog and the PrecisionInsert preset, which accepts a caller-supplied affected-turn list) and `settings.rs`.

## Build

1. **Loading.**
   - When an episode loads (and after Reload or any job reload), run `load_turns` and `load_changes` on a worker thread. Keep the results in app state: `ScriptTurns`, `Changes`, the `script_sha256` that was loaded, and any error.
   - Without them, the inspector keeps showing FIRST WORDS with "Full text unavailable: <reason>".
2. **Inspector, side-panel mode (default):**
   - "Turn view" / "Edit" tabs at the top of the Selected turn inspector.
   - Turn view shows TRANSCRIPT: the turn's full body text, wrapped, instead of FIRST WORDS.
   - Edit shows the speaker and verification line; TRANSCRIPT as a multi-line `TextEdit` holding `body` (min height `t::EDITOR_MIN_HEIGHT`, a new token of 120, growing with content up to the available height); "Unsaved draft" with the alert-circle icon in danger colour when the draft differs; Save draft (primary); Cancel; and the footnote "Audio remains unchanged. To replace rendered audio, save this draft and confirm synthesis."
   - Add the alert-circle icon. Its SVG is in the Figma file; use the Lucide `circle-alert` path at 14 px in danger colour, the same way `settings-16-text.svg` was added, registered in `ui/design/assets/icons/index.json` and `theme::icon`.
3. **Table-row mode (Settings option):**
   - The editor opens as a block under the selected row, following design 03b: "Inline transcript edit" plus a "Draft" badge, "Turn NNN · Speaker", the field, the footnote, and Cancel / Save draft.
   - With virtualised rows, insert it as an extra-height item after the selected row; it must scroll with the table.
4. **One draft model** shared by both modes: `Draft { turn: usize, text: String, original: String }`.
   - Switching modes keeps the draft.
   - Selecting another turn with an unsaved draft asks to "Keep editing / Discard".
   - Esc cancels the draft only while the editor has focus.
   - ⌘S or ⌘Return saves.
   - While the editor has focus, the existing shortcuts (Space, the arrow keys) must not fire.
5. **Opening the editor:** the E key with a turn selected, a double-click on a table row, or a double-click on a timeline block. Each selects the turn and opens the editor in the current mode.
6. **Save:**
   - Call `script::save` on a worker thread.
   - On success, re-run both loaders, show a status line ("Saved turn NNN to <script name> · checkpoint vNN"), and close the editor.
   - On a hash conflict, show "The script changed on disk. Reload to edit." and keep the draft.
7. **Edited markers.**
   - Use `edited_turns(changes, manifest_len, script_len)`. When it returns rows, show the Verification column as "Edited" in accent and draw a dashed accent outline on that timeline block.
   - When it returns `None`, show one banner above the table: "Turns were added or removed since render. Re-render to sync."
   - Playing an edited turn shows "Audio is from before this edit" in the inspector.
8. **Re-render.** A "Re-render edited turns" button (inspector, Edit tab, shown when any edits exist) opens the existing PrecisionInsert confirmation, with the affected-turn list taken from `Changes` (inserted, modified, and the deleted count). After the job, the existing reload reselects the turn.
9. **Settings.** Add "Editing → Edit transcripts in: Side panel / Table row" to `settings.rs`, persisted in `Preferences` with a `#[serde(default)]` default of side panel.
10. **Checkpoints list.** Show the kind (audio or script edit). A script-edit restore confirmation names the absolute script path it will replace.
11. **Capture flags** for evidence: `--edit` (opens the editor on the selected turn) and `--edit-mode <panel|row>`.

## Tests (required)

- The draft state machine: open, change, cancel, switch mode keeps the draft, and the select-away guard.
- The shortcut suppression rule while the editor has focus.
- Mapping `Changes` to row markers, including the `None` banner case.

## Acceptance

```bash
cd ui && cargo fmt && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs
```

**Live, on the `/tmp` copy only:**
1. Open the copy.
2. Capture `--select 6 --edit --edit-mode panel` and `--edit-mode row`.
3. Save one edit to the copy's script through a test or a debug path (not by clicking).
4. Show that `--changes-json` now lists that turn.
5. Restore the script-edit checkpoint and confirm the bytes are back.

If screenshots time out ("waiting for the screenshot event"), the display may be asleep: record that and move on. Do not work around it by driving other apps.

Save captures as `plans/Native_UI/evidence/phaseD2-*.png`. Write `plans/Native_UI/evidence/phaseD2-findings.md` early and keep it current.
