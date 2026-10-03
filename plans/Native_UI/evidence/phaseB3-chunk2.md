# Evidence

## Diff Stat
```
 ui/src/audio.rs   | 167 +++++++++++++++++++++++++++++++++++++++++++-----------
 ui/src/capture.rs |   4 +-
 ui/src/main.rs    |  66 ++++++++++++++++++++-
 3 files changed, 200 insertions(+), 37 deletions(-)
```

## Command Output
```
    Finished `release` profile [optimized] target(s) in 23.95s
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 15 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.94s
```

## Capture
The capture shows turn `005` playing. 
At an offset of 5000 ms from the start of playback, turn 004 (which only lasts 0.89 seconds) has already completed, and playback has gaplessly advanced to turn 005.

## Verification
All required changes have been successfully implemented:
- `TurnQueue` handles gapless offset calculations and test coverage passes.
- `AudioOut` supports `queue_next` and maintains accurate state tracking across segments.
- `main.rs` properly sets up auto-advancement for gapless audio with no silence intervals, updating selection while preserving audio streams.
- The `gapless` setting was successfully added to preferences and the settings UI.
