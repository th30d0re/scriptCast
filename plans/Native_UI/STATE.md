# Native UI Implementation State

## Design snapshot

- Captured 2026-10-02 from Figma file `8mO8UiLLEMNEutFSkjalPD` into `ui/design/` (screens 01–06, Workspace state sheet, flow notes, tokens, 21 icons). Agent screens 07–08 excluded by the operator.
- Figma MCP calls used this month: 10 of ~20 (1 Workspace state, 2 metadata, 7 frames). Loops make none.

## Operator decisions

| # | Decision | Status | Choice |
|---|---|---|---|
| 1 | Checkpoint before audio replacement | Decided 2026-10-02 | In scope: `checkpoint.rs` clones affected files into `<episode>/_backups/ui/`, with Restore (plan steps 12, 14, 16) |
| 2 | Where the Job runner opens | No preference 2026-10-02 | Plan default: "Jobs" button in the filter bar + ⌘J; revisit after Phase C |
| 3 | Fonts | Decided 2026-10-02 | OpenDyslexic Regular/Bold + OpenDyslexicMono instead of Inter / IBM Plex Mono; copied from the operator's files into `ui/assets/fonts/` |

## Phase A — Episode Browser

- Last run: —
- Status: **not started** (ready)
- Verifier: —

### Checklist

| # | Check | Result | Notes |
|---|---|---|---|
| 1 | Files from plan.md steps 1-8 exist; `ui/target/` ignored; OpenDyslexic fonts embedded and rendering | ⬜ | FONT_SCALE: |
| 2 | cargo build --release / test / clippy -D warnings green | ⬜ | |
| 3 | Loader tests cover all plan.md step 6 cases | ⬜ | |
| 4 | No hex colour literals outside theme.rs | ⬜ | |
| 5 | EP03: real header, 4 real speaker lanes, timeline end matches manifest, Not verified vs Failed correct | ⬜ | |
| 6 | Search, speaker filter, Failed only, segment click-to-select work | ⬜ | |
| 7 | Error state on a bad directory; current episode stays open | ⬜ | |
| 8 | Design fidelity vs screens 01, 02 | ⬜ | |
| 9 | EP03 load < 50 ms; smooth scroll; idle CPU ~0% | ⬜ | |
| 10 | No Python changes; EP03 directory untouched; no Figma calls | ⬜ | |

## Phase B — Playback

- Last run: —
- Status: **blocked on Phase A**

### Checklist

| # | Check | Result | Notes |
|---|---|---|---|
| 1 | audio.rs exists; build/test/clippy green | ⬜ | |
| 2 | Space play/pause, Esc stop (by ear) | ⬜ | |
| 3 | Switching turns while playing: no overlap (by ear) | ⬜ | |
| 4 | Multi-segment turn plays in order; time and seek advance (by ear) | ⬜ | |
| 5 | Auto-advance follows the active filter (by ear) | ⬜ | |
| 6 | Space in the search box does not play | ⬜ | |
| 7 | Missing WAV: Missing audio row, other turns still play | ⬜ | |
| 8 | Device menu works; lost device pauses and shows Unavailable device (by ear) | ⬜ | |
| 9 | Design fidelity vs screen 03 | ⬜ | |
| 10 | Idle CPU ~0% when not playing | ⬜ | |

## Phase C — Job Runner and Verify Overlay

- Last run: —
- Status: **blocked on Phase B**
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
