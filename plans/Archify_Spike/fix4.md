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
