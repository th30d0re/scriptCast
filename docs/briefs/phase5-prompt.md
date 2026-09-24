You are implementing **Phase 5** in `/Users/emmanuel/Documents/Theory/scriptCast` (branch
`v0.2-video-tooling`, HEAD after commit "Fix clip seek"): a redesign pass on the Remotion card
components in `video/src/cards/`. Append a `## Phase 5` section to `docs/briefs/FINDINGS-v0.2.md`.

Rules:
1. Do NOT run `git commit/add/checkout/stash`. Leave changes uncommitted.
2. Do not modify `/Users/emmanuel/Documents/Theory/TheOriginalPower`. Read only. Never touch `video/public/episode/` (a render is using it right now) and never run `remotion render/still` or start headless Chrome (blocked in your sandbox; the orchestrator renders). No network. No API keys.
3. Python tests: `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q` (baseline 165 passed). TS: `cd video && npx tsc --noEmit && npm run check-copy && npm run check-quiver`.
4. Do not use `rm -rf`. Do not edit `README.md`; add to `[0.2.0]` in `CHANGELOG.md`.

## Problem (observed on real renders by the user and the orchestrator)

Cards are sparse and small: content sits in the top third of the ~950x883 safe box with the
rest empty, body text is tiny at phone size, and the art slot is a 64px strip, too small for a
rifle silhouette or a map. Also: (a) a card that omits an optional field inherits it from the
composition's `defaultProps`, which leaked a Louisiana quote from the G-11 example onto an
unrelated card; (b) a missing `svgAsset` file renders a broken-image icon; (c) overflow is
only caught by markup assertions, not by a real measurement.

Look at the current code first: `video/src/cards/{Frame,TimelineCard,StatBarsCard,TitleCard,CompareCard}.tsx`,
`video/src/{Root,safeZone,theme,Episode}.tsx|ts`, `video/scripts/check-copy.tsx`,
`video/examples/*.json`, `scriptcast/tools/video.py`, `scriptcast/tools/video_plan.py`.

## Deliverables

1. **Use the whole safe box, larger type.** Redesign `Frame` and the four cards so content is
   distributed over the full usable height and body text is at least ~30px, headline ~56px,
   detail lines ~28px, sources line ~22px (at 1080 wide; it is viewed on a phone). Compute a
   deterministic `density` scale in TypeScript from item count and total text length (shrink
   type stepwise for dense cards, never below 24px body, and never let it be so small the card
   is unreadable) so all existing examples still fit. Layouts stay in the shared safe-zone
   constants. No new colours; use `theme.ts`. Keep the QR placeholder box and sources line.
2. **Hero art area.** Add an optional prop on all cards: `art`: `{src: string; size?: "strip" | "hero"; caption?: string}`
   where `hero` is a large area (about 40% of the safe box height, full width, `objectFit: contain`)
   and `strip` is the current small strip. Keep `svgAsset` working as an alias for
   `art: {src, size: "strip"}` so `examples/g10_svg.json` and existing JSON still work. Same path
   validation as now (relative, under `public/`, `.svg`). When `hero` art is present, the card's
   text content flows in the remaining height.
3. **No default leakage.** Composition `defaultProps` in `Root.tsx` must contain only the minimum
   required fields with neutral placeholder text and none of the optional fields (`note`,
   `quote`, `attribution`, `lead`, `art`, `svgAsset`, ...). Verify by test that rendering a card JSON
   that omits an optional field produces markup without any text from another card's example. Add
   a check for this in `check-copy` (e.g. render StatBarsCard with only required fields; assert
   no "Louisiana", no "racially neutral").
4. **Missing assets fail fast, in Python.** In `scriptcast/tools/video.py`, before `still` and
   `render` shell out, resolve every `svgAsset`/`art.src` in the card JSON (and in the plan's
   cards) against `video/public/`; if any file is missing, exit non-zero with a clear message
   listing card and path (do not render). Add `--allow-missing-assets` to override (renders
   with the slot omitted; the component must render no `<img>` when `art.src` is empty/absent).
   Unit tests with mocked `subprocess.run`, neutral fixtures.
5. **Runtime overflow guard.** In `Frame`, after layout, if the content section's
   `scrollHeight > clientHeight` (or the card's content exceeds the safe box), throw an Error
   naming the card headline, so a render fails loudly instead of clipping. Implement with
   `useLayoutEffect` inside the browser only (skip during `renderToStaticMarkup`); use
   `delayRender`/`continueRender` if needed. Keep it simple and document that you could not run
   it (no Chrome), so the orchestrator will verify.
6. **Stat bars and compare cards are more informative.** `StatBarsCard`: allow an optional
   `sublabel` per item and per-item `color`, and remove the forced grey colour on single-item
   comparisons. `CompareCard`: allow `points` lists of up to 6 short lines per side and an
   optional `verdict` line under both columns. `TitleCard`: allow `items` with optional
   `emphasis` (a big-number line, e.g. "$100") and support 1 to 5 items. `TimelineCard`
   unchanged in props (keep G-10 copy exact) but scaled per rule 1.
7. **Examples and checks.** Keep `examples/g10.json`, `g11.json`, `g10_svg.json` required copy
   exact (existing check-copy assertions must still pass unless the assertion was about the old
   64px strip or old font sizes; update those with justification). Add neutral examples for the
   new features: `examples/hero_art.json` (CompareCard with `art.size: "hero"`, referencing
   `assets/example.svg`), `examples/title_emphasis.json`, and extend `check-copy` to render
   them. Update the safe-zone checks for the new geometry.
8. Update the Python `COMPONENTS`/validation in `video_plan.py` only if the new props require it.

## Acceptance (paste raw output into findings)

```bash
/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q
cd video && npx tsc --noEmit && npm run check-copy && npm run check-quiver
```

State plainly: renders were not run; the density scale and overflow guard are unverified in a
browser. End with files changed, raw outputs, and every unverified item.
