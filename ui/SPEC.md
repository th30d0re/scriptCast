# scriptCast UI — Spec

A lightweight native desktop window for scriptCast, written in Rust with egui (eframe).

## Goals

- Start instantly and stay small: one binary, no webview, no Python in the UI process.
- Browse a rendered episode: every turn, its speaker, timing and status.
- Audition any turn without opening a DAW.
- Launch the existing Python tools and stream their output.

## Non-goals

- Synthesis, Whisper verification, repair, mixing, and export inside the egui process. The app calls tools. Python remains the parser and the existing Kokoro, Chatterbox, OmniVoice, ElevenLabs, and archive engines.
- Vendoring outside Python into those tools. Engines and DSP added after 2026-10-18 are Rust binaries the tools call (`plans/Native_UI/parity-brief.md`: J9 chunking, K4 `[tone]` instruct, N6–N8 `soundcraft-dsp`).
- Editing the engine's output files. The UI treats them as read-only except for checkpoints and explicitly confirmed Restore; other writes belong to the Python tools.
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
| `stress.json` | Verification joined by turn_id: score, expected/heard text, repetition and stress checks. |
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
5. **Playback.** Space plays or pauses the selected turn; Up and Down move the selection; optional auto-advance to the next turn.
6. **Run panel.** Buttons for the existing commands against the current episode: Detect changes, Precision insert, Regenerate selected, Verify, Repair. Output streams live into a log pane. One job at a time, with a Cancel button.
7. **Refresh.** Reload from disk, and automatically after a job finishes.

## Running jobs

- Jobs opens the runner from the filter bar or Cmd+J. Shift-click selects a visible range; Cmd-click toggles additional turns.
- Each command resolves independently: Settings folder → alphabetical project `.venv*/bin` folders containing that command → login-shell PATH → dev checkout. The existing `preferences.cli_dir` setting is the bin directory; no separate folder setting exists.
- Login PATH loads once on a background thread with a two-second timeout and a process-PATH fallback. All subprocesses receive it. Checkout modules come from `[project.scripts]`; a project venv must import the entry module before it is selected as interpreter.
- Jobs run in the project root with stdin null. The only renderer modes are Detect changes, Precision insert, and Regenerate selected; a bare render and the overwrite flag are never emitted.
- Turn-targeted synthesis requires an unchanged script hash. Missing script, project configuration, or voices key/file disables the dependent preset with an explanation.
- Synthesis and replacement of existing verification output show the exact shell-quoted command and scope in a confirmation dialog. Precision insert accepts an optional caller-supplied affected-turn list for Phase D.
- After confirmation, an explicit file plan is copied to `_backups/ui/vNN-<timestamp>-<preset>/`. `checkpoint.json` is written last; incomplete checkpoints are ignored. Failure prevents process start.
- Checkpoints list version, time, preset, turns, file count and size. Restore first checkpoints current files as `pre-restore`, then copies via same-directory temporary files and rename. The app never deletes checkpoints or episode files. Newly created files absent from a checkpoint remain after Restore.
- stdout and stderr stream on separate reader threads; the log retains 5,000 lines. Completion waits for both streams to drain, reports the exit code, and reloads. Cancel and closing the window kill the process group, including subprocesses.
- Verification shows real report contents; missing and stale IDs are never converted to passes. Failed-only review shows timeline markers and last-verification time.
- Mix is deferred: its required `--config` and `--asr` cannot be derived from episode metadata.
- Self-capture: `--inspector selected|verification|jobs`; `--confirm <preset>` opens without confirming; `--run detect-changes|verify` runs and captures after exit. All synthesis is refused by `--run`. `--running-screenshot PATH` additionally saves an active-job image. Captures are muted.
- Visual specification: [design/DESIGN.md](design/DESIGN.md). The UI uses embedded OpenDyslexic and OpenDyslexicMono fonts.

## Performance targets

- Playback requests a repaint every 250 ms on built-in MacBook speakers and 50 ms on other outputs. Table follow scrolling is immediate on built-in speakers to avoid animation bursts between turns.
- Cold start to first frame: under 300 ms.
- Load a 400-turn manifest: under 50 ms.
- Scroll and repaint at 60 fps with all turns loaded.
- Idle: repaint only on input or job output, so no CPU use when nothing happens.

## Decisions and open questions

- Audio backend: rodio (wav decoding included in its defaults).
- Folder dialog: rfd.
- Command source is visible in tooltips and the runner; project virtualenv discovery precedes login PATH.
- Open: script text is not in the manifest (only first words). Showing full turn text means parsing the markdown script in `source_file`. Defer to v2.

## v2 ideas

- Waveform per turn.
- Stale-turn detection against the script hash, mirroring `--detect-changes`.
- Inline script editing with regenerate for a single turn: now planned as Phase D (`plans/Native_UI/plan.md`, Flow 6).
- Drag to retime gaps.
- Editor for `voices.yaml` and `pronunciations.yaml`.

## Acceptance for v1

- `cargo build --release` succeeds with no warnings.
- Opening `outputs/ATO_EP03_five_levels` shows 392 turns, four speakers, and a timeline whose total length matches the manifest.
- Playing a turn plays its processed wav; selecting another turn while playing switches cleanly.
- Running a verify job streams output and ends with the exit status shown.
- Running with no episode open, or on a broken manifest, shows an error rather than crashing.
