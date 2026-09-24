You are implementing **Phase 4** of `docs/briefs/v0.2-video-tooling.md` (scriptCast, branch
`v0.2-video-tooling`): the Quiver AI "Arrow 2" client. Read the whole brief (Phase 4
deliverables) and `docs/briefs/FINDINGS-v0.2.md`; append a `## Phase 4` section. Phases 1 to 3b
are committed (HEAD 9052cbd).

Rules:

1. Do NOT run `git commit/add/checkout/stash`. Leave changes uncommitted.
2. Do not modify `/Users/emmanuel/Documents/Theory/TheOriginalPower`.
3. Python tests: `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q`
   (baseline **159 passed**, must stay green plus new tests).
4. **Network is enabled for this run only to `npm install @quiverai/sdk` (and its
   dependencies) inside `video/`.** Nothing else, no other host. **You have no API key and
   must never try to obtain, read from the environment, print, or write one.** Every call
   to `api.quiver.ai` is forbidden: tests and verification use recorded/hand-written
   fixtures and `--dry-run` only. Never write a key into any file.
5. Headless Chrome is blocked in your sandbox; do not attempt renders (orchestrator runs
   them). Do not edit `README.md`; add to `[0.2.0]` in `CHANGELOG.md`.
6. Fixtures neutral and synthetic. Do not delete anything with `rm -rf`.

## Sources of truth for the API (no guessing)

Saved docs in `docs/reference/`: `quiverai-llms-full.txt` (whole docs site: auth, errors,
rate-limit headers, pricing, sandbox/test keys), `quiver_developers_models_text-to-svg.md`
(request parameters for `POST /v1/svgs/generations`, fully documented),
`quiver_developers_guides_errors-and-debugging.md`,
`quiver_developers_guides_sandbox-and-test-keys.md`, `quiver_developers_models_arrow-2.md`.
The endpoint pages for animations and edits list only the path. **The request and response
shapes for `POST /v1/svgs/animations` and the response envelope for generations are not in
the saved docs.** Read them from the installed SDK: `npm install @quiverai/sdk` in `video/`
and read its shipped TypeScript types / models (`node_modules/@quiverai/sdk/`), e.g. the
`animateSVG` request and result types and the `generateSVG` result. Record in the findings
file exactly which SDK files you took each field from, and the SDK version. If a shape
cannot be determined from the SDK either, implement that call as "not verified": accept
the request body as opaque JSON, and say so plainly in findings. Do not invent fields.

## Deliverables

Language: **TypeScript in `video/src/quiver/`** (the Remotion workspace already has Node
and tsc). State that choice in findings. Do not use the SDK at runtime if plain `fetch`
suffices (Node 22, no extra runtime dependency); if you do use it, justify.

- `client.ts`: `generateSvg(req)`, `animateSvg(req)`, `listModels()`. Base
  `https://api.quiver.ai/v1`, `Authorization: Bearer $QUIVERAI_API_KEY` read from the
  environment only at call time; missing key is a clear error that names the variable
  and never echoes any value. Honour `Retry-After` on 429/503 with bounded retries and
  injectable sleep. Error envelope `{status, code, message, request_id}` surfaces as a
  typed error including `request_id`. `fetch` is injectable so tests use fakes.
- `cache.ts`: on-disk cache under `video/public/assets/quiver/` keyed by a stable hash of
  the canonical request (sorted-key JSON of model + params, excluding the key); each
  result stored as `<hash>.svg` plus an entry in `manifest.json` (`prompt`, `model`,
  `request_id`, `date`, `endpoint`, `hash`). A cache hit makes **no** request. Add
  `video/public/assets/quiver/*.svg` policy: commit the manifest, `.gitignore` nothing
  else unless justified; state your decision.
- CLI `video/scripts/quiver.ts` (compiled the same way `check-copy` is, no new tools) and a
  Python passthrough in `scriptcast/tools/video.py`: `scriptcast-video svg generate
  "<prompt>" [--model arrow-2] [--instructions ...] [--n 1] [--dry-run]`,
  `scriptcast-video svg animate <svg-path> [--prompt ...] [--dry-run]`,
  `scriptcast-video svg models`. `--dry-run` prints the exact method, URL, and JSON body
  (never the key) and exits 0 without needing a key.
- Cards accept generated art: `svgAsset` (already a prop slot from Phase 3a) now renders
  the referenced SVG from `video/public/` via `Img`/`staticFile` inside the safe-zone
  box, in all four card components; add one example `video/examples/g10_svg.json` using a
  small hand-written neutral SVG committed at `video/public/assets/example.svg`
  (a plain shape, not generated). Extend `check-copy` to assert the slot renders and stays
  inside the safe-zone constants.

## Tests and acceptance (paste raw output into findings)

```bash
/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q
cd video && npx tsc --noEmit && npm run check-copy && npm run check-quiver
```

Add `npm run check-quiver` (TypeScript test script, no new test framework) covering with a
fake `fetch`: request body and headers are exactly as documented; missing key error
message; `Retry-After` honoured then success; typed error carries `request_id`; cache hit
makes zero fetches; cache key is stable across key-order and unaffected by the API key;
manifest entry written; `--dry-run` output contains no key and needs none. Python tests
(mocked `subprocess.run`) cover the `svg` subcommand argv construction.

## Findings and final message

Update the findings file early. End with: files changed, raw outputs of pytest / tsc /
check-copy / check-quiver, the SDK version and which fields came from where, and every
unverified item: no live API call was made, animation/response shapes if any not derived,
renders not run, whether the installed `scriptcast-video svg` console path was smoke-tested.
