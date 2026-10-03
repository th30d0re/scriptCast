# Phase A visual fixes, round 1

Same worktree and rules as `plans/Native_UI/briefs/phase-a.md`:
- **no git writes**;
- write only inside this worktree (plus `~/.cargo`, `ui/target/` and PNGs under `/tmp/`);
- never write to `/Users/emmanuel/Documents/Theory/TheOriginalPower/`;
- no Python changes;
- do not touch `ui/design/**`, `ui/assets/**` or `plans/` except the findings file.

Colours and design sizes still come only from `ui/src/theme.rs`.

## Context

The orchestrator compared the app's own captures with the design:
- empty workspace vs `ui/design/screens/01-no-episode-open/screenshot.png`;
- EP03 vs `ui/design/screens/02-episode-browser-idle/screenshot.png`.

Layout, colours, region heights, the table, the timeline lanes and selection all match. The defects below come from one cause: **OpenDyslexic is about 45% wider than the design's Inter.** This was measured: "Your episode" in Bold at 30 px is 304 px, which is what the app draws, so the font renders correctly. The fix is to size text-bearing boxes from measured text, using the design token as the **minimum**. Do not change the font, `FONT_SCALE`, region heights or row heights.

Add one helper in `theme.rs` (or a small `ui/src/measure.rs`), for example `text_width(ui, text, font_id) -> f32` using `ui.fonts(|f| f.layout_no_wrap(..)).size().x`, and use it below.

## Fixes

1. **Empty workspace (screen 01): the "Expected directory contents" list is clipped away.**
   - Cause: `main.rs` around line 588. The left column is a fixed-height `ScrollArea`, and the two-line hero title pushes the four list lines below its visible edge, so only the heading shows.
   - Fix: lay out the two-column block at its content height, centred vertically in the workspace, with no inner scroll unless the window is shorter than the content. Give the 1 px divider the height of the taller column.
   - The four mono lines must be visible at 1440×900.
2. **Speaker names are truncated** ("Tous…", "Emma…", "Lee …" in the timeline lane labels; "Toussa…" in the table).
   - Timeline label column width = max(`TIMELINE_LANE_LABEL_WIDTH`, widest "▎<display name>" in the speaker-label font), capped at 200.
   - Table Speaker column width = max(`TABLE_COL_SPEAKER`, the same measurement), capped at 200.
   - Names longer than the cap still ellipsize.
3. **The table header "Durat…" is truncated.** Duration column width = max(`TABLE_COL_DURATION`, width of the header "Duration" in the header font, widest duration value). Apply the same rule to every fixed-width column (Turn, Time, Verification): header and values must fit without ellipsis.
4. **Search field: the "Cmd+F" hint is cut to "C", and the field touches the "All speakers" button.**
   - Width = max(`SEARCH_WIDTH`, icon + gap + "Search turns" + gap + "Cmd+F" + horizontal padding).
   - Keep the design's 10 px gap (`gap-[10px]` on the filter bar) between every control in the bar.
5. **Zoom controls look like buttons.** The design (screen 02 `design-context.tsx`, "Zoom controls") shows plain text at 11 px muted: `−  Zoom  +` and `Fit episode`, 14 px apart, with no frame or fill. Render them as frameless clickable text, brightening to `TEXT_PRIMARY` on hover. Behaviour stays the same.
6. **The overview strip is a dense stripe** (about 100 sections with 2 px gaps on a 64-minute episode). The design uses 12 sections for a 34-minute episode, about 3 minutes each.
   - Use `n = clamp(round(total_minutes / 3), 12, 32)` sections, with the same dominant-speaker colouring, 48% opacity outside the visible range, and the visible-range box.
   - Failed-turn markers stay positioned by time, independent of section boundaries.
7. **The header metadata shows a raw ISO timestamp** (`Rendered 2026-09-18T14:26:45.706211Z`) in a different face from the rest of the line.
   - Format as the design does: `Rendered 18 Sep 2026, 10:26`, converted to the machine's local time zone. Use `chrono` with `clock` if it is already in the lockfile, otherwise the lightest correct option.
   - Render the whole line in the same sans 11 px muted style. If parsing fails, show the original string in the same style.
   - Apply the same date format to Recent episodes "when" values if they are absolute dates.
8. **The playback bar's current-audio text is truncated** ("Select a turn to auditi…"). Its block width = max(226, the measured width of the longer of the two lines). The seek track keeps filling the remaining space.

## Acceptance

```bash
cd ui
cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
./target/release/scriptcast-ui --screenshot /tmp/sc-v1-empty.png
./target/release/scriptcast-ui --screenshot /tmp/sc-v1-ep03.png /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
./target/release/scriptcast-ui --screenshot /tmp/sc-v1-ep03-sel.png --select 4 /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
```

- All commands exit 0.
- Add unit tests for:
  - the overview section count rule (12 at 20 minutes, 21 at 64 minutes, 32 at 3 hours);
  - the timestamp formatter (a valid ISO string, and an unparseable string passed through);
  - column-width helpers returning at least the token minimum.
- The colour grep still prints nothing:

  ```bash
  grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' ui/src | grep -v theme.rs
  ```

**Not yours to claim:** whether it now looks right. The orchestrator judges the PNGs.

## Findings

Append a section "Visual fixes round 1" to `plans/Native_UI/evidence/phaseA-findings.md`: what changed for each numbered item, any new dependency, and anything you could not do.
