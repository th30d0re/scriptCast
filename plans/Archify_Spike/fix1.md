# Read first
This is a fresh session continuing earlier work in this worktree. Read `plans/Archify_Spike/brief.md` (original brief: prohibitions, facts, owned files) and `plans/Archify_Spike/findings.md` (what the previous session built), then `git status` and `git diff` to see the uncommitted implementation. Then do the fixes below.

# Archify spike, fix round 1

The orchestrator rendered the specimen (`render-kit-videos.mjs --only ArchDiagram`). Stills: `plans/Archify_Spike/evidence/arch_0.35.png` (mid-entrance) and `arch_0.98.png` (final). Open both images before changing anything. Same prohibitions and owned files as `brief.md`; additionally you now own `tests/test_svg_offload_harness.py` (the CONDITIONS assertion only) and `plans/Archify_Spike/evidence/`.

## Defects seen in the final still (arch_0.98.png)
1. **Box inside a box.** Each node shows the DrawRect frame and a second FitBox frame inside it. For "Gather sources" and "Evidence check" the inner FitBox frame spills out of the bottom of the node. A node must read as one box with its label inside it. Use FitBox props only (see `video/src/stage/kit/motion/FitBox.tsx` and `FitBox.opus.tsx`), or draw the label without FitBox if FitBox cannot go frameless; do not edit FitBox. Two-line labels must sit inside the node's height.
2. **Edge labels cover nodes.** "synthesize" covers the start of "Draft claim" (it reads "aft claim"); "collect" and "release" touch their neighbour nodes. Archify's workflow `labels` entries carry measured `x, y, width, height`; place labels from that box (not a guessed `labelAt`) and size the text to fit that width. Add an adapter check: if any edge-label box intersects a node box after normalization, the script exits non-zero naming the label.
3. **Missing arrowheads.** Only the vertical "verify" edge shows a readable arrowhead; the horizontal edges end in a small tick hidden near the label or under the target node. Every edge must end in a visible arrowhead that touches, not enters, the target node's border.
4. **Too small for video.** The specimen box is 1040x430, so node labels end up around 15px on a 1920x1080 frame. Make the specimen box fill the stage area (about 1700x700) and choose label sizes so node labels render at 22px or more and edge labels at 18px or more at that box. Record the final rendered px sizes (source size x scale) in findings.

## Also fix
5. Condition C in `experiments/svg_offload/run.py` hard-codes `/Users/emmanuel/Dev/Tools/scriptCast-w-archify/...`. That worktree is deleted after merge. Use the repo root derived in the harness (as other paths do) so it points at the main checkout `video/scripts/archify-layout.mjs`.
6. Update the `CONDITIONS` assertion in `tests/test_svg_offload_harness.py` to the three-tuple. Leave the two Muse trust-file tests alone (sandbox-only failure).

## Acceptance
- `npx tsc --noEmit -p video` passes.
- `.venv/bin/python -m pytest tests/test_svg_offload_harness.py -q` passes except the two Muse trust-file tests; list any other failure.
- Adapter label-overlap check: demonstrate it firing on a constructed bad layout and passing on the fixture.
- Update `findings.md` with a "Fix round 1" section: what you changed per defect, and what you could not verify (rendering stays the orchestrator's).

## Stop condition
If archify's label boxes themselves overlap nodes in its own output (not just after our font change), stop and report the numbers; that is an archify limitation, not ours.
