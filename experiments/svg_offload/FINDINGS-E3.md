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

## Round 3: condition C, archify layout via `ArchDiagram` (2026-11-15)

Pre-registered in `plans/Archify_Spike/brief.md` before any data: 4 node-and-edge cues (doctrine-chain, practice-network, router-tree, comparator-loop), condition C with Codex gpt-5.6-terra against the existing condition-B Terra cells. Adoption bar: C first or tied first on at least 2 of 4, none unusable.

Result: **null**. In a blind X/Y ranking (`exp_C/blind/`, key in `exp_C/blind/key.txt`), the author picked Y, X, X, X (doctrine-chain, practice-network, comparator-loop, router-tree), which the key resolves to B on all 4 cues. C placed first or tied first on 0 of 4, so it is below the bar. Ranking in `author_rank_C.csv`. `ArchDiagram` stays a kit specimen. It does not become a routing lane.

Caveats: C was rendered on the current book HEAD. B's four diffs no longer apply there, because the scene-note fixes touched the same scenes, so B used its 2026-11-06 frames. The surrounding scene therefore differs slightly. The harness `tsx_typecheck` read "fail" for all C cells, but that is an environment artifact: the fresh worktrees have no `node_modules`, and `tsc` cannot resolve React (6,123 errors on the unmodified stage). All four C cells rendered.

Orchestrator observations, recorded after the ranking:
- C diagrams were mounted at roughly their archify size inside large scene boxes, so in three cues (practice-network, router-tree, comparator-loop) the labels were too small to read at video size. This was judged by eye, not measured. The 12px floor in `archify-layout.mjs` is calibrated to the 1700x700 specimen and is not enforced at mount size.
- practice-network node labels were truncated ("schoolin", "employme").
- comparator-loop's C diagram sat under the scene's existing UPDATE label instead of replacing the routed wires.
- B drew each asset to the scene's composition (curved funnel wires, orbit circles, bracketed comparator). Archify's grid layout cannot express those.

If revisited: enforce the readable-size floor at the mount box (fail when scale x font < 12px), and limit C to plain box-and-arrow cues. Do not spend more on it without a cue where layout correctness, not styling, is the hard part.
