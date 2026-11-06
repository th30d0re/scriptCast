# Stage dashboard engine findings

Implemented the dashboard-engine upgrade in the permitted Stage, site, demo, script, package, and findings paths. No git commands or Chromium launches were used.

## What changed

- Added `flyTo(ms, keys, ease?)`, backed by `d3-interpolate`'s van Wijk `interpolateZoom`. Key timestamps remain authoritative. The d3 path controls inverse zoom and progress along the move; the returned geographic centre is constrained to the endpoint great-circle, including shortest-path longitude handling. `DemoMap` now uses `flyTo` for world to Portugal.
- Extended `MapStage` without changing its default rendering: optional engraved land, internal-border, stepped-graticule, screen-space rhumb-spoke, and SVG clip layers; Mercator projection; and a top-level `fit={{center, pxPerDeg}}` alternative to `camera={{center, zoom}}`.
- Fit semantics: equirectangular and Mercator use exact horizontal pixels per longitude degree at the equator. Natural Earth and orthographic use the local screen-space distance of one longitude degree at the requested centre. Legacy zoom still means zoom 1 fits the world.
- Added token-driven `CompassRose` (16 tips, two rings, north label) and `Route`. `Route` projects lon/lat waypoints, builds a deterministic Catmull-Rom/cardinal cubic path, approximates its length with a fixed 32 samples per segment, reveals it with dash offset, and supports selected waypoint markers.
- Installed and loaded IBM Plex Sans 400/500/600 and IBM Plex Mono 400/500 in both hosts. Remotion imports package CSS directly and delays rendering until `document.fonts.ready`; the browser host imports the same package CSS. Tokens now expose `fonts.sans` and `fonts.mono` and use Plex Sans as the default body/heading family.
- Added `MathText` and exported `parseMathText`. HTML output uses real `sub`/`sup` elements; SVG mode emits `tspan` runs with baseline shifts. The parser handles single-code-point and braced markup while preserving Greek text.
- Added `GlassPanel`, using the token's rgba glass fill, a 1 px token hairline, token radius, configurable padding, and deterministic `backdrop-filter`/`-webkit-backdrop-filter` blur.
- Added all five `Instrument.grow` directions. Expanded/settings target sizes are re-anchored to the matching compact corner (or centre), then existing time-driven weights interpolate the rect. Site hover/settings overrides use the same anchored geometry. The top-right demo instrument grows left-down.
- Expanded the Stage checks for the fly-to midpoint corridor and zoom bound, all required MathText strings plus superscripts, Mercator, fit scale, and growth anchoring.

No scene uses CSS transitions, `Math.random`, `Date.now`, or `requestAnimationFrame`.

## Installed package versions

All requested pins existed and were installed exactly:

```text
@fontsource/ibm-plex-mono@5.1.1
@fontsource/ibm-plex-sans@5.1.1
@types/d3-interpolate@3.0.4
d3-interpolate@3.0.1
```

The machine's default npm cache contains root-owned entries, so installation used `/tmp/scriptcast-npm-cache`. npm reported the repository's pre-existing audit summary (19 vulnerabilities); no audit fix or unrelated package change was attempted.

## Visual verification handoff

Chromium cannot launch in this sandbox, as stated in the brief. The CSS and production bundle contain `backdrop-filter`, but whether Remotion's specific headless Chrome build composites the blur correctly cannot be verified here. The orchestrator must render a frame with content behind `GlassPanel` and check the blur, hairline, alpha fill, font loading, fly-to framing, clipping, and instrument growth visually.

## Verification tails

TypeScript, using the installed `video/` toolchain from the repository root:

```sh
npm_config_cache=/tmp/scriptcast-npm-cache npx --prefix video tsc --noEmit -p video
```

```text
[no output]
exit 0
```

The literal root command `npx tsc --noEmit -p video` cannot resolve the child workspace's compiler in this checkout (there is no root package.json) and also encounters the machine npm-cache ownership issue. `--prefix video` makes npx use the pinned `typescript@5.9.3`; `./video/node_modules/.bin/tsc --noEmit -p video` also passed with no output.

Stage unit checks:

```sh
cd video && npm run check-stage
```

```text
> scriptcast-video@0.2.0 check-stage
> node scripts/run-check-stage.mjs

PASS: motion/fly-to, springs, cues, instrument growth, four map projections + fit, and MathText parsing.
exit 0
```

Site production build:

```sh
cd video && npx vite build --config site/vite.config.ts
```

```text
✓ 219 modules transformed.
dist/assets/index-DG4s5Jfv.css       11.36 kB │ gzip:   1.73 kB
dist/assets/index-crT9_P4K.js     1,056.49 kB │ gzip: 334.82 kB
(!) Some chunks are larger than 500 kB after minification.
✓ built in 854ms
exit 0
```

The known bundle-size warning remains attributable mainly to the locally bundled world atlas; the new package font files are emitted as local assets.

Complete Python suite:

```sh
PYTHONPATH=$PWD /Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q
```

```text
........................................................................ [ 66%]
........................................................................ [ 99%]
.                                                                        [100%]
217 passed, 5 warnings in 5.50s
sys:1: DeprecationWarning: builtin type swigvarlink has no __module__ attribute
exit 0
```

The five warnings are existing third-party deprecations from torch, SWIG/importlib, and misaki resource loading.
