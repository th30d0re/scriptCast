# L1: resizable panes — evidence

Date: 2026-10-03  
Branch: `ui/layout-settings`  
Episode: `/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels` (read-only)

## git diff --stat

```
 ui/src/capture.rs   | 34 ++++++++++++++++++++++
 ui/src/inspector.rs |  7 +++++
 ui/src/main.rs      | 66 ++++++++++++++++++++++++++++++++++++++----
 ui/src/theme.rs     | 10 +++++++
 ui/src/timeline.rs  | 11 ++++---
 ui/src/view.rs      | 83 ++++++++++++++++++++++++++++++++++++++++++++++++++++-
 6 files changed, 201 insertions(+), 10 deletions(-)
```

## Command output

```
$ cd ui && cargo fmt && cargo build --release 2>&1 | tail -2
    Finished `release` profile [optimized] target(s) in 0.33s

$ cargo test 2>&1 | grep "test result"
test result: ok. 13 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 15 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo clippy --all-targets -- -D warnings 2>&1 | tail -1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s

$ awk 'length>120{print FILENAME":"FNR}' src/*.rs
(no output)

$ grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs
(no output)
```

## Captures (muted, EP03, select 6)

```
$ ./target/release/scriptcast-ui --mute --select 6 \
    --screenshot /tmp/scriptcast-layout/l1-default.png \
    /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
scriptcast-ui: Login shell PATH loaded
scriptcast-ui command: scriptcast · Project virtualenv: …/.venv-voice/bin/scriptcast
scriptcast-ui command: scriptcast-export · Dev checkout: …/.venv/bin/python (…/layout; -m scriptcast.tools.export_session)
scriptcast-ui command: scriptcast-repair · Project virtualenv: …/.venv-voice/bin/scriptcast-repair
scriptcast-ui command: scriptcast-verify · Project virtualenv: …/.venv-voice/bin/scriptcast-verify
screenshot: /tmp/scriptcast-layout/l1-default.png 2996x1800

$ ./target/release/scriptcast-ui --mute --layout 420x480 --select 6 \
    --screenshot /tmp/scriptcast-layout/l1-resized.png \
    /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
scriptcast-ui: Login shell PATH loaded
… (same command discovery lines) …
screenshot: /tmp/scriptcast-layout/l1-resized.png 2996x1800
```

## What the PNGs show

Both shots are the same EP03 workspace with turn 006 selected (Selected turn inspector).

**`l1-default.png`** — default layout: timeline at `TIMELINE_HEIGHT` (188), inspector at Selected-turn default (376). Speaker lanes are compact; the turns table takes most of the review row.

**`l1-resized.png`** — `--layout 420x480`: timeline is clearly taller (filter bar sits lower; speaker lanes are taller / more spacious, up to `TIMELINE_LANE_MAX`). Inspector is wider (~480 logical px; table columns compress). Approximate physical-pixel measure at 2×: inspector left edge moves from ~2244 → ~2034 (width ~752 → ~962 ≈ 376 → 481 points).

Nothing failed: build, tests, clippy (`-D warnings`), line-length, colour-literal checks, and both captures succeeded.
