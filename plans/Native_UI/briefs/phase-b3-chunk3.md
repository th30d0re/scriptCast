# B3 chunk 3: open the output at the files' sample rate (fixes the speaker whine)

Working directory: this git worktree (branch `ui/phase-b3`).

## Rules

- **No git commands that write.** Change only `ui/src/audio.rs`, plus `ui/src/main.rs` if it calls `AudioOut::open`.
- Put scratch files under `/tmp/scriptcast-b3/`.
- Lines under 120 characters, written as normal readable Rust like the surrounding code. Run `cargo fmt`. Do **not** delete existing comments.
- Mute every playback you start. The real project is read-only.

## Diagnosis (measured by the orchestrator, 2026-10-03)

- `system_profiler SPAudioDataType` reports "MacBook Pro Speakers … Current SampleRate: 44100". The episode WAVs are 48 kHz mono.
- rodio 0.22 opens the device at its default config (`DeviceSinkBuilder::from_device`, `rodio-0.22.2/src/stream.rs:205-217`), so it resamples 48 → 44.1 kHz itself.
- The orchestrator simulated this on 40 EP03 turns. Linear interpolation, compared with a high-quality polyphase resampler, adds 7–10 dB in 14–22 kHz, plus a spurious tone at 15.3 kHz that is 17.5 dB louder. That matches the operator's report: a high-pitched whine in quiet passages, on the built-in speakers only, not in Live.

## Task

Open the output stream at **48 000 Hz** when the device supports it, so that no resampling happens.

1. Write a pure function that is testable without a device:
   ```rust
   /// Pick the config range to open at `want` Hz: the first range whose
   /// min..=max contains `want`, preferring the device's default channel count.
   fn pick_rate(ranges: &[(u16 /*channels*/, u32 /*min*/, u32 /*max*/)], default_channels: u16, want: u32)
       -> Option<usize /*index into ranges*/>
   ```
2. In `AudioOut::open`:
   - get `rodio::stream::supported_output_configs(&device)` (or the cpal equivalent the code already uses);
   - map the configs to tuples and call `pick_rate(…, 48_000)`;
   - if a range matches, build with `DeviceSinkBuilder::from_device(device)?` then `.with_supported_config(&range.with_sample_rate(48_000))` (adapt to the real 0.22 API: read `stream.rs:283-360`), then `.with_error_callback(…)`, then open as the code does today;
   - if nothing matches, or opening fails, fall back to exactly the current default path.
3. Store the actual stream sample rate in `AudioOut` and expose `pub fn stream_rate(&self) -> u32`. Show it in the output-device dropdown's tooltip, for example "Output at 48 kHz", so the operator can see it.

## Tests (required)

`pick_rate` must cover:
- a match on the default channel count;
- a match only on another channel count;
- no range containing 48 000;
- an empty list.

## Check yourself, then report

```bash
cd ui && cargo fmt && cargo build --release 2>&1 | tail -2 && cargo test 2>&1 | grep "test result" && cargo clippy --all-targets -- -D warnings 2>&1 | tail -1
awk 'length>120{print FILENAME":"FNR}' src/*.rs
git diff --stat
```

Then add a temporary `eprintln!` (or use an existing debug path) to print `stream_rate()` once at startup. Run the release binary briefly, muted, on EP03 with `--screenshot /tmp/scriptcast-b3/chunk3.png`, and record the rate it printed. **Remove the eprintln afterwards.**

Write `plans/Native_UI/evidence/phaseB3-chunk3.md` with:
- the diff stat;
- all command output;
- the printed stream rate.

If anything fails, say so plainly.
