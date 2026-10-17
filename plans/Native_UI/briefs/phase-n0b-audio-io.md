# Phase N0b: native decoding and multi-resolution peaks from SoundCraft audio-io

Worktree: branch `ui/n0b-audio-io`, from `dev` (app 0.20.1). A parallel agent adds `egui_kittest` as a dev-dependency on another branch. Avoid unrelated `Cargo.toml` churn.

## Rules

- **No git writes.**
- **Never start other agent CLIs**, and never use `osascript`.
- **No Python changes.**
- **The real project is read-only.** A copy with real `.mp4`/`.m4a` clip sources is at `/tmp/scriptcast-k/proj/Architecting_the_operation/archive/sources/`.
- Lines under 120 characters; `cargo fmt`; delete no comments.
- Redirect cargo output to a file.

## Source

SoundCraft (MIT OR Apache-2.0), pinned at commit `f3a19480d8cc2d7a98cda492f5464d8c54558c9f`. A read-only clone is at `/private/tmp/claude-501/-Users-emmanuel-Dev-Tools-scriptCast/bf35c9bf-0d0b-4d7b-9bf1-8b66645c0f23/scratchpad/soundcraft`.
- Its `crates/audio-io` depends only on `serde`, `thiserror`, `log` and `symphonia` (with the `isomp4`, `aac`, `alac` and other features; see their workspace `Cargo.toml`).
- Use it as a **git dependency pinned to that rev** (`soundcraft-audio-io = { git = "https://github.com/storytold/soundcraft", rev = "…" }`).
- If the workspace-inherited fields make a git dependency impossible, vendor **only** `crates/audio-io` into `ui/vendor/soundcraft-audio-io/`, with its LICENSE-MIT, LICENSE-APACHE, NOTICE and a `SOURCE.md` naming the rev, and say so in the findings.

## Problems this fixes (measured)

- `ui/src/waveform.rs` decodes through rodio, and falls back to spawning **ffmpeg** (`ffmpeg_wav` :64, `ffmpeg_f32le` :137) for `.mp4`/`.m4a` sources, because rodio can't read them.
- The Phase M clip maker draws only about the first third of the waveform for `jalachantelle_2026a.mp4`.
- Earlier, re-decoding every frame pinned the CPU.
- ffmpeg is an external dependency, and it is slow to start.

## Build

1. **Decoding:**
   - decode every source (episode WAVs, clip sources, references) through `soundcraft-audio-io`;
   - keep rodio for **output only**, as `audio.rs` does now: feed it decoded samples. If any path today relies on rodio decoding, route it through the new decoder;
   - keep the ffmpeg path only as a last-resort fallback for formats audio-io refuses, and log when it is used.
2. **Waveforms:** use its `Peaks` (the multi-resolution min/max/RMS overview) for every waveform: timeline blocks, the playback bar, the clip detail, the clip maker and the reference maker.
   - Build `Peaks` once per file on a background thread, and cache them in memory, keyed by path and mtime.
   - Query them per drawn pixel range, so zoom is exact. Drop the `covers_window` heuristics.
3. **Accurate ranges:** clip range playback and drawing use exact sample ranges from the decoded buffer.
4. **Resampling:** respect the 48 kHz output stream that fixed the whine. Resample with good quality where a source differs; never route through rodio's cheap resampler. Use SoundCraft's `dsp::offline::resample` only if you also take `soundcraft-dsp`. Otherwise keep the current approach, and say which in the findings.

## Acceptance

1. `cargo fmt --check`, clippy with `-D warnings` and `cargo test` must all be green.
2. New tests:
   - decode a short `.m4a` and `.mp4` fixture (create tiny ones with ffmpeg **once**, at test-fixture creation time, and commit them under `ui/tests/fixtures/audio/`);
   - `Peaks` range queries match a brute-force min/max;
   - the cache is built once across repeated frames;
   - the clip-maker envelope spans the whole window.
3. On the `/tmp` copy, decode every clip source in `clips.yaml`. Report the count decoded natively, the count needing the ffmpeg fallback, and the timings.
4. A grep shows ffmpeg is no longer called on the normal path.
5. Bump the version to 0.20.2.

## Findings

Write `plans/Native_UI/evidence/n0b-findings.md` early.
