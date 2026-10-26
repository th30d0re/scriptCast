# Phase H2 (UI) findings

Started: 2026-10-04. Worktree: `scriptCast-worktrees/phase-h2` on `ui/phase-h2`.
Completed live run the same day.

## Scope

Wire `clip_transcripts.json` into the native UI: resolver probe for
`scriptcast-clip-transcribe`, Jobs preset, loader with staleness check, clip
read-along, inspector ensemble block, "Use ensemble transcript", strip
`[clip:<id>]` from Turn-view transcript display.

## What landed

| Piece | Location |
|---|---|
| Engine-aware resolver | `ui/src/cli.rs` — for this command only, probe `fermion`, `mlx_whisper`, `scriptcast.asr.engines` (6 s timeout); prefer such a venv over installed-scriptcast ordering |
| Loader | `ui/src/clip_transcripts.rs` — `manifest_sha256` staleness like word timings |
| Preset | `Preset::ClipTranscripts` in `runner.rs` / `jobs.rs` / `checkpoint.rs` |
| Clip read-along | `table.rs` + inspector ensemble block; row highlight only when script word count matches ensemble |
| Use ensemble | `clip_transcripts::use_ensemble_transcript` + Edit button |
| Tag strip | Inspector TRANSCRIPT uses `script::display_body` |

## Resolver measurement

Cold `import fermion, mlx_whisper, scriptcast.asr.engines` on `.venv-fermion`
takes about **2.8 s**. The shared 2 s entry-module timeout killed the probe and
fell back to PATH `python3`. Clip-engine probes now use **6 s**.

With `/tmp/scriptcast-h2` symlinking `.venv-fermion` and `.venv-voice` (read-only
to Theory), discovery logs:

```
scriptcast-clip-transcribe · Dev checkout: .../.venv-fermion/bin/python
  (.../phase-h2; -m scriptcast.tools.clip_transcribe)
```

Other commands still prefer `.venv-voice` when it has an installed scriptcast.

## Live run — EP03 turn 220

Copy under `/tmp/scriptcast-h2/` (episode, `scriptcast.toml`, script, voices,
`clips.yaml`; venv dirs as symlinks). Theory tree untouched.

```bash
SCRIPTCAST_PROJECT=/tmp/scriptcast-h2 \
PYTHONPATH=<worktree> SCRIPTCAST_FERMION=<Theory>/.venv-fermion/bin/fermion \
  <Theory>/.venv-fermion/bin/python -m scriptcast.tools.clip_transcribe \
  /tmp/scriptcast-h2/outputs/ATO_EP03_five_levels \
  --script .../ATO_EP03_redefining_racism_five_levels.md \
  --turns 220
```

| Field | Value |
|---|---|
| Wall clock | **42.32 s** real |
| Words | **159** |
| Disagreement | **11.8%** |
| `listen_manually` | false |
| Coverage warnings | `[]` |
| Script word count (turn body) | **94** |
| Row mapping | **no** (counts differ; years expand to words) |

## Evidence captures

### `phaseH2-select220.png`

- Turn 220 selected (Lee Atwater). Table shows Audio clip badge + `atwater_1981`;
  no raw `[clip:…]` in the row or inspector TRANSCRIPT.
- Inspector Turn view shows the operator script (years as digits: 1954 / 1968).
- **ENSEMBLE TRANSCRIPT** block below: note “Highlighting follows the
  recording's transcript”, “Whisper disagreement 11.8%”, then the merged text
  with “nineteen fifty four” / “nineteen sixty eight”.
- Idle: no read-along pill. Footer: Original recording · Not synthesized.
- All words are `phonon-2`, so no mixed-engine labels.

### `phaseH2-select220-play4000.png`

- Playing turn 220 at about **00:04.214** / 00:51.899 (capture waits for
  `--at` + 200 ms).
- Same ensemble meta visible above the fold.
- At 4214 ms the timing gap is between `nigger.` (ends 3440–4160) and `By`
  (4240), so **no word is active** at the capture instant; the ensemble text
  shows no accent pill. Row text is also unhighlighted (script/ensemble counts
  disagree). Brief asked for `--at 4000`; this gap is real, not a UI miss.
- Word that covers 4000 ms exactly: `nigger.` (historical archive audio).

## Acceptance

```
cd ui && cargo fmt && cargo build --release && cargo test && \
  cargo clippy --all-targets -- -D warnings
```

All green (2026-10-04). Line-length and colour greps empty outside `theme.rs`.

Note: leave `SCRIPTCAST_PROJECT` unset when running `cargo test`; pointing it at
`/tmp/scriptcast-h2` makes fixture jobs fail to resolve the fixture transcript.

## Tests added

- Resolver probe ordering (`cli_resolution.rs`)
- Loader staleness + use-ensemble keeps tag + clip word lookup
  (`clip_transcripts.rs`)
- Tag strip display (`inspector.rs`)
- Clip transcripts preset argv / overwrite (`runner_presets.rs`)
