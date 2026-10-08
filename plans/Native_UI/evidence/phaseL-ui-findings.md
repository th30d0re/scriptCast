# Phase L UI findings

## Status

Finished 2026-10-08. No GUI launch, synthesis, Python edits, or git writes.
`/Users/emmanuel/Documents/Theory/TheOriginalPower/` was not touched. The real copy is `/tmp/scriptcast-k/proj`.

Model: Composer.

The three contract blockers in the earlier section are resolved on this branch. The UI now follows that resolution.

## Contract resolution used

- `show --json` includes a complete `fields` object for every speaker, clip, and pronunciation entry. Pronunciation entries also keep `readings` (the same object).
- `set_speaker` and `set_clip` send that complete `fields` mapping with the draft edits applied. `set_entry` sends the complete `readings` mapping. Unchanged keys stay byte-identical in the tool. This replaces the brief's "only changed keys" line for those three ops.
- TOML still sends one `set` op per changed key.
- `scriptcast-project init --root PATH` creates the comment-header `scriptcast.toml`. The picker button calls that command. The UI does not write the file itself.

## Audit (before this pass) and what changed

| Item | Was | Now |
|---|---|---|
| L1 recent projects, folder picker, episode table, async load, double-click, summary card, header | Present | Unchanged |
| L1 "Create scriptcast.toml here" | Disabled, with a note that the tool had no create op | Calls `init --root` on click only |
| L2 voices form, live required errors, rename warning, engine-change confirmation | Present | Unchanged. Reference-audio and TOML Choose… store a project-relative path when the file is inside the project |
| L2/L3 save of speakers and clips | Refused: omitted keys were treated as destructive | `voice_ops` / `clip_ops` send the full mapping. Clip start, end, citation, origin URL, and content note edit `fields` |
| L3 waveform, search, missing source, disabled Play, disabled Register clip | Present | Play's disabled reason is "Missing source" |
| L4 hearings, pronunciations, read-only rates, Settings → Project | Present | Deleting the last hearing for a word drops that row. Project data tabs are the five in the brief; TOML stays in Settings → Project |
| Acceptance fixture | Speaker `fields` only; clips and pronunciations had no `fields` | Recaptured |
| Acceptance one-line edit | Asserted that a speed-only op fails validation and writes nothing | Full `fields` plus `speed: 1.04` inserts one line |
| Capture flags and `capture-phaseL.sh` | Present | Unchanged |
| Version | `ui/Cargo.toml` is 0.19.0 | Unchanged |

## Files changed

- `ui/tests/fixtures/phase_l/show.json` — recaptured with `/tmp/scriptcast-l/venv/bin/python -m scriptcast.tools.project_config show --json --root /tmp/scriptcast-k/proj` and `PYTHONPATH` set to this worktree. 18 speakers, 32 clips, 5 pronunciations. Every speaker, clip, and pronunciation entry has `fields`.
- `ui/src/project.rs` — complete `set_speaker` / `set_clip` ops, `init_project`.
- `ui/src/phase_l.rs` — init button, clip `fields` editing, relative Choose… paths, hearing-row cleanup, five project-data tabs.
- `ui/tests/phase_l.rs` — fixture `fields` checks, complete-mapping ops, real-tool one-line diff and `init`.
- This findings file.

## Tests

`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean.
`cargo test` wrote `/tmp/l-ui.log` and exited 0. `tests/phase_l.rs`: 11 passed, including `real_tool_speed_edit_changes_exactly_one_line` (Aisha `speed: 1.04` is the only inserted line; a stale hash exits as `ToolError::Stale` and leaves the file unchanged; `init` writes only the comment header).

The log's ignored tests are the existing live-copy tests (`phase_d2`, `phase_i`, and two others). They were not run.

## Deviations

- Speaker and clip saves send the complete `fields` object. Pronunciation saves send the complete `readings` object. The brief's "only changed keys" line applies to TOML `set` ops only.
- "Create scriptcast.toml here" uses `init`, which did not exist when the brief was written.
- Project settings are the Settings → Project section (`--settings-project`), not a sixth tab on the project data window.
- Validation from exit 1 is a banner of `op[i]:` / `file:` lines. Live required-field errors stay under the fields. Exit 3 shows "The file changed on disk. Reload?" plus a reload button.

## Visual round 1

Captured against `SCRIPTCAST_PROJECT=/tmp/scriptcast-k/proj` and compared with `ui/design/L1`–`L4`. No git writes, Python edits, agent CLIs, or GUI launch from this pass.

- The project picker paints `BG_WINDOW`, the app header (logo, project name, root, secondary Project settings, primary New episode…), bordered columns, selected recent-project cards, and status chips (Rendered in `STATUS_PASSED`, Interrupted render in `STATUS_FAILED`, Not rendered in `TEXT_MUTED`).
- Last rendered is local time on two lines (`05 Oct 2026` / `10:32`). Review dates stay on the UTC calendar date (`05 Oct 2026`).
- Episode rows match `episode_manifest.json` `source_file` resolved from the project root. `render_state.json` still points at `/Users/emmanuel/Documents/Theory/TheOriginalPower/...`, which is why `ATO_EP04_what_power_is.md` was listed again as Not rendered.
- The summary card and the double-click helper sit under the episode table.
- Project data is a full-window view: header with Close (Esc does the same), accent tab pills, list plus detail, footer with the reason on the left and Revert plus primary `Save <file name>` on the right.
- Voices use the mock labels, engine chips, a right-aligned turn count, `Speakers  N · M invalid`, a full-width speed slider (`0.5` / `1.5`, `1.0 · default`), and reference audio as path + Choose… + Play + duration on one row.
- Clip source cells are one truncated line at `PROJECT_ROW_HEIGHT` (44). The full path is in the detail panel. Search and `+ Register clip…` share a row.
- When rodio cannot decode a clip source, the waveform and Play use `ffmpeg -v error -ss <start> -t <len> -i <src> -f wav -ac 1 -ar 48000 -` in memory. The Phase K clip picker uses the same fallback inside `AudioOut::preview_range`. A missing ffmpeg is the visible note `ffmpeg is not available`.
- "Used in" reads the open episode's turn `first_words` (the loaded turn text, which includes `[clip:…]`). The line is `019  ATO_EP04 · Jala Chantelle`. On this copy, turn 019 is `jalachantelle_2026a` and turn 021 is `jalachantelle_2026b`.
- Accepted hearings are a headed table. Delete is a danger-outline button, right-aligned.
- Settings → Project puts each path and its Choose… button on one row.

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` are green. The log is `/tmp/l-ui-visual.log`.

## Visual round 2

`--project-data clips --select jalachantelle_2026a` stayed busy because `load_project_media` started a new decode on every frame while the previous one was in flight, and again after an error (`media` stayed empty, so the cache check never hit). A decode is now one background job per `(source, start, end)` — start and end rounded to milliseconds, or the whole file for reference audio. The result, including a decode error, is cached. The worker requests one repaint when it finishes. The clip table does not decode; only the selected clip's detail panel does. `clip_decode_is_requested_once_across_frames` paints that selection four times and expects a single request.

Voices: the speaker list and the form are each a bordered panel. A speaker row keeps the name on the left; the engine chip then the turn count sit on the right. Reference audio is one row: path field, Choose…, Play, duration. The speed slider spans the form, with `1.0 · default` right-aligned above it and `0.5` / `1.5` under the ends. Rename… is on the right of the Id row.

Recent project cards take the left column width. The status chip is the width of its label plus padding, not the source column.

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` are green. The log is `/tmp/l-ui-visual2.log`.

## Visual round 3

Voices tab only. Round 2 built each speaker row as a vertical child of the allocated rect, so the colour dot, the name, and the engine chip landed on three successive lines. The chip and turn count painted one row below their name, and the selection fill stayed on the allocated rect.

Each speaker is now one `allocate_exact_size` of `PROJECT_ROW_HEIGHT`. The dot, name, chip, and turn count are painted inside that rect. Click and the selection fill use that same rect. `speaker_row_chip_overlaps_name` paints the voices tab and requires every name rect to overlap its chip rect vertically.

The speed label line is `Speed` on the left and `1.0 · default` on the right. `slider_width` is the form width minus the value box and the item gap, then restored. `0.5` and `1.5` sit under the track ends.

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` are green. The log is `/tmp/l-ui-visual3.log`.

## Unverified

The GUI was not launched. Screenshot comparison against `ui/design/L1-project-picker.png`, `L2-voices.png`, `L3-clips.png`, and `L4-small-lists.png` is for the orchestrator via `plans/Native_UI/evidence/capture-phaseL.sh`.

That script already calls:

- `--project-picker [ROOT]`
- `--project-data <tab> [--select id]` (`voices --select aisha`, `clips`, `accepted-hearings`, `pronunciations`, `speaker-rates`)
- `--settings-project`

Captures are muted. "Used in" lists clip turns after the open episode's script loader fills `editing.turns`.
