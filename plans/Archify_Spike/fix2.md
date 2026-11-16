# Archify spike, fix round 2

Fresh session. Read `plans/Archify_Spike/brief.md`, `plans/Archify_Spike/fix1.md` and `plans/Archify_Spike/findings.md` (including "Fix round 1"), then `git status` / `git diff`. Same prohibitions and owned files as before.

## The fix-1 stop was a misread contract, not an archify limitation (orchestrator-verified)
In `--layout-json`, a workflow label's `x, y` are the **text anchor** (center x, baseline y), not a top-left corner. Evidence, fixture `video/src/stage/kit/fixtures/archdiagram.workflow.candidate.json`:

layout-json entry:
```
{"edge":"collect","label":"collect","x":205.8,"y":109,"width":43.6,"height":14}
```
archify's own rendered HTML for the same candidate (`archify render workflow ...`):
```
<rect x="184" y="99" width="43.6" height="14" rx="3" class="c-mask"/> <text x="205.8" y="109" class="t-muted" font-size="8" text-anchor="middle">collect</text>
```
So the box is `left = x - width/2`, `top = y - 10`, `width`, `height`. With that, `collect` spans 184-227.6, inside the gap between `question` (ends 180) and `sources` (starts 231.6). Re-run your overlap numbers with the corrected box; all four reported overlaps should disappear. Archify's own example `examples/agent-tool-call.workflow.json` shows the same pattern.

## Do
1. Fix the adapter's label-box conversion as above. Keep the overlap guard, now on the correct box; show it passes on the fixture and fires on a constructed bad layout.
2. Edge-label size: archify sizes edge labels at 8px source (node text 11px). Size each edge label to the largest that fits the free space along its edge segment between the two nodes (not archify's 8px box width), capped at the node label size. Report the rendered px for each fixture edge label at the 1700x700 specimen box. Target 16px rendered or more; if any falls below, report it rather than overlapping a node.
3. Keep everything from fix round 1 (frameless node labels, chevron arrowheads, specimen box, condition C path, test tuple).

## Acceptance
- `npx tsc --noEmit -p video` passes.
- `.venv/bin/python -m pytest tests/test_svg_offload_harness.py -q`: only the two Muse trust-file tests may fail.
- `node video/scripts/archify-layout.mjs workflow video/src/stage/kit/fixtures/archdiagram.workflow.candidate.json video/src/stage/kit/fixtures/archdiagram.workflow.layout.json` exits 0 and regenerates the committed fixture layout.
- Findings: add "Fix round 2" with the corrected contract and the per-label px table. Rendering stays the orchestrator's.
