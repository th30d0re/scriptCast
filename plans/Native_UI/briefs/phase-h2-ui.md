# Phase H2 (UI): clip transcripts in the app

Working directory: a git worktree of scriptCast on branch `ui/phase-h2`, from `dev` (app 0.9.0 plus the H1/H2 Python).

## Rules (read first)

- **No git writes.**
- **Never start other agent CLIs, and never use `osascript` or drive other apps.**
- **The real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only**, including its `.venv*` folders. You may *run* their interpreters. Live runs use `/tmp/scriptcast-h2/` copies.
- **No Python changes.** The `scriptcast-clip-transcribe` tool from H1/H2 is finished; you only call it.
- No synthesis. Mute captures.
- Lines under 120 characters, readable code, `cargo fmt`, no deleted comments, no colour or size literals outside `theme.rs`.

## Read

1. `plans/Native_UI/plan.md`, "Phase H", item H2.
2. `plans/Native_UI/evidence/phaseH1-findings.md` and `scriptcast/tools/clip_transcribe.py`, for the output format of `clip_transcripts.json`: per `turn_id`, `{clip_id, words: [{text, start_ms, end_ms, engine}], coverage_warnings, whisper_disagreement_pct, listen_manually}`, plus `manifest_sha256`.
3. The existing code:
   - `word_timings.rs` and `read_along.rs` (Phase E/F: the timings loader with staleness check, and the in-row highlight);
   - `runner.rs` and `jobs.rs` (presets: the "Read-along timings" preset is the model to copy);
   - `cli.rs` (command resolution, step C0);
   - `editing.rs` (the clip-turn transcript-correction rules from Phase G);
   - `inspector.rs`.

## Established facts (measured 2026-10-04)

- `scriptcast-clip-transcribe` runs only from `TheOriginalPower/.venv-fermion`, which now has both the Phonon stack and `mlx-whisper`. `.venv-voice` lacks the Phonon stack (`fermion`).
- The C0 resolver's dev-checkout fallback currently prefers venvs with an installed `scriptcast` (`.venv-voice`), and probes only by importing the entry module. `clip_transcribe` imports its engines lazily, so that probe would wrongly accept `.venv-voice`.
- On a copy of EP03, turn 220 (`atwater_1981`) took about 42 s and gave 159 words, 11.8% disagreement, and no coverage warnings.

## Build

1. **Command resolution** for `scriptcast-clip-transcribe`. Choose an interpreter that can import `fermion`, `mlx_whisper` and `scriptcast.asr.engines`. Probe exactly those imports, with a short timeout, and prefer such a venv over the "installed scriptcast first" ordering for this one command only. Unit-test it with fake venvs.
2. **Job preset "Clip transcripts".**
   - Runs `scriptcast-clip-transcribe <episode> [--script]` over all clip turns.
   - Copy the Read-along timings preset: it checkpoints `clip_transcripts.json` when that file exists, and synthesizes nothing.
   - Disabled, with a tooltip, when the episode has no clip turns.
3. **Loader** `clip_transcripts.rs`: parse the file at episode open and after job reloads, with the same staleness check as word timings (`manifest_sha256`).
4. **Read-along for clip turns:** when a clip turn plays and has a current transcript, highlight its words inside the row, as Phase F does for synthesized turns. Words come from `clip_transcripts.json` instead of `word_timings.json`. Map words in order; if the counts disagree with the row text, see item 5.
5. **Clip transcript text.**
   - Clip rows keep showing the script's transcript text (what the operator wrote).
   - In the inspector's Turn view for a clip turn, add an "Ensemble transcript" block: the merged text; the engine sources as small muted labels when not all words are `phonon-2`; the disagreement %; and any coverage warnings.
   - When `listen_manually` is true, show "Listen manually: engines disagree by N%" in the danger colour.
   - **Read-along on clips** uses the ensemble words. When the script text and the ensemble words differ in count, highlight within the *ensemble* text shown in the inspector, not in the row, and note "Highlighting follows the recording's transcript".
6. **"Use ensemble transcript"** in the clip editor (Phase G rules still apply): replace the draft body's text after the `[clip:<id>]` tag with the ensemble text, keeping the tag. It stays a transcript-only correction.
7. **Strip the raw `[clip:<id>]` tag** from the inspector's TRANSCRIPT display; the badge already shows the id. The editor still shows and requires the tag.
8. **Missing or stale hints,** like read-along: "No ensemble transcript yet. Run Jobs → Clip transcripts." with a Run… button.

## Tests (required)

- The resolver probe ordering for this command.
- Loader staleness.
- Clip read-along word lookup.
- Tag stripping for display.
- "Use ensemble transcript" keeps the tag.

## Acceptance

```bash
cd ui && cargo fmt && cargo build --release && cargo test && cargo clippy --all-targets -- -D warnings
awk 'length>120{print FILENAME":"FNR}' src/*.rs tests/*.rs
grep -rnE '#[0-9a-fA-F]{6}|from_rgb|from_rgba|Color32::from' src | grep -v theme.rs
```

**Live, on a `/tmp/scriptcast-h2` copy of EP03**, with `SCRIPTCAST_PROJECT` pointed at a copy of the project files (script, toml, voices). The `.venv-fermion` interpreter may be referenced at its real path, read-only.

- Run the Clip transcripts job through a test or debug path for turn 220.
- Capture `--select 220` (the inspector with the ensemble block) and `--select 220 --play --at 4000` (the highlight) into `plans/Native_UI/evidence/phaseH2-*.png`.
- Look at both captures and describe them honestly.

Write `plans/Native_UI/evidence/phaseH2-findings.md` early and keep it current.
