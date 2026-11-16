# Brief: Stage, a time-driven scene engine for video and web

Read all of this before writing code. Branch `v0.3-stage`, worktree
`/Users/emmanuel/Documents/Theory/scriptCast-stage`.

## Rules

1. Write only inside this worktree: `video/src/stage/`, `video/site/`, `video/episodes/_demo/`,
   `video/scripts/`, `video/remotion.config.ts`, `video/tsconfig.json`, `scriptcast/tools/stage_plan.py`,
   `scriptcast/tools/stage.py`, `pyproject.toml` (one console-script line), `tests/test_stage_plan.py`,
   `docs/briefs/stage-engine.findings.md`. Leave every existing card, `Episode.tsx`, `video.py` and
   `video_plan.py` behaviour unchanged.
2. No `git` commands. No `npm install`: every dependency is already installed (`d3-geo`,
   `topojson-client`, `world-atlas`, `vite`, `@vitejs/plugin-react`, their types). If you need one
   more package, stop and say so in the findings file.
3. You cannot launch Chromium in this sandbox, so `remotion render` and `remotion still` will fail.
   Do not retry them. The orchestrator renders. Your checks are `npx tsc --noEmit -p video`,
   `npx vite build` for the site, the TS unit tests, and `pytest tests/test_stage_plan.py`.
   Python: `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python`. That venv's
   editable install points at the *other* checkout (`../scriptCast`), so run everything here as
   `PYTHONPATH=$PWD <python> -m pytest …` and `PYTHONPATH=$PWD <python> -m scriptcast.tools.stage …`;
   the `scriptcast-stage` console script will not exist until the orchestrator reinstalls.
4. Every animation is a pure function of time. No `Math.random`, no `Date.now`, no CSS
   transitions or `requestAnimationFrame` inside scenes. The same ms must produce the same pixels
   in Remotion and in the browser.

## What Stage is

Today `Episode.tsx` places static cards on the turn clock. Stage replaces that for new work: a
scene is a React component that reads **episode-absolute milliseconds** from a clock context and
**named cues** (absolute ms, resolved from spoken words). Two clocks drive the same scenes:

- Remotion: ms comes from `useCurrentFrame()`. This produces the video.
- Browser site: ms comes from scroll position or from an `<audio>` element. This produces the
  scrollable website. Instruments there also respond to hover and click.

Episode content (scene components, copy, data) lives outside scriptCast, in the book repo. The
engine resolves it through an `@episode` import alias.

## Deliverables

### 1. Plan compiler (`scriptcast/tools/stage_plan.py`, console script `scriptcast-stage`)

Inputs: the episode's `episode_manifest.json` (turns with `turn_index`, `turn_id`, `start_ms`,
`end_ms`), its `word_timings.json` (`{"turns": {turn_id: [{text, start_ms, end_ms}, …]}}`) and a
`stage.yaml` in the episode directory:

```yaml
episode: YT02v2_ov
outputs: ../../../../outputs/YT02v2_ov     # relative to stage.yaml
fps: 30
width: 1920
height: 1080
shots:
  - shot_id: S00-01
    turn_id: 0edfe226e42abaa3
    scene: TitleOpen
    instance: open          # optional; consecutive shots sharing it stay mounted as one segment
    props: {title: "…"}
    cues:
      archives: {word: archives}
      tests:    {word: tests, at: end}            # at: start (default) | end
      ep1:      {word: episode, occurrence: 2, offset_ms: -120}
      late:     {ms: 9000}                        # turn-relative fallback
```

- Determine whether word timings are turn-relative or absolute by checking them against the
  manifest (turn 1, `a57c5a84a6c02ce8`, starts at 13500 ms in `YT02v2_ov`). Normalise to absolute
  and record which you found in the findings file.
- Word match: lowercase, strip punctuation on both sides, compare tokens. `word` may be several
  words (`"tripartite model"`), matched as a consecutive run. `occurrence` counts from 1 within the
  turn. A missing word raises an error that prints the turn's tokens.
- Cue ids are global: `<shot_id>.<cue>` is the canonical id, and the bare name is also exported if
  it is unique in the plan.
- Shot window: from its turn's `start_ms` to the next shot's turn `start_ms` (gaps belong to the
  earlier shot); the last shot ends at its turn's `end_ms` plus 800 ms.
- `--range S00-01..S00-06` keeps those shots and sets `range.start_ms/end_ms`.
- Output `stage_plan.json`:

```ts
type StagePlan = {
  episode: string; fps: number; width: number; height: number;
  duration_ms: number; range: {start_ms: number; end_ms: number};
  audio: string;                         // path under video/public, see below
  cues: Record<string, number>;          // absolute ms
  segments: {instance: string; scene: string; start_ms: number; end_ms: number;
             shots: {shot_id: string; turn_id: string; turn_index: number;
                     start_ms: number; end_ms: number; props: Record<string, unknown>}[]}[];
  words?: Record<string, {text: string; start_ms: number; end_ms: number}[]>;  // by turn_id, absolute
  warnings: string[];
};
```

- Copy or symlink the episode audio (`<outputs>/<episode>.mp3` sits one level up from the episode
  directory: check where it actually is) into `video/public/stage/<episode>/` and reference it
  relative to `public`.
- Subcommands of `scriptcast-stage`: `plan <stage.yaml> [--range] [-o]`, `render <plan.json> -o out.mp4`
  (shells out to `npx remotion render src/stage/index.ts StageEpisode`), `render --per-shot <outdir>`
  (one mp4 per shot named `<shot_id>.mp4`, using `--frames`, video only), `still <plan.json> --shot
  S00-01 --at 0.5 -o x.png`, and `site <plan.json>` (starts the Vite dev server). Each sets
  `STAGE_EPISODE_DIR` to the directory holding `stage.yaml`. Reuse the browser-executable,
  concurrency and chunking handling in `scriptcast/tools/video.py` rather than copying it. Each
  render subcommand supports `--dry-run`, which prints the argv.

### 2. Engine (`video/src/stage/`)

- `clock.tsx`: `ClockContext` with `{ms, fps, interactive: boolean}`; `useStageMs()`;
  `RemotionClock` (ms from frame, absolute to the episode: add the segment start); `ManualClock`
  ({ms}) for the site.
- `plan.tsx`: `PlanContext`; `useCue(id) → {at, since, on}`; `useCueProgress(id, durationMs,
  ease?) → 0..1`; `useWords(turnId)`. An unknown cue id throws with the list of known ids.
- `motion.ts`: easing functions (linear, cubic in/out/inOut, expo out, back out), `clamp`, `mix`,
  `progress(ms, from, to)`, `tween(ms, keys: {at: number; value: number | number[]}[], ease)` with
  vector support, and `settle(sinceMs, {stiffness, damping})`, a closed-form critically or
  under-damped response so springs stay deterministic.
- `StageEpisode.tsx` + `index.ts` (Remotion root registering `StageEpisode`, size and duration
  from the plan via `calculateMetadata`, plus `StageStill` for single frames). Audio is trimmed to
  `range`; frame 0 is `range.start_ms`. Each segment mounts once inside a `Sequence`, so a scene
  that spans several shots (the dashboard) keeps its state and camera across turns.
- `registry.ts`: `type SceneProps = {segment; shots; shotAt(ms)}`;
  `type SceneRegistry = Record<string, React.FC<SceneProps>>`. Scenes come from
  `import {scenes} from '@episode'`. An unknown scene name renders a visible red placeholder naming
  it and adds to a warnings overlay in the site; in Remotion it throws.
- `@episode` alias: in `remotion.config.ts` (`Config.overrideWebpackConfig`) and the site's
  `vite.config.ts`, alias `@episode` to `process.env.STAGE_EPISODE_DIR` (default
  `video/episodes/_demo`) and make modules imported from that directory resolve `react`,
  `remotion`, `d3-geo` and the engine from `video/node_modules` and `video/src/stage` (the episode
  directory has no node_modules of its own). Also export the engine as `@stage` so episode code
  imports `import {useCue, MapStage} from '@stage'`. Add both to `tsconfig.json` `paths`.

### 3. Primitives (design-neutral; colours and fonts from a `Tokens` object passed by the episode)

- `tokens.ts`: `type Tokens` (colours, font families, sizes, radii, stroke widths) and
  `TokensContext`; defaults from `video/src/theme.ts`.
- `MapStage`: world map with `d3-geo` and `world-atlas` (`countries-50m.json`, imported from
  node_modules, never fetched). Props: `camera: {center: [lon, lat]; zoom: number}` (the scene
  computes it with `tween` from cues), `projection` (`'naturalEarth' | 'equirectangular' |
  'orthographic'`), `highlight: {id: string; color?: string; amount: number}[]` by ISO 3166 numeric
  id, `graticule`, `children`. Provides `useProject()` returning `[lon, lat] → [x, y]` in screen
  space so markers, routes (great-circle arcs drawn progressively with a 0..1 `draw` value) and
  labels track the camera. SVG rendering; zoom 1 fits the world, zoom 8 frames a country like
  Portugal.
- `Instrument`: a panel shell with three modes, `compact | expanded | settings`, and a 0..1
  transition between any two (size, position and content cross-fade interpolated, no CSS
  transitions). Props: `mode` (controlled) or `modeTimeline: {at: number; mode}[]`; `compact`,
  `expanded`, `settings` render props; `anchor` rect for compact and expanded. When the clock is
  `interactive` (site), hover expands and a gear button opens settings, overriding the timeline.
- `InstrumentRail`: lays out compact instruments in named slots around a centre region
  (`topLeft`, `top`, `topRight`, `left`, `right`, `bottomLeft`, `bottom`, `bottomRight`) with gaps
  and safe margins, so a dashboard is a map plus a rail.
- `Reveal`: children fade and rise in on a cue, optional stagger; `WordHighlight`: renders a turn's
  text from `useWords` and highlights each word as it is spoken.

### 4. Site (`video/site/`)

Vite + React, built with `npx vite build --config video/site/vite.config.ts`. Loads a plan JSON and
its audio (path given by `?plan=` or a default copied into `video/public/stage/`). Two modes:

- **Scroll**: a sticky 16:9 viewport, scaled to the window width; the page height is the sum of
  segment durations at a fixed px-per-second (setting, default 120). Scroll position maps to ms.
  Segment titles appear in a thin side index that jumps to a shot.
- **Play**: an `<audio>` element drives ms; play/pause, a scrubber with shot ticks, keyboard
  space and arrows.

Both modes render the same `StageEpisode` content through `ManualClock` with `interactive: true`.

### 5. Demo episode and tests

- `video/episodes/_demo/`: `index.ts`, `stage.yaml` and two scenes, `DemoTitle` (Reveal on two
  cues) and `DemoMap` (MapStage tweening from world view to Portugal, ISO 620, on a cue, with one
  Instrument switching compact → expanded → compact on cues), plus a tiny synthetic manifest and
  word timings so the demo plan compiles without the book repo.
- TS tests in `video/scripts/check-stage.ts` (compiled like the existing `check-*.ts` scripts, add an
  npm script `check-stage`): tween and ease endpoints, `settle` converging, vector tween, cue lookup
  errors, `Instrument` mode resolution from a timeline, `MapStage` projection of Lisbon landing
  inside the viewport at zoom 8 centred on Portugal.
- `tests/test_stage_plan.py`: word match with punctuation and case, multi-word match, occurrence,
  `at: end`, offset, `ms` fallback, missing word error text, range trimming, segment merging by
  `instance`, relative-vs-absolute normalisation, and a run against the real files when present:
  `/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/YT02v2_ov/episode_manifest.json` and
  `word_timings.json` (skip if absent).

## Done means

`npx tsc --noEmit -p video`, `npm run check-stage` (in `video/`), `npx vite build --config
video/site/vite.config.ts` and `pytest tests/test_stage_plan.py` all pass; `scriptcast-stage plan
video/episodes/_demo/stage.yaml` writes a plan; `scriptcast-stage render … --dry-run` prints the
argv with `STAGE_EPISODE_DIR` set. Write `docs/briefs/stage-engine.findings.md`: what you built, the
word-timing normalisation you found, anything in this brief that was wrong or ambiguous, and every
check's output tail.
