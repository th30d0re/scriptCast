# L1: resizable panes

Working directory: this git worktree (branch `ui/layout-settings`).

## Rules

- **No git commands that write.** Change only `ui/src/main.rs`, `ui/src/theme.rs`, `ui/src/view.rs` and `ui/src/inspector.rs`.
- **Never start other agent CLIs** (`codex`, `kimi`, `agy`, `gemini`, `cursor-agent`, `claude`). Do the work yourself.
- Put scratch files under `/tmp/scriptcast-layout/`. Never put them in the worktree.
- Lines under 120 characters, written as normal readable Rust like the surrounding code. Run `cargo fmt`. **Do not delete or reword existing comments** unless you remove the code they describe.
- No colour or size literals outside `ui/src/theme.rs`. Add tokens there, named for what they are.
- Mute every playback you start. The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only.

## Operator request (2026-10-03)

> "I want the windows to be resizable … like if I want to expand the top part so I can see more, I want to be able to … adjust and play with that."

## Current layout (`main.rs`, around `let timeline = Rect::from_min_size(workspace.min, vec2(workspace.width(), t::TIMELINE_HEIGHT));`)

- The workspace is split top to bottom into:
  - the timeline, at a fixed `t::TIMELINE_HEIGHT`;
  - the filter bar, at a fixed `t::FILTER_BAR_HEIGHT`;
  - the review row.
- In the review row, the inspector takes a fixed width at the right: `InspectorMode::width()` in `inspector.rs`, which is 376, 436 or 510 depending on mode, capped at half the row.

## Build

1. **Two draggable splitters:**
   - **Horizontal** (between the timeline and the filter bar): drag up or down to change the timeline's height.
   - **Vertical** (between the table and the inspector): drag left or right to change the inspector's width.

   For each:
   - a hit area `t::SPLITTER_HIT` (new token, 6) thick, centred on the boundary, with `Sense::drag()` and the matching resize cursor (`CursorIcon::ResizeVertical` or `ResizeHorizontal`) on hover and while dragging;
   - a 1 px line in `t::BORDER_DEFAULT`, turning `t::ACCENT` while hovered or dragged;
   - double-click resets that splitter to its default.
2. **State.** Add `timeline_height: Option<f32>` and `inspector_width: Option<f32>` to the persisted `Preferences`, with `#[serde(default)]`; `None` means the default.
   - The timeline height is clamped between `t::TIMELINE_MIN` (new token, 140) and 70% of the workspace height.
   - The inspector width is clamped between `t::INSPECTOR_MIN` (new token, 320) and 60% of the review row's width.
   - One width applies to every inspector mode once the operator has set it. Until then, each mode's own default width is used, as today.
3. **Timeline at a new height.** The timeline must lay out correctly at any height. Look at `timeline::show` and the constants it uses: the extra height must go to the speaker lanes area, spread evenly across lanes up to a sensible maximum lane height (add a token), without overlapping anything. If `timeline.rs` needs changes for that, you may edit it too.
4. **Repaints.** Dragging must repaint smoothly while it happens, and there must be no continuous repaint when idle.

## Check yourself, then report

```bash
cd ui && cargo fmt && cargo build --release 2>&1 | tail -2 && cargo test 2>&1 | grep "test result" && cargo clippy --all-targets -- -D warnings 2>&1 | tail -1
awk 'length>120{print FILENAME":"FNR}' src/*.rs
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs   # must print nothing
git diff --stat
```

Add a hidden CLI flag in `capture.rs` that sets both sizes for a capture, for example `--layout 420x480` meaning timeline height × inspector width; you may edit `capture.rs` for this. Then capture EP03, muted, twice:

- default: `--select 6 --screenshot /tmp/scriptcast-layout/l1-default.png`;
- resized: `--layout 420x480 --select 6 --screenshot /tmp/scriptcast-layout/l1-resized.png`.

Look at both PNGs and say plainly what changed.

Write `plans/Native_UI/evidence/layout-l1.md` with:

- the diff stat;
- all command output;
- what the PNGs show.

If anything fails, say so plainly.
