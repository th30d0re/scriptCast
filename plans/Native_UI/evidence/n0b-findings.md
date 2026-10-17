# N0b findings

Branch `ui/n0b-audio-io`. App version is 0.20.2. No git writes. The GUI was not launched.

## Dependency

`soundcraft-audio-io` and `soundcraft-dsp` are git dependencies of `https://github.com/storytold/soundcraft` at rev `f3a19480d8cc2d7a98cda492f5464d8c54558c9f`. Workspace-inherited fields did not block that. Nothing was vendored.

## Decoding and output

Episode WAVs, clip sources, and references go through `soundcraft_audio_io::decode`. Rodio no longer decodes. `AudioOut` appends a `SamplesBuffer` of already-decoded samples.

ffmpeg remains only inside `ffmpeg_wav` / `ffmpeg_pcm` in `ui/src/waveform.rs`. `decode_path` calls the full-file PCM fallback after soundcraft returns an error, and prints `scriptcast-ui: ffmpeg fallback for …`. `phase_l.rs` still calls `ffmpeg_wav` only when `play` has already returned an error. A search for `Decoder` in `ui/src` is empty. `Command::new("ffmpeg")` appears only in those two fallback functions.

## Waveforms

Each file is decoded once per `(path, mtime)` on the caller’s thread (the timeline and the clip maker already spawn that work). The cache holds the planar samples, a mono mix, and `soundcraft_audio_io::peaks::Peaks`. A repeated `envelope` of the same file reuses those `Arc`s. Columns come from `Peaks::spans` / `Peaks::minmax` over the decoded samples. `covers_window` is gone, so the clip maker draws the requested time range across the whole allocated rect.

## Resampling

Playback uses `soundcraft_dsp::offline::resample` (windowed sinc) when the source rate differs from the open stream rate. `AudioOut::open` still asks for 48 kHz. A source already at the stream rate is not resampled, so rodio’s sample-rate converter sees equal rates and copies samples through. Clip ranges are sliced by sample index before that resample.

## Clip sources

`/tmp/scriptcast-k/proj/Architecting_the_operation/archive/sources/` is not on disk. The measurement read the real project at `/Users/emmanuel/Documents/Theory/TheOriginalPower/` and did not write there. `clips.yaml` has 39 unique sources. Release decode (`cargo test --release --lib archive_clip_sources -- --ignored`) finished in 6.77s:

- native: 39
- ffmpeg fallback: 0
- failed: 0

`jalachantelle_2026a.mp4` is 76.417s of audio and decoded in 0.118s. The slowest file was `atwater_1981.mp3` (2518.021s of audio in 1.433s). Every other source was under 0.32s.

## Checks

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` passed. New tests cover the `.m4a` and `.mp4` fixtures under `ui/tests/fixtures/audio/`, exact `Peaks` min/max, a single cache build across repeated queries, a clip-maker envelope that stays loud in the last third of the window, and an exact sample slice. The fixtures were created with ffmpeg once and are on disk. They were not committed, because this brief forbids git writes.
