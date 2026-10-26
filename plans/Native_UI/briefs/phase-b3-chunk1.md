# B3 chunk 1: remove the bar before speaker names

Working directory: this git worktree (branch `ui/phase-b3`).

## Rules

- **No git commands that write.** Change only `ui/src/table.rs`, `ui/src/timeline.rs`, and, only if the bar is drawn there, `ui/src/view.rs`.
- Put any scratch files under `/tmp/scriptcast-b3/`. Never put them in the worktree.
- Keep lines under 120 characters, written as normal readable Rust like the surrounding code.

## Task

A thin coloured bar is drawn just left of each speaker name:

- **In the table's Speaker column** (`table.rs`): a `rect_filled` of size `vec2(t::SPACE_2, t::SPACE_12)` in the speaker colour, with the name offset by `t::SPACE_6`.
- **In the timeline speaker lanes** (`timeline.rs`): find the equivalent.

Remove the bar in both places. The name keeps its speaker colour and starts where the bar used to start (no leftover indent). Change nothing else.

## Check yourself, then report

Run these and paste the output into the findings file:

```bash
cd ui && cargo build --release 2>&1 | tail -2 && cargo test 2>&1 | grep "test result" && cargo clippy --all-targets -- -D warnings 2>&1 | tail -1
git diff --stat
./target/release/scriptcast-ui --mute --select 12 --screenshot /tmp/scriptcast-b3/chunk1.png /Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/ATO_EP03_five_levels
```

Open the PNG and confirm no bar is visible before the names in either place.

Write `plans/Native_UI/evidence/phaseB3-chunk1.md` with:

- the diff stat;
- the command output;
- what you saw in the PNG.

If anything fails, say so plainly; do not claim success.
