# Phase B3 — Waveforms, speaker-label fix, playback-gap experiments (implementer brief)

Your working directory is a git worktree of scriptCast on branch `ui/phase-b3`, created from `dev` after Phase C was merged (`847e9a2`).

## Prohibitions (read first)

- **No git writes.** The orchestrator commits.
- **Write only inside this worktree**, plus `~/.cargo`, `ui/target/` and `/tmp/scriptcast-b3/`.
- **The real project is read-only:** `/Users/emmanuel/Documents/Theory/TheOriginalPower/`. Reading EP03's WAVs is fine.
- **No jobs:** run no synthesizing or overwriting job, and no Verify against real episodes.
- **No Python changes.**
- **Do not modify** `ui/design/**` or `ui/assets/**`. Under `plans/**`, edit only the findings file.
- **No new colour or size literals outside `theme.rs`.**
- **Captures are muted.** Never call Figma; the designs are already saved locally. Never launch Live or Logic.
- **Keep every line under 120 characters.** Write normal, readable Rust that matches the surrounding code. A previous pass shipped minified one-line blocks and had to be repaired.

## Goal

Implement plan steps B3.1–B3.5 (`plans/Native_UI/plan.md`, "Phase B3"), plus the playback-gap experiments below.

## Read

1. `plan.md`, Phase B3.
2. `ui/design/screens/03d-waveforms/timeline.png` and `playback-bar.png`: the targets.
3. `ui/design/page-metadata.xml`, nodes `27:*` (timeline waveforms) and `4:18360` (Waveform viewport: 36 high, fill `#1e2024` = `bg/surface`, 1 px `border/default`, radius 3, 2 px playhead in `text/primary` at the centre, 32 px fades from `bg/surface` to transparent at both ends).
4. `plans/Native_UI/evidence/phaseC-findings.md`, for API notes and the current structure.
5. The existing code: `timeline.rs`, `table.rs`, `audio.rs`, `main.rs`, `jobs.rs`, `theme.rs`, `capture.rs`.

## Build

- **B3.1.** Remove the coloured bar before speaker names in the timeline lanes and the table's Speaker column. Show the name alone, in the speaker colour, as in Figma. Check any other place that draws it, and leave the inspector's identity line as the design shows it.
- **B3.2–B3.5.** As in the plan:
  - envelopes are computed on a background thread at episode open and cached in memory, never written to disk; the 392-turn EP03 must stay responsive (record the time taken);
  - the timeline blocks draw waveform bars;
  - the playback bar shows the scrolling waveform viewport, and click or drag in it seeks;
  - on built-in MacBook speakers (`audio::is_builtin_speakers`), the bar falls back to the plain seek track.

## Playback-gap experiments (operator report, 2026-10-03)

The operator hears a faint high-pitched whine **between clips**:

- only from the MacBook's built-in speakers;
- not in headphones;
- not in Live.

The orchestrator measured EP03's WAVs and found no tone in their quiet frames. Lowering the playback repaint rate to 250 ms on the speakers **did not remove it** (operator, after Phase C).

Facts:

- rodio 0.22 opens the device with about a 50 ms buffer by default (`rodio-0.22.2/src/stream.rs:207-216`). The app keeps one output stream open and feeds it zeros when no `Player` is attached.
- Auto-advance starts the next turn only when a repaint notices `Player::empty()`, so on the speakers the gap between turns can be up to 250 ms of silence.

Build two operator-selectable experiments. Add both to the settings gear popover under "Playback", persist them, and add a capture/CLI flag for each:

1. **Gapless auto-advance** (default **on**). When auto-advance is on, append the next visible turn's segments to the same `Player` ahead of time, so there is no gap and no repaint dependence. The UI's notion of the playing turn must follow the audio: track segment boundaries by summed durations against the wall-clock position. Stop, pause, seek, selection change and filter change must all still behave. Unit-test the turn-boundary bookkeeping.
2. **Release the speakers when idle** (default off). After 2 s with no `Player` attached, drop the device sink and reopen it on the next play. Measure and record the reopen latency.

The operator will A/B these by ear. You cannot judge the whine, so do not claim either fixes it.

## Acceptance

```bash
cd ui && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs   # nothing
awk 'length>120{print FILENAME":"FNR}' src/*.rs                                     # nothing
```

Save these captures of EP03, muted, to `plans/Native_UI/evidence/`:

- `phaseB3-timeline.png`;
- `phaseB3-playing-waveform.png`: playing, with a non-speaker device forced by a flag, so the waveform viewport shows;
- `phaseB3-playing-speakers.png`: the seek-track fallback.

## Findings file

Write `plans/Native_UI/evidence/phaseB3-findings.md` early and keep it current. Record decisions, timings, test output, the capture paths, and anything left undone. List any scratch files you create; better still, keep scratch work in `/tmp/scriptcast-b3/`.
