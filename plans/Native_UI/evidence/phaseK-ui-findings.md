# Phase K UI findings

Status: Rust implementation, automated checks, and visual round 1 fixes are complete.
Version: 0.18.0. No git writes, no Python edits, and no synthesis.
Only the Contract section of `plans/Native_UI/briefs/phase-k-py.md` was read.

## Files changed

- `ui/src/structure.rs`: insert, delete, move, and set_speaker ops, speaker/clip registry, exit 3 and exit 1 messages, archive-clip Save gate.
- `ui/src/tags.rs`: local tag spans using the CLI rules, save blocking for malformed pause and clip tags, pill `LayoutJob` with brackets left in the text.
- `ui/src/tag_editor.rs`: shared toolbar (Pause 800 ms, Beat, Emphasis, Tone, Clip picker, Raw script) and caret insertion for inline, side panel, and new-turn drafts.
- `ui/src/phase_k.rs`: turn menu, new-turn draft, script-order table after a structural edit, delete confirmation, retime preview window, capture windows.
- `ui/src/retime.rs`: `scriptcast-retime` preview and Apply.
- `ui/src/script.rs`: `--edit-json` after a checkpoint, `--tags-json`, `--speakers-json`. Mode-specific flags only: `--script` on turns, changes, tags, and edit; `--voices` on speakers and edit.
- `ui/src/editing.rs`: toolbar state, body editors call the tag editor, Save runs `--tags-json` and keeps the report.
- `ui/src/table.rs`, `ui/src/timeline.rs`: row and block context menus. Timeline blocks map back to the script index.
- `ui/src/jobs.rs`: Jobs button "Retime script". Detect changes clears the structure-changed state.
- `ui/src/inspector.rs`: authority tag lines for the selected turn.
- `ui/src/main.rs`, `ui/src/capture.rs`: shortcuts, script-order layout, capture flags.
- `ui/src/audio.rs`: clip-range preview on the existing output, including mute.
- `ui/src/theme.rs`: tag field, picker, and pill tokens. No new colour literals outside this file.
- `ui/src/lib.rs`: module exports.
- `ui/Cargo.toml`, `ui/Cargo.lock`: 0.18.0.
- `ui/tests/phase_k.rs`: the acceptance tests below.
- `plans/Native_UI/evidence/capture-phaseK.sh`: muted fixture captures. Executable.

## Tests

From `ui/`, cargo output in `/tmp/k-ui-fmt.log`, `/tmp/k-ui-clippy.log`, and `/tmp/k-ui.log`:

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: 202 passed, 0 failed, 6 ignored.
- `git diff --check`: passed. Rust sources have no line over 120 characters.

`ui/tests/phase_k.rs` has 5 tests:

1. `operations_serialize_the_exact_contract` — ops JSON for insert, delete, move, and set_speaker; exit 3 and exit 1 text.
2. `real_python_structural_round_trips_and_stale_atomic_failure` — real `scriptcast-export` on a `/tmp/scriptcast-k` fixture copy: move, delete, set_speaker, insert, each followed by a turns reload; stale hash; a rejected speaker leaves the file unchanged.
3. `local_tags_agree_with_real_python_and_archive_gating` — local spans match `--tags-json` on the fixture bodies; archive Save gating.
4. `layouter_keeps_every_character_and_paints_backgrounds` — pill sections keep the original characters; danger background is painted; caret hit-testing matches character indexes. The egui 0.36 layouter stop condition was not hit.
5. `retime_preview_is_read_only_apply_checks_hash_and_has_checkpoint` — preview leaves the script unchanged, Apply checkpoints then writes the reviewed bytes, a second Apply reports the hash mismatch, and diff parsing drops unchanged rows.

## Deviations

- Retime's default (no `--apply`) writes `--map-out` and does not rewrite the script. Preview still passes `--apply` on a temporary copy so Apply can commit those exact bytes. The episode script is unchanged until Apply, which checkpoints and checks the hash again.
- The speaker row is a coloured `●`, the name, and the engine, with `Current speaker ✓` on the active id.
- After a structural edit the turn table becomes a script-order list. The timeline keeps real waveforms.
- "Unsaved · structure changed" is on the selected script row and in the side panel. "Detect changes → Precision insert" appears only after that save.
- Disabled Move, Delete, and Change speaker items include the reason in the item label.
- While typing, `[pause:abc]` is shown as "[pause:abc] is not a number of milliseconds". The stored local message still matches `--tags-json` ("unrecognised tag; it will be spoken aloud") and still blocks Save.
- A body Save stores the `--tags-json` report and the selected-turn inspector shows those lines.

## Unverified

No window was opened, so none of `phaseK-*.png` exist. Menu clicks, the clip picker popover, raw-script pills, and audible clip preview were not exercised. Preview uses the muted output path; `capture-phaseK.sh` passes `--mute` on every launch.

From the worktree root, after `cargo build --manifest-path ui/Cargo.toml`:

```sh
sh plans/Native_UI/evidence/capture-phaseK.sh
```

The script copies `ui/tests/fixtures` under `/tmp/scriptcast-k/capture` and writes:

- `phaseK-K1.png` — `--context-menu 1` with the speaker list open
- `phaseK-K2.png` — `--new-turn-after 1`
- `phaseK-K2-archive.png` — same, `--new-turn-speaker archive`
- `phaseK-K3.png` — `--tag-toolbar 1` panel editor
- `phaseK-K3-row.png` — same, `--edit-mode row`
- `phaseK-K3-clips.png` — `--clip-picker`
- `phaseK-K3-raw.png` — `--raw-script`

## Visual round 1

The orchestrator captured `outputs/ATO_EP04` with `SCRIPTCAST_PROJECT=/tmp/scriptcast-k/proj`.

Cause of the missing panel screenshot: `--new-turn-after` switched the turn table to a non-virtualized list of every script turn and put the draft at the insertion index. That list opens scrolled to turn 001, so the draft row sat below the fold (the "Episode text" view with no draft). The same frame also built the side-panel editor. A local rerun of `--new-turn-after 14 --new-turn-speaker aisha --edit-mode panel` exited 0 in about 8s and wrote a PNG of that wrong layout; on a slower pass the 339-row list can miss the 20s capture deadline, which leaves no PNG. There was no panic in that rerun.

Fixes:

- The context menu is an egui popup anchored to the row (`Popup::context_menu`), about 260 px wide. The caption uses the table's turn number and speaker, for example "TURN 014 · TOUSSAINT". Shortcuts are right-aligned text: Opt+Cmd+Up, Opt+Cmd+Down, Cmd+Up, Cmd+Down, Delete. Change speaker opens a submenu to the right; the current speaker is checked. Delete stays in the danger colour. Capture no longer opens a second menu on the timeline block.
- `--context-menu`, `--new-turn-after`, and `--tag-toolbar` select `turn.index`, and the menu caption uses that same number.
- A new turn stays on the waveform table. Row mode inserts a highlighted draft between the neighbours ("New · unsaved", speaker dot and engine, estimated "MM:SS est.", placeholder, Cancel, Save Cmd+Return). Panel mode shows the same form in the side panel ("After 014 / before 015", SPEAKER, TIME · EDITABLE, TRANSCRIPT). The estimate is the midpoint of the neighbouring script headers, and Save still sends a null timestamp while the field ends in "est.".
- The tag toolbar is one row: Pause, ms, Beat, Emphasis, Tone, text, Clip. Raw script sits on its own line under the body. The body editor asks for 6 rows. The clip picker is 300 px under the Clip button; a capture opens it with the first row selected, highlighted, and "Insert selected <id>" enabled.

`capture-phaseK.sh` takes `EPISODE_DIR` as `$1` and otherwise copies the small fixture.

Checks after this round, from `ui/`, with `SCRIPTCAST_PROJECT` unset: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` passed. `cargo test` passed: 202 tests, 6 ignored. Logs: `/tmp/k-ui-fmt.log`, `/tmp/k-ui-clippy.log`, `/tmp/k-ui.log`.
