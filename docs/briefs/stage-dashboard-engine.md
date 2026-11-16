# Brief: Stage engine upgrades for the runtime dashboard

Worktree `/Users/emmanuel/Documents/Theory/scriptCast-stage`, branch `v0.3-stage`. Read
`docs/briefs/stage-engine.md` (the original brief) and `docs/briefs/stage-engine.findings.md`
first; the same rules apply (deterministic scenes, no CSS transitions, no git), except:

- **Network is on** for this run. You may `npm install` exact pinned versions of these packages in
  `video/` and nothing else: `d3-interpolate@3.0.1`, `@types/d3-interpolate@3.0.4`,
  `@fontsource/ibm-plex-sans@5.1.1`, `@fontsource/ibm-plex-mono@5.1.1` (if a pinned version does not
  exist, use the nearest existing 5.x and say so).
- You may edit any file under `video/src/stage/`, `video/site/`, `video/episodes/_demo/`,
  `video/scripts/check-stage.ts`, `video/package.json`, `video/package-lock.json`, and write
  `docs/briefs/stage-dashboard-engine.findings.md`.
- Chromium still cannot launch in your sandbox; the orchestrator renders.

## Changes

1. **Fly-to camera.** `flyTo(ms, keys: {at, camera: {center, zoom}}[], ease?)` in `motion.ts`, using
   `d3-interpolate`'s `interpolateZoom` (van Wijk smooth zoom) between consecutive keys, so a
   world → Portugal move zooms out, pans, then zooms in instead of passing over the wrong region.
   Duration stays whatever the keys say. Update `DemoMap` to use it. Unit test: the camera centre
   at the midpoint of world → Portugal stays within the great-circle corridor between the two
   centres, and zoom at the midpoint is ≤ the larger of the endpoints.
2. **Engraved map style.** Extend `MapStage` with optional props (all token-driven, all default off
   so the demo is unchanged): `land: {fill, stroke, strokeWidth}`, `borders: {stroke, strokeWidth,
   opacity}`, `graticule: {step, stroke, strokeWidth, opacity}`, `rhumb: {centres: [lon,lat][],
   spokes: number, stroke, strokeWidth, opacity}` (lines radiating from each centre across the
   viewport, drawn in screen space from the projected centre), `clip: {x, y, width, height}` (an
   SVG clipPath so nothing draws outside it), `projection` adds `'mercator'`, and a `fit` mode that
   takes `{center: [lon, lat], pxPerDeg}` as an alternative to `zoom` (pxPerDeg at the equator for
   equirectangular; document what it means for other projections). Export a `CompassRose` SVG
   component (centre, radius, 16-point star, two rings, N label) and a `Route` component that draws
   a Catmull-Rom spline (`tension` prop) through projected lon/lat waypoints with a 0..1 `draw`
   progress (stroke-dasharray on the path length, computed deterministically) and markers at
   chosen waypoint indices.
3. **Fonts.** Load IBM Plex Sans (400, 500, 600) and IBM Plex Mono (400, 500) from the
   `@fontsource` packages in both hosts (Remotion: `staticFile`-free import of the CSS from the
   packages is fine, but Remotion must wait for fonts: use `delayRender`/`continueRender` around
   `document.fonts.ready`). Add `fonts.sans` and `fonts.mono` to `Tokens` defaults.
4. **Subscripts.** A `<MathText>` component that renders strings with `_x` and `_{xyz}` subscript
   markup (and `^` superscripts) as real `<sub>`/`<sup>`-styled spans in HTML and `tspan` with
   `baseline-shift` in SVG (`<MathText svg />`). Greek letters pass through. Unit test the parser:
   `ρ_τ`, `Φ_{load}`, `O_{racialized}`, `W = jψ_s`, `ψ_m + jψ_s`.
5. **Glass panel.** `<GlassPanel>`: a div with token fill (rgba), 1 px hairline border, radius,
   padding, and `backdrop-filter: blur(Npx)`; verify in the findings whether Remotion's headless
   Chrome renders backdrop-filter (you cannot run it; say the orchestrator must check).
6. **Instrument growth.** `Instrument` gets `grow: 'right-down' | 'left-down' | 'right-up' |
   'left-up' | 'centre'`: expanded and settings views anchor at the compact panel's corner and
   grow in that direction (interpolated rect, still deterministic). Hover expansion on the site
   respects it.

## Done means

`npx tsc --noEmit -p video`, `npm run check-stage` (with the new tests), `npx vite build --config
site/vite.config.ts` (in `video/`), and `PYTHONPATH=$PWD <venv python> -m pytest tests -q` all pass.
Findings file lists what changed, the package versions installed, and every check's tail.
