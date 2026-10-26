# Native UI: full parity with the CLI (brief)

Written 2026-10-07 against `dev` at e442e36 (UI 0.16.4).

**Goal.** Everything you can do with scriptCast's command-line tools, you can do from the app, without a terminal and without editing YAML or markdown by hand.

**Already covered:**
- Browse an episode.
- Play it.
- Edit turn text, inline or in the side panel.
- Find, and find & replace.
- Detect changes.
- Precision insert.
- Regenerate selected turns.
- Verify, then repair.
- Read-along timings.
- Clip transcripts.
- Second opinion (ensemble).
- Review a discrepancy and save the decision.
- Review stats.
- Checkpoints and restore.
- Open in Live and Open in Logic.
- Settings.

What follows is the gap, grouped into phases that can each ship on their own. In the original, each item names the CLI command it wraps. The UI never re-implements Python logic; it calls the tool and reads what the tool writes.

## Ground rules (carry over from every phase so far)

- The UI only runs CLI tools, through `runner.rs` presets. It parses JSON the tools emit. Any new read-only Python mode needs operator approval first, as `--turns-json` did (decision 5).
- Before any write to project files, take a checkpoint in `_backups/ui/`. Script saves stay hash-checked.
- The real project (`TheOriginalPower/`) is read-only for agents. Tests use fixtures.
- Agents make no git writes, start no other agent CLIs, never use `osascript`, and never launch Live or Logic.
- Tooltips only explain why something is disabled, or show information that isn't visible (0.16.2 rule).

## Phase J: Episode lifecycle (render from scratch and the post-render chain)

Today you can only open an episode that is already rendered.

| # | Feature | Wraps | Notes |
|---|---|---|---|
| J1 | **New episode**: pick a script `.md` and an episode id, then do a dry run (parse and manifest) and show the turns before any synthesis | `scriptcast --dry-run` | Shows parser warnings (text before the first header, unknown speakers) as a list you can act on |
| J2 | **Full render**, with a progress bar per turn and Cancel; options sheet for `--gap-ms`, `--overwrite`, `--no-trim`, `--tail-ms`, `--speech-threshold`, `--skip-als`, `--fcpxml`, `--max-turns` | `scriptcast` | Synthesizing: confirm first. Long-running, so it must survive the app being closed (detached job and a log file) or warn |
| J3 | **Relayout** (gap slider, preview of the new episode length) | `scriptcast-relayout` | Reloads the timeline when done. Note that it deletes `.asd` files |
| J4 | **Stitch** to MP3; show the path; reveal in Finder | `scriptcast-stitch` | |
| J5 | **Turn index** (TURN_INDEX.csv) | `scriptcast-turn-index` | |
| J6 | **Retime script** from the measured audio: diff of the timestamps before saving | `scriptcast-retime` | Writes the script, so checkpoint and hash check |
| J7 | **Exports**: regenerate the ALS, FCPXML and Logic dry-run script without re-synthesis | `scriptcast --skip-…` flags / existing export paths | Check the flags support export-only; if not, file it as a Python gap (needs approval) |
| J8 | **One-click pipeline**: Render → Verify → Repair → Relayout → Stitch → Turn index, each step skippable, stopping at the first failure | presets in sequence | The README's six-step loop as a single button |

## Phase K: Script structure editing

Today only the body of a turn can be edited (`script.rs::splice`).

| # | Feature | Notes |
|---|---|---|
| K1 | Insert a turn before or after the selection (choose the speaker from voices.yaml) | Needs an insert/delete splice in `script.rs`; then flows into Detect changes → Precision insert |
| K2 | Delete a turn; move a turn up or down | Same |
| K3 | Change a turn's speaker | Header rewrite; warn if the new speaker uses a different engine |
| K4 | Tag helpers: insert `[pause:Nms]`, `[beat]`, `[emphasis]`, `[tone:…]`, `[clip:id]` (clip picker from the registry) | Plain text inserts at the caret; validate on save |
| K5 | Show the whole script as raw text (read and edit) for anything the structured editor can't do | Hash-checked save |

## Phase L: Project configuration editors

Today this means hand-editing YAML/TOML.

| # | Feature | File | Notes |
|---|---|---|---|
| L1 | **Project picker** (find `scriptcast.toml`; list episodes under `outputs`) | `scriptcast.toml` | Replaces Open… on a bare directory |
| L2 | **Voices editor**: speakers, engine, the fields each engine requires (validated), speed, the reference audio picker with a play button | `voices.yaml` | Validate against the README's required fields per engine |
| L3 | **Clips registry**: list, play, edit start/end/citation/url/note | `archive/clips.yaml` | |
| L4 | **Pronunciations** list (view, edit, delete) | `pronunciations.yaml` | |
| L5 | **Accepted hearings** list, with the same risky-word guard as 0.16.1 | `scriptcast/accepted_hearings.yaml` | Today it can only be added to |
| L6 | **Speaker rates** (view) | `speaker_rates.json` | Read-only is enough at first |
| L7 | `scriptcast.toml` settings | | The keys from the README table, with unknown-key validation |

## Phase M: Voice and clip tools

| # | Feature | Wraps |
|---|---|---|
| M1 | **Make clip**: pick a source file, then a Whisper transcript with word timings; select the first and last words by clicking; fill in the citation and URL; write the registry entry; copy the generated turn | `scriptcast-clip` |
| M2 | **Make reference**: raw recording → reference WAV and exact transcript | `scriptcast-reference` |
| M3 | **Audition**: score candidate references against a speaker; show results side by side with playback | `scriptcast-audition` |
| M4 | **Calibrate pronunciation**: for a heteronym, try spellings, listen to them, and write the winner | `scriptcast-calibrate --write` |
| M5 | **Stress check** for a single heteronym in a turn (from the context menu) | `scriptcast-stress` |
| M6 | **Outliers**: a list of turns whose audio doesn't match the text, which you can click through | `scriptcast-outliers` |

## Phase N: Post-production

| # | Feature | Wraps |
|---|---|---|
| N1 | **Mix**: overlay and per-turn gain editor (mix.yaml), anchored to the script text; render the mix; A/B against the plain stitch | `scriptcast-mix` |
| N2 | **Tape-stop** at the end of a turn (choose from the table) | `scriptcast-tapestop` |
| N3 | **Captions** (generate, preview over playback) | `scriptcast-captions` |
| N4 | **Craft cue sheet** export | `scriptcast-craft` |
| N5 | **Loudness meter**: integrated, short-term and momentary LUFS plus true peak, for the episode and per turn. Turns far from the episode level are flagged; the target defaults to −16 LUFS | SoundCraft `soundcraft-dsp` `LoudnessMeter` (ITU-R BS.1770) |
| N6 | **Per-speaker effects chain**: Channel Strip (trim → gate → compressor → 3-band EQ + HPF), De-Esser, optional light Room Reverb and EQ 7-band. Archive clips get Gate/EQ matching. Stored in the project (`fx.yaml`, or an `fx` field per speaker), with a bypass A/B | SoundCraft `soundcraft-dsp` plugins |
| N7 | **Master chain**: Maximizer to the loudness target, Dither, DC removal, applied to the stitched episode | SoundCraft `soundcraft-dsp` |
| N8 | **`scriptcast-fx` CLI** (Rust, built from the same DSP code). Stitch and Mix call it so exported audio matches what the app plays. Live/Logic sessions keep the dry audio | new Rust binary in `ui/` workspace |

**Phase N prerequisites (from the SoundCraft review, 2026-10-17):**
- **N0a:** add `egui_kittest` headless snapshot tests, so agents verify visuals themselves instead of relying on orchestrator captures.
- **N0b:** adopt `soundcraft-audio-io` (symphonia with isomp4/aac, MP4 atom guards, multi-resolution `Peaks`) for decoding and waveforms. This replaces the ffmpeg fallback and fixes the partial clip-maker waveform.

Pin SoundCraft to a commit (pre-alpha), take only `audio-io`, `dsp` and `time`, which have no SoundCraft dependencies, and keep their LICENSE/NOTICE. Do not take `mix`, `playback` or `ui-egui`; they pull in plugin hosts, MIDI and video.

## Phase O: Video (decide scope first)

`scriptcast-video` (Remotion cards), `scriptcast-shotspec` (shot lists to timing) and `scriptcast-tapefx` (video tape FX) run a separate Remotion workspace in `video/`.

**Operator decision needed:** should the native app drive these (a job runner plus a shot-list view on the timeline), or should video stay CLI-only? It's out of parity scope until that's decided.

## Phase P: App completeness (not CLI-driven)

| # | Feature |
|---|---|
| P1 | Menu bar with every action and its shortcut (File, Edit, Episode, Jobs, View) |
| P2 | Undo/redo for edits, beyond checkpoints |
| P3 | Job queue and history: several queued jobs; a log viewer for past runs |
| P4 | First-run setup: find or create the venvs, check `ffmpeg`, Whisper and `misaki`, and explain what's missing |
| P5 | Notifications when a long job finishes while the app is in the background |
| P6 | Proper bundle: an icon, a signed and notarized build, a version shown in About (Settings already shows it), and auto-reload when the episode files change on disk |
| P7 | Open the remaining review follow-up: discrepancy cards show the surrounding phrase |

## Suggested order and routing

1. **J (lifecycle), then K (structure).** These have the highest everyday value. Without J you still need the terminal to make an episode at all. Use Codex `gpt-6-astra` at medium, one phase per run.
2. **L (config editors).** Repetitive form work: Cursor `auto`.
3. **M, then N.** Each item is a wrapper plus a small view: Codex `gpt-5.6-sol` medium or Cursor, in one-item chunks.
4. **P**, interleaved as small chunks; P4 before any release.
5. **O** only after the operator decides.

Each phase needs a Figma pass for its new screens first: New episode sheet, pipeline progress, the voices and clips editors, the clip maker, and the mix editor. Use one consolidated Figma prompt per phase.

## Operator decisions (2026-10-07)

1. **Quitting during a long job:** a popup asks whether to keep rendering. "Keep rendering" detaches the job; it carries on with a log file, and the app reattaches on next launch. "Stop" cancels it cleanly. Applies to J2, J8 and any synthesizing job.
2. **Video (Phase O):** in scope. It follows the same pattern: a job runner for `scriptcast-video`, `scriptcast-shotspec` and `scriptcast-tapefx`, plus a shot-list view on the timeline. It needs its own Figma pass.
3. **Python additions:** approved. Add the minimum to the CLI, each with tests:
   - an export-only mode (regenerate ALS, FCPXML or Logic script without synthesis);
   - insert, delete and move turn operations through `scriptcast-export`;
   - a speaker change through `scriptcast-export`.

   The UI calls these and does not splice structure itself.
4. **Scope:** everything listed is used. No phase drops out, so parity means J through P in full.
