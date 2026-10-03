# Native UI Implementation State

## Design snapshot

- Captured 2026-10-02 from Figma file `8mO8UiLLEMNEutFSkjalPD` into `ui/design/` (screens 01–06, Workspace state sheet, flow notes, tokens, 21 icons). Agent screens 07–08 excluded by the operator.
- Figma MCP calls used this month: 15 (5 on 2026-10-03 for the inline-edit A/B; the operator upgraded the plan) (1 Workspace state, 2 metadata, 7 frames). Loops make none.

## Operator decisions

| # | Decision | Status | Choice |
|---|---|---|---|
| 1 | Checkpoint before audio replacement | Decided 2026-10-02 | In scope: `checkpoint.rs` clones affected files into `<episode>/_backups/ui/`, with Restore (plan steps 12, 14, 16) |
| 2 | Where the Job runner opens | No preference 2026-10-02 | Plan default: "Jobs" button in the filter bar + ⌘J; revisit after Phase C |
| 3 | Fonts | Decided 2026-10-02 | OpenDyslexic Regular/Bold + OpenDyslexicMono instead of Inter / IBM Plex Mono; copied from the operator's files into `ui/assets/fonts/` |
| 4 | Inline editing location | Decided 2026-10-03 after a Figma A/B | Both, toggled in Settings; default Side panel (Alternative B). Table row (Alternative A) kept as an option |
| 5 | Python read-only modes for editing (`scriptcast-export --turns-json/--changes-json`) | Approved 2026-10-03 | Needed by Phase D so the UI never re-implements the markdown parser |

## Phase A — Episode Browser

- Last run: 2026-10-03 ~00:30 local
- Status: **passed** — merged to `dev` at `8da185f` (fast-forward from `ui/phase-a`)
- Implementer: Codex (gpt-6-astra, high) for steps 1–8; Kimi for the self-capture flags and visual round 1 (Codex usage limit hit 2026-10-02 23:40, resets 01:41)
- Verifier: orchestrator. Live UI control of the app was declined by the operator, so visuals were checked with the app's own `--screenshot` capture.
- Visual rounds: 1 (seven text-fitting and layout fixes, all resolved)
- Evidence: `evidence/phaseA-01.png`, `evidence/phaseA-02-select4.png`, `evidence/phaseA-findings.md`

### Checklist

| # | Check | Result | Notes |
|---|---|---|---|
| 1 | Files from plan.md steps 1-8 exist; `ui/target/` ignored; OpenDyslexic fonts embedded and rendering | ✅ | FONT_SCALE: 1.0. CFF OTFs render in epaint 0.36.2; font test checks glyph atlas UVs differ from the replacement glyph |
| 2 | cargo build --release / test / clippy -D warnings green | ✅ | 40 tests + 1 ignored (EP03) |
| 3 | Loader tests cover all plan.md step 6 cases | ✅ | plus malformed CSV, cancellation, stale stress IDs, nullable `repetition` |
| 4 | No hex colour literals outside theme.rs | ✅ | grep empty |
| 5 | EP03: real header, 4 real speaker lanes, timeline end matches manifest, Not verified vs Failed correct | ✅ | 13 Passed / 0 Failed / 379 Not verified; 5 stress.json turn_ids are stale (report predates the 2026-09-18 re-render) and surface as a header warning |
| 6 | Search, speaker filter, Failed only, segment click-to-select work | ⚠️ | Selection verified via `--select`; search/filter logic unit-tested (`filters_intersect_and_never_manufacture_verification`); not live-clicked (no UI control) |
| 7 | Error state on a bad directory; current episode stays open | ⚠️ | Unit-tested (`failed_open_keeps_episode_and_recent_history`); not captured |
| 8 | Design fidelity vs screens 01, 02 | ✅ | After round 1. Intentional differences: OpenDyslexic (wider), real data, 0-based turn numbers (the CLI index), "Cmd+O"/"Cmd+F" text (font lacks ⌘), native title bar |
| 9 | EP03 load < 50 ms; smooth scroll; idle CPU ~0% | ✅ | 9.6 ms; idle 0.0% CPU, ~117 MB RSS; scroll smoothness not live-tested |
| 10 | No Python changes; EP03 directory untouched; no Figma calls | ✅ | EP03 hashes unchanged |

## Phase B — Playback

- Last run: 2026-10-03 ~11:40 local
- Status: **passed** — merged to `dev` at `6259d3e`. Operator listened on EP03 on 2026-10-03: "It's sounding really good."
- Implementer: Kimi (09:54–11:14; its weekly quota ended while finalising findings, after acceptance had passed). Orchestrator completed the findings and fixed the state-card copy.
- Evidence: `evidence/phaseB-playing.png`, `phaseB-sel.png`, `phaseB-device-lost.png`, `phaseB-missing-audio.png`, `evidence/phaseB-findings.md`
- API discoveries: rodio 0.22's `speakers` module requires the `experimental` feature (devices via cpal + `DeviceSinkBuilder::from_device` instead); `Player::get_pos()` resets per appended segment (position is wall-clock); `try_seek` only seeks within the current segment.

### Checklist

| # | Check | Result | Notes |
|---|---|---|---|
| 1 | audio.rs exists; build/test/clippy green | ✅ | 49 tests + 2 ignored |
| 2 | Space play/pause, Esc stop (by ear) | ✅ | operator, 2026-10-03 |
| 3 | Switching turns while playing: no overlap (by ear) | ✅ | operator, 2026-10-03 |
| 4 | Fixture two-segment turn plays both chunks in order; time and seek advance | ✅ | muted ignored test passes (400 ms); EP03 capture shows 00:04.829 / 00:08.359 and progress |
| 5 | Auto-advance follows the active filter | ⚠️ | unit-tested (next visible turn, end of list, skips missing audio); by-ear pending |
| 6 | Space in the search box does not play | ⚠️ | shortcut suppression unit-tested; not live-typed |
| 7 | Missing WAV: Missing audio row + state, other turns still play | ✅ | capture on fixture copy |
| 8 | Device menu works; lost device pauses and shows Unavailable device | ⚠️ | simulated loss captured (paused at 00:02.05, selection kept); real unplug pending (operator) |
| 9 | Design fidelity vs screen 03 | ✅ | after removing specimen captions/notes from state cards |
| 10 | Idle CPU ~0% when not playing | ✅ | repaint only while playing (code review); Phase A idle 0.0% |

## Phase B2 — Open in DAW

- Requested 2026-10-03 by the operator: "Open in Live" and "Open in Logic". Operator approved a small Python change (`scriptcast-export`).
- Status: **passed** — merged to dev (UI part by Codex). Operator, 2026-10-03: "all the buttons work." Header shows Open / Reload / Open in Live / Open in Logic / a settings gear (the B2 "CLI…" button, renamed after the operator found the label confusing).
- Follow-ups: the render path's own `--fcpxml` still declares stereo (it does not set `audio_channels`); `--logic` AppleScript exporter is non-functional on Logic 12.4 — candidates for a later cleanup.

### Checklist

| # | Check | Result | Notes |
|---|---|---|---|
| 1 | `scriptcast-export --fcpxml` writes 392 assets for EP03 to `--out`, mono channels, EP03 untouched | ✅ | max offset error 16.67 ms (accepted 30 fps rounding, operator-visible in --help) |
| 2 | Python tests for export + generator green; full suite run reported | ✅ | 28 targeted; full 213 passed / 11 failed = dev's pre-existing pronunciation failures |
| 3 | Open in Live opens EP03's `.als` in Live 12 (operator) | ✅ | operator, 2026-10-03 |
| 4 | Open in Logic imports the FCPXML with clips at the right times (operator) | ✅ | operator, 2026-10-03 ("all the buttons work"); exact clip timing not separately reported |
| 5 | Buttons disabled with tooltip when the `.als` is missing or the CLI is unavailable | ✅ | unit-tested disabled reasons; export source (Folder/PATH/DevCheckout) shown in tooltip |

## Phase C — Job Runner and Verify Overlay

- Last run: —
- Status: **ready** (Phases A, B, B2 passed)
- Checkpoint timing (Precision-insert scope, EP03 clone): —

### Checklist

| # | Check | Result | Notes |
|---|---|---|---|
| 1 | Files from plan.md steps 11-17 exist; build/test/clippy green | ⬜ | |
| 2 | Preset and checkpoint tests pass; no `--overwrite`; no delete calls in ui/src | ⬜ | |
| 3 | All jobs spawn with stdin null | ⬜ | |
| 4 | Detect changes streams and exits; no confirmation, no checkpoint | ⬜ | |
| 5 | Verify: confirmation with exact argv + checkpoint note; checkpoint exists before the CLI starts; column updates | ⬜ | |
| 6 | Restore round-trip: stress.json back byte for byte; pre-restore checkpoint exists | ⬜ | |
| 7 | Checkpoint failure blocks the job and names the path | ⬜ | |
| 8 | Cancel kills the job within 1 s; state Canceled | ⬜ | |
| 9 | Edited script disables Regenerate selected (scratch copy only) | ⬜ | |
| 10 | Wrong bin dir: CLI unavailable state; browsing and playback still work | ⬜ | |
| 11 | Verification inspector shows real score, expected, heard, stress[] | ⬜ | |
| 12 | Design fidelity vs screens 04, 05, 06 | ⬜ | |
| 13 | SPEC.md updated (presets, mix deferred, checkpoints, font, design link) | ⬜ | |
| 14 | No Python changes; real EP03 directory unchanged (shasum of stress.json before/after) | ⬜ | |

## Blockers

- None recorded. Phase A is ready to start.

## Phase D — Inline Editing

- Requested 2026-10-03 by the operator ("the ability to edit the words and re-render"). Plan: `plan.md` Flow 6 and steps D1–D6.
- Status: **planned; starts after Phase C** (needs its runner, confirmation dialog and checkpoints). D1 approved. Figma captured: `screens/03b-alt-a-inline-row-edit/`, `screens/03c-alt-b-side-panel-edit/` (Figma calls this month: 15).

## Phase E — Read-along word highlighting

- Requested 2026-10-03 (shown in Figma Alternative B). Planned after D; needs per-word timings from Python (captions.py alignment).
