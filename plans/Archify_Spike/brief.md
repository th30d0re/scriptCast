# Archify spike: archify layout drawn with kit primitives (`ArchDiagram`)

Suggested worker: Codex gpt-5.6-terra, medium effort, in a scriptCast worktree off `dev` (`../scriptCast-w-archify`).

## Prohibitions
- Do not touch `review/`, `.claude/launch.json`, `uv.lock`, `router_log.jsonl`, `video/tools/` or `experiments/svg_offload/exp_*`. They contain uncommitted work that is not yours or is past experiment data.
- Do not edit the archify copy at `/Users/emmanuel/Dev/Tools/vendor/archify-a7ed6fd/`. Treat it as read-only. Do not copy it into the repo.
- Do not edit `FitBox`, `DrawRect`, `DrawLine`, any motion variant, or any existing kit component. They are shared primitives used in every scene.
- Do not touch the book repo (`~/Documents/Theory/TheOriginalPower`). The orchestrator runs the experiment cells there.
- No pushes, no rebases, no branch switching. Commit only files listed under "Files you own". If your sandbox blocks commits, leave the changes uncommitted and say so.

## Goal
Find out whether archify's checked layout can drive video diagrams drawn in the kit's own style. A good outcome is:
1. a script that turns an archify candidate JSON into a layout JSON,
2. a kit component `ArchDiagram` that draws that layout with the kit primitives and animates it on cue timing,
3. a specimen in the kit sheet, and
4. a new condition `C` in the E3 harness, so the orchestrator can run it on four YT02 cues.

"Not viable" is an acceptable answer if you show evidence. For example, the layout JSON might lack data a scene needs, or the coordinates might not map to a 1920x1080 stage. Record the reason in the findings file and stop.

## Established facts (orchestrator-measured, 2026-11-15)
- Archify 3.2.0 is pinned at commit `a7ed6fd6657da90414831d1d42cf95987188750c`, and the copy is at `/Users/emmanuel/Dev/Tools/vendor/archify-a7ed6fd/`. The CLI is `node /Users/emmanuel/Dev/Tools/vendor/archify-a7ed6fd/archify/bin/archify.mjs`. It has no runtime dependencies and needs no network. `doctor` printed `Archify is ready.`
- Layout export is `archify validate <type> <candidate.json> --layout-json`. It covers only two types. Asked for a sequence, it printed, verbatim: `--layout-json is currently supported for architecture and workflow diagrams only.`
- Workflow layout (`examples/agent-tool-call.workflow.json`) has top-level keys `contract, viewBox, requiredViewBox, columns, nodes, edges, labels, diagnostics`. Raw first items:
  ```
  nodes {"id":"user","lane":"ui","col":0,"x":48,"y":94,"width":132,"height":52}
  edges {"id":"approval-check","from":"router","to":"approval","points":[[659.2,279],[659.2,302],[577.2,302],[577.2,387],[593.2,387]]}
  labels {"edge":"approval-check","label":"needs approval?","x":618.2,"y":292,"width":82,"height":14}
  ```
  The viewBox is `[1108, 684]`. The layout output does not carry node label text. That text lives in the candidate JSON, so join the two by `id`.
- Architecture layout (`examples/web-app.architecture.json`) has keys `ok, diagram_type, layout, viewBox, components, boundaries, connections, labels`. Raw first items:
  ```
  components {"id":"users","type":"external","label":"Users","x":40,"y":300,"width":120,"height":60,"pos":[40,300]}
  boundaries {"kind":"region","label":"AWS Region: us-west-2","x":220,"y":120,"width":820,"height":400,"wraps":["cdn","lb","api","cache","db","s3","queue","worker"]}
  connections {"from":"users","to":"cdn","label":"HTTPS","variant":"emphasis","route":"auto","points":[[160,330],[250,330]],"labelAt":[205,320]}
  ```
  The viewBox is `[1080, 588]`. The two types use different shapes, so normalize both into one type.
- Archify's HTML output uses its own fonts and CSS-variable themes. It is **not** used. Only the coordinates are used, and all drawing is in kit style.
- How scenes draw diagrams today: see `minutetwo/SenseTwo.tsx` in the book stage (`/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/yt02/stage/minutetwo/SenseTwo.tsx`, lines 54–90; read only). It uses an `<svg>` with `DrawRect` inside `<foreignObject>` for boxes, `<text>` in IBM Plex, and `DrawLine` with `points` and `end="arrow"` in an absolutely positioned div. Colors come from `color('color/...')` tokens. Scenes import the kit through `@stage`. `video/remotion.config.ts:21` maps that to `video/src/stage/api.ts`.
- Kit exports are in `video/src/stage/kit/index.ts`. Primitives are at `video/src/stage/kit/motion/{DrawRect,DrawLine,FitBox}.tsx`. Read their props there and do not guess them.
- E3 harness: `experiments/svg_offload/run.py`.
  - `CONDITIONS = ("A", "B")` is at line 287.
  - Brief text per condition is in `brief_text()`, around line 346.
  - Each cell runs in a throwaway worktree of the book repo.
  - Past results: Terra placed first or tied first on 7/10 cues in round 1 and 6/8 in round 2 (`experiments/svg_offload/FINDINGS-E3.md`, artifact-backed by `author_rank*.csv`).

## Authorities
- What counts as a cue's asset is the description in `experiments/svg_offload/candidates.md`, under each cue's `**Asset:**` line.
- Kit look: `docs/stage-kit.md` and the existing components. Use tokens only, with no raw hex colors.

## Tasks
1. **Findings file first** (see below). Record your plan and a check of the facts above. Re-run the two `--layout-json` commands yourself and confirm the shapes.
2. **Layout script** `video/scripts/archify-layout.mjs <architecture|workflow> <candidate.json> <out.layout.json>`.
   - It runs the pinned archify CLI. The path comes from the `ARCHIFY_BIN` env var, with the vendor path above as the default.
   - It joins node and component labels from the candidate JSON.
   - It writes one normalized shape: `{viewBox:[w,h], nodes:[{id,label,x,y,w,h,kind?}], edges:[{id,from,to,points:[[x,y]...],label?,labelAt?}], groups:[{label,x,y,w,h}]}`.
   - It exits non-zero on archify validation failure and prints archify's diagnostics. Never write a partial layout.
3. **Type** `ArchLayout` in `video/src/stage/kit/components/types.ts`. Add it only; do not change existing types.
4. **Component** `video/src/stage/kit/components/ArchDiagram.tsx`, exported from `components/index.ts`.
   - Props: `layout: ArchLayout`, `box:{x,y,w,h}` (the stage box to fit into, preserving aspect ratio), and `progress` (0–1 entrance) or per-node start frames. Pick one and justify it in findings.
   - Drawing uses the existing kit primitives:
     - nodes with `DrawRect`
     - labels with `FitBox`, so long labels shrink rather than overflow
     - edges with `DrawLine` along `points`, with an arrow at the end
     - groups with a dashed hairline
     - edge labels as muted mono text at `labelAt`
   - Reveal order: groups, then nodes, then edges, each staggered.
5. **Specimen**: add a `case 'ArchDiagram'` to `video/src/stage/kit/KitSheet.tsx` and an entry in `KitGallery.tsx`. Feed it a committed fixture layout generated by your script. Put the fixture at `video/src/stage/kit/fixtures/archdiagram.workflow.layout.json` and its candidate next to it. Use a 5–7 node workflow written fresh, not an archify example.
6. **E3 condition C** in `experiments/svg_offload/run.py`:
   - Add `"C"` to `CONDITIONS`.
   - Add a `brief_text` branch telling the cell worker to:
     1. author an archify workflow or architecture candidate JSON for the cue's asset,
     2. run `video/scripts/archify-layout.mjs` (absolute path into the main scriptCast checkout),
     3. commit the layout JSON beside the scene, and
     4. mount `<ArchDiagram>` from `@stage` at the cue in place of the inline SVG, gated on the cue timing.
   - Conditions A and B must stay byte-identical in `--dry-run` output. Diff the dry-run before and after for A and B.
7. **Docs**: add a short `ArchDiagram` section to `docs/stage-kit.md`, covering props, the layout script, and the two supported types.

## Files you own
- `video/scripts/archify-layout.mjs` (new)
- `video/src/stage/kit/components/ArchDiagram.tsx` (new)
- `video/src/stage/kit/components/index.ts`, export line only
- `video/src/stage/kit/components/types.ts`, the `ArchLayout` addition only
- `video/src/stage/kit/fixtures/archdiagram.workflow.*` (new)
- `video/src/stage/kit/KitSheet.tsx` and `KitGallery.tsx`, the `ArchDiagram` entries only
- `experiments/svg_offload/run.py`, condition C only
- `docs/stage-kit.md`, the new section only
- `plans/Archify_Spike/findings.md`

## Acceptance (run these)
- `node video/scripts/archify-layout.mjs workflow <your fixture candidate> /tmp/x.json` exits 0. Running it on a deliberately broken candidate exits non-zero and leaves no output file.
- `npx tsc --noEmit -p video` passes, or the repo's equivalent video typecheck.
- `.venv/bin/python experiments/svg_offload/run.py --dry-run --cues <file listing doctrine-chain>` prints cells for A, B and C. The A and B lines match the output from before your change, so diff them.
- Any existing `run.py` or kit tests still pass. Find them with `ls experiments/svg_offload tests` and `grep -rl svg_offload tests`.

## Not yours to claim
- Rendering. Your sandbox cannot bind localhost ports, so Remotion renders and the `#kit` page are the orchestrator's job. Do not claim the specimen "looks right".
- Running E3 cells or any book-repo work.
- Whether condition C beats B. The author ranks that.

## Pre-registered comparison (orchestrator runs it; recorded here so it is fixed before data)
- Cues: `doctrine-chain`, `practice-network`, `router-tree`, `comparator-loop`. These are the node-and-edge cues in `candidates.md`. `debate-funnel` is a polygon funnel, which neither archify type can express, so it is excluded.
- Cells: condition C with Codex gpt-5.6-terra, compared against the existing condition-B Terra output for the same cues. The author ranks the pairs blind on the zoom contact sheets, using the same method as `author_rank*.csv`.
- Adoption bar: C places first or tied first on at least 2 of 4 cues, and none of its 4 is ranked unusable. Below that bar, `ArchDiagram` stays a kit specimen and does not become a routing lane. A null result is recorded in `FINDINGS-E3.md` exactly like a hit.

## Stop conditions
- If `ArchDiagram` needs a change to `FitBox`, `DrawRect`, `DrawLine` or any motion variant, stop and report what is missing.
- If the layout cannot be fitted into a stage box without distortion, stop on task 4 and report the numbers. For example, labels might become unreadable below 18px at a 1240px-wide box.
- If archify's `--layout-json` output differs from the shapes quoted above, record the real shape in findings and build to the real one. If it is missing node or edge geometry, stop.

## Findings file
Write `plans/Archify_Spike/findings.md` before any code change: your plan, the re-verified layout shapes, and any wrong fact in this brief. Update it as you go. End with a short list of what you could not verify.
