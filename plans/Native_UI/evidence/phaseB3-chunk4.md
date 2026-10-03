# Phase B3 Chunk 4: Waveform data and timeline-block waveforms

## Diff Stat

`git diff --stat` (tracked edits only):

```
 ui/src/lib.rs      |  1 +
 ui/src/main.rs     |  8 ++++++++
 ui/src/theme.rs    |  3 +++
 ui/src/timeline.rs | 46 +++++++++++++++++++++++++++++++++++++++-------
 4 files changed, 51 insertions(+), 7 deletions(-)
```

New untracked file (not shown by `git diff --stat`):

```
 ui/src/waveform.rs | 205 lines (new)
```

## Command Output

`cd ui && cargo fmt && cargo build --release 2>&1 | tail -2 && cargo test 2>&1 | grep "test result" && cargo clippy --all-targets -- -D warnings 2>&1 | tail -1`

```
    Finished `release` profile [optimized] target(s) in 36.90s
test result: ok. 12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 15 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.93s
```

`awk 'length>120{print FILENAME":"FNR}' src/*.rs`

```
(no output)
```

`./target/release/scriptcast-ui --mute --select 6 --screenshot /tmp/scriptcast-b3/chunk4.png …/ATO_EP03_five_levels` (after eprintln removal):

```
scriptcast-ui: Login shell PATH loaded
scriptcast-ui command: scriptcast · Project virtualenv: …
scriptcast-ui command: scriptcast-export · Dev checkout: …
scriptcast-ui command: scriptcast-repair · Project virtualenv: …
scriptcast-ui command: scriptcast-verify · Project virtualenv: …
screenshot: /tmp/scriptcast-b3/chunk4.png 2996x1800
```

## Timing

Temporary `eprintln!` left in place for one release run on EP03 (392 turns with audio), then removed:

```
Waveform loading took 3.887942208s for 392 turns
```

## PNG

`/tmp/scriptcast-b3/chunk4.png` (2996×1800). **Yes — the timeline blocks show bars.** Turn numbers sit at the left of each block; the remaining width is filled with thin vertical bars of varying height in the speaker/label colour, centred vertically. Selected turn 006 shows a clear bar pattern inside its block.

## What Changed In Review (this finish pass)

Inherited implementation already matched the brief on the three review points:

- `envelope` stores max abs sample per 10 ms bucket (rodio `Sample` = f32 full scale); no per-turn peak rescale.
- background thread calls `ctx.request_repaint()` every 50 completed turns, plus once at the end.
- `Drop for Waveforms` sets the shared `AtomicBool`; the thread checks it between turns.

Finish-pass edits only:

1. Moved worktree scratch junk to `/tmp/scriptcast-b3/`: `ui/src/bin/scratch.rs` (removed empty `ui/src/bin/`), `scratch_pick_rate`, `scratch_pick_rate_test`.
2. Added `#[expect(clippy::too_many_arguments)]` above `timeline::show` (same pattern as `table::show` / `inspector::show`).
3. Recorded EP03 load timing, then removed the temporary timing `eprintln!` and its locals from `waveform.rs`.
4. Wrote this evidence file.

No other brief mismatches required code changes.
