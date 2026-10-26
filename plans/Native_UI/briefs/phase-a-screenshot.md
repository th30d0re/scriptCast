# Phase A follow-up: self-screenshot flags for visual verification

Same worktree, same rules as `plans/Native_UI/briefs/phase-a.md`: **no git writes**, write only inside this worktree (plus `~/.cargo` and `ui/target/`), never write to `/Users/emmanuel/Documents/Theory/TheOriginalPower/`, no Python changes, and do not touch `ui/design/**`, `ui/assets/**` or `plans/` except the findings file.

## Goal

The orchestrator must compare the running app with the design screenshots, but macOS screen capture is not available to it (no Screen Recording permission, and UI control was declined). Add a way for the app to capture itself through egui, so verification needs no OS permissions.

## What to add (in `ui/src/main.rs` or a new `ui/src/capture.rs`)

1. `--screenshot <path.png>`: start normally (an optional positional episode dir still works), wait until any episode load has finished, at least 3 frames have painted and no repaint is pending, then request a screenshot with egui's viewport screenshot command. Find the exact 0.36 API in `~/.cargo/registry/src/*/egui-0.36*` (`ViewportCommand::Screenshot` and the matching `Event::Screenshot` carrying a `ColorImage`). Write the image as PNG at full pixel density, print the path and pixel size to stdout, and close the app with exit code 0. On any failure, print the error to stderr and exit non-zero; never hang. A 20 s timeout counts as a failure.
2. `--size <W>x<H>`: the inner window size in points. Default 1440x900, the design size.
3. `--fresh`: ignore persisted state (no remembered episode, no recent episodes) and do not save state on exit, so the empty workspace (screen 01) is reproducible. `--screenshot` implies `--fresh` unless an episode dir is given; in that case it still never writes persisted state.
4. `--select <turn_index>`: after loading, select that manifest turn (scrolled into view) before capturing. This is for future screen 03/04 checks.
5. Nothing changes when none of these flags is given.

Use the `image` crate (PNG feature only) or `png` for encoding, whichever is lighter given the current lockfile. Add the dependency to `ui/Cargo.toml`.

## Acceptance

```bash
cd ui
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
./target/release/scriptcast-ui --screenshot /tmp/sc-empty.png
./target/release/scriptcast-ui --screenshot /tmp/sc-ep03.png /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
./target/release/scriptcast-ui --screenshot /tmp/sc-ep03-sel.png --select 3 /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
```

Each screenshot command must exit 0 and write a non-empty PNG of about 2880×1800 px on a Retina display. Report the sizes. Running the GUI binary for these captures is allowed; do not try to judge how it looks (the orchestrator does that). Add unit tests for argument parsing.

## Findings

Append a section "Screenshot flags" to `plans/Native_UI/evidence/phaseA-findings.md`: the API you used, the frame and settle logic, the measured PNG sizes, and any limitation.
