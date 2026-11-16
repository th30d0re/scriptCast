# Archify spike, fix round 3 (polish before merge)

The orchestrator rendered your round-2 result. Stills: `plans/Archify_Spike/evidence/r2_final.png` (final) and `r2_mid.png` (mid-entrance). Open both first. Same prohibitions and owned files as before.

## Author feedback (verbatim intent)
The edge labels on the arrows clip over the boxes, and the author doesn't want that. There needs to be a rule that prevents it, shrinking the text if needed. The vertical `verify` label is fine. The `release` label should sit further left so it doesn't clip the `Publish` box. Everything else is good.

What the still shows: the label's dark background patch (not just its glyphs) cuts into the neighbouring node's frame or corner bracket. Examples: `release` at Publish's top-left corner, `collect` at Question's top-right corner, and `synthesize` and `revise` touching their nodes.

## Rule to implement (in `ArchDiagram` and the adapter guard)
1. **Clearance.** The full drawn label box, including its background padding, must keep at least 8px (rendered, at the specimen scale) from every node box. That means the frame and the corner brackets, which extend past the rect.
2. **Placement.** Centre each label on the free part of its routed segment: the stretch of line that is outside every node box plus that clearance. `release`, for example, moves left along its horizontal run away from `Publish`. For a horizontal segment, the label sits just above the line, as now. Vertical segments keep the current `verify` treatment.
3. **Shrink to fit.** If the label still does not fit the free stretch, reduce its font size until it does, down to a floor of 12px rendered.
4. **Below the floor.** If it still does not fit at 12px, the adapter fails with an error naming the label, its edge and the space available. It must never draw an overlapping label.
5. **Guard.** The adapter's overlap guard must use the same box (padding and clearance included). Show it fails on a constructed bad layout and passes on the fixture.

## Also (the mid-entrance still)
6. An arrowhead and its edge label must not appear before their line has finished drawing. In `r2_mid.png`, faint chevrons and labels show at the ends of lines that are still undrawn, and a drawing line briefly shows two arrowheads. The overlay chevron and the label should fade in only once that edge's line reaches its end, and only one arrowhead should be visible at a time.

## Acceptance
- `npx tsc --noEmit -p video` passes.
- `.venv/bin/python -m pytest tests/test_svg_offload_harness.py -q`: only the two Muse trust-file tests may fail.
- Fixture layout regenerates with exit 0.
- Add a "Fix round 3" section to findings with a table per label: rendered font px, clearance to the nearest node in rendered px, and the label's centre x/y. Every clearance must be 8px or more.
- Rendering is the orchestrator's.
