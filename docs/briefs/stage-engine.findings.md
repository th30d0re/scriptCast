# Stage engine findings

Implemented in the specified worktree without git commands, dependency installation, browser launch, or edits to existing cards, Episode.tsx, video.py or video_plan.py.

## Status and scope conflict

The engine, compiler, CLI, demo, browser site, and executable test runner are implemented. The exact done criterion `npm run check-stage` is **not met**: rule 1 excludes `video/package.json`, while deliverable 5 requires editing it. A scope question was submitted; no answer had arrived when these findings were written. The permitted runner passes directly:

```sh
node video/scripts/run-check-stage.mjs
```

The single pending package.json scripts entry is:

```json
"check-stage": "node scripts/run-check-stage.mjs"
```

Rule 1 also excludes `video/public/`, while deliverables request staging files there. No files were written there during this task. The compiler implements audio copying into public for subsequent authorized use, and that behavior was verified against a temporary public directory. The silent demo compiles without touching public. The site serves `/stage/default.json` through Vite middleware in development and emits it into `video/site/dist/stage/default.json` for builds, using the compiled demo plan by default. Existing public assets are copied into the site build by Vite.

## What was built

- `stage_plan.py`: phrase matching with punctuation/case normalization, occurrences, start/end timing, offsets, explicit turn-relative ms, global canonical and unique bare cue ids, complete absolute word data, shot windows including gaps, inclusive shot-id range selection, consecutive instance merging, media discovery/copying, validation, and warnings.
- `stage.py`: plan/render/per-shot/still/site commands; serialized `episode_dir` metadata keeps plans usable with `-o` elsewhere. All execution commands set `STAGE_EPISODE_DIR`; site also sets `STAGE_PLAN`. Render dry runs have no render/file side effects. The existing video module supplies browser discovery, frame chunk splitting and parallel chunk rendering. Whole-range chunked output uses a subsequent range-trimmed audio mux; per-shot output is muted.
- Shared absolute clock and plan contexts, cue/word hooks, deterministic scalar/vector motion and closed-form springs. Remotion sequences mount each segment once, and account for frame rounding when calculating the absolute clock. Both hosts use the same scene renderer and registry.
- Token context with existing theme defaults; static world-atlas SVG map, projections, highlights, marker/label and progressively drawn great-circle route; instrument shell with interpolated geometry/content; rail positions; cue reveal and spoken-word highlight.
- Vite site with scroll clock, sticky scaled viewport, shot index, scroll speed setting, audio clock, scrubber/ticks, keyboard controls, notices and visible missing-scene placeholders. Scene code contains no RAF, random calls, wall-clock reads or CSS transitions. RAF is used only in the browser audio host to sample audio.currentTime smoothly.
- Demo title and world-to-Portugal dashboard; two dashboard shots share a segment. Demo includes manifest, words, YAML and generated plan. It deliberately has no MP3; the plan explicitly reports silence.
- `@stage` points to `api.ts`, keeping Remotion registration side effects out of episode imports. `@episode` resolves to the episode folder, using the demo by default. Webpack and Vite pin runtime dependencies to video/node_modules. TypeScript paths use the demo registry as the build-time example.

## Real timing and audio findings

The real YT02v2_ov files were present and tested. Word times are **turn-relative**, not absolute. Manifest turn 1, `a57c5a84a6c02ce8`, starts at 13,500 ms. Its first word, “The”, has start_ms/end_ms 0/0; “common” runs 0–340 ms. Normalization adds 13,500 ms to those values. The first turn's `archives` cue resolves to 3,480 ms and its second `episode` minus 120 ms resolves to 7,000 ms.

The MP3 is actually `/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/YT02v2_ov/YT02v2_ov.mp3`, alongside episode_manifest.json and word_timings.json. The compiler checks the configured outputs directory first, then its parent.

## Interpretations and limitations

- `duration_ms` remains the complete shot-plan duration; `range` is the selected interval. Composition duration is derived from range. Cues remain globally available even when shots are range-filtered, preserving references to earlier cue anchors.
- `still --at` is a normalized shot fraction (0..1); the last frame is clamped inside the shot. Per-shot frame intervals are inclusive and relative to the composition range.
- Shared consecutive instance ids require the same scene; nonconsecutive uses mount separately. Shot turn starts must strictly increase.
- Timing mode is inferred from unambiguous manifest bounds across turns. Conflicting evidence raises an error. Completely ambiguous data defaults to relative (a zero-start turn has identical results either way).
- `Instrument.modeTimeline` transitions over 400 ms by default. `transitionMs` changes this. Explicit `transition: {from, to, progress}` supports controlled transitions between any modes. Interrupted timelines preserve current blended weights. Interactive hover/settings overrides are immediate and do not alter the episode clock.
- Map markers and labels use SVG children and `useProject`; other map overlays can use the same projection hook. Orthographic route paths clip through d3; custom marker children must implement horizon visibility if needed.
- External episode folders need an index.ts/index.tsx exporting `scenes`. Static builds use one registry selected by STAGE_EPISODE_DIR; `?plan=` can choose another plan using that registry, not dynamically import arbitrary new scene modules.
- @types/react-dom was not installed. A minimal typed declaration for the one used createRoot API avoids adding a package. No dependency was installed or required.
- Existing TS check scripts use CommonJS output, but d3-geo is ESM. The Stage runner compiles/bundles with the already-installed esbuild and executes the result with Node. Output stays in `video/scripts/.stage-check/`.
- No Chromium renders/stills or visual/manual QA were attempted, per the sandbox restriction. Orchestrator visual review remains necessary.

## Verification output tails

TypeScript, run from video/ against installed dependencies (`npx --no-install tsc --noEmit -p .`, equivalent to the requested project check):

```text
[no output]
exit 0
```

TS unit runner (`node video/scripts/run-check-stage.mjs`):

```text
PASS: easing/tween endpoints, vectors, spring convergence, cue errors, timeline transitions, Lisbon projections (3 types).
exit 0
```

Requested npm entry (`cd video && npm run check-stage`):

```text
npm error Missing script: "check-stage"
exit 1
```

Site production build (`cd video && npx --no-install vite build --config site/vite.config.ts`):

```text
vite v6.0.7 building for production...
✓ 180 modules transformed.
dist/index.html                     0.35 kB │ gzip:   0.25 kB
dist/stage/default.json             2.42 kB │ gzip:   0.68 kB
dist/assets/index-CP577h0J.css      0.86 kB │ gzip:   0.47 kB
dist/assets/index-9bvkQd02.js   1,050.60 kB │ gzip: 332.39 kB
(!) Some chunks are larger than 500 kB after minification.
✓ built in 743ms
exit 0
```

The static atlas contributes to the bundle-size warning; it is bundled locally and never fetched from a map service.

Python (`PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=$PWD /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests/test_stage_plan.py -q -p no:cacheprovider`):

```text
...............                                                          [100%]
15 passed in 0.13s
exit 0
```

Includes actual real-file normalization, range/instance behavior, phrase cues, audio staging, and CLI dry runs including chunk audio trimming.

Demo plan:

```text
wrote video/episodes/_demo/stage_plan.json
exit 0
```

Whole render dry run:

```text
STAGE_EPISODE_DIR=/Users/emmanuel/Documents/Theory/scriptCast-stage/video/episodes/_demo npx --no-install remotion render src/stage/index.ts StageEpisode /private/tmp/stage-demo.mp4 --props=/Users/emmanuel/Documents/Theory/scriptCast-stage/video/episodes/_demo/stage_plan.json --timeout=120000 --codec h264 --concurrency=8 '--browser-executable=/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
exit 0
```

Per-shot dry run produced S00-01.mp4 (`--frames=0-89`), S00-02.mp4 (`--frames=90-239`) and S00-03.mp4 (`--frames=240-323`), all with `--muted`, correct episode env and exit 0.

Still dry run:

```text
... remotion still src/stage/index.ts StageStill /private/tmp/stage.png ... --frame=165 ...
exit 0
```

Site dry run:

```text
STAGE_EPISODE_DIR=/Users/emmanuel/Documents/Theory/scriptCast-stage/video/episodes/_demo STAGE_PLAN=/Users/emmanuel/Documents/Theory/scriptCast-stage/video/episodes/_demo/stage_plan.json npx --no-install vite --config site/vite.config.ts
exit 0
```

Tool-location note: running npx from the repository root attempts registry lookup because dependencies live under video/. The initial root TypeScript invocation failed with `npm error ENOTFOUND`, resolving registry.npmjs.org/tsc. Use the video working directory as shown above; no installation is necessary.

Additional check history: the first TypeScript pass found the missing ReactDOM declaration and an incorrect explicit generic on Composition; both were corrected before the passing check. Root invocations with video/node_modules/.bin prepended to PATH still failed with `ENOTFOUND registry.npmjs.org/tsc` and `ENOTFOUND registry.npmjs.org/vite`; npm resolves packages by workspace, so PATH does not fix this. The successful checks use cwd=video.

Final site adjustment resets scroll position when entering Play mode, preventing a viewport left offscreen after a long scroll. Revalidation:

```text
TypeScript: no output, exit 0
Vite: ✓ 180 modules transformed.
dist/assets/index-RKiNLPhn.js   1,050.63 kB │ gzip: 332.40 kB
✓ built in 923ms
exit 0 (same atlas bundle-size warning)
```
