# E3 / R5 — SVG offload cue candidates (YT02 stage)

Fifteen vector-heavy cue assets from the YT02 stage scenes. The operator picks
ten; `run.py --cues` takes the selection. Constraints applied: diagrams, icons,
network-node assets only — no people (excludes `PersonMark`/`PersonIcon` usage
and the stick figure inside `UnequalTreatment`'s "refusing" tile) and no
typeset equations (plotted chart curves are admitted; rendered math is not).

Paths are relative to the stage root:
`/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/yt02/stage/`
Cue ids resolve in `stage_ov.yaml`. Line numbers point at the first SVG
element of the asset.

## 1. book-stack
- **Scene:** `firstminute/TitleOpen.tsx:33` — cue `title` (shot S00-00)
- **Asset:** the book *The Original Power* as a layered icon: four offset page
  rects, cover rect with gold stroke, inner frame, title rule.
- **Why vector-heavy:** five stacked rounded rects with deliberate offsets plus
  stroke/fill layering — pure geometric composition, no text layout.

## 2. doctrine-chain
- **Scene:** `minutetwo/SenseTwo.tsx:54` — cue `t8Determine` (shot S00-08)
- **Asset:** the doctrine's claimed causal chain — four boxed nodes joined by
  marker arrows, inside a dashed rounded boundary with two orbit circles and a
  caption chip.
- **Why vector-heavy:** node-and-arrow flow diagram with `<marker>` defs,
  dashed container, and repeated box geometry.

## 3. practice-network
- **Scene:** `minutetwo/SenseThree.tsx:56` — cue `t9Practices` (shot S00-09)
- **Asset:** a system-of-practices network: five rounded nodes connected by
  dashed-in wires (`data-network-wire`), plus a two-bar comparison meter.
- **Why vector-heavy:** genuine node/wire graph — many positioned nodes and
  edges, exactly the topology a vector model must keep consistent.

## 4. debate-funnel
- **Scene:** `min3_7/a/HeartsAudit.tsx:48` — cue `t13Audit` (shot S00-13)
- **Asset:** public-debate funnel polygon narrowing into an audit-beam
  triangle, with three ordering text-rule strokes.
- **Why vector-heavy:** multi-segment polygon path plus a triangular beam
  region — shape construction with no typography.

## 5. router-tree
- **Scene:** `min3_7/a/HeartsAudit.tsx:90` — cue `t14Default` (shot S00-14)
- **Asset:** three lines routed through elbow (quadratic-curve) joints into a
  single router output.
- **Why vector-heavy:** routing topology — parallel paths that must converge
  cleanly; path continuity is the hard part for a generator.

## 6. group-boundary
- **Scene:** `min3_7/b/APAGroupDefinitions.tsx:66` — cue `t19Ingroup` (shot
  S00-19)
- **Asset:** ingroup boundary circle over a membership-token lattice, with an
  exterior shading mask, identification ring, and detachment/competition/
  opposition arrows leaving the boundary.
- **Why vector-heavy:** mask defs, a 12-token grid, and multiple directed
  arrows crossing one boundary — the most node-dense asset in min3_7/b.

## 7. compressed-bus
- **Scene:** `min3_7/c/HostWetware.tsx:66` — cue `t26Bandwidth` (shot S00-26)
- **Asset:** the wetware compression bus: four input sockets (stroked
  circles) feeding a horizontal spine, a vertical riser, and three output
  channels fanning out to cards.
- **Why vector-heavy:** bus topology with fan-in/fan-out; sockets, spine, and
  channels must stay aligned as one network.

## 8. sorting-bands
- **Scene:** `min3_7/c/UsThemSorting.tsx:125` — cue `t24Bands` (shot S00-24)
- **Asset:** us/them sorting field: two rounded band frames, a dashed alliance
  link, a quadratic trust bridge, a red threat arc, a belonging wrap, and a
  self-ring ellipse.
- **Why vector-heavy:** six interacting primitives (rects, ellipse, curved
  bridges) with paired symmetry — relational geometry a generator must keep
  coherent.

## 9. term-glyphs
- **Scene:** `min3_7/c/BiasDefinition.tsx:39` — cue `t28Categories` (shot
  S00-28)
- **Asset:** the four 48px term icons: tilt line with dot (bias), two linked
  circles (prejudice), arrow (discrimination), arrow into a bar (act).
- **Why vector-heavy:** an icon *set* — one generator must hold stroke weight,
  grid size, and visual language constant across four glyphs.

## 10. comparator-loop
- **Scene:** `min3_7/d/PredictiveCoding.tsx:64` — cue `t30Compares` (shot
  S00-30)
- **Asset:** predictive-coding comparator: an axis, a stroked comparator
  circle, and three routed wires (prediction in, evidence in, update out) plus
  a comparison bracket pair.
- **Why vector-heavy:** closed-loop routing diagram; wires must visibly enter
  and leave the comparator node. Chart-free, equation-free.

## 11. pulse-network
- **Scene:** `min3_7/d/InstalledRacialPrior.tsx:150` — cue `t33Institutions`
  (shot S00-33)
- **Asset:** institutional pulse network: a dashed pulse path with node
  circles travelling it, over a field of twelve stereotype-token squares.
- **Why vector-heavy:** node points constrained to a path plus a repeated
  token grid — counting and spacing discipline in one asset.

## 12. bayes-curves
- **Scene:** `min7_12/e/DocumentedExpectation.tsx:346` — cue `t35Updated`
  (shot S00-35)
- **Asset:** expectation-update chart: axis with tick, prior curve, dashed
  evidence curve, posterior curve, and a dashed start leader. Plotted curves,
  no typeset math.
- **Why vector-heavy:** three overlapping smooth paths that must read as
  distinct distributions — curve-shape control is the whole task.

## 13. dual-rails
- **Scene:** `min7_12/e/ExplicitImplicitChannels.tsx:212` — cue `t39Implicit`
  (shot S00-39)
- **Asset:** explicit/implicit channel lanes: a gold explicit rail with
  in/out spurs, a red implicit rail with three node dots, and two dashed meter
  leads.
- **Why vector-heavy:** two parallel routed lanes with node markers and
  leader lines — lane separation and node placement are generator-hard.

## 14. taxonomy-panels
- **Scene:** `min7_12/g/ThreePartTaxonomy.tsx:179` — cue `t54Stereotyping`
  (shot S00-54)
- **Asset:** Fiske's three-part taxonomy as three glyph panels: dashed-linked
  box chain (stereotyping), dashed target circle (prejudice), arrow over a
  gold tick (discrimination).
- **Why vector-heavy:** three small diagrams that must stay stylistically
  unified while encoding three distinct ideas — a set-consistency test.

## 15. speech-bubble
- **Scene:** `min7_12/h/UnequalTreatment.tsx:641` — cue `t56Verbal` (shot
  S00-56)
- **Asset:** the verbal-exclusion speech balloon: rounded balloon path with
  tail, two dashed text rules, one vertical red slash.
- **Why vector-heavy:** single composite path (balloon + tail curve) with
  internal rule geometry — path-heavy, trivial to describe, easy to judge.

---

**Selection protocol:** the operator marks ten of the fifteen; pass them to
the harness as `run.py --cues slug1,slug2,...`. Until then the harness
defaults to candidates 1–10 (`DEFAULT_CUES` in `run.py`).
