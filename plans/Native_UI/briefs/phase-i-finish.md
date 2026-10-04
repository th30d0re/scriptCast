# Finish Phase I (find words / find turns / find & replace)

Branch `ui/phase-i`, this worktree.

## Rules

- No git writes.
- Never start other agent CLIs; never use `osascript`.
- `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only. Replace tests use `/tmp/scriptcast-i/` copies.
- No Python. No synthesis. Mute captures.
- Lines ≤120 characters, `cargo fmt`, keep comments, no colour or size literals outside `theme.rs`.
- **Send build and test output to files** (for example `cargo test > /tmp/scriptcast-i/test.log 2>&1`) and read only the tail.

## Spec

The spec is `plans/Native_UI/briefs/phase-i-find-replace.md`, Build items 1–4 and its Tests list. **Read only that file plus the code**; do not read `plan.md`.

## State (measured)

- Codex built `ui/src/find.rs`, `ui/src/find_replace.rs` and `ui/tests/find_replace.rs`, plus edits elsewhere, then ran out of quota before validating.
- `cargo build`, `cargo test` (157 passed) and clippy are green.
- `plans/Native_UI/evidence/phaseI-findings.md` says "Validation, capture … pending".

## Do

1. **Audit Build items 1–4 against the code.** Mark each as done, partly done, or missing in the findings.
2. **Fill the gaps.** Small gaps get the full fix. For gaps over about 150 lines, implement the smallest correct version and say so.
3. **Make sure these tests exist:**
   - matcher: case, whole word, regex, invalid regex, clip-tag exclusion;
   - wrap-around navigation;
   - Turns mode unchanged;
   - the replace planner, including skips inside clip tags;
   - `save_many`: LF and CRLF byte preservation, one checkpoint per batch, a hash conflict writes nothing, restore gives the original bytes.
4. **Live check, on the `/tmp/scriptcast-i` copy only:**
   - capture find "racism" in Words mode to `plans/Native_UI/evidence/phaseI-find.png`;
   - replace-all one word across several turns through a test or debug path;
   - run `--changes-json` and confirm those turns are listed;
   - restore the checkpoint and show the original bytes with `shasum`.
5. **Rewrite `phaseI-findings.md`** as the final report.
