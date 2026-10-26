# Phase B — scriptcast-ui playback (implementer brief)

You are implementing Phase B of the scriptCast native UI (Rust, eframe/egui 0.36). Your working directory is a git worktree of the scriptCast repo on branch `ui/phase-b`, created from `dev` after Phase A was merged and verified (font switch `4616e98`, this brief committed on top).

## Prohibitions (read first)

- **No git writes.** Do not run `git commit`, `push`, `branch`, `checkout`, `switch`, `reset`, `restore`, `clean`, `stash`, `rebase` or `merge`. The orchestrator reviews and commits. `git status` and `git diff` are fine.
- **Write only inside this worktree**, plus `~/.cargo`, `ui/target/`, and scratch files and PNGs under `/tmp/scriptcast-phase-b/` (create it).
- **The real episode is read-only:** `/Users/emmanuel/Documents/Theory/TheOriginalPower/` and everything under it. To test missing audio, copy the **fixture** episode into `/tmp/scriptcast-phase-b/` and remove a WAV there; never touch the real episode.
- **No Python changes:** `scriptcast/`, `tests/`, `pyproject.toml`, `video/`, `swift/`.
- **Do not modify** `ui/design/**`, `ui/assets/**`, or `plans/**` except the findings file named below.
- **No network** other than Cargo fetching crates. Never call Figma.
- **Every playback you start yourself must be muted** (see `--mute` below). It is late at night where the operator is.

## Goal

Implement plan steps 9–10 of `plans/Native_UI/plan.md` Section 4:
- `audio.rs`;
- playback wired into the app, built to design screen 03;
- the Missing audio and Unavailable device workspace states.

The done state:
- the operator can select a turn and press Space to hear it;
- pause, resume and stop work;
- the playback bar shows real progress;
- auto-advance moves to the next visible turn;
- the output device can be chosen;
- a Selected turn inspector appears.

If part of the plan is not viable as written, say so in the findings file with evidence.

## Read these, in order

1. `ui/design/DESIGN.md`: the Playback bar, Inspector "Selected turn", Keyboard, and sample-data sections, and the font override.
2. `ui/design/screens/03-selected-turn-playing/`: `screenshot.png` is the target. `design-context.tsx` gives exact sizes and copy; it is a spec, not code to port.
3. `ui/design/components/workspace-state/`: the Missing audio and Unavailable device cards.
4. `plans/Native_UI/plan.md`: Flow 2, Observations, and Steps 9–10.
5. `plans/Native_UI/evidence/phaseA-findings.md`: egui 0.36 API notes already learned (`ui()` instead of `update()`, `Ui::put` advancing the cursor, headless test texture cleanup, measured-width helpers).
6. The existing code: `ui/src/main.rs`, `theme.rs`, `episode.rs`, `table.rs`, `timeline.rs`, `view.rs`, `capture.rs`.

## Established facts (measured 2026-10-03)

- Phase A is verified on `dev`, with 40 tests and clippy clean. The app already has a disabled playback bar, single selection (`select()`), Up/Down navigation, a `--select <turn_index>` flag, and `--screenshot/--fresh/--size` self-capture (`capture.rs`). EP03 loads in about 10 ms.
- Fonts: the proportional face is **OpenDyslexic 3**, with OpenDyslexic 0.99 registered as a fallback for symbols, and the mono face is OpenDyslexicMono with the same fallback (`theme.rs` `apply`). **Use the mono face for every time readout, file name and number** ("00:04.620 / 00:09.060", "turn_0004.wav", "48 kHz"). The fonts lack ⌘ and ⌄; use egui widgets that draw their own arrows (for example `ComboBox`), and write "Cmd+".
- rodio 0.22.2 is already a dependency (default features include WAV decoding). Its source is at `~/.cargo/registry/src/*/rodio-0.22.2/`.
  - `Player` (`src/player.rs`): `connect_new(&mixer)`, `append(source)`, `play()`, `pause()`, `is_paused()`, `stop()`, `clear()`, `empty()`, `get_pos()`, `try_seek()`, `set_volume()`.
  - Default output: `rodio::DeviceSinkBuilder::open_default_sink()`, then `.mixer()`.
  - Specific device: `rodio::speakers::available_outputs()` returns `Output` values that implement `Display` (the device name) and have a default flag. Pass one to `SpeakersBuilder::new().device(output)?.default_config()?.open_mixer()` (`src/speakers/builder.rs:130-175, 549-561`).
  - Device-loss detection: `examples/error_callback.rs` shows `DeviceSinkBuilder::from_device(cpal_device)?.with_error_callback(|err| …)`, catching `cpal::StreamError::DeviceNotAvailable | StreamInvalidated`. rodio re-exports `cpal`. **Check every signature in the source; do not trust memory of older rodio** (`OutputStream` and `Sink` are gone).
- EP03 WAVs are mono, 48 kHz, Int16; turn 0 is 8.36 s. EP03 has **no** multi-segment turns; the fixture's turn 2 has two segments (use it for chunk-order tests).
- EP03 turn 4 is 0.89 s, and turn 0 is 8.27 s by manifest timing.

## What to build

**Step 9, `ui/src/audio.rs`:**
- `AudioOut`, with:
  - `open(device: Option<&str>) -> Result<Self, String>` (`None` means the default output; a named device falls back to the default with a returned notice if it is gone);
  - `outputs() -> Vec<String>`;
  - `play(turn: usize, wavs: &[PathBuf]) -> Result<(), String>`, which stops the current player, creates a new `Player` and appends each segment in order, decoded from `BufReader<File>`; a missing or undecodable file returns `Err` naming the path;
  - `pause()`, `resume()`, `stop()`, `is_paused()`, `is_finished()`, `position() -> Duration` (from `Player::get_pos`), `duration() -> Duration` (the sum of decoded segment durations, or manifest timing if a decoder cannot report one), `playing() -> Option<usize>`, `set_muted(bool)`, `device_name()`;
  - a device-error channel the app polls each frame (`take_device_error() -> Option<String>`).
- Keep the sink alive for the app's lifetime. One player at a time.

**Step 10, `ui/src/main.rs` and friends, built to screen 03:**
- **Keys:** Space plays the selected turn, or toggles pause if that turn is already playing. Esc stops. Up/Down keep the Phase A behaviour, and selecting a different turn while playing switches playback immediately. Shortcuts are ignored while a text field has focus (Phase A has the rule).
- **Playback bar:** Play/Pause primary button (play or pause icon from `ui/design/assets/icons/`) and Stop. The current-audio block shows "Turn 004 · Toussaint" and `mm:ss.mmm / mm:ss.mmm` in mono. The seek track has an accent elapsed fill. "Auto-advance" is a checkbox, persisted. The output device menu is a `ComboBox` of `outputs()`, persisted by name. The bar is disabled at 45% when nothing is selected or there is no audio.
- **Auto-advance:** when the player finishes, select and play the next **visible** turn, respecting search, speaker filter and Failed-only. At the end of the list, stop.
- **Timeline:** a 1 px `TEXT_PRIMARY` playhead across the speaker lanes at the playing position; the playing segment uses the selected style.
- **Table:** the playing row shows "▶" before its number. ▶ comes from the fallback font and must render; check it in a capture.
- **Selected turn inspector (right, 376 wide, from `theme.rs` tokens), shown when a turn is selected:**
  - heading "Selected turn" and "<n> / <total>";
  - speaker row with the verification state;
  - fields:
    - TRANSCRIPT, with the `expected` text from `stress.json` when this turn has an entry, otherwise FIRST WORDS with the `TURN_INDEX.csv` text. **Never label first words as a transcript.**
    - AUDIO: `start → end · seconds`.
    - FILE: wav name(s) in mono, plus "mono/stereo · <kHz>" from the decoder.
  - the accent status sentence;
  - Previous / Next buttons;
  - the shortcut list (Space, ↑ ↓, Esc).
  - No "Review failure" button yet; Phase C adds it.
- **Status bar:** while playing, "Playing turn 004 · selected in timeline and table".
- **Repaint:** `request_repaint_after(50 ms)` only while playing, and no repaint when idle. Idle CPU must stay about 0%.
- **Missing audio:** if a selected turn's WAV is missing, the row shows "Missing audio" (Phase A computes the state) and Play shows the Missing audio state for that turn (the card's text with real names). "Regenerate selected" is shown disabled; Phase C enables it. Other turns still play.
- **Unavailable device:** on a device error, pause, keep position and selection, and show the Unavailable device state with "Choose output", which opens the device menu. Browsing keeps working.

**Self-capture extensions (`capture.rs`), so the orchestrator can verify without UI control:**
- `--play`: after load and `--select`, start playback of the selected turn.
- `--at <ms>`: seek the player to that position after starting (`try_seek`).
- `--mute`: volume 0. **`--screenshot` together with `--play` always implies `--mute`.** A capture run never makes sound.
- With `--play`, the capture waits until the position has advanced at least 200 ms past `--at`, then captures **while playing**, so the Pause button and playhead are visible.
- `--simulate-device-loss`: after `--play`, inject a device error through the same channel the real error callback uses. Use it only for capturing the Unavailable device state.
- Keep the existing flags working.

## Contract (Phase C builds on this)

- `lib.rs` adds `pub mod audio;`.
- `AudioOut`'s public methods are as above.
- The inspector is a panel with a mode enum, `InspectorMode::{SelectedTurn, …}`, so Phase C can add `Verification` and `JobRunner` at 436 and 510 px.
- No colour or size literals outside `theme.rs`.

## Acceptance (run all; all must pass)

```bash
cd ui
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
mkdir -p /tmp/scriptcast-phase-b
EP=/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
./target/release/scriptcast-ui --screenshot /tmp/scriptcast-phase-b/b-03-playing.png --select 0 --play --at 4620 "$EP"
./target/release/scriptcast-ui --screenshot /tmp/scriptcast-phase-b/b-03-paused-sel.png --select 4 "$EP"
./target/release/scriptcast-ui --screenshot /tmp/scriptcast-phase-b/b-device-lost.png --select 0 --play --at 2000 --simulate-device-loss "$EP"
cp -R ui/tests/fixtures /tmp/scriptcast-phase-b/fixtures-missing
# then delete one WAV of turn 2 in that copy
./target/release/scriptcast-ui --screenshot /tmp/scriptcast-phase-b/b-missing.png --select 2 --play /tmp/scriptcast-phase-b/fixtures-missing/episode_small
```

Every command exits 0. The missing-audio run must still exit 0, with the Missing audio state captured.

Tests that must exist and pass:
- unit tests for:
  - next visible turn under a filter;
  - auto-advance target at the end of the list;
  - Space/Esc state transitions (an audio-free state machine);
  - `mm:ss.mmm` formatting;
  - TRANSCRIPT vs FIRST WORDS labelling;
- an `#[ignore]` test `fixture_two_segment_turn_plays_in_order` that plays the fixture's turn 2 **muted** and asserts that the player finishes and that `duration()` equals the sum of both WAVs (0.4 s ± 20 ms). Run it with `cargo test --release -- --ignored` and report the result.
- The colour grep still prints nothing:

  ```bash
  grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' ui/src | grep -v theme.rs
  ```

## Not yours to claim

The orchestrator and operator run these legs. Do not claim them:
- that audio sounds right;
- by-ear checks;
- real headphone unplugging;
- visual fidelity against screen 03.

## Stop conditions

- **If rodio cannot open any output device in your environment,** do everything that does not need audio, mark the device-dependent acceptance items as blocked with the exact error, and stop. Do not stub audio to fake a pass.
- **If `Player::get_pos()` does not advance correctly across appended segments,** record measurements in the findings file and track position yourself (an `Instant` adjusted for pause and seek). Say which you used.
- **If a feature here conflicts with Phase A behaviour or tests,** keep Phase A behaviour and record the conflict.

## Findings file

Create `plans/Native_UI/evidence/phaseB-findings.md` **before** you start step 10, and keep it current. Record:
- brief, plan or design defects and what you did;
- rodio and egui API notes;
- the position-tracking method;
- the PNG paths and sizes;
- the acceptance output summary;
- anything left undone.
