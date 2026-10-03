# Phase C — repair and finish (Codex, second pass)

You built Phase C in this worktree and ran out of quota near the end. A different agent (Gemini) then attempted the finish brief (`plans/Native_UI/briefs/phase-c-finish.md`) and damaged the integration. **Every prohibition in `plans/Native_UI/briefs/phase-c.md` still applies:** no git writes; the real project is read-only; never synthesize; never launch Live or Logic; no Python changes.

## What the orchestrator measured at 15:13 (raw)

- `cargo build` fails. `src/main.rs` no longer contains `mod jobs;`, and `Browser` has no `jobs` field, so `src/jobs.rs` (rewritten by Gemini, last modified 14:37) is not compiled at all. `table::show` gained `multi` and `animated_scroll` parameters, but the call site passed 7 of its 9 arguments.
- The orchestrator added `SPEAKER_REPLAY_TICK` (250 ms) after `REPLAY_TICK` in `main.rs`, and patched the `table::show` call to pass `&self.jobs.selected` and `!on_builtin_speakers` and to call `self.multi_select(i, modifiers)`. Those lines fail only because the jobs wiring is gone. Keep their intent.
- **Your own pre-Gemini integration is preserved** as `ui/main.patch`: a diff of `ui/src/main.rs` against commit `359bf6b`, saved at 14:14. It contains `+mod jobs;`, `+    jobs: jobs::Jobs,` and the `multi_select` call sites. `ui/jobs.patch` is empty. There is no saved copy of your original `jobs.rs`.
- Gemini's claimed fixes are **unverified**. Treat each as a hypothesis:
  - a pipe-hang fix in `runner.rs` `Job::start` (it says the Verify child's inherited stdout/stderr kept the reader threads blocked after exit);
  - `audio::is_builtin_speakers` plus tests;
  - the 250 ms tick;
  - `animated_scroll` disabled on built-in speakers;
  - captures saved to `ui/phaseC-05-detect.png`, `ui/phaseC-06.png` and `ui/phaseC-precision-insert.png` (the wrong folder; move them to `plans/Native_UI/evidence/`);
  - "the Verify capture times out headlessly".
- About 30 untracked scratch files sit in `ui/`: `fix_*.py`, `format_*.py`, `revert_auto_run.py`, `test_fmt.rs`, `check_baselines.sh`, `*.patch`. Also `fix_main_ui.py` at the worktree root. Do not delete them; list them in the findings. The orchestrator will leave them out of the commit.

## Do

1. Restore the Phase C integration in `main.rs` from `ui/main.patch`, and reconcile it with the current `jobs.rs`, `runner.rs`, `table.rs` and `audio.rs`. Keep whatever of Gemini's changes is correct and verified.
2. Keep the speaker-safe redraw from the finish brief (item 4) working, with tests.
3. Make every long line readable. Afterwards, `awk 'length>120' src/*.rs` must print only unsplittable string literals.
4. Run the full acceptance block and the live checks from `phase-c.md` on the existing clone `/tmp/scriptcast-c/outputs/ATO_EP03_five_levels`, including Verify and the byte-for-byte Restore.
   - If the Verify capture really cannot complete headlessly, record the exact symptom, and prove the Verify job and checkpoint ordering another way (a test or the log).
   - Re-check the real-file hashes and `git status --porcelain` against `/tmp/scriptcast-c/real-before.*`.
5. Rewrite `plans/Native_UI/evidence/phaseC-findings.md` as the final report. Write it early. It must say which of Gemini's claims held.

## Stop conditions

Same as `phase-c.md`. If any real-file hash or the git status differs from the baseline, stop and report immediately.
