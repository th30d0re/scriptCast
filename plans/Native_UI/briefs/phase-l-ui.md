# Phase L (UI): project picker and project data editors

Working directory: a git worktree of scriptCast on branch `ui/phase-l`, from `dev` (app 0.18.0). The Phase L Python tool `scriptcast-project` is merged on this branch.

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs**, never use `osascript`, and never launch Live or Logic.
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.** A realistic copy exists at `/tmp/scriptcast-k/proj` (`SCRIPTCAST_PROJECT`). For write tests, copy it under `/tmp/scriptcast-l/`.
- **No Python changes**, and no synthesis.
- Lines under 120 characters; `cargo fmt`; delete no comments; no colour or size literals outside `theme.rs`.
- **Tooltips:** only for a disabled reason, or for information that is not visible.
- Redirect cargo output: `cargo test > /tmp/l-ui.log 2>&1; tail -30 /tmp/l-ui.log`.
- **Mute** captures.
- **The UI never writes these files itself.** Every write goes through `scriptcast-project edit`.

## Design

Match `ui/design/L1-project-picker.png`, `L2-voices.png`, `L3-clips.png` and `L4-small-lists.png` with the existing tokens.

**Ignore in the mock-ups:**
- the bottom annotation strips;
- the ▶ glyph waveform in L3 (draw the real one);
- L4's "illustrative config keys". Show the **real** `scriptcast.toml` keys from `show --json`: `voices`, `pronunciations`, `speaker_rates`, `clips`, `clip_sources`, `references`, `scripts`, `outputs`, `omnivoice_python`, `tail_ms`;
- L4's Pronunciations columns. Use the real `readings` shape from `show --json`.

## Contract: `scriptcast-project`

The authority is `plans/Native_UI/briefs/phase-l-py.md`, section "Contract" (read only that section), plus the deviation in `plans/Native_UI/evidence/phaseL-py-findings.md`: `toml` has a `problems` list.
- `show --json [--root]`
- `validate --json`
- `edit --ops OPS --expect-sha256 FILE=HEX ... [--root]`. Exit 3 means stale; exit 1 means validation failed (`op[i]: message` / `file: message`). On success, it prints the `show` JSON.
- Edits keep comments and the quoting of unchanged keys. Make sure that holds by sending only the changed keys in `fields`.

## Established facts (`file:line` on dev)

- `main.rs:72` `recent: Vec<RecentEpisode>` (preferences, truncated to 5 at :512). Screen 01 is the "No episode open" state. Settings live in `settings.rs`.
- The Phase J dry-run, New episode and job reattach code is in `phase_j.rs`, `new_episode.rs` and `detached.rs`. An `_jobs/*.json` without a `done` line marks an episode as "Interrupted render".
- Command resolution is in `jobs.rs`/`runner.rs`. Resolve `scriptcast-project` the same way as the other tools.
- `ruamel.yaml` may be missing from a project venv. If `scriptcast-project` fails with `ModuleNotFoundError`, show a visible state: "scriptcast-project needs ruamel.yaml in <python path>". **Never** install anything.

## Build

**L1. Project picker** (replaces screen 01, and opens with ⌘⇧O and File → Open Project…)
1. Left column:
   - **recent projects**, stored in preferences next to `recent`, with name, root and last opened;
   - **Open project folder…**;
   - for a folder with no `scriptcast.toml`, the "Not a scriptCast project" card with "Create scriptcast.toml here". That writes a minimal `scriptcast.toml` holding only a comment header, and only on click.
2. Right column: the episodes table from `<outputs>/*/episode_manifest.json`, plus the scripts under `scripts` that have no output yet. Columns:
   - episode id;
   - script file;
   - turns;
   - duration;
   - last rendered;
   - the verification summary, from `stress.json` and the existing loader;
   - a status chip: Rendered, Interrupted render (from `_jobs`) or Not rendered.

   The table loads asynchronously, off the UI thread.
3. Behaviour:
   - **Double-click** opens the episode, or opens New episode prefilled for a "Not rendered" script.
   - **The selected row** shows a summary card below the table.
   - **Header:** the project name and root, plus **Project settings** (which opens Project data) and **New episode…**.

**L2–L4. Project data window** (Episode or File → Project Data…, ⌘;). It is one window with tabs: Voices · Clips · Pronunciations · Accepted hearings · Speaker rates.

The rules for every tab:
- The data comes from `show --json`.
- Edits are held as a local draft, with "Unsaved changes" in the header.
- **Save** sends one `edit` call for that tab's file, with the sha256 of the file as loaded.
- **Revert** discards the draft.
- After an exit 3, show "The file changed on disk. Reload?". After an exit 1, show each message inline under its field, or as a banner.
- **Closing with unsaved changes** asks whether to save, discard or cancel.

**L2. Voices**
1. A speaker list with a colour dot, name, engine chip, turn count in the current episode, and a red dot when the speaker has problems; "+ Add speaker".
2. The form fields depend on the engine:
   - **OmniVoice:**
     - `reference_audio`: picker, Play, duration;
     - `reference_text`: required, multiline;
     - the expressive pair, collapsed;
     - `speed`: slider 0.5–1.5, default 1.0;
     - `character_profile`;
   - **Kokoro:** `kokoro_voice`, `lang_code`, `speed`;
   - **Chatterbox:** `reference_audio`, `exaggeration`, `temperature`, `cfg_weight`;
   - **ElevenLabs:** `elevenlabs_voice_id`;
   - **Archive:** Name only, plus the note shown in the mock-up.
3. Required-field errors show inline, live, and Save is disabled with a visible reason in the footer. The footer note reads "Comments in the file are kept."
4. **Rename…** runs `rename_speaker`, and warns that scripts use the speaker's display name, not the id.
5. Changing the engine keeps the fields of the new engine and drops the others, but only after a confirmation that lists the fields being removed.

**L3. Clips**
1. A searchable table: id, source file (red "Missing source" when `source_exists` is false), start, end, duration, citation, and Play (disabled when the source is missing).
2. **Detail:**
   - a waveform of the source, made by decoding it through the existing audio/waveform code. If it cannot be decoded, show a visible note instead;
   - the start/end range drawn with draggable handles, linked to the start/end fields;
   - Citation, Origin URL and Content note;
   - "Used in:", listing the turns of the current episode that use the clip.
3. "+ Register clip…" is disabled with the reason "Coming in Phase M (clip maker)".

**L4. Small lists**
1. **Accepted hearings:**
   - each row is word → heard, the review date, a "Common word" chip when `risky`, and Delete;
   - Delete removes that hearing (op `delete`);
   - the muted note: "These hearings count as a match in every turn of the project."
2. **Pronunciations:** a list with add, edit and delete (`set_entry` / `delete_entry`).
3. **Speaker rates:** read-only.
4. **Settings → Project section:** the real `scriptcast.toml` keys as labelled fields, with Choose… for paths and a number field for `tail_ms`. Save sends `toml` `set` ops.

## Acceptance

1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` must all be green. The new tests cover:
   - parsing `show --json`, using a fixture captured from the real copy (run `scriptcast-project show --json --root /tmp/scriptcast-k/proj` once and save it under `ui/tests/fixtures/phase_l/`);
   - op building per tab, sending only the changed keys;
   - exit 3 and exit 1 handling;
   - engine field sets and live required-field validation;
   - project discovery and episode status classification;
   - an end-to-end edit using the **real** tool on a `/tmp` copy: change one speaker's speed, then assert that the file diff is exactly one line.

   Use `/tmp/scriptcast-l/venv/bin/python`, which has `ruamel.yaml`, with `PYTHONPATH=<worktree>` for this test, and skip it with a message if that venv is absent.
2. Add capture flags:
   - `--project-picker [ROOT]`;
   - `--project-data <tab> [--select <id>]`;
   - `--settings-project`.

   Add `plans/Native_UI/evidence/capture-phaseL.sh`. You cannot launch the GUI, so the orchestrator runs it.
3. Bump the version to 0.19.0.

## Findings file

Write `plans/Native_UI/evidence/phaseL-ui-findings.md` early: the files changed, the tests, deviations, and what is unverified.

## Later work (2026-10-18) — not part of the contract above

The Voices form above is shipped. `character_profile` stays the description field it already is. It is not sent as a delivery instruct.

When the Rust instruct engine from `briefs/phase-k-py.md` lands, add its fields to the engine switch in L2, with the same live required-field check the other engines use. The reference row stays one clip, with Play. A second sample waits until that engine accepts more than one.
