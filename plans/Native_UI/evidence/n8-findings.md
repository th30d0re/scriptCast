# N8 findings: scriptcast-fx

Branch `claude/zen-edison-pkv554`. The branch tree is equal to `origin/dev` (072b16a).

## Files

- `fx/Cargo.toml`: package `scriptcast-fx`, edition 2024, `GPL-3.0-or-later`, `publish = false`. Two dependencies only: `soundcraft-dsp` and `soundcraft-audio-io`, both pinned to rev `f3a19480d8cc2d7a98cda492f5464d8c54558c9f` of `https://github.com/storytold/soundcraft`.
- `fx/Cargo.lock`: the lock for the binary.
- `fx/.gitignore`: ignores `/target`.
- `fx/src/lib.rs`: presets, `--set` parsing and checks, chain build, offline run, meter, WAV read and write, JSON output. JSON is written by hand, so the crate needs no JSON dependency.
- `fx/src/main.rs`: argument parsing and the two commands.
- `fx/tests/cli.rs`: tests that run the binary on audio made in the test.

No other file changed. The binary does not call stitch or mix. Nothing in `scriptcast/`, `ui/`, `video/` or `media/` calls it yet.

## Plugin ids (checked against the pinned rev)

`soundcraft_dsp::plugin_info` knows all seven ids: `channel_strip`, `de_esser`, `room_reverb`, `eq_7band`, `maximizer`, `dither`, `dc_offset_removal`. A unit test (`every_chain_plugin_exists`) keeps this true. Plugins are made with `soundcraft_dsp::create`.

## Chains

- speaker: `channel_strip`, `de_esser`, then `room_reverb` with `--room`, then `eq_7band` with `--eq7`.
- master: `maximizer`, `dither`, `dc_offset_removal`. `--room` or `--eq7` with master is an error.
- Each plugin starts on its own defaults. The tool adds no tuned values. `--set` applies after the defaults.
- `channel_strip` is the only trim, gate and compressor. Its gate and compressor stay off unless `--set channel_strip.gate_on=1` or `comp_on=1`.

## apply_plugin choice

The tool uses `soundcraft_dsp::offline::apply_plugin`, not `apply_plugin_with_tail`. `apply_plugin` keeps the buffer length, compensates latency, and drops any tail after the last input sample (for example the room reverb tail). It calls `prepare` with the real sample rate and channel count, then `reset`. Parameters set before the call stay set, because `reset` clears only the DSP state.

## Maximizer param ids

`threshold` (dB, -30..0, default 0), `ceiling` (dB, -30..0, default -0.3), `release` (ms, 1..1000, default 50). There is no LUFS-target parameter. The tool leaves the maximizer on its defaults and does not store -16 anywhere.

Other param ids at this rev, for `--set`:

- `channel_strip`: input_gain, gate_on, gate_threshold, gate_range, gate_release, comp_on, comp_threshold, comp_ratio, comp_attack, comp_release, comp_makeup, eq_on, hpf_on, hpf_freq, low_gain, low_freq, mid_gain, mid_freq, mid_q, high_gain, high_freq, output_gain.
- `de_esser`: freq, range, threshold, listen.
- `room_reverb`: mix, pre_delay, decay, size, damping, diffusion, width, low_cut.
- `dither`: bits (choice 16/20/24, default 16), noise_shaping.
- `dc_offset_removal`: cutoff.

## --set rules

- The form is `plugin.param=value`. The value must be a finite number.
- An unknown plugin id is an error. A known plugin that is not in this chain is also an error.
- An unknown param id is an error. The message lists the real param ids.
- A value outside the param range is an error. The tool does not clamp it silently.
- Each error is one stderr line (`scriptcast-fx: ...`) and exit code 1. The chain is checked before any audio is read, so no output file is written.
- With `--bypass`, the `--set` values are still checked, but no plugin runs.

## Audio I/O

- Decode: `soundcraft_audio_io::decode`, with the file extension as the hint.
- Encode: `soundcraft_audio_io::encode`, WAV, `BitDepth::Float32`, no dither, no BWF chunk. The rate and channel count are the input's.
- `encode` clamps samples to [-1, 1]. Thus `--bypass` gives the same samples only when the input is inside [-1, 1]. A float input with peaks above 0 dBFS is clamped on write.

## Meter

`LoudnessMeter::new(rate, channels)` reads the whole buffer once. `--json` prints exactly:

```json
{"integrated_lufs":-9.65944,"short_term_lufs":-9.959283,"momentary_lufs":-11.413449,"true_peak_db":-6.027722}
```

Silence gives -144 for the LUFS values (the meter floor). This value is finite.

## Chain JSON

```json
{"preset":"speaker","plugins":["channel_strip","de_esser","room_reverb","eq_7band"],"bypass":false,"frames":72000,"sample_rate":48000,"wrote":"/tmp/.../s.wav"}
```

`frames` and `sample_rate` are of the written file. `plugins` is the chain, also when `bypass` is true.

## Test results

From `fx/`:

- `cargo test`: 11 passed, 0 failed (5 unit tests in `src/lib.rs`, 6 CLI tests in `tests/cli.rs`).
- `cargo clippy --all-targets -- -D warnings`: exit 0, no warnings.

CLI tests:

- `meter_silence_is_quiet_and_finite`: 1 s of stereo silence. The four keys are in order. `integrated_lufs` is finite and at or below -70.
- `meter_sine_has_a_true_peak`: 1 kHz sine at 0.5. `true_peak_db` is finite and above -40.
- `speaker_chain_keeps_frame_count`: plain chain, and chain with `--room --eq7 --set channel_strip.hpf_on=1`. The frame count, rate and channel count stay the same. All samples are finite.
- `master_chain_keeps_frame_count`: the frame count stays the same.
- `bypass_writes_input_samples`: the output buffer is equal to the input buffer.
- `bad_arguments_fail_with_one_line`: unknown param, unknown plugin, `--room` and `--eq7` on master, and an unknown preset. Each exits non-zero with one stderr line and writes no file.

All test audio is made in the test, in a temporary directory. No episode directory is used.
