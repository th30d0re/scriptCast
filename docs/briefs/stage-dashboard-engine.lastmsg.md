Implemented all Stage dashboard upgrades, including fly-to cameras, engraved map layers, Mercator/fit modes, CompassRose/Route, IBM Plex fonts, MathText, GlassPanel, and directional Instrument growth.

Verification:

- TypeScript: pass
- `npm run check-stage`: pass
- Vite production build: pass
- Python: `217 passed`
- No forbidden scene transitions, randomness, wall-clock reads, or RAF usage

The literal root `npx tsc -p video` cannot locate the child workspace compiler; `npx --prefix video tsc --noEmit -p video` passes.

Full details and check tails: [stage-dashboard-engine.findings.md](/Users/emmanuel/Documents/Theory/scriptCast-stage/docs/briefs/stage-dashboard-engine.findings.md)

Chromium remains unavailable, so the orchestrator must visually verify Remotion’s `backdrop-filter` rendering.