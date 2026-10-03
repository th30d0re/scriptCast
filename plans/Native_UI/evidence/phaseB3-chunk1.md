# Phase B3 - Chunk 1 Evidence

## git diff --stat

```
 ui/src/table.rs    | 10 +---------
 ui/src/timeline.rs | 10 +---------
 ui/src/view.rs     |  2 +-
 3 files changed, 3 insertions(+), 19 deletions(-)
```

## Command Output

```
$ cd ui && cargo build --release 2>&1 | tail -2 && cargo test 2>&1 | grep "test result" && cargo clippy --all-targets -- -D warnings 2>&1 | tail -1
    Finished `release` profile [optimized] target(s) in 31.82s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 15 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.40s
```

```
$ ./target/release/scriptcast-ui --mute --select 12 --screenshot /tmp/scriptcast-b3/chunk1.png /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
scriptcast-ui: Login shell PATH loaded
scriptcast-ui command: scriptcast · Project virtualenv: /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/scriptcast
scriptcast-ui command: scriptcast-export · Dev checkout: /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv/bin/python (/Users/emmanuel/Dev/Tools/scriptCast-worktrees/phase-b3; -m scriptcast.tools.export_session)
scriptcast-ui command: scriptcast-repair · Project virtualenv: /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/scriptcast-repair
scriptcast-ui command: scriptcast-verify · Project virtualenv: /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/scriptcast-verify
screenshot: /tmp/scriptcast-b3/chunk1.png 2996x1800
```

## Screenshot verification

Checking `/tmp/scriptcast-b3/chunk1.png`:
- The coloured bar has been removed in the table's Speaker column.
- The coloured bar has been removed in the timeline's speaker lanes.
- Speaker names are still rendered in their respective speaker colours.
- Speaker names are left-aligned and start where the bar used to start (no leftover indent).
