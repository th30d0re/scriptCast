# Follow-up in this session: mark the words a failed turn did not have heard

This worktree is now on branch `ui/failed-marks`, from `dev`, which includes your clip work. Same rules as before:

- no git writes;
- no other agent CLIs, no `osascript`;
- the real project is read-only;
- send build and test output to files;
- lines ≤120 characters, keep comments, no literals outside `theme.rs`.

Spec: `ui/design/DESIGN.md`, the last section, "failed turns show the words that were not heard". Targets: `ui/design/screens/03g-failed-turns/03-A.png` and `03-B.png`.

1. **Shared helper** `not_heard(expected: &str, heard: &str) -> Vec<(usize, usize)>`, giving byte ranges in the **displayed transcript text** of the expected words that were deleted or replaced. Tokenise and normalise like `verify_render._normalize` (lowercase; strip punctuation). Align with a diff, then map tokens back to byte ranges in the displayed text.
   - Use the `stress.json` entry's `expected` text for alignment.
   - When `expected` differs from the turn's current displayed text (the script has been edited since), map by tokens. If that mapping fails, show nothing.
   - Show nothing when `stress.json` is stale for that turn.
2. **A (inline):** for Failed rows, draw not-heard words in danger colour, bold, inside the Transcript text. Under the text, add a danger line: `N words not heard: "<words>"`, truncated with "…".
3. **B (side panel):** for Failed rows, draw a danger pill behind not-heard words, with a small "not heard" label (about 9 px) inside the pill after them. It must coexist with the accent read-along pill. If both apply to the same word, the read-along pill wins.
4. Reuse the measured-height and galley code you already know in `table.rs` and `read_along.rs`. Row heights must include the extra A-mode line.
5. **Tests:** the helper (deletion, replacement, punctuation, text edited since verification, no-diff); row height including the A-mode line.
6. **Capture:** build a `/tmp` copy of EP03 whose `stress.json` marks one turn as failed, with its `heard` text missing two words. Capture `--select <that turn> --edit-mode row` and `--edit-mode panel` to `plans/Native_UI/evidence/failed-marks-{A,B}.png` and describe both. Add the results to a new `plans/Native_UI/evidence/failed-marks-findings.md`.
