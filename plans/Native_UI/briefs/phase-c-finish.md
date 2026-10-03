# Phase C — finish (follow-up brief)

You are finishing Phase C in this worktree (branch `ui/phase-c`). The previous implementer (Codex) built almost everything and ran out of quota before its final steps. **First read `plans/Native_UI/briefs/phase-c.md` in full: every prohibition there applies to you unchanged.** In short: no git writes; write only inside this worktree, `~/.cargo`, `ui/target/` and `/tmp/scriptcast-c/`; the real project `/Users/emmanuel/Documents/Theory/TheOriginalPower/` is read-only; never run Precision insert, Regenerate or Repair; never launch Live or Logic; no Python changes; muted captures; no Figma.

## State you inherit (measured by the orchestrator)

- `cargo build --release` and `cargo test` pass: 31 + 16 + 15 + 6 + 4 + 4 + 3 + 1 tests, with some ignored scratch-clone tests.
- The scratch clone already exists at `/tmp/scriptcast-c/outputs/ATO_EP03_five_levels`. Baselines of the real files are in `/tmp/scriptcast-c/real-before.sha` and `real-before.status`.
- Checkpoint timing is measured: a Precision-insert scope of 396 files and 365 MB took 0.094 s.
- `cargo clippy --all-targets -- -D warnings` **fails**: `possible_missing_else` at `src/jobs.rs:461`.
- **About 30 lines are minified**: hundreds of characters each, with whole blocks on one line. `rustfmt` cannot reflow them: `src/jobs.rs:420-464` (the confirmation and restore dialog, and the checkpoint list) and `src/main.rs:1668-1722`. Find them with `awk 'length>120{print FILENAME":"FNR}' src/*.rs`.
- The findings file `plans/Native_UI/evidence/phaseC-findings.md` is stale: it still says "implementation pending".
- No live checks or captures were done.

## Do

1. Rewrite the minified regions as normal, readable Rust that matches the surrounding code's style, with the **same behaviour**:
   - break them into small helper functions where that reads better;
   - fix the clippy lint;
   - afterwards, `awk 'length>120' src/*.rs` must print nothing except string literals you cannot split, and you must list any such line in the findings.
2. Run the acceptance block of `phase-c.md` and paste the output into the findings.
3. Run the live checks of `phase-c.md` on the existing clone:
   - Detect changes;
   - Verify via `--run verify`, using `SCRIPTCAST_PROJECT=/Users/emmanuel/Documents/Theory/TheOriginalPower`;
   - Restore of the Verify checkpoint, checked byte for byte;
   - the `--confirm verify` capture;
   - the `--confirm precision-insert` capture on the fixture;
   - the final real-file hash and `git status --porcelain` comparison against the saved baselines.

   Save captures as `plans/Native_UI/evidence/phaseC-*.png`. If Verify cannot run (GPU, model, environment), record the exact error and continue with the rest.
4. **Speaker-safe redraw (operator request 2026-10-03).** The operator hears a faint high-pitched whine between clips, only from the MacBook's built-in speakers. It is absent in headphones and absent in Live. The orchestrator measured EP03's WAVs and found no tone in their quiet frames; the app keeps one output stream open and feeds it silence between turns. So this is acoustic coil whine from rendering bursts, not audio. Fix:
   - add `audio::is_builtin_speakers(name: &str) -> bool`: case-insensitive, true when the name contains both "macbook" and "speakers" (the device is listed as "MacBook Pro Speakers");
   - while playing on built-in speakers, request repaints every 250 ms (new constant `SPEAKER_REPLAY_TICK` beside `REPLAY_TICK` in `main.rs`) instead of 50 ms. Every other output keeps 50 ms;
   - on built-in speakers, the turn switch during auto-advance must not trigger extra repaint requests beyond one frame. Check for animated scrolling or repeated `request_repaint` on selection change, and disable them for this case only;
   - unit-test the name check (built-in name, headphones, AirPods, external DAC, empty string) and the tick selection;
   - you cannot judge the whine itself; the operator listens.
5. Rewrite `phaseC-findings.md` to cover:
   - what was built;
   - the command resolution on this machine (the source each of `scriptcast`, `scriptcast-verify`, `scriptcast-repair` and `scriptcast-export` resolved to);
   - the checkpoint timing;
   - the acceptance output;
   - the live-check results with hashes;
   - the captures;
   - anything left undone.

   Write it **early** and update it as you go.

## Stop conditions

- If reformatting a region would change its behaviour in a way you cannot verify, leave the behaviour exactly as it is and say so in the findings.
- If any real-file hash or the git status differs from the baseline, stop immediately and report it.
