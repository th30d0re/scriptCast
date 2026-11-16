# N1 Mix findings

## Implemented

- Added a full-window Mix editor reached by a **Mix** action immediately after **Stitch MP3** in the Jobs controls.
- Kept the existing episode header styling, theme tokens, audio output, and waveform column painter. Overlay anchors are painted over the existing episode waveform data.
- Added `mix.yaml` loading, canonical YAML saving, missing-file-as-empty behavior, and disk reload for Revert. The app had no existing comment-preserving YAML writer, so the writer emits only the fields accepted by `scriptcast.tools.mix`.
- Overlay gain is clamped to −40…0 dB and defaults to −22 dB. Turn gain is clamped to −12…+12 dB.
- Unmatched `after_word` validation is local and deterministic: lowercase ASCII alphanumeric normalization plus the configured regular expression. It does not call Whisper or a model.
- Added the `scriptcast-mix` runner preset. It passes the episode directory, episode script, project-root `mix.yaml`, `{episode}_mixed.mp3`, and the documented `{episode}.captions.json.asr.json` only when that recognised-word cache exists.
- A/B playback uses the app audio output. Stitched resolves the episode's single non-mixed MP3; Mixed remains disabled until `{episode}_mixed.mp3` exists.
- Loudness readouts remain deliberately unmeasured (`—`) and show the single target `−16 LUFS`.

## Verification

- `cargo check --locked --offline` — passed.
- `cargo test --locked --offline --test mix` — 6 passed. Covers fixture loading, missing-file behavior, YAML save/reload, gain bounds, unmatched-word error text/normalization, and render argv/ASR gating.
- `cargo test --locked --offline --test runner_presets --test phase_j` — 28 passed.

## Notes

- Save only writes `overlays` and `gains` with the tool's existing rule fields; it does not run ffmpeg.
- Render progress uses the existing preset-runner state. `scriptcast-mix` itself does not publish progress JSON, so the strip remains indeterminate/empty until the runner completes.
