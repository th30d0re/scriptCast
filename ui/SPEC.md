# scriptCast UI — Spec

A lightweight native desktop window for scriptCast, written in Rust with egui (eframe).

## Goals

- Start instantly and stay small: one binary, no webview, no Python in the UI process.
- Browse a rendered episode: every turn, its speaker, timing and status.
- Audition any turn without opening a DAW.
- Launch the existing Python tools and stream their output.

## Non-goals

- Reimplementing synthesis, Whisper verification, repair, mixing or exporters in Rust. Python remains the source of truth.
- Editing the engine's output files. The UI treats them as read-only; only the Python tools write them.
- Cross-platform support. macOS on Apple Silicon only, matching scriptCast.

## Architecture

```
ui/
  Cargo.toml
  SPEC.md
  src/
    main.rs      eframe app, window layout, input handling
    episode.rs   load and model an episode output directory
    audio.rs     per-turn playback (rodio)
    runner.rs    spawn scriptcast CLIs, stream stdout/stderr
```

The UI is a thin front-end over files on disk plus subprocesses. It links no scriptCast Python code.

## Data sources

An episode is an output directory (for example `outputs/ATO_EP03_five_levels/`). Verified layout:

| File | Used for |
|---|---|
| `episode_manifest.json` | Episode id, engine, sample rate, speakers, and every turn with `turn_index`, `turn_id`, `speaker_id`, `source_timestamp`, `start_ms`, `end_ms` and `segments[]` (`segment_wav` relative path, `duration_ms`, `speech_duration_ms`, `gap_after_ms`, `checksum`). |
| `render_state.json` | Per-turn fingerprints (`text_hash`, `voice_hash`, `speech_text_hash`), `source_file`, `source_hash`, `rendered_at`. |
| `TURN_INDEX.csv` | `first_words` preview for each turn. |
| `stress.json` | Heteronym stress-check results, when present (later). |
| `Samples/Processed/<speaker>/<turn_id>_chunk_NNNN.wav` | Audio for playback. |

Rules:
- Wav paths in the manifest are relative to the episode directory.
- Missing optional files (`TURN_INDEX.csv`, `stress.json`) are not errors; the matching column or overlay is simply empty.
- A missing or unparsable `episode_manifest.json` is an error shown in the window, never a silent empty episode.

## v1 features

1. **Open episode.** Native folder picker, plus `scriptcast-ui <episode_dir>` on the command line. Remember the last directory.
2. **Header.** Episode id, engine, sample rate, turn count, rendered-at time.
3. **Timeline strip.** One block per turn, width proportional to duration, coloured by speaker. Click selects a turn. The selected turn and the playing turn are highlighted.
4. **Turn table.** Virtualised (392+ turns must scroll smoothly). Columns: index, time, speaker, first words, duration. Filter by speaker; text search over first words.
5. **Playback.** Space plays or stops the selected turn; Up and Down move the selection; optional auto-advance to the next turn.
6. **Run panel.** Buttons for the existing commands against the current episode: render, verify, repair, mix. Output streams live into a log pane. One job at a time, with a Cancel button.
7. **Refresh.** Reload from disk, and automatically after a job finishes.

## Running jobs

- The UI invokes the installed console scripts (`scriptcast`, `scriptcast-verify`, `scriptcast-repair`, `scriptcast-mix`) by name from `PATH`. A setting overrides the executable path, so the UI can target the Documents install or a venv for `dev`.
- Commands run with the episode's project directory as the working directory.
- stdout and stderr are streamed line by line on a background thread; the UI thread never blocks.
- Render never overwrites by default. Anything that spends ElevenLabs credits or overwrites output needs an explicit confirmation dialog.
- A non-zero exit code is shown in the log pane and on the job status line.

## Performance targets

- Cold start to first frame: under 300 ms.
- Load a 400-turn manifest: under 50 ms.
- Scroll and repaint at 60 fps with all turns loaded.
- Idle: repaint only on input or job output, so no CPU use when nothing happens.

## Decisions and open questions

- Audio backend: rodio (wav decoding included in its defaults).
- Folder dialog: rfd.
- Which Python install the UI runs by default: `PATH` first, with a settings override. Open: should the UI auto-detect a `.venv` next to the project?
- Open: script text is not in the manifest (only first words). Showing full turn text means parsing the markdown script in `source_file`. Defer to v2.

## v2 ideas

- Waveform per turn.
- Stale-turn detection against the script hash, mirroring `--detect-changes`.
- Inline script editing with regenerate for a single turn.
- Verify and repair results overlaid on turns, from `stress.json`.
- Drag to retime gaps.
- Editor for `voices.yaml` and `pronunciations.yaml`.

## Acceptance for v1

- `cargo build --release` succeeds with no warnings.
- Opening `outputs/ATO_EP03_five_levels` shows 392 turns, four speakers, and a timeline whose total length matches the manifest.
- Playing a turn plays its processed wav; selecting another turn while playing switches cleanly.
- Running a verify job streams output and ends with the exit status shown.
- Running with no episode open, or on a broken manifest, shows an error rather than crashing.
