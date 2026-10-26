# B3 chunk 2: gapless auto-advance

Working directory: this git worktree (branch `ui/phase-b3`).

## Rules

- **No git commands that write.** Change only `ui/src/audio.rs`, `ui/src/main.rs` and `ui/src/capture.rs`.
- Put scratch files under `/tmp/scriptcast-b3/`. Never put them in the worktree.
- Lines under 120 characters, written as normal readable Rust like the surrounding code.
- Mute every playback you start (`--mute`). Never launch Live or Logic. The real project folder is read-only.

## Why

When a turn ends, `on_playback_finished` (main.rs) calls `stop_audio()`, then `select(next)`, then `start_audio(next)`. That restart only happens when a repaint notices `Player::empty()`, so there is a gap of silence of up to 250 ms on MacBook speakers. The operator hears a whine in exactly those gaps.

Goal: when auto-advance is on, line up the next turn's audio **behind** the current turn's audio in the same rodio `Player`, so the audio never stops between turns.

## Design (follow it)

1. **In `audio.rs`, a pure struct** with no rodio types, so it can be tested:
   ```rust
   pub struct TurnQueue { entries: VecDeque<(usize /*turn*/, Duration)> }
   // push(turn, duration); clear();
   // current(elapsed: Duration) -> Option<(usize /*turn*/, Duration /*offset in that turn*/, usize /*how many entries finished before it*/)>
   ```
   `elapsed` is the time since the first queued turn began. `current` walks the summed durations.
2. **In `AudioOut`:**
   - `play()` resets the queue to `[(turn, total)]` as it does today.
   - New `queue_next(turn, &[PathBuf]) -> Result<(), String>` appends that turn's segments to the **same** player and pushes `(turn, total)`. It does nothing if no player exists.
   - Add `queued_after_current() -> bool`.
   - `playing()`, `position()` and `duration()` must report the **current** turn, as determined from the queue and the existing wall-clock tracking. When the wall clock passes a boundary, drop the finished entries and move `base` and `started` on, so `position()` is the offset within the new turn.
   - `is_finished()` stays true only when the whole player has drained.
3. **In `main.rs`, `poll_audio`**, while `Transport::Playing(turn)`, with `preferences.auto_advance && preferences.gapless`:
   1. If `audio.playing()` now reports a turn other than `turn`, the audio has crossed into the next turn. Set `self.transport = Transport::Playing(new)` and call `self.select(new)`. Do **not** restart audio. Then check whether `new` equals `advance_target(...)` for the turn before it, under the current filters. If it does not (for example the filters changed), call `start_audio(target)` so the right turn plays.
   2. If nothing is queued after the current turn and `advance_target` returns `Some(next)` with audio, call `queue_next(next, wavs)` once.
   - `on_playback_finished` stays as the fallback, used at the end of the list or when gapless is off.
   - Selecting a turn manually, Stop, Esc, seek, or turning auto-advance off still go through the existing `play` and `stop` paths, which clear the queue.
4. **Setting.** Add `gapless: bool` (default `true`) to `Preferences`, with `#[serde(default)]` so old saved preferences still load. Add a "Gapless auto-advance" checkbox to the settings gear popover, under a "Playback" heading. Add the CLI flag `--no-gapless` in `capture.rs` for testing.

## Tests (required)

`TurnQueue::current` must cover:
- before the first boundary;
- exactly at a boundary;
- past the last turn (returns `None`);
- an empty queue.

## Check yourself, then report

```bash
cd ui && cargo build --release 2>&1 | tail -2 && cargo test 2>&1 | grep "test result" && cargo clippy --all-targets -- -D warnings 2>&1 | tail -1
awk 'length>120{print FILENAME":"FNR}' src/*.rs
git diff --stat
./target/release/scriptcast-ui --mute --select 4 --play --at 5000 --screenshot /tmp/scriptcast-b3/chunk2.png /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
```

Turn 004 lasts 0.89 s, so by 5 s playback should have moved on to a later turn without stopping. If the capture flags cannot show that, say so. Do not invent a result.

Write `plans/Native_UI/evidence/phaseB3-chunk2.md` with:
- the diff stat;
- all command output;
- which turn the capture shows playing;
- anything you could not verify.

If anything fails, say so plainly.
