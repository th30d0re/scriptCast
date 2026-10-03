# B3 chunk 4: waveform data and timeline-block waveforms

Working directory: this git worktree (branch `ui/phase-b3`).

## Rules

- **No git commands that write.** Files you may change:
  - create `ui/src/waveform.rs`;
  - edit `ui/src/lib.rs` (add `pub mod waveform;`), `ui/src/timeline.rs` and `ui/src/main.rs`.
- Put scratch files under `/tmp/scriptcast-b3/`. Never put them in the worktree.
- Lines under 120 characters, written as normal readable Rust like the surrounding code. Run `cargo fmt`. **Do not delete or reword existing comments.**
- No colour or size literals outside `ui/src/theme.rs`. Add any new size tokens there, named for what they are.
- Mute every playback you start. The real project is read-only. Write no files under the episode.

## Target

`ui/design/screens/03d-waveforms/timeline.png`. Each turn block shows its number at the left, then a row of thin vertical bars filling the rest of the block. The bars are drawn in the speaker colour, centred vertically, and their heights follow the turn's loudness.

## Design (follow it)

1. **`waveform.rs`.**
   - `pub struct Envelope { pub peaks: Vec<f32> }`. Each value is 0.0..=1.0, the max absolute sample per 10 ms bucket, across all of a turn's segments in order.
   - `pub fn envelope(paths: &[PathBuf]) -> Result<Envelope, String>`. Decode with `rodio::Decoder`, which the crate already uses, converting samples to f32. Normalise to 0..1 by full scale; do **not** rescale each turn to its own peak, so quiet turns look quiet.
   - `pub fn bars(env: &Envelope, n: usize) -> Vec<f32>`. Resample the peaks into `n` bars by taking the max per bar. Return an empty `Vec` when `n == 0` or the envelope is empty.
   - `pub struct Waveforms`: background loading. `Waveforms::start(episode: &Episode, ctx: egui::Context) -> Self` spawns **one** thread that computes envelopes for every turn with audio, in order, and sends `(turn_index, Envelope)` over an `mpsc` channel, calling `ctx.request_repaint()` about every 50 turns, not for each one.
   - `poll(&mut self)` drains the channel into a `HashMap<usize, Envelope>`. `get(turn) -> Option<&Envelope>`.
   - Dropping `Waveforms` must stop the thread soon. Use a shared `AtomicBool` that the thread checks between turns.
2. **`main.rs`.**
   - Create `Waveforms` whenever an episode finishes loading, including on Reload and after a job reload. Drop the old one first.
   - Call `poll()` once per frame.
   - Pass `&Waveforms` (or an `Option`) into `timeline::show`.
3. **`timeline.rs`.**
   - In the block-drawing code (around the `view::line(... "{:03}" ...)` call), only when `block.width()` leaves room after the turn number: compute the label's width, take the remaining rect, shrunk by `t::SPACE_4`, and draw `bars(env, n)` with `n = remaining_width / (bar width + gap)`.
   - Each bar is a filled rect of width `t::WAVE_BAR` (new token, 1.5) with gap `t::WAVE_GAP` (new token, 1.5) and height `max(peak * remaining_height, t::WAVE_MIN)` (new token, 1.0), centred vertically.
   - Draw the bars in the label's colour, with the same opacity rule as the label.
   - Without an envelope yet, draw nothing extra.

## Tests (required, in `waveform.rs`)

- `bars` on an empty envelope, with `n == 0`, with n larger than the number of peaks, and with the max-per-bar behaviour.
- `envelope` on the fixture WAVs: `ui/tests/fixtures/episode_small/` (look at its `Samples/`). The number of buckets matches the duration within ±1, and every value is within 0..=1.

## Check yourself, then report

```bash
cd ui && cargo fmt && cargo build --release 2>&1 | tail -2 && cargo test 2>&1 | grep "test result" && cargo clippy --all-targets -- -D warnings 2>&1 | tail -1
awk 'length>120{print FILENAME":"FNR}' src/*.rs
git diff --stat
./target/release/scriptcast-ui --mute --select 6 --screenshot /tmp/scriptcast-b3/chunk4.png /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
```

Open the PNG and say plainly whether the timeline blocks show bars. Also time how long envelope loading takes for all 392 EP03 turns, with a temporary `eprintln!` that you **remove** afterwards.

Write `plans/Native_UI/evidence/phaseB3-chunk4.md` with:

- the diff stat;
- all command output;
- the timing;
- what the PNG shows.

If anything fails, say so plainly.
