# E3 findings: inline SVG by worker (condition B), 2026-11-06

10 vector-heavy YT02 cues, 4 workers, each cell isolated in its own book-repo worktree. Local SVG generation (condition A) not run: no SVGen/OmniSVG weights on disk.
Rendered with `judge.py` (cue still + approved reference); zoomed comparison sheets in `judge/zoom/`. Ranking by the author (`author_rank.csv`).

| Worker | Produced a change | Author top placements (ties count for each) | Notes |
|---|---|---|---|
| Codex gpt-5.6-terra | 10/10 | **7** (debate-funnel, router-tree, group-boundary, compressed-bus, term-glyphs, comparator-loop, pulse-network) | most consistent |
| agy gemini-3.8-flash-low | 8/10 | **5** (debate-funnel, compressed-bus, term-glyphs, comparator-loop, dual-rails) | near-Terra quality on the cheapest quota |
| Codex gpt-5.6-luna | 10/10 | 2 (practice-network, comparator-loop) | one unusable result (compressed-bus) |
| Muse | 3/10 | 2 (doctrine-chain; pulse-network, slightly ahead of Terra) | best taste when it delivers; 7/10 errors or no change |

## Routing rule for `svg_asset` (inline)
- Default: Codex **gpt-5.6-terra**.
- Mechanical or budget-constrained: agy **gemini-3.8-flash-low** (ties Terra on half the cues).
- Do not route to Luna without a critic gate (it can produce an unusable scene).
- Muse: high quality but unreliable headless; use only as a second opinion on hero cues, never as the only worker, until its failure cause is fixed (harness should capture Muse stdout per cell).

## Not settled
- Automatic critic: MiniCPM-V returned schema-invalid output on 21/37 renders and found 0 defects in the rest, so it can't rank yet (R3 needs a stricter prompt/retry).
- Condition A (local generation) awaits model weights.

## Round 2: Muse rerun with output capture (2026-11-07)

Muse delivered 8/10 cues (pulse-network and dual-rails lost to quota exhaustion). Author ranking in `author_rank_round2.csv`. Muse rerun placed first or tied first on 4 of 8 (group-boundary alone, where all others were unusable; debate-funnel, term-glyphs, comparator-loop tied), was unusable on compressed-bus, made no visible change on router-tree, and looked unfinished on doctrine-chain. Terra placed first or tied first on 6 of 8; Luna was last or unusable on 4. Two cues (practice-network, comparator-loop) did not separate workers at all, so they carry no routing signal.

Routing rule unchanged: Terra default, Flash when cheap. Muse is worth a second-opinion cell on layout-heavy cues (it alone rescued group-boundary), not as the default; its output also varies run to run (doctrine-chain best in round 1, unfinished in round 2).
