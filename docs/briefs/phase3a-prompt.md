You are implementing **Phase 3a** of `docs/briefs/v0.2-video-tooling.md` (scriptCast, branch
`v0.2-video-tooling`). Read the whole brief, then `docs/briefs/FINDINGS-v0.2.md` (Phases 1
and 2 are committed; append a `## Phase 3a` section). Phase 3a is the **card half** of
Phase 3: the `video/` workspace, safe-zone constants, data-driven card components with
G-10 and G-11 ported, and the `scriptcast-video still` command. The `Episode` assembly
composition and `render` subcommand are **Phase 3b, not yours**. Do not start Phase 4.

Rules:

1. Do NOT run `git commit`, `git add`, `git checkout`, `git stash`, or any history
   command. Leave changes uncommitted; the orchestrator reviews and commits.
2. Do not modify `/Users/emmanuel/Documents/Theory/TheOriginalPower`. Read-only.
3. Python tests: `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q`.
   Baseline **139 passed**; must stay green plus new tests.
4. **Network is enabled for this run only for `npm install` of these packages and their
   dependencies: `remotion`, `@remotion/cli`, `react`, `react-dom`, `typescript`,
   `@types/react`.** Nothing else. No telemetry setup, no signup, no API keys. Do not
   call any other host.
5. Remotion is free for individuals; if install or render asks for a license
   acknowledgement or payment, stop and report (brief stop condition).
6. Do not edit `README.md`. Add to the `[0.2.0]` entry in `CHANGELOG.md`. Add
   `video/node_modules/`, `video/out/`, `video/.remotion/` to `.gitignore` (create it if
   absent, do not disturb existing entries).
7. Fixtures and examples for tests must be neutral, except the two acceptance examples
   below (`video/examples/g10.json`, `g11.json`), whose copy is fixed by the brief and
   must be reproduced **exactly**.

## Deliverables

- `video/package.json`, `tsconfig.json`, `remotion.config.ts`, `src/index.ts` (registers
  compositions), no monorepo tooling. TypeScript strict.
- `src/safeZone.ts`: the constants from the brief (1080x1920; top 14%, bottom 35%, sides
  6%, bottom-right deeper region; usable box). Cards lay out **inside** the usable box
  (content may not sit in the top 14% or bottom 35%). This deliberately differs from the
  reference HTML, which is full-bleed and gets scaled in the edit; here the safe zone is
  built in.
- `src/theme.ts`: palette (navy `#0f1b33`, navy2 `#16264a`, cream `#f4ead2`, gold
  `#d9a441`, red `#e2573f`, teal `#6ec3c0`, mute `#9fb0cf`), typography (system stack
  `"Helvetica Neue", Helvetica, Arial, sans-serif`).
- `src/cards/`: `TimelineCard`, `StatBarsCard`, `TitleCard`, `CompareCard`. Each takes a
  typed props object from JSON: `headline`, its own items, `sources` (string), optional
  `svgAsset` slot (a path under `public/`, unused for now, Phase 4 fills it), and always
  renders the outlined 120x120 "QR" placeholder box under the sources line. Export a
  `Card` composition per component (id = the component name) at 1080x1920, 30 fps,
  1 frame.
- `video/examples/g10.json` (TimelineCard) and `g11.json` (StatBarsCard): ports of
  `/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/cards_html/g10.html`
  and `g11.html`. **Copy and numbers must match exactly.** Required strings, all must
  appear in the rendered markup: G-10: "The timeline they don't put in the press
  release", "Chapter 135 signed", "69 DAYS LATER", "Emergency preamble signed",
  "93,229 signatures submitted", "78,707 signatures verified", "Sources: Ballotpedia ·
  Foley Hoag LLP · AP / NBC Boston (Oct 2, 2024)". G-11: "Louisiana, 1898: a
  neutral-sounding cutoff", "130,344", "5,320", "(−96%)" (U+2212 minus), "164,088",
  "125,437", "(−24%)", "racially neutral on its face", "Guinn v. United States (1915)".
  Bar widths in StatBarsCard are computed from values against the group max (do not
  hardcode percentages).
- Python: `scriptcast/tools/video.py`, entry `scriptcast-video` in `pyproject.toml`, with
  **`still <card.json> <out.png> [--component NAME]`** that shells out to
  `npx remotion still` in `video/` with `--props`, and passes
  `--browser-executable` = `SCRIPTCAST_CHROME` env var if set, else
  `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome` if it exists, else
  Remotion's own. The component defaults from the JSON's `"component"` field. Unit-test
  the argument construction with a mocked `subprocess.run` (no Node in Python tests).
  `render` is **not** part of this phase; do not add it.

## Acceptance (run these, paste raw output into the findings file)

```bash
/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q
cd video && npm install && npx tsc --noEmit
```

Markup check (write a small script, e.g. `video/scripts/check-copy.tsx` run with
`npx tsx` or compiled with `tsc`, choose the lighter option and state it): render each
card with `react-dom/server` `renderToStaticMarkup` from its example JSON and assert
every required string above is present and that no element is positioned inside the top
14% or bottom 35% of the canvas (assert on your layout constants, not on pixels).

Renders (may be blocked by the sandbox if headless Chrome cannot launch; if so, say so
plainly and do not fake results, the orchestrator will run them):

```bash
cd video
npx remotion still src/index.ts TimelineCard /tmp/g10.png --props=examples/g10.json --browser-executable="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
npx remotion still src/index.ts StatBarsCard /tmp/g11.png --props=examples/g11.json --browser-executable="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
file /tmp/g10.png /tmp/g11.png     # both 1080 x 1920
# render each twice to /tmp/*_b.png; shasum of the pair must be equal
```

## Findings and final message

Update `docs/briefs/FINDINGS-v0.2.md` early and as you go (brief defects, decisions,
package versions installed, what you could not verify). End your final message with:
files changed, raw outputs of pytest / tsc / the markup check / the renders, and every
unverified item, including whether the installed `scriptcast-video` console script was
smoke-tested.
