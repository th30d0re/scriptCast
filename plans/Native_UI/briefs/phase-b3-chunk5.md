# B3 chunk 5: scrolling waveform in the playback bar; remove the speakers-only special cases

Working directory: this git worktree (branch `ui/phase-b3`).

## Rules

- **No git commands that write.** Change only `ui/src/main.rs`, `ui/src/theme.rs`, `ui/src/table.rs`, `ui/src/waveform.rs` and `ui/src/audio.rs`.
- Put scratch files under `/tmp/scriptcast-b3/`. Never put them in the worktree.
- Lines under 120 characters, written as normal readable Rust like the surrounding code. Run `cargo fmt`. **Do not delete or reword existing comments** unless the code they describe is removed.
- No colour or size literals outside `ui/src/theme.rs`. Add tokens there, named for what they are.
- Mute every playback you start. The real project is read-only.

## Background

- An earlier fix made playback on built-in MacBook speakers repaint every 250 ms instead of 50 ms, and turned off animated table scrolling there. The theory was that redraws caused a whine.
- The real cause turned out to be resampling. It is fixed in `audio.rs` by opening the output at 48 kHz, and the operator has confirmed it by ear.
- So the speakers-only special cases are now pointless, and they make playback on the speakers look choppier.
- `ui/src/waveform.rs` already exists, from the previous chunk: `Waveforms` loads per-turn `Envelope`s in the background; `bars(env, n)` resamples them.

## Task

1. **Remove the speakers-only special cases.**
   - Delete `SPEAKER_REPLAY_TICK` and `playback_tick`; use `REPLAY_TICK` everywhere.
   - In `table::show`, remove the `animated_scroll` parameter and restore the scroll animation's default behaviour, and update the call site.
   - Remove the test that asserted the speaker tick (around `main.rs` "SPEAKER_REPLAY_TICK"), or adapt it.
   - **Keep** `audio::is_builtin_speakers` and its tests; it is harmless and may be useful later.
2. **Replace the seek track with the waveform viewport** (target: `ui/design/screens/03d-waveforms/playback-bar.png`; Figma node `4:18360` in `ui/design/page-metadata.xml`). Do this in the playback bar code where the `track` rect is drawn today (search `SEEK_TRACK_HEIGHT` in `main.rs`).
   - **Viewport rect:** same horizontal extent as today's track, height `t::WAVE_VIEWPORT_HEIGHT` (new token, 36), centred vertically. Fill `t::BG_SURFACE`, 1 px `t::BORDER_DEFAULT` stroke, radius `t::RADIUS_WAVE_VIEWPORT` (new token, 3).
   - **Content:** the playing (or selected) turn's envelope, drawn as vertical bars (`t::WAVE_BAR` width, `t::WAVE_GAP` gap, `t::WAVE_MIN` minimum height, all existing tokens), centred vertically, using up to 80% of the viewport height.
   - **Scrolling:** a fixed window of `t::WAVE_VIEWPORT_SECONDS` seconds (new token, 6.0) of audio spans the viewport width. The current playback position sits at the **horizontal centre**, so the waveform scrolls left as audio plays. Before position 0 and after the turn's end, the area is empty.
   - **Colours:** bars left of the centre (already played) use `t::ACCENT`; bars right of it use `t::TEXT_MUTED`.
   - **Playhead:** a 2 px wide, full-height rect at the centre in `t::TEXT_PRIMARY`.
   - **Fades:** a `t::WAVE_FADE` (new token, 32) wide gradient at each end, from `t::BG_SURFACE` to transparent. Use an egui `Mesh` with per-vertex colours for the gradient.
   - **Seeking:** clicking or dragging in the viewport seeks. The new position is the current position plus `(pointer_x - centre_x) / viewport_width * WAVE_VIEWPORT_SECONDS`, clamped to 0..duration, through the existing seek path. Use `ui.interact(rect, id, Sense::click_and_drag())`.
   - **Nothing playing or selected, or no envelope yet:** draw the empty viewport box only.

## Check yourself, then report

```bash
cd ui && cargo fmt && cargo build --release 2>&1 | tail -2 && cargo test 2>&1 | grep "test result" && cargo clippy --all-targets -- -D warnings 2>&1 | tail -1
awk 'length>120{print FILENAME":"FNR}' src/*.rs
grep -n "SPEAKER_REPLAY_TICK\|animated_scroll" src/*.rs     # must print nothing
git diff --stat
./target/release/scriptcast-ui --mute --select 7 --play --at 4000 --screenshot /tmp/scriptcast-b3/chunk5.png /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
```

Open the PNG and say plainly what the playback bar shows.

Write `plans/Native_UI/evidence/phaseB3-chunk5.md` with:

- the diff stat;
- all command output;
- what the PNG shows.

If anything fails, say so plainly; do not claim success.
