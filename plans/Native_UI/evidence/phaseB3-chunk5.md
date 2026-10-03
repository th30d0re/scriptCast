# Phase B3 Chunk 5: scrolling waveform playback bar; remove speaker special cases

## Diff Stat

`git diff --stat`:

```
 ui/src/main.rs     | 205 ++++++++++++++++++++++++++---------------------------
 ui/src/table.rs    |   2 -
 ui/src/theme.rs    |   6 ++
 ui/src/waveform.rs |  34 +++++++++
 4 files changed, 142 insertions(+), 105 deletions(-)
```

## Command Output

`cd ui && cargo fmt && cargo build --release 2>&1 | tail -2 && cargo test 2>&1 | grep "test result" && cargo clippy --all-targets -- -D warnings 2>&1 | tail -1`

```
   Compiling scriptcast-ui v0.1.0 (/Users/emmanuel/Dev/Tools/scriptCast-worktrees/phase-b3/ui)
    Finished `release` profile [optimized] target(s) in 37.46s
test result: ok. 13 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.18s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 15 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.75s
```

`awk 'length>120{print FILENAME":"FNR}' src/*.rs`

```
(no output)
```

`grep -n "SPEAKER_REPLAY_TICK\|animated_scroll" src/*.rs`

```
(no output)
```

`./target/release/scriptcast-ui --mute --select 7 --play --at 4000 --screenshot /tmp/scriptcast-b3/chunk5.png …/ATO_EP03_five_levels`

```
scriptcast-ui: Login shell PATH loaded
screenshot: /tmp/scriptcast-b3/chunk5.png 2996x1800
```

## PNG

`/tmp/scriptcast-b3/chunk5.png` (2996×1800). Cropped bar: `/tmp/scriptcast-b3/chunk5-bar.png`.

**What the playback bar shows:** Pause + Stop on the left; “Turn 007 · Aisha” with `00:04.374 / 00:15.869`; a bordered waveform viewport in the middle with a centred white playhead, accent bars to the left of centre and muted bars to the right, edge fades at both ends; Auto-advance / Space hint / MacBook Pro Speakers on the right. Matches `ui/design/screens/03d-waveforms/playback-bar.png`.

## What Changed

1. Removed `SPEAKER_REPLAY_TICK`, `playback_tick`, and `mod speaker_redraw_tests`. Playback always uses `REPLAY_TICK` (50 ms).
2. Removed `animated_scroll` from `table::show`; scroll animation is default again. Removed unused `output_name` (only served the speaker tick).
3. Kept `audio::is_builtin_speakers` and its tests.
4. Replaced the thin seek track with a scrolling waveform viewport (`WAVE_VIEWPORT_*` tokens, playhead, Mesh edge fades, click/drag seek).
5. Added `waveform::peak_in_range` (+ test) for time-window peak sampling.

Workers (codex, kimi, agy) were all quota-exhausted; implementation was done in-session from the SPEC.
