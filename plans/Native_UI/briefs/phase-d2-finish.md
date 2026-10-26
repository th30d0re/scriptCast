# Finish Phase D part 2 (inline-editing UI): verify, fix small issues, write the report

Working directory: this git worktree (branch `ui/phase-d2`). **Read `plans/Native_UI/briefs/phase-d2.md` first; all of its prohibitions apply to you.** In particular:

- no git writes;
- never start other agent CLIs, and never use `osascript` or drive other apps;
- the real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only, scripts included;
- no synthesis: never confirm Precision insert;
- mute captures.

Codex implemented the UI and ran out of quota before its final checks. Your job is to **verify and finish**, not to redesign.

## State you inherit (measured by the orchestrator)

- `cargo build --release`, `cargo test` (118 passed) and clippy are clean.
- One line is over 120 characters: `ui/src/jobs.rs:651`.
- New: `ui/src/editing.rs` (650 lines: `Draft`, worker loaders and saves, both editor views), plus edits across `inspector.rs`, `table.rs`, `timeline.rs`, `main.rs`, `jobs.rs` and `settings.rs`, and the `circle-alert-14-danger.svg` icon.
- `plans/Native_UI/evidence/phaseD2-findings.md` is partly written; "Implementation and validation" still says "In progress".
- Codex prepared a live copy under `/tmp/scriptcast-d2/live` (EP03 output, script, config, voices) with an export shim. Check that it exists before relying on it.
- No captures exist yet. **No live save, change check, or restore has been run.**

## Do

1. Fix the long line. Run `cargo fmt`.
2. Read `editing.rs` and the diff against `phase-d2.md`, items 1–11. List in the findings any item that is missing or only partly done. **Fix small gaps**, meaning under about 30 lines each. Report larger ones; do not implement them.
3. Run the live checks on the `/tmp` copy only:
   1. Open it via `SCRIPTCAST_PROJECT`.
   2. Capture `--select 6 --edit --edit-mode panel` to `plans/Native_UI/evidence/phaseD2-panel.png` and `--edit-mode row` to `phaseD2-row.png`.
   3. Save one edit to the **copy's** script through a test or debug path.
   4. Run `--changes-json` on the copy and show that the turn now appears.
   5. Restore the script-edit checkpoint and show `shasum` before and after (bytes back).

   Before and after everything, `shasum -a 256` the real EP03 script (`/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/podcasts/ATO_EP03_redefining_racism_five_levels.md`) and confirm it is unchanged.

   If a screenshot times out "waiting for the screenshot event", the display is asleep: record that and continue. Do not work around it.
4. Run the acceptance block from `phase-d2.md` and paste the output.
5. Rewrite `phaseD2-findings.md` so it is complete and honest:
   - what was built;
   - what you verified;
   - what is unverified;
   - remaining gaps;
   - captures;
   - hashes.
