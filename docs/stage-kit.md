# Stage kit

The Stage kit is the reusable visual language for Stage episodes. New scenes compose from `video/src/stage/kit` (available through `video/src/stage/api.ts`) instead of defining episode-local versions of these parts.

Phase 0 is a harvest of the approved YT02 implementation. Component markup, `data-*` attributes, element IDs, geometry, typography, and colours are retained. Import paths changed to point at the engine. Sample copy appears only in `KitGallery`.

## Layers

- **Theme** — `kit/theme/tokens.ts` is the YT02 token file and default kit theme. It retains `colors`, `dimensions`, `Tokens`, and `color()` with the approved token keys.
  The theme and kit barrels export the theme value as `KitTokens`, avoiding a collision with the engine's `Tokens` type in the Stage API.
- **Primitives** — border geometry, notch geometry, person marks, and the approved icon family.
- **Components** — reusable cards, headers, term treatments, source treatments, fields, gauges, plots, and the scene frame.
- **Motion** — deterministic animated primitives live in `kit/motion`. Canonical props are shared across Opus, Argon, Astra, Kimi, and selected Flash treatments exposed by each component's `variant` prop. The engine's existing `motion.ts`, `Reveal.tsx`, `MapStage.tsx`, and `GlassPanel.tsx` remain available.

## Motion

All timing is frame-derived and deterministic. Pass `progress` (0–1) for parent-controlled choreography, or `startFrame` and `durationFrames`. Variant implementations accept the canonical Opus prop type, so a builder can change the treatment without changing scene layout. Defaults are component-specific: Argon for `DrawRect`, `DrawLine`, and all `BootIn` modes; Opus for `Flicker`, `PowerDown`, `FitBox`, and `StickFigure`. For `BootIn`, `variant` selects the implementation and `mode` selects `snap`, `scan`, or `glitch`.

| Export | Canonical props | Variants | Use |
| --- | --- | --- | --- |
| `DrawRect` | `box`, timing, `corners?`, colour tokens, `strokeWidth?`, `notch?`, `seed?`, `intensity?`, `brackets?`, `variant?` | **argon (default)**, opus, astra, kimi | Argon for dual-corner construction, closure brackets, and seeded panel ignition. Opus when terminal heat and exact notch choreography matter. Astra is calmer; Kimi is compact and gated. |
| `DrawLine` | `points`, timing, colour tokens, `strokeWidth?`, `tail?`, `origin?`, `nodes?`, `end?`, `seed?`, `variant?` | **argon (default)**, opus, astra, kimi | Argon for a clean settled trace with a graduated hot wake. Opus when terminals, vertex ticks, and energized trails are important. Astra is restrained; Kimi is denser. |
| `Flicker` | `children`, timing, `seed?`, `intensity?`, `glowToken?`, `style?`, `variant?` | **opus (default)**, argon, astra, kimi | Opus for the house seeded ignition. Argon for an authored relay/dip/overshoot rotation with no spatial jitter. Astra is quieter; Kimi has a bounded CRT pulse. |
| `PowerDown` | Flicker props plus `box?`, `mode?`, `variant?` | **opus (default)**, argon, astra, kimi | Opus for CRT squeeze or in-place collapse. Argon supplies the matching voltage-sag, flyback, and ember shutdown. Astra is understated; Kimi is clean and bounded. |
| `FitBox` | `text`, `x`, `y`, timing, anchor/wrap/type/padding/token/tag/notch props, `variant?` | **opus (default)**, argon, astra, kimi, flash | Opus for loaded-font precision, tags, and typing. Argon for deterministic kit-metric measurement followed by border and text power-on. Astra favors explicit mono labels; Kimi is compact; Flash suits chassis badges. |
| `StickFigure` | PersonMark props, timing, `pose?`, `animate?`, `variant?` | **opus (default)**, argon | Opus for poseable figures (`stand`, `open`, `raise`). Argon for a compact 28×40 technical mark with sequential head/torso/limb tracing. Both preserve PersonMark anchoring and group colours. |
| `BootIn` | `box`, `children`, timing, `variant?`, `mode?`, seed/intensity/frame-token props | **argon (default)**, opus, astra, kimi | Argon is the default across `snap`, `scan`, and `glitch`, pairing its border and power envelope with reticle, raster, or slice-lock details. Opus is the established rotation; Astra is restrained; Kimi adds stronger texture. |

Helpers include `Trace`, `fitBoxLayout`, font measurement helpers, seeded flicker/hash functions, phase constants, cue generators, and the `MotionBox`/`MotionPoint` geometry types. The `KitMotion` composition is a 300-frame, 1920×1080 comparison reel; each 50-frame section labels the default and selected alternates side by side.

### Sound cues

`motion/sounds.ts` exports `motionSounds`, `soundCueTable`, and `defaultSounds`. Every cue is keyed as `component.variant.event` and contains `{frameOffset, sound, gain_db}` plus an operator note. Default cues follow the visual defaults: Argon for DrawRect, DrawLine, and BootIn; Opus for Flicker, PowerDown, FitBox, and StickFigure. Parent presets such as BootIn own their nested cues; do not also trigger the child DrawRect/Flicker cues.

Wrap narrated scenes in `<KitSound mode="beats">` to play only motion components marked `beat`; the default `mode="all"` ignores `beat` and preserves all existing cues. Use `beatCue` only when a beat needs a different sound-table component key: `<KitSound mode="beats"><DrawRect beat box={box} startFrame={30}/></KitSound>`.

Argon adds authored cues for DrawRect corner/notch/closure/fill events, DrawLine launch/turn/lock events, both Flicker directions, FitBox construction and text ignition, StickFigure limb stages, and each BootIn preset (`snap`, `scan`, `glitch`). The rotation table below remains a compact overview; use `motionSounds` for the complete Argon event list and exact offsets.

| Component | Opus events (default) | Astra events | Kimi events | Flash events |
| --- | --- | --- | --- | --- |
| DrawRect | corner start, edges run, lines meet, notch, seeded fill pulses/ignite/sag | corner start, lines meet, fill on | opposite corners ignite, four edges meet, fill on/pulse/dropout/settled | — |
| DrawLine | origin on, head leaves, measured vertices, arrive | head launch, head settles | head starts, vertex, path complete | — |
| Flicker | seeded pulses, ignite, sag | power on, sparse pulses, steady | first ignition, three pulses, steady output | — |
| PowerDown | flare, two dips, collapse, off | reverse-envelope decays, power off | power cut, last dropout, fully off | — |
| FitBox | border construction, optional tag, spaced key ticks, cursor out | enclosure/fill, clustered text pulses, steady | border/fill, text reveal, text settled | chassis draw, type reveal |
| StickFigure | head, limbs land | — | — | — |
| BootIn snap | border start/meet, content pulses/ignite | border/fill/power/content/settled | border closed, content reveal | — |
| BootIn scan | border/fill, scan sweep, scan lands | border/fill/power/content/settled | border closed, scan sweep, settled | — |
| BootIn glitch | border/fill, two tears, lock | border/fill/power/content/settled | border closed, glitch reveal, settled | — |

Offsets are relative to the component start at 30 fps and scale with custom duration. Structural events generally sit near −20 to −29 dB; texture is quieter, typically −30 to −40 dB. Use the exact generated table in code rather than copying offsets from this overview.

## Primitives

| Export | Props | YT02 source |
| --- | --- | --- |
| `CardBorder` | `notch?: number` | `dashboard/CardBorder.tsx` |
| `CompactBorder` | none | `dashboard/CompactBorder.tsx` |
| `SceneBorder` | `notch?: number` | `dashboard/SceneBorder.tsx` |
| `notchPath` | `(y0, dir, p)` | `dashboard/notch.ts` |
| `PersonMark` | `x`, `y`, `group`, `emphasis`, `scale` | `min3_7/shared/PersonMark.tsx` |
| `PersonIcon` | `tone?` | `min3_7/a/PackageAVisuals.tsx` |
| `ActionIcon` | none | `min3_7/a/PackageAVisuals.tsx` |
| `SystemIcon` | `tone?` | `min3_7/a/PackageAVisuals.tsx` |
| `WordIcon` | none | `min3_7/a/PackageAVisuals.tsx` |
| `HeartIcon` | none | `min3_7/a/PackageAVisuals.tsx` |
| `NumberBadge` | `value`, `active?` | `min3_7/a/PackageAVisuals.tsx` |
| `CausalArrow` | `x1`, `x2`, `y`, `p?`, `accent?`, `width?`, `id?` | `min3_7/a/PackageAVisuals.tsx` |
| `ScanLines` | `x`, `y`, `w`, `h`, `progress` | `min3_7/a/PackageAVisuals.tsx` |
| `QRCode` | `url`, `size`, `fg?`, `bg?`, timing | Crisp SVG QR with a two-module quiet zone; defaults to medium error correction. |

## Components

Absolute `box` props use `{x, y, w, h}` in the 1920×1080 scene coordinate system. Progress, emphasis, and state numbers are normalized from 0 to 1 unless noted.

| Export | Props | YT02 source |
| --- | --- | --- |
| `StageSceneFrame` | `segment`, `title?`, `source?`, `notch?`, `children?` | `min3_7/shared/StageSceneFrame.tsx` |
| `SourceLine` | `text`, `x?`, `y?`, `tone?`, `qr?` | `min3_7/shared/SourceLine.tsx`; pass a URL through `qr` to place a 96px QR beside the citation. |
| `TermBlock` | `term`, `summary?`, `glyph`, `state`, `accent`, `unit?` | `min3_7/shared/TermBlock.tsx` |
| `DefinitionCard` | `term`, `definition`, `source`, `paraphrased`, `progress`, `accent` | `min3_7/shared/DefinitionCard.tsx` |
| `CitationCard` | `authors`, `year`, `title`, `progress`, `compact?` | `min3_7/shared/CitationCard.tsx` |
| `UsThemSortingField` | `marks`, `partitionProgress`, `ingroupProgress`, `outgroupProgress`, `labels`, `mode` | `min3_7/shared/UsThemSortingField.tsx` |
| `NeuralField` | `layers`, `inputs`, `weights`, `activationProgress`, `compressionProgress`, `labels` | `min3_7/shared/NeuralField.tsx` |
| `PredictiveUpdatePlot` | `evidenceMean`, `priorMean`, `precision`, `showEquation`, `termHighlight`, `posteriorProgress` | `min3_7/shared/PredictiveUpdatePlot.tsx` |
| `PrecisionGauge` | `value`, `derived`, `label`, `lowLabel`, `highLabel` | `min3_7/shared/PrecisionGauge.tsx` |
| `TrainingTextStrip` | `tokens`, `patternKey`, `progress`, `emphasis` | `min3_7/shared/TrainingTextStrip.tsx` |
| `EvidenceToken` | `label`, `kind`, `progress`, `emphasis`, `box`, `sourceSocket?`, `id?`, `size?` | `min7_12/shared/EvidenceToken.tsx` |
| `SignalLane` | `box`, `label`, `progress`, `accent`, `direction`, `tokens`, `dashed?`, `bare?`, `id?` | `min7_12/shared/SignalLane.tsx` |
| `QuestionHeader` | `box`, `text`, `progress`, `accentTerm?`, `accent?`, `id?` | `min7_12/shared/QuestionHeader.tsx` |
| `TermTriad` | `boxes`, `terms`, `states`, `sources?`, `progress`, `pulses?`, `socketProgress?`, `layout?`, `id?` | `min7_12/shared/TermTriad.tsx` |
| `GroupDirectionField` | `box`, `marks`, `leftLabel`, `rightLabel`, `forwardProgress`, `reverseProgress`, `insideLabels?`, `boundaryProgress?`, `gutter?`, `id?` | `min7_12/shared/GroupDirectionField.tsx` |
| `Label` | `id`, `box`, `children`, typography/alignment options | `min7_12/shared/audit.tsx` |
| `Panel` | `id`, `box`, `opacity?`, `accent?`, `style?`, `children?` | `min7_12/shared/audit.tsx` |

The kit also exports the geometry and audit helpers colocated with those components, including `evidenceTokenSize`, `laneGeometry`, `lanePoint`, `questionHeaderFit`, `triadBlockLayout`, `groupFieldGeometry`, and `groupMarkPlacement`.

## Gallery

`KitGallery` and `KitGallery2` are 1920×1080, 90-frame Remotion compositions registered in `video/src/Root.tsx`. Each specimen receives its natural layout dimensions and is uniformly scaled to fit its labelled cell. The second sheet includes the world map and separate dashboard border specimens. The kit theme loads locally bundled IBM Plex Sans (including the display weights and italic) and Mono using the same Fontsource CSS imports as YT02 and the engine. Render review stills with:

```sh
cd video
npx remotion still src/index.ts KitGallery /tmp/kitgallery.png --browser-executable="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
npx remotion still src/index.ts KitGallery2 /tmp/kitgallery2.png --browser-executable="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
```

## Kit review

Open the site at `#kit` (or select **Kit review** in the native Stage column) to play every kit item and record notes, priority, status, and tags. Reviews save to `review/kit-review.json`; the dev-server endpoint merges each item independently and writes atomically. Use `scriptcast-stage kit-review export` to create `review/kit-review.md` for dispatch. The export includes only Change and Cut items, sorted P0 through P3, followed by items with no priority.

## Left in episode

- `dashboard/notch.ts`'s `MAP_SCENES`, `NOTCH_MS`, and `useNotch()` remain episode policy because they name YT02 scenes (`DashboardBoot`, `Dashboard1486`, and `RoadmapBoot`). The kit retains the visual `notchPath()` primitive; episode code computes and passes `notch` progress.
- `min3_7/a/PackageAVisuals.tsx`'s `AuditedTermBlock` remains in YT02 because it binds equation-term IDs, spoken-word pulse timing, and `ColoredTerm` from YT02's equation package.
- `min3_7/a/PackageAVisuals.tsx`'s scene helpers (`enter`, `SceneSource`, and package-specific layout/audit wrappers) remain in YT02 because they encode episode timing or duplicate the newer shared audit primitives. Its content-neutral icons and marks were harvested.
- YT02 scene/package files outside the named `shared` directories remain episode-local because they contain narration-specific words, cue names, scene timing, and composition logic.

## Composition rule

New Stage scenes import visual building blocks from the kit and compose them with episode data. Episode code owns narration copy, cue lookup, scene transitions, and policy such as which scenes collapse a border notch. Reusable styling or geometry belongs in the kit and must be reviewable in `KitGallery`.
