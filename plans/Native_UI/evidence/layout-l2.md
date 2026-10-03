# L2: Settings window (⌘,) — evidence

Date: 2026-10-03  
Branch: `ui/layout-settings`  
Episode: `/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels` (read-only)

## git diff --stat

Tracked modifications plus new `ui/src/settings.rs` (untracked; 213 lines):

```
 ui/src/capture.rs   |  10 ++++
 ui/src/inspector.rs |   2 +-
 ui/src/jobs.rs      |   6 ++-
 ui/src/main.rs      |  97 +++++++++++++---------------------
 ui/src/settings.rs  | 213 ++++++++++++++++++++++++++++++++++++++++++++++++++++ (new)
 ui/src/table.rs     |   2 +-
 ui/src/theme.rs     | 146 +++++++++++++++++++++++++++++++++++++++++++++++++---
 ui/src/timeline.rs  |  14 +++--
 ui/src/view.rs      |   6 ++-
 9 files changed, 418 insertions(+), 78 deletions(-)
```

(`git diff --stat` alone omits the untracked `settings.rs`; line count from `wc -l`.)

## Command output

```
$ cd ui && cargo fmt && cargo build --release 2>&1 | tail -2
   Compiling scriptcast-ui v0.1.0 (/Users/emmanuel/Dev/Tools/scriptCast-worktrees/layout/ui)
    Finished `release` profile [optimized] target(s) in 27.88s

$ cargo test 2>&1 | grep "test result"
test result: ok. 16 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 15 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo clippy --all-targets -- -D warnings 2>&1 | tail -1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s

$ awk 'length>120{print FILENAME":"FNR}' src/*.rs
(no output)

$ grep -rn "t::ACCENT\b\|[^_]ACCENT[^_]" src/*.rs | grep -v "theme.rs"
(no output)

$ grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs
(no output)
```

## Capture (failed)

```
$ ./target/release/scriptcast-ui --mute --settings --select 6 \
    --screenshot /tmp/scriptcast-layout/l2-settings.png \
    /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
scriptcast-ui: Login shell PATH loaded
scriptcast-ui command: scriptcast · Project virtualenv: …/.venv-voice/bin/scriptcast
scriptcast-ui command: scriptcast-export · Dev checkout: …/.venv/bin/python (…/layout; -m scriptcast.tools.export_session)
scriptcast-ui command: scriptcast-repair · Project virtualenv: …/.venv-voice/bin/scriptcast-repair
scriptcast-ui command: scriptcast-verify · Project virtualenv: …/.venv-voice/bin/scriptcast-verify
scriptcast-ui: timed out after 20 s while waiting for the screenshot event
```

No PNG was written. The settle phase completed (timeout message is `Phase::Requested`), but eframe never delivered `Event::Screenshot`. The same timeout occurs for a clean rebuild of L1 commit `59b7384` in this environment; `screencapture -x` also fails with `could not create image from display`. Earlier L1 PNGs under `/tmp/scriptcast-layout/l1-*.png` (18:34–18:35) remain from a prior session that still had display capture.

## What the PNG shows

Unavailable — capture failed as above. Re-run the screenshot command from a GUI session that can receive egui screenshot events (or has Screen Recording permission for `screencapture`) to produce `/tmp/scriptcast-layout/l2-settings.png`.

## Implementation summary (for re-capture)

- Gear and ⌘, open a centred Settings window (`ui/src/settings.rs`): Appearance (accent + four speaker colours + text size 85–130% + reset), Playback (auto-advance + gapless), Commands (folder Choose/Clear + resolved command list).
- Runtime `Appearance` in `theme.rs` with `accent()` / `speaker_colour` / scaled fonts; persisted on `Preferences.appearance`.
- Timeline lanes grow up to `2 × TIMELINE_LANE_MAX`; track column background fills to the timeline bottom.
- Hidden `--settings` opens the window for capture.
