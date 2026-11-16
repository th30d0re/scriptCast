# R4 repair loop: first real run (2026-11-07)

Target: S00-13 (scene HeartsAudit, `stage/min3_7/a/HeartsAudit.tsx`), flagged by System 1: audit-beam p=0.98, debate-funnel 0.48, debate-panel 0.21.

What ran: `stage_repair.py --dispatch` with Codex gpt-5.6-terra, one combined brief, in a throwaway book-repo worktree; 21 HeartsAudit stills re-rendered there.

Result:
- The worker made no source change. It reported that HeartsAudit.tsx already matches the approved reference; the flagged still was an older render. The scene check `check_min3_7_a.mjs` passed.
- Verified by hand: rendering the current source at cue `t13Audit` (175.5 s, inside shot S00-13) passes S1 at 0.2, every element <= 0.04. The flagged still was stale, not a live defect.
- Loop verdict was `after: None`: the re-check looked for `S00-13.png`, but the renderer writes `<Scene>_<cue>.png`, and the critic's cue `end` means the scene end (199.6 s), outside the shot (171.6-180.5 s). Scoring `HeartsAudit_end.png` or `t13Intentions` against the reference fails at p=1.0 only because those frames show a different moment.

Gaps to fix:
1. Map a shot still to a rendered frame by timestamp (the reference's ms), not by cue name; re-check that frame.
2. The loop's rewrite of `episode_dir` in the worktree plan drops `/stage`, and ends up in the saved diff. Rewrite in place without changing the value's shape, and exclude it from the diff.
3. Before dispatching, re-render the flagged frame from current source; skip the worker when it already passes (would have saved this whole dispatch, ~180 s and ~1.07M tokens).
4. Attempt counting now ignores report-only rows (fixed, bce1eef+).
