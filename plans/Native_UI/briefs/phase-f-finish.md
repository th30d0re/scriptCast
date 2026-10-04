# Finish Phase F (align screen 03 with the 2026-10-04 Figma update)

Working directory: this git worktree (branch `ui/phase-f`). **Read `plans/Native_UI/briefs/phase-f-figma-0410.md` first; all of its prohibitions apply to you.** In particular:

- no git writes;
- never start other agent CLIs, and never use `osascript` or drive other apps;
- the real project is read-only; save tests run on `/tmp` copies;
- no synthesis; mute captures.

Codex started this and ran out of quota after 5 minutes. **Finish it; do not restart it.**

## State you inherit (measured by the orchestrator)

- About 530 lines changed: `table.rs` (+411/−…, variable-height rows), `read_along.rs` (rewritten), `editing.rs` (+149), plus `inspector.rs`, `main.rs`, `settings.rs`, `theme.rs`, `capture.rs` and tests.
- `cargo build --release` and `cargo test` (123 passed) are green. One line is over 120 characters: `ui/src/inspector.rs:295`.
- `plans/Native_UI/evidence/phaseF-findings.md` still says "in progress"; nothing in it is verified.

## Do

1. **Audit** the diff against the brief's Build items 1–6 and the two target screenshots. For each item, record in the findings: done, partly done, or missing.
2. **Finish** the missing and partly done items yourself. If an item needs more than about 150 lines, implement the smallest correct version and say so.
3. **Add the required tests** (row-height prefix sums and `scroll_to`; word mapping, including unequal counts giving no highlight; the mode toggle writing the setting) if they are not already there.
4. Run the acceptance block and the three captures from the brief. For the read-along capture, build the `/tmp` copy with `word_timings.json` as the brief describes.
5. **Look at each PNG** and describe it honestly against the target screenshots: what matches and what doesn't. If a capture times out because the display is asleep, record it.
6. Rewrite `phaseF-findings.md` as the final report.
