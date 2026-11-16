# Archify spike findings

## Plan

1. Re-run Archify's two supported layout exports and record their actual
   contracts.
2. Normalize each export into a small, renderer-neutral JSON shape, joining
   workflow node labels from the source candidate.
3. Draw that normalized shape with the existing kit primitives, fit uniformly
   into a stage box, and expose it in the kit sheet.
4. Add the E3 condition without changing A or B, then run the requested
   script/type checks.

## Re-verified layout exports

On 2026-11-15, the pinned CLI successfully exported both types.

- Workflow: `contract`, `viewBox`, `requiredViewBox`, `columns`, `nodes`,
  `edges`, `labels`, and `diagnostics`; view box `[1108, 684]`. Nodes contain
  `id`, lane/column, and `x/y/width/height`; edges contain routed `points`;
  labels reference an edge and provide `x/y/width/height`.
- Architecture: `ok`, `diagram_type`, `layout`, `viewBox`, `components`,
  `boundaries`, `connections`, and `labels`; view box `[1080, 588]`.
  Components contain the label and geometry; boundaries contain group geometry;
  connections contain routed points and optional `labelAt`.

One path in the brief was slightly wrong: the supplied examples live under
`archify/examples/`, not the vendor root's `examples/` directory. The emitted
shapes otherwise match the quoted facts.

## Decisions

`ArchDiagram` takes one `progress` value (0–1), rather than frame starts per
node. It keeps cue timing owned by the scene, matches the kit's composable
motion style, and permits deterministic group → node → edge staggering without
introducing scene-frame assumptions into layout data.

## Unverified

- Remotion rendering and visual legibility in the live kit page (the
  orchestrator owns rendering in this environment).

## Implementation result

- `video/scripts/archify-layout.mjs` exported the committed six-node workflow
  fixture and an architecture example successfully. A semantically invalid
  workflow caused Archify diagnostics and wrote no output file.
- The workflow fixture is `[989, 404]`; the specimen's `1040 × 430` scene box
  has a uniform scale of about `1.05`, so the adapter fits it without
  distortion.
- `npx tsc --noEmit -p video` passes. The E3 dry-run planner can produce A/B/C
  cells when the existing condition-A weight check is satisfied (verified by
  exercising the planner with that local availability predicate).

## Test notes

The existing harness test suite still has an assertion for the old two-item
`CONDITIONS` tuple, so that assertion necessarily fails after adding C. Two
Muse-cell tests also attempt to write `/Users/emmanuel/.config/muse/trust.json`,
which the workspace sandbox denies. The existing condition-A missing-weight gate
is intentionally preserved, including for dry-runs, to keep A/B behavior
unchanged; with weights absent, invoke the smoke dry-run for `B,C`, or run the
full A/B/C matrix on the orchestrator that has the A weights.

## Fix round 1

- Replaced `FitBox` node labels with frameless, centered mono text. `FitBox`
  always draws its own `DrawRect`, so it cannot supply an in-node label without
  the duplicate frame seen in the first render. Two-line labels use the node's
  inner height and `overflow: hidden`.
- The adapter now carries Archify workflow labels as their measured
  `labelBox: {x, y, w, h}`. It rejects a normalized layout when a label box
  intersects any node, naming both the edge label and node; the component uses
  that exact rectangle and reduces the font to its measured width.
- `ArchDiagram` retains `DrawLine end="arrow"` and overlays a 2px terminal
  chevron whose tip is the final routed point. Archify's final point is on the
  destination rectangle border, so arrowheads touch rather than enter nodes.
- Enlarged the kit-sheet specimen box to `1700 × 700`. The fixture's
  `989 × 404` layout scales by `1.718`; node source text is 15px, so it renders
  at **25.8px**, and edge text is up to 12px, so it renders at **20.6px**.
- Condition C now derives the main checkout's layout-script path from the
  harness `ROOT`, rather than embedding this disposable worktree path.

Rendering remains orchestrator-owned. I could not independently verify the
updated stills in this sandbox.

## Fix round 2

- Corrected the workflow-label contract in the adapter: exported `x, y` are the
  text's center anchor and baseline, rather than a box origin. The normalized
  guard box is therefore `{x: x - width / 2, y: y - 10, w: width, h: height}`.
  On the fixture, `collect` is `184–227.6 × 99–113`, wholly inside the
  `180–231.6` gap; the corrected guard passes. A deliberately constructed
  normalized layout with that box moved onto `question` fails and names the
  edge and node.
- Edge text now uses the longest size that fits the straight routed segment
  nearest Archify's label anchor, based on Archify's measured 8px advance,
  capped at the 15px node-label source size. It remains positioned from the
  corrected anchor/baseline rather than stretching to Archify's 8px label box.
- At the `1700 × 700` specimen scale (1.718), fixture edge-label rendered
  sizes are: `collect` 16.3px, `synthesize` 15.6px, `verify` 25.5px,
  `revise` 16.6px, and `release` 17.0px. `synthesize` is the only label below
  the 16px target: its 66-unit inter-node segment is the limiting space, so it
  is reported rather than allowed to overlap either node.

## Fix round 3

Inspected `evidence/r2_final.png` and `evidence/r2_mid.png`. The old guard
checked Archify's original mask, while the renderer enlarged that mask and
added padding. The adapter now computes the full drawn background rectangle
and the renderer uses it verbatim. Horizontal placement subtracts every node's
8px clearance region from the selected segment, centers in the longest remaining
interval, and finds the largest fitting font up to 15 source px. Padding is
3 source px horizontally and 2 vertically; IBM Plex Mono advances are 0.6em.
The minimum is 12 rendered px. Failure names the label, edge, and available
space and occurs before any output write. Clearance and floor are calibrated
to the requested 1700×700 specimen; other boxes scale the geometry uniformly.

| Label (edge id) | Rendered font px | Nearest node clearance px | Background center x/y (source) |
| --- | ---: | ---: | --- |
| collect | 14.853 | 8.000 | 205.800 / 104.680 |
| release | 17.740 | 8.000 | 699.073 / 103.840 |
| revise (repair) | 15.037 | 8.000 | 585.000 / 228.626 |
| synthesize | 14.522 | 8.000 | 396.600 / 104.776 |
| verify | 25.518 | 32.925 | 495.600 / 173.577 |

`release` moves left from anchor x=701.4 to 699.073. Vertical `verify`
retains its round-2 baseline, center x, and font size. Table centers describe
the actual padded rectangles; text uses the separately stored `labelAt` baseline.
The overlap guard expands every node by the specimen's 8px source equivalent
and checks these same rectangles. Fixture check exits 0; a constructed layout
placing `collect` just 1 source px beyond Question's right edge exits 1 and
names `collect` and `question` (a clearance failure without literal overlap).

The line now uses `end="none"`. Its drawing finishes at 85% of each edge's
local entrance; only then do the single terminal chevron and label fade in
over the final 15%. Shared primitives are unchanged.

## Fix round 4

The adapter and `ArchDiagram` now share the terminal open-chevron geometry
(8 source-px wings and 13 source-px depth). The adapter expands every
chevron's exact three-point bounding box by the same specimen-calibrated 8px
rendered clearance used for nodes, both when choosing a horizontal free
interval and when validating the finished padded label patch. For narrow
endpoint gaps, a horizontal patch is lifted only enough to clear the chevron;
it still stays at least 4 rendered px above its own line. This preserves the
12px rendered font floor while keeping the label patch from masking an arm.
A constructed `collect` patch touching its own chevron now fails the guard
with `intersects chevron collect`.

| Label (edge id) | Rendered font px | Nearest node clearance px | Nearest chevron clearance px |
| --- | ---: | ---: | ---: |
| collect | 14.853 | 8.000 | 8.000 |
| release | 17.740 | 8.000 | 8.000 |
| revise (repair) | 15.037 | 8.000 | 8.000 |
| synthesize | 14.522 | 8.000 | 8.000 |
| verify | 25.518 | 32.925 | 36.097 |

`verify` is unchanged from round 3: its baseline, center, and font remain
the same because it already cleared all terminal chevrons.

Fixture regeneration succeeds. TypeScript passes; harness tests report 27
passed and only the two expected Muse trust-file permission failures.
Rendering and visual confirmation of this round remain orchestrator-owned.
