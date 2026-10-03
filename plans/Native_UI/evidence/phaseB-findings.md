# Phase B findings

Model: Kimi Code CLI (moonshotai/K2.5-class). Scope: plan steps 9–10 — `audio.rs`, playback wiring, screen 03, Missing audio / Unavailable device states, capture flags `--play/--at/--mute/--simulate-device-loss`.

Started 2026-10-03 on branch `ui/phase-b` (worktree `phase-b`). No git writes were or will be made.

## Constraints and decisions (kept current as work progresses)

- **rodio device API substitution.** The brief names `rodio::speakers::available_outputs()` + `SpeakersBuilder`, but in rodio 0.22.2 `pub mod speakers` is gated on `#[cfg(all(feature = "playback", feature = "experimental"))]` (`src/lib.rs:187`), and `experimental` (which pulls `dep:atomic_float`) is **not** in rodio's default features (`Cargo.toml [features]`). Rather than change the dependency shape, audio.rs uses the equally supported `DeviceSinkBuilder` path from rodio's own `examples/error_callback.rs`: enumerate devices via the re-exported `rodio::cpal` (`default_host().output_devices()`, cpal 0.17.3 `DeviceTrait::description().name()`), then `DeviceSinkBuilder::from_device(dev)?.with_error_callback(cb).open_sink_or_fallback()`. The same builder takes the device-loss error callback, which the `speakers` builder's `open_mixer()` also supports but the brief's default-sink path (`open_default_sink()`) does not. `Cargo.toml` is unchanged.
- **Position tracking: wall-clock, not `Player::get_pos()`.** Reading `player.rs:104-162`: every `append()` wraps its source in a fresh `track_position()` and a `periodic_access(5 ms)` closure that writes the shared `controls.position`. `get_pos()` therefore reports the position **within the currently playing segment only** and resets to ~0 at each segment boundary (also reset to zero by `stop()`/`clear()`). Multi-segment turns (fixture turn 2) would show a sawtooth position. `AudioOut` therefore tracks position itself: `base: Duration + started: Instant`, frozen into `paused_at` on pause, rebased on seek, clamped to `duration()`. This is the brief's documented fallback. Measured `get_pos()` samples confirming the per-segment reset are recorded below (ignored test output).
- **Seek granularity.** `Player::try_seek` applies to the *current* source only (player.rs:238-257; a `SeekOrder` consumed by the playing source's periodic access). Seeking into a later segment of a multi-segment turn is therefore not possible through the player; `--at` and device-loss resume seek within the turn's first segment. All acceptance runs use single-segment turns. Noted as a limitation, not worked around with fake queue surgery.
- **`outputs()` is an associated function** (`AudioOut::outputs()`, no `&self`) so the device menu can be populated even when `open()` failed at startup. All other briefed methods are instance methods with the briefed signatures.
- **Device fallback notice.** `open(Some(name))` with a missing device falls back to the default output; the notice is carried on the struct and read with `take_notice()` (the briefed `open` signature `Result<Self, String>` cannot return a notice otherwise).
- **Error callback filter** follows `examples/error_callback.rs`: only `cpal::StreamError::DeviceNotAvailable | StreamInvalidated` are sent to the app; other stream errors go to stderr. `--simulate-device-loss` sends through the same `mpsc` channel the callback uses (`AudioOut::simulate_device_loss_for_capture`).
- **Muted captures.** `Cli::effective_muted()` = `--mute || (--screenshot && --play)`; mute maps to `Player::set_volume(0.0)`, re-applied to every new player in `play()`. A capture run never makes sound.
- **Auto-advance skips unplayable turns.** The brief says "moves to the next visible turn" and separately "Other turns still play" for missing audio. Auto-advance selects the next *visible* turn whose WAVs all exist; a missing-audio turn never auto-plays (pressing Play on it shows the Missing audio card instead). At the end of the visible list playback stops and the selection stays.
- **"Choose output"** opens a device menu anchored at the card button (`egui::Popup::menu(&response)`, egui 0.36.2 `popup.rs:236`) listing the same `AudioOut::outputs()` as the playback-bar ComboBox. The playback-bar ComboBox draws its own arrow (fonts lack ⌄, Phase A finding).
- **Playback bar enabled rule.** Transport/seek/auto-advance are disabled (45%) when no episode, no selection, or audio output failed. The device ComboBox stays enabled in every episode-loaded state so a lost device can be recovered from the bar as well as from the card. Play on a missing-audio turn is *enabled* and opens the Missing audio card (the brief requires the click to show the state).
- **Inspector** lives in `ui/src/inspector.rs` with `InspectorMode::{SelectedTurn}` (width from `theme::INSPECTOR_WIDTH_SELECTED_TURN`); Phase C adds `Verification`/`JobRunner`. Shown iff a turn is selected.
- **FILE field** decodes the first WAV of the selected turn per selection change (cached by turn id): `<names> · mono|stereo · <kHz>`, all mono face. Multi-segment turns list every file name joined with " + ".
- **TRANSCRIPT vs FIRST WORDS**: `TRANSCRIPT` only when the turn has a `stress.json` entry with non-empty `expected`; otherwise `FIRST WORDS` with the `TURN_INDEX.csv` text (never labelled transcript).
- **Reload stops playback** (`accept_episode` resets the transport): reloaded WAV paths/bytes may differ under a playing player.
- **Status bar**: "Playing turn NNN · selected in timeline and table" while playing, "Paused turn NNN · …" while paused (same suffix, per screen 03 pattern).

## rodio / egui 0.36 API notes (verified against registry sources)

- rodio 0.22.2: `Player::connect_new(&Mixer)`, `append(S: Source + Send + 'static)`, `play/pause/is_paused/stop/empty/get_pos/try_seek/set_volume` (`src/player.rs`). `Player::drop` stops its sounds; `MixerDeviceSink` must outlive players (drop ends output).
- `DeviceSinkBuilder::from_device(cpal::Device) -> Result<DeviceSinkBuilder, DeviceSinkError>`; `.with_error_callback(F: FnMut(cpal::StreamError) + Send + Clone + 'static)`; `.open_sink_or_fallback()` requires the callback `Clone` — an `mpsc::Sender<String>` capture satisfies this.
- `MixerDeviceSink::log_on_drop(false)` silences the drop-time stderr notice.
- cpal 0.17.3 via `rodio::cpal`: `default_host()`, `HostTrait::output_devices()`, `HostTrait::default_output_device()`, `DeviceTrait::description() -> Result<DeviceDescription,_>`, `DeviceDescription::name() -> &str`.
- `Decoder::new(BufReader<File>)` (decoder/mod.rs:386); `Source::total_duration()/channels()/sample_rate()` on the decoded WAV give exact durations and stream info. Symphonia WAV `total_duration()` is `Some` for our files.
- egui 0.36.2: `egui::Popup::menu(&response).show(...)` anchors a menu at a button; `ComboBox::from_id_salt(...).selected_text(...).show_ui(...)` as before.

## Capture state machine additions

- `--play` requires `--select`; `--at` and `--simulate-device-loss` require `--play` (usage error, exit 2).
- With `--play`: at the Loading→Settling transition the app starts playback of the selected turn, then seeks to `--at` (try_seek), then optionally injects the device error. Settling ends when the tracked position is ≥ `--at + 200 ms` (default `--at` = 0), or immediately once the Missing audio / Unavailable device card is up — the two failure runs must still exit 0 with the card captured. The 500 ms quiet-window rule is skipped for `--play` captures (playing requests a 50 ms repaint every frame by design); tiny embedded icons finish loading well inside the 200 ms audio-advance window.

## Position-tracking measurement (fixture turn 2, two 0.2 s WAVs, muted)

`cargo test --release fixture_two_segment_turn_plays_in_order -- --ignored --nocapture` (this machine, default output): the player finished in ~0.6 s and `duration()` reported exactly 400 ms from the two decoded WAVs. Sampled every 20 ms, rodio's `Player::get_pos()` read:

```text
[0, 0, 0, 35, 35, 80, 125, 125, 125, 170, 170, 20, 20, 65, 65, 110, 110, 160]
```

It climbs through segment 1 (~0→200 ms), **resets to ~20 ms at the segment boundary**, and climbs again — confirming the per-segment `track_position()` in `player.rs:122-143`. Conclusion: `get_pos()` cannot report turn position for multi-segment turns, so `AudioOut::position()` is wall-clock (`base + started.elapsed()`, frozen on pause, rebased on seek, clamped to `duration()`), exactly the brief's documented fallback. `is_finished()` uses `Player::empty()` (queue drained), which is unaffected.

## Acceptance output summary

(Filled by the orchestrator: Kimi's weekly quota ended while it was finalising this file, after its last acceptance run had passed.)

Re-run by the orchestrator on 2026-10-03 after merging `dev` (font alignment, scriptcast-export) into `ui/phase-b`:
- `cargo build --release`: green.
- `cargo test`: 4 lib + 29 bin + 15 loader + 1 font pass. 2 ignored: the EP03 load test and the audio test.
- `cargo clippy --all-targets -- -D warnings`: green.
- Colour grep outside `theme.rs`: empty.
- `cargo test --release -- --ignored fixture_two_segment`: passes (muted). Both chunks play in order and `duration()` = 400 ms.
- All four briefed captures exit 0 at 2880×1800: playing, selected, device lost, and missing audio.

## PNG paths and sizes

Downscaled to 1440×900 and saved in `plans/Native_UI/evidence/`:
- `phaseB-playing.png`: EP03, `--select 0 --play --at 4620`, muted.
- `phaseB-sel.png`: EP03, `--select 4`.
- `phaseB-device-lost.png`: EP03, `--simulate-device-loss`.
- `phaseB-missing-audio.png`: a fixture copy with `turn_2_1.wav` removed.

## Left undone / orchestrator changes

- **Orchestrator fix after review:** the workspace-state cards showed the Figma specimen caption ("Workspace state / …") and the designer's footnotes. Both were removed (DESIGN.md has been corrected). The raw device error moved from the body text into a tooltip on the card title.
- **Operator legs, not yet done:** audible playback, a real headphone unplug, and the device menu with real devices.
- **Known limits, from the findings above:**
  - seeking works only within the current segment, which matters for multi-segment turns;
  - position is wall-clock.
