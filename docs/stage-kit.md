# Stage kit

The Stage kit is the reusable visual language for Stage episodes. New scenes compose from `video/src/stage/kit` (available through `video/src/stage/api.ts`) instead of defining episode-local versions of these parts.

Phase 0 is a harvest of the approved YT02 implementation. Component markup, `data-*` attributes, element IDs, geometry, typography, and colours are retained. Import paths changed to point at the engine. Sample copy appears only in `KitGallery`.

## Layers

- **Theme** — `kit/theme/tokens.ts` is the YT02 token file and default kit theme. It retains `colors`, `dimensions`, `Tokens`, and `color()` with the approved token keys.
  The theme and kit barrels export the theme value as `KitTokens`, avoiding a collision with the engine's `Tokens` type in the Stage API.
- **Primitives** — border geometry, notch geometry, person marks, and the approved icon family.
- **Components** — reusable cards, headers, term treatments, source treatments, fields, gauges, plots, and the scene frame.
- **Motion** — the engine's existing `motion.ts`, `Reveal.tsx`, `MapStage.tsx`, and `GlassPanel.tsx` remain in place and are re-exported by `kit/index.ts`.

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

## Components

Absolute `box` props use `{x, y, w, h}` in the 1920×1080 scene coordinate system. Progress, emphasis, and state numbers are normalized from 0 to 1 unless noted.

| Export | Props | YT02 source |
| --- | --- | --- |
| `StageSceneFrame` | `segment`, `title?`, `source?`, `notch?`, `children?` | `min3_7/shared/StageSceneFrame.tsx` |
| `SourceLine` | `text`, `x?`, `y?`, `tone?` | `min3_7/shared/SourceLine.tsx` |
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
| `TermTriad` | `boxes`, `terms`, `states`, `sources?`, `bracketLabel?`, `progress`, `bracketBox?`, `pulses?`, `socketProgress?`, `layout?`, `bracketAccent?`, `id?` | `min7_12/shared/TermTriad.tsx` |
| `GroupDirectionField` | `box`, `marks`, `leftLabel`, `rightLabel`, `forwardProgress`, `reverseProgress`, `insideLabels?`, `boundaryProgress?`, `gutter?`, `id?` | `min7_12/shared/GroupDirectionField.tsx` |
| `Label` | `id`, `box`, `children`, typography/alignment options | `min7_12/shared/audit.tsx` |
| `Panel` | `id`, `box`, `opacity?`, `accent?`, `style?`, `children?` | `min7_12/shared/audit.tsx` |

The kit also exports the geometry and audit helpers colocated with those components, including `evidenceTokenSize`, `laneGeometry`, `lanePoint`, `questionHeaderFit`, `triadBlockLayout`, `triadBracketBox`, `groupFieldGeometry`, and `groupMarkPlacement`.

## Gallery

`KitGallery` and `KitGallery2` are 1920×1080, 90-frame Remotion compositions registered in `video/src/Root.tsx`. Each specimen receives its natural layout dimensions and is uniformly scaled to fit its labelled cell. The second sheet includes the world map and separate dashboard border specimens. The kit theme loads locally bundled IBM Plex Sans (including the display weights and italic) and Mono using the same Fontsource CSS imports as YT02 and the engine. Render review stills with:

```sh
cd video
npx remotion still src/index.ts KitGallery /tmp/kitgallery.png --browser-executable="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
npx remotion still src/index.ts KitGallery2 /tmp/kitgallery2.png --browser-executable="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
```

## Left in episode

- `dashboard/notch.ts`'s `MAP_SCENES`, `NOTCH_MS`, and `useNotch()` remain episode policy because they name YT02 scenes (`DashboardBoot`, `Dashboard1486`, and `RoadmapBoot`). The kit retains the visual `notchPath()` primitive; episode code computes and passes `notch` progress.
- `min3_7/a/PackageAVisuals.tsx`'s `AuditedTermBlock` remains in YT02 because it binds equation-term IDs, spoken-word pulse timing, and `ColoredTerm` from YT02's equation package.
- `min3_7/a/PackageAVisuals.tsx`'s scene helpers (`enter`, `SceneSource`, and package-specific layout/audit wrappers) remain in YT02 because they encode episode timing or duplicate the newer shared audit primitives. Its content-neutral icons and marks were harvested.
- YT02 scene/package files outside the named `shared` directories remain episode-local because they contain narration-specific words, cue names, scene timing, and composition logic.

## Composition rule

New Stage scenes import visual building blocks from the kit and compose them with episode data. Episode code owns narration copy, cue lookup, scene transitions, and policy such as which scenes collapse a border notch. Reusable styling or geometry belongs in the kit and must be reviewable in `KitGallery`.
