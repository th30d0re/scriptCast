# Seeded session (compacted from the round-3 worker)
You continue the round-3 worker's session. Its own final-state summary follows; trust it as your memory, then do fix round 4 below. Background files: plans/Archify_Spike/brief.md and findings.md.

# Final state: Archify fix round 3

Implemented round 3 after opening both r2 evidence stills. No commits made.
Props remain `ArchDiagram({layout: ArchLayout, box: {x,y,w,h}, progress})`.
Nodes: question, sources, claim, publish, check, revise.
Edges: collect, release, repair (text revise), synthesize, verify.

Adapter now stores `labelBox` as the FULL drawn padded background rectangle,
`labelAt` as text center-x/baseline-y, and `labelSize` in source units. Do not
apply the old 8px-mask conversion in the renderer. Horizontal labels center
on the longest free interval of the selected segment after subtracting nodes
expanded by 8 rendered px at the 1700×700 specimen scale. Font fits IBM Plex
Mono 0.6em advances plus 6 source px padding; minimum 12 rendered px, maximum
15 source px. Vertical verify keeps round-2 placement and font size. Guard
uses the exact drawn box against clearance-expanded nodes. Fixture passes;
constructed collect box only 1 source px from Question fails the guard.

DrawLine uses end="none", finishes by local edge progress .85. The one
overlay chevron and label fade in during .85–1; no primitive edits.

Touched this round: video/scripts/archify-layout.mjs,
video/src/stage/kit/components/ArchDiagram.tsx, components/types.ts,
video/src/stage/kit/fixtures/archdiagram.workflow.layout.json,
plans/Archify_Spike/findings.md, .session-log.md, .session-summary.md.
All earlier dirty changes remain in place, including condition C and specimen.

Checks: fixture regeneration succeeds; TypeScript passes; pytest has 27 passes
and only the two permitted Muse trust.json permission failures. See findings
Fix round 3 table for fonts/centers/clearance (minimum 8px, fonts >=14.5px).
Open verification: orchestrator must render final and mid-entrance stills.
Scale contract: 8px clearance/12px floor are calibrated to 1700×700; smaller
mount boxes uniformly reduce them. Architecture labels without workflow
label metadata remain a pre-existing limitation, outside this fixture polish.
Prohibitions continue: no vendor/book/shared primitive edits, no rendering,
no review/, .claude/launch.json, uv.lock, router_log.jsonl, video/tools/ or
experiments/svg_offload/exp_* edits. Do not commit session log/summary.

---

# Archify spike, fix round 4: labels must clear their own arrowhead

Fix round 3 is merged into dev (`9b61c89`); this worktree is that branch. Same prohibitions and owned files as before.

## Author feedback
"We're clipping the arrow on the horizontal arrow ones, including the release. So it needs to be moved a little bit back or shrunk so that it doesn't clip its own arrow."

Orchestrator check on the round-3 render (zoomed final frame): on `collect`, `synthesize` and `release`, the upper arm of the terminal chevron rises into the bottom-right of the label. The label's background patch hides part of the chevron arm. `revise` sits close to the same problem. Node clearance (round 3) is fine; this rule is about the edge's own arrowhead.

## Rule to add (in `ArchDiagram` and in the adapter guard; both use the same geometry)
1. Compute each edge's chevron bounding box from the exact chevron geometry the component draws (tip at the final routed point, both arms). Treat that box like a node: the padded label box must keep at least 8px rendered clearance from it, as with nodes. This also applies to every other edge's chevron, not just the label's own.
2. Free-interval placement as in round 3, with the chevron's clearance region also subtracted. The label moves back along its segment, away from the arrowhead, then shrinks to fit (12px floor). Below the floor, the adapter fails naming the label, its edge and the space available.
3. The label must not overlap its own line either. For horizontal segments, the bottom of the padded box stays at least 4px rendered above the line.
4. Vertical `verify` already clears; leave its look unchanged unless the new rule forces a change, and report it if it does.

## Acceptance
- `npx tsc --noEmit -p video` passes.
- `.venv/bin/python -m pytest tests/test_svg_offload_harness.py -q`: only the two Muse trust-file tests may fail.
- Fixture layout regenerates with exit 0. The guard fails on a constructed layout whose label touches its own chevron, and the error names the label and the chevron.
- Add a "Fix round 4" table to findings with one row per label: rendered font px, clearance to the nearest node, and clearance to the nearest chevron, all in rendered px.
- Rendering is the orchestrator's job.
