# Phase M UI findings

## Status

Implemented on `ui/phase-m`. Version is 0.20.0. No git writes. The GUI was not launched; the orchestrator runs `plans/Native_UI/evidence/capture-phaseM.sh`.

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` are green. Test log: `/tmp/m-ui.log` (`tests/phase_m.rs`: 9 passed). Clippy log: `/tmp/m-ui-clippy.log`.

## Rules observed

- No git writes, no other agent CLIs, no `osascript`, no Live or Logic.
- `/Users/emmanuel/Documents/Theory/TheOriginalPower/` was not opened. Tests do not run Whisper or synthesis.
- No Python changes.
- Colour and size literals for the new meter live in `ui/src/theme.rs` (`METER_WIDTH`).

## Files changed

| File | Change |
|---|---|
| `ui/src/phase_m.rs` | Word range, argv, winner, heard-word diff, ratio class, JSON parsers, Voices-draft assign |
| `ui/src/phase_m_ui.rs` | Sheets, detached jobs, capture fixtures |
| `ui/src/lib.rs` | `pub mod phase_m` |
| `ui/src/main.rs` | Episode → Find outliers, poll and windows |
| `ui/src/capture.rs` | Capture flags |
| `ui/src/phase_l.rs` | Register clip, Make reference, Audition, Calibrate |
| `ui/src/phase_k.rs` | Turn menu “Check stress” |
| `ui/src/jobs.rs` | Jobs panel “Find outliers” |
| `ui/src/theme.rs` | `METER_WIDTH` |
| `ui/Cargo.toml` | 0.20.0 |
| `ui/tests/phase_m.rs` | Fixture tests |
| `ui/tests/fixtures/phase_m/` | JSON fixtures |
| `plans/Native_UI/evidence/capture-phaseM.sh` | M1–M4 captures |

## Tests

`ui/tests/phase_m.rs` covers:

- word-range click, shift-click, and handle snap to word boundaries
- id collision (exact, trimmed, blank)
- argv for clip words, clip register, reference candidates, reference write, audition, calibrate (`--write`), outliers, and stress
- audition winner (first highest `mean`) and heard-word diff
- outlier ratio bands (`> 1.5` or `< 0.6` is danger; `1.5` and `0.6` are within)
- stress card parse, including `{"error":...}`
- `assign_reference` changes only `reference_audio` and `reference_text`
- the recorded Aisha endpoints from the Python findings

## Behaviour

- Transcribe, register, sentence runs, reference write, audition, calibrate, outliers, and stress launch through `detached::launch` with Cancel. They do not run on the UI thread.
- Words JSON is cached per source path and mtime.
- Clip id collision is an inline error. The result card appears only after a successful register. Insert uses the Phase K `insert` op with the archive speaker id and the clip body.
- Audition shows `mean` × 100, `clean/of`, temperature, take `score` and `run`. Differing heard words are underlined in `STATUS_FAILED`. There is no WER.
- Calibrate “Write” runs the tool again with `--json --keep --write` (comments stay because the tool uses `config_edit`). The pronunciations tab is marked stale.
- Outlier row click selects that manifest turn. Stress `heard` is drawn with the mono face.
- “Use for speaker” writes the Voices draft only. The user still presses Save.

## Deviations

- Mock-ups’ WER, illustrative waveforms, annotation strips, and the separate collision / “after saving” panels are not in the UI. Collision is inline. The result card is post-save only.
- `words-aisha.json` keeps the recorded endpoints only (`The` 0.00–0.18, `it.` 15.12–15.38, duration 15.41, last index 44). The other 43 tokens were not in the findings file, so they were not invented.
- `words.json` follows the Python unit-test shape (`Hello` / `world` / `later`) and is what M1 capture loads.
- Audition “Use this reference” sets `reference_text` only when a sibling `.txt` exists. Otherwise only `reference_audio` changes, and the sheet says so. Reference-maker “Use for … in Voices” sets both from the write result.
- Calibrate and outliers share one window so M4 can be captured together. Closing it closes both.
- The stress result is a card window opened from the turn menu, not a block inside the inspector scroll.
- Reference-maker handles drag in seconds. Clip-maker handles snap to word boundaries.

## Unverified

- Pixel match to `ui/design/M1-clip-maker.png`, `M2-reference-maker.png`, `M3-audition.png`, and `M4-calibrate-outliers.png`. Run `sh plans/Native_UI/evidence/capture-phaseM.sh`.
- A live `scriptcast-clip` / audition / calibrate / outliers / stress process from the GUI.
- Playback of fixture WAVs (those paths are not files, so Play stays disabled).

## Visual round 1

Captured against `ui/design/M1-M4` on `SCRIPTCAST_PROJECT=/tmp/scriptcast-k/proj`. The four tools no longer open as small collapsible `egui::Window`s.

- Clip maker is a centred modal sheet (`CLIP_MAKER_WIDTH` 1100, `CLIP_MAKER_HEIGHT` 720). Title “Clip maker” is left-aligned. The step row is “1 Source · 2 Pick words · 3 Details”, with the current step in the accent colour. Cancel closes it. Left column: source path, Choose…, Transcribe, progress, waveform with range handles, “start → end · duration”, Play range, then the transcript as one flowing line. The selected run is a single accent band. Right column: Id and Pad ms on one row, Speaker, Citation, Origin URL, Content note, and Cancel / Register clip at the bottom right.
- The waveform and Play range use the Phase L path: `waveform::clip_envelope` (rodio, then cached ffmpeg) for peaks, and `audition_project_media` (rodio play, then `ffmpeg_wav`) for playback. That is the same fallback the Clips detail waveform uses, so an `.mp4` that rodio reports as `isomp4: packet out-of-bounds` still draws and plays.
- Reference maker, Audition, and Calibrate/Outliers replace the Voices or Pronunciations content with `bordered_columns` and `tool_footer`. Headers are “Reference maker”, “Audition”, and “Calibrate pronunciation  |  Outliers · {episode}”. Status sits on the left of the footer; Cancel and the primary action sit on the right.
- Reference candidates are a table: Rank, Time range / duration, Sentence text, Score / 100 with the meter, Play. The selected row expands in the right panel: waveform, Start / End / Play on one row, Exact transcript, Reference name and Peak on one row. Write reference is the footer primary.
- Audition puts the speaker picker, reference chips, and Run audition on one row. Each candidate is an equal-width bordered card. The big number is mean × 100: ≥ 90 `STATUS_PASSED`, ≥ 80 `TEXT_PRIMARY`, otherwise `STATUS_WARNING`. “clean/of takes” sits beside it. The winning card uses the accent border and a Winner badge at the top right. Each take is sentence, “WHISPER HEARD”, underlined heard text, Play. One footer primary, “Use this reference for {speaker}”, applies to the winner.
- Calibrate lists readings as radio rows and puts the speaker picker beside Run calibration. Results are a grid: Spelling, Score / 100, Take 1–3 with Play and ✓/✗. The best spelling uses the accent colour. Write is the footer primary. Outliers are a table: Turn / speaker, Pred. s, Actual s, Ratio (meter and value), Reason, with the selected-turn card under the table.
- Pronunciation entries label the key “Word”. Readings are a two-column table, Reading key and Spoken as, with Delete on each row.

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` were run again after this round. Logs: `/tmp/m-ui-fmt.log`, `/tmp/m-ui-clippy.log`, `/tmp/m-ui.log`. All three exited 0. The GUI was not launched.

## Visual round 2

- Audition results use `ui.columns(n)`. The columns are equal width and top-aligned, and each card is at least as tall as the column. Inside a card the stack is vertical: reference name with the Winner badge on the right, the big score beside “clean/of takes”, then each take as sentence, “WHISPER HEARD”, the heard line, and “score · run” with Play on the right. The heard line is one wrapping galley at the card’s inner width, with differing words in `STATUS_FAILED` and underlined. A binary test lays out a card and checks that galley’s wrap width against the card inner width.
- The Speaker picker keeps a current id only when that person is in the project. Otherwise it uses the Voices-tab selection (`select_id`), then the first eligible speaker. The footer primary is “Use this reference for {display name}”, so a selected Aisha reads “Use this reference for Aisha”. A fixture id that is not a project speaker, such as `guest`, does not stay in the picker.
- Clip-maker peaks no longer treat a short rodio decode as the whole file. `clip_envelope` keeps a native decode only when the peaks cover the requested window. Otherwise it asks ffmpeg for that window as mono 48 kHz PCM (`-ss` before `-i`, `-t` and `-vn` after `-i`) and refuses a result that still ends early. While that decode is in flight the waveform says “Loading waveform…”. A finished envelope that covers the window is drawn across the full width.
