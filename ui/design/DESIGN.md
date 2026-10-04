# scriptCast UI — Design Snapshot

This folder is a local, offline copy of the Figma design for `scriptcast-ui`. Build from it. **Never call the Figma MCP from an implementer or verifier loop**: the account is on Figma's Starter plan (about 20 MCP calls a month), and everything needed is already here.

- Figma file: `8mO8UiLLEMNEutFSkjalPD` ("Untitled"), Page 1. Captured 2026-10-02.
- Target: native macOS window, dark only, designed at 1440×900.
- **Font override:** the app uses **OpenDyslexic** (Regular, Bold) and **OpenDyslexicMono** from `ui/assets/fonts/`, not the design's Inter and IBM Plex Mono (operator decision, 2026-10-02). Keep every size, weight role, colour and layout dimension from the design; only the typeface changes. OpenDyslexic runs about 45% wider with a taller line box, so expect glyph shapes and text widths to differ from the screenshots; fixed-width columns ellipsize, content-sized controls grow, and a single `FONT_SCALE` in `theme.rs` may shrink all text if heights clip. Weight 500 (Medium) maps to Regular, 600 to Bold.
- Scope captured: core screens 01–06, the workshop flow notes, and the Workspace state component sheet. Screens 07–08 and the agent components (agent editing, proposals, agent offline) were deliberately **not** captured; they belong to the future agent phase (plan.md Flow 0) and are out of scope for Phases A–C.

## How to use these files

| File | What it is | How to use it |
|---|---|---|
| `screens/NN-*/screenshot.png` | Figma render, 1024×640 (the frame is 1440×900; scale 0.711) | The visual target. Compare against the running app. Never ship it as an asset. |
| `screens/NN-*/design-context.tsx` | Figma's React+Tailwind rendering of the frame | A precise spec, not code to port. Read sizes, spacing, colours, type and copy from the class names (`h-[28px]`, `text-[#a4abb6]`, `gap-[12px]`). `data-node-id` attributes map back to Figma nodes. |
| `tokens.json` | Every colour, type role, radius and fixed size used across the captures, with semantic names | The single source for `ui/src/theme.rs`. Do not hard-code hex values elsewhere. |
| `assets/icons/*.svg` + `index.json` | The 21 Lucide icons used, already sized and coloured per use | Load with `egui::include_image!` (needs `egui_extras` with the `svg` feature). `index.json` maps each file to the Figma asset hash used in the `.tsx` files. |
| `components/workspace-state/` | Component sheet for full-panel states (Empty, Loading, Error, Missing audio, Unavailable device, CLI unavailable, Agent offline) | Build the first six. Skip Agent offline. |
| `flow-notes/` | The designer's flow notes and principles | Read once. The LISTEN, REPAIR and RECOVER flows are in scope; EDIT and RESOLVE are future. |
| `page-metadata.xml` | Node tree of the whole Figma page (ids, names, positions, sizes; no styling) | Use to see the structure of component sheets that were not captured. |

## Screens and where they land in the plan

Plan step numbers refer to `plans/Native_UI/plan.md` Section 4.

| Screen | Figma node | Shows | Phase / step |
|---|---|---|---|
| 01 · No episode open | 4:17810 | Empty workspace: hero, Open Episode, expected directory contents, Recent episodes, disabled playback bar | A / 8 |
| 02 · Episode browser — idle | 4:17895 | Header, timeline (overview + speaker lanes + ruler), filter bar, full-width turn table, disabled transport | A / 8 |
| 03 · Selected turn — playing | 4:18121 | Playhead, playing row (▶), Selected turn inspector (376 px), Pause/Stop, seek progress, current time | B / 10 |
| 04 · Verification review | 4:18375 | Failed-only filter active, failed markers on overview, Verification inspector (436 px): score, expected/heard, stress verdicts, Regenerate/Repair/Verify again | C / 14 |
| 05 · Job runner — active | 4:18609 | Job runner inspector (510 px): five presets, active job, progress, streaming log, Cancel | C / 14 |
| 06 · Confirm audio replacement | 4:18874 | Modal scrim + 540 px confirmation dialog: scope, warning, command disclosure, Cancel / Confirm | C / 14 |
| Workspace state sheet | 4:20383 | Empty, Loading, Error (A / 8); Missing audio, Unavailable device (B / 10); CLI unavailable (C / 14) | A–C |

## Window layout (top to bottom)

| Region | Height | Notes |
|---|---|---|
| Title bar | 38 | Drawn by macOS. Use native decorations and set the window title to `scriptCast — <episode_id>` (`scriptCast — Episode workspace` when none). Do not draw fake traffic lights. |
| Episode header | 76 | `bg/surface`, bottom border. Brand (audio-lines icon 22 + "scriptCast" 18/600), 1×30 divider, episode name 15/600 over a metadata line 11/400 muted, then Open and Reload buttons (Reload disabled at 45% when no episode). |
| Review workspace | fills | Timeline (188), filter bar (42), then a row: turn table (fills) + optional right inspector. |
| Playback bar | 64 | `bg/raised`, top border. Always present; disabled (45%) until a turn is selected and audio is available. |
| Status bar | 26 | `bg/window`, caption 10 px. Left: state sentence (muted; accent while a job runs). Right: "Local workspace" (subtle). |
| Workshop annotation | 56 | **Figma annotation only. Do not build.** |

## Components

**Buttons.** Height 30, radius 5, 1 px `border/default`, padding x 11, Semi Bold 12, optional 16 px icon with 7 gap. Primary: `accent` fill, `on-accent` text. Secondary: `bg/control` fill, `text/primary`. Disabled: 45% opacity. Toggle-on (Failed only active): `bg/danger` fill, `status/failed` border and text, label prefixed "✓ ".

**Timeline (188 high, padding 20×10, gap 6).**
- Heading row: "Timeline" 12/600 + "<total mm:ss.mmm> · <n> turns" 11 muted; right side "−  Zoom  +" and "Fit episode" (11 muted, clickable).
- Overview strip (17 high): the whole episode split into equal sections with 2 px gaps, each coloured by the speaker who talks most in that slice; sections outside the visible range at 48% opacity. A 21-high `accent` outline box marks the visible range. Failed turns draw a full-height `status/failed` tick at their position.
- Ruler (18 high, left pad 80): mono 10 subtle ticks at a readable interval for the zoom (15 s in the design).
- Speaker lanes: a 66 px label column ("▎Name", 12/500, speaker colour) and one 20 px lane per speaker (`bg/surface`, 4 gap). Each turn is a segment block positioned by time: `bg/segment` fill, 1 px speaker-colour border, radius 2, padding x 4, turn number in mono 9 speaker colour. Selected: `bg/selected` fill, 2 px `accent` border. Failed and selected: `bg/danger` fill, 2 px `accent` border, number in `status/failed`. Playhead: 1 px `text/primary` vertical line across all lanes.

**Filter bar (42 high, `bg/raised`, padding x 16, gap 10).** Search field 270×27 (`bg/window`, search icon 14, placeholder "Search turns", "⌘F" hint), "All speakers ⌄" dropdown button, "Failed only · <n>" toggle, right-aligned "Turn view" + list icon (static label in v1).

**Turn table.** Header row 30 high (`bg/window`, 11/600 muted). Rows 28 high, 1 px bottom border, padding x 16, column gap 12. Columns: Turn 44 (mono 11, zero-padded to 3), Time 92 (mono 11, `mm:ss.mmm`), Speaker 82 ("▎Name" 12/500 colour), First words (fills, 12 primary, ellipsis), Duration 60 (mono 11, "8.24 s"), Verification 106 (11). Row states: default `bg/surface`; failed `bg/danger-row`; selected `bg/selected` with turn number in `accent`; playing = selected plus "▶" before the number. Footer: "Showing a–b of n turns" left, "↑ ↓  Select turn" right (11 subtle); in Failed-only mode "<f> failures · <p> passed · <m> missing audio" and "Last verified HH:MM" (mtime of `stress.json`).

**Verification column values.** "✓  Passed" (`status/passed`), "×  Failed" (`status/failed`), "Not verified" (`text/subtle`) for turns with no `stress.json` entry, "Missing audio" (`status/failed`) when a WAV is absent. The last two states exist in Figma component sheets that were not captured; use these colours.

**Inspector (right panel, left border, `bg/surface`).** Heading 39 high (`bg/raised`, 12/600 title, 10 muted right label), body padding 16 (12 for the job runner). Three modes:
- *Selected turn* (376, screen 03): speaker + verdict row; TRANSCRIPT, AUDIO (`start → end · seconds`), FILE (`<wav name> · mono|stereo · <kHz>`) fields with 10/600 uppercase muted labels; accent status sentence; Previous / Next / Review failure buttons; shortcut list (Space, ↑ ↓, Esc).
- *Verification* (436, screen 04): verdict + "Score <score> / <threshold>" (mono 12); EXPECTED TEXT; HEARD TEXT (in `status/failed` when failed); STRESS CHECKS list; Regenerate selected (primary) + Repair; Verify again + "Replaces verification output only"; footer caution.
- *Job runner* (510, screen 05): heading "Job runner" + "Local CLI · available|unavailable"; wrapping row of the five presets; active job title + subtitle + state ("● Running" accent); 3 px progress bar; log well (`bg/window`, border, radius 5, mono 11 lines, header "OUTPUT · auto-scroll on", spinner line while running); footer "<mm:ss> elapsed · reload when complete" + Cancel.

**Playback bar.** Play/Pause primary (play or pause icon) + Stop secondary; current-audio block 226 wide ("Turn 004 · Jordan" 12/500, "00:04.620 / 00:09.060" mono 11); seek track (fills, 4 high, radius 2, `bg/control`, `accent` elapsed fill); "☑ Auto-advance" checkbox; "Space  Play / pause" hint; output device menu "<device> ⌄".

**Confirmation dialog (screen 06).** Scrim `bg/scrim` over the workspace; dialog 540 wide, `bg/raised`, 1 px border, radius 12, padding 24, gap 18, `shadow.dialog`. Shield-check icon 26 + title 20/600; 13 muted description; scope box (`bg/window`, radius 5, padding 14) with EPISODE and AFFECTED TURNS fields; `status/failed` warning row with triangle-alert 16; command disclosure box (`bg/surface`, border, radius 5, padding 12) with the command in mono 11; actions right-aligned: Cancel (secondary), "Confirm · <preset>" (primary).

**Workspace states (component sheet).** Card 680 wide, `bg/surface`, border, radius 8, padding 16. Icon 22 + title 18/600, body 13 muted, recovery button(s). Loading adds a 3 px progress bar and Cancel. **Do not render** the sheet's caption line ("Workspace state / …", 13/600) or its 11 px footnote: both are designer annotations on the specimen, not app copy.

**Empty workspace (screen 01).** Two columns centred, gap 64, 1×350 divider. Left (410): folder-open 42, "Your episode starts here." 30/600, 14 muted body, Open Episode (primary, folder-open 16 on-accent) + "⌘O", EXPECTED DIRECTORY CONTENTS list in mono 12. Right (386): "Recent episodes" + "Open directory ↗"; rows with name 14/500, "<when> · <n> turns · <duration>" 12 muted, path mono 10 subtle; footer "● Local files  Nothing uploaded. No connection required."

## Open in Live / Open in Logic (operator request, not in Figma)

Two secondary buttons in the episode header, after Reload, in the same style (height 30, radius 5, Semi Bold 12, no icon): "Open in Live" and "Open in Logic". Disabled at 45% with a tooltip naming the reason (no `.als` in the episode; Logic export command unavailable). While Logic's FCPXML is being generated, the button label reads "Preparing for Logic…" and the status bar shows progress; errors appear in the status bar in `status/failed`.

## Keyboard

⌘O open episode · ⌘J jobs · ⌘F focus search · ↑ ↓ select previous/next visible turn · Space play/pause · Esc stop playback (and close a dialog) · ⌘R reload. Shortcuts are ignored while a text field has focus.

## The design shows sample data; the app must show real data

- Speaker names (Alex, Maya, Jordan, Sam), "Kokoro v1.0", turn text, scores, "385 passed · 7 failed", file names like `turn_004.wav`, and dates are **illustrative**. Use the manifest, `render_state.json`, `TURN_INDEX.csv` and `stress.json`. The header metadata line becomes `<engine> / <sample rate kHz> / <n> turns / <n> speakers / Rendered <rendered_at>`.
- **Never manufacture a verification result.** `stress.json` often covers only some turns (the real EP03 file has 18 entries for 392 turns). Turns without an entry are "Not verified", not "Passed". Failed-only counts come from real entries.
- The design's STRESS CHECKS rows ("Text alignment", "Emphasis / stress", "Clipping / silence") are concepts. Render what `stress.json` actually contains: the score, the `repetition` flag if set, and each `stress[]` entry (word, reading, verdict).
- The design's command disclosure says "illustrative only". The app must show the **exact argv** it will run (plan step 11, `JobSpec::display`), and drop the "illustrative" caption.
- Job progress: the CLIs do not report a completion fraction. Show an indeterminate (animated) bar while running, never a fake percentage.
- "A checkpoint is saved first" / "checkpoint v03 saved": **implemented by the app** (operator decision). Before any job that synthesizes audio or overwrites `stress.json`, `checkpoint.rs` clones the files that job can change into `<episode>/_backups/ui/vNN-<time>-<preset>/`; the job starts only if that succeeds. Show the real version number. Checkpoints are reversible through Restore in the Job runner inspector (plan Flow 3b).

## Not captured (consult `page-metadata.xml`; capture later only if needed)

Component sheets for Timeline block, Turn row, Speaker badge, Playback, Verification, Job status, Agent proposal and Confirmation. Their key states already appear in screens 02–06. States not seen in any capture: Turn row and Verification "Missing audio" / "Not verified", Playback "Missing audio" / "Unavailable device", Job status Idle / Complete / Failed / Canceled, Confirmation "Replace verification" (the Verify confirmation). Build these from the tokens and the patterns above. If one needs exact Figma values, the operator can approve one `get_design_context` call per sheet (node ids in `page-metadata.xml`).

## Operator decisions (2026-10-02)

1. **Checkpoints: in scope.** The app saves a checkpoint before every audio- or verification-replacing job and offers Restore (see the sample-data section above and plan steps 12, 14, 16). The Job runner inspector gains a **Checkpoints** section below the log: one row per checkpoint (version, time, preset, turns, files, size) with a secondary "Restore" button, and the total size of `_backups/ui/` at the foot. Style it with the existing list-row and button patterns; Restore uses the confirmation dialog pattern with a "Restore checkpoint vNN?" title and the list of files to be replaced.
2. **Job runner entry point: plan default.** A secondary "Jobs" button at the right end of the filter bar and ⌘J. The operator had no preference; revisit after Phase C.
3. **Fonts: OpenDyslexic.** See the font override at the top.

## Fidelity check

Run the app at 1440×900 against `outputs/ATO_EP03_five_levels`, take a screenshot of the same state, and compare side by side with the matching `screenshot.png`. Layout, colours, sizes and type must match; text content will differ because the design uses sample data.

## Update 2026-10-04: screen 03 A/B behaviour (supersedes 03b/03c where they differ)

Captured from Figma frames `4:18121` (A, direct inline editing) and `23:2` (B, side-panel editing) into `screens/03e-update-2026-10-04/`.

- **Table.** The "First words" column is now **Transcript** and shows the turn's full text. It wraps to at most 2 lines, then ellipsis, so row height varies with text. The design shows 2-line rows of about 52 px.
- **A, direct inline editing.**
  - The selected row's Transcript cell itself becomes the editor, with a caret in the text.
  - Under the text, inside the same row: **Save draft** (primary, small), **Cancel**, and on the right "Unsaved draft" with the alert icon.
  - There is no separate editor block under the row.
- **Mode toggle.** A filter-bar segmented button "Inline / Side panel" sits after "Failed only", showing the active mode as a small pill ("Inline"). It is the same setting as Settings → Editing.
- **Playback bar.** The second line of the current-audio block reads "Turn 004 · Jordan · Paused" when paused.
- **B, read-along.** The current word is highlighted **inside the playing row's Transcript text**: an accent pill, radius 3, padding 4×2, semibold, in on-accent text. There is no extra read-along row under the playing row.
