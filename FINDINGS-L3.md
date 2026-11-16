# L3 findings: select and annotate in the Stage player

Review mode is mounted in the browser player (`video/site/main.tsx`). The Remotion entry (`video/src/stage/index.ts` → `Root.tsx` → `StageEpisode.tsx`) does not import it. Press **R**, or the Review button, to turn it on. Turning it on pauses the episode audio. The stage shifts right by the notes column so the list does not cover the picture.

While review is on, the pointer outlines the nearest visible element that carries `data-text` or `data-shape`. A point inside several boxes selects the smallest box. A point outside every box selects the closest box. A click freezes scene, element id, stage milliseconds, and the nearest cue, and opens the note box. The nearest cue is the cue closest in time; when a bare name and its shot-qualified twin share that time, the bare name is kept. The side list shows notes with `status: "open"`. Choosing one seeks the player to that millisecond and draws a solid outline on the element.

## Persistence

Notes are stored by a Vite dev middleware, `GET` and `PUT /review/notes.json`, which reads and writes `review/notes.json` at the worktree root. The middleware is registered in `configureServer` only, so a Remotion bundle never includes it.

The File System Access API was set aside because it needs a picker gesture and does not write a known repo path on a fresh load. The verifier's check is: add notes, reload, and find them still in `review/notes.json`. The dev endpoint does that. `Cache-Control: no-store` is set on the response. A body that is not a list of `{scene, element, ms, cue, note, author, status: "open"}` is rejected with HTTP 400.

`video/site/vite.config.ts` also allows `../scriptCast-stage/video/node_modules` when that directory exists. The book episode's `index.ts` imports IBM Plex from that absolute path, and the player cannot compile APAGroupDefinitions without it.

## What was exercised in the player

The player was started with `STAGE_EPISODE_DIR` set to the book repo's `yt02/stage` and `STAGE_PLAN` set to `ov/stage_plan.json`, at http://localhost:5179/.

- **R** turned review on. The two sample notes loaded into the list.
- At 262.3s, hovering **INGROUP BIAS** outlined `definition-term` (dashed) and showed that id.
- The click opened the note box: scene `APAGroupDefinitions`, `262330 ms`, cue `t21Conduct` (that cue is at 262430, 100 ms away).
- Choosing the `allocation-tray` note sought the playhead to 263.9s and drew the solid outline on the tray.
- Saving a third note from the box wrote `review/notes.json`. After reload, review mode showed all three. That third note was then removed so the sample file is the two notes below. A malformed `PUT` returned HTTP 400 and left the file in place.

Screenshots:

- `review/screenshots/01-hover.png`
- `review/screenshots/02-note.png`
- `review/screenshots/03-list.png`

## Determinism

`video/scripts/check-review-determinism.tsx` renders `DemoTitle` at 2000 ms through `PlanContext` and `ClockContext` with `interactive: false`, the clock `RemotionClock` supplies. That markup is compared, byte for byte, with `video/scripts/fixtures/demo-title-ms2000.html`. The fixture is the scene as it renders with review absent: `DemoTitle` was not edited, and the markup contains no review marker. The same test checks that the render-path sources do not mention the review module, and that `review/notes.json` holds two open notes on `APAGroupDefinitions`.

The fixture is 252 bytes, sha256 `a3bb136877660a791f54f5989466c4e452507f59b00529f2654135858a8b6175`.

## Acceptance tails

From `video/`:

```
$ npx tsc --version
Version 5.9.3
$ npx tsc --noEmit -p tsconfig.json
tsc_exit:0

$ node scripts/run-check-review.mjs
PASS: review determinism 252 bytes sha256 a3bb136877660a791f54f5989466c4e452507f59b00529f2654135858a8b6175
```

`npm run check-review` is the same command.

## Sample notes

`review/notes.json`:

```json
[
  {
    "scene": "APAGroupDefinitions",
    "element": "definition-term",
    "ms": 255890,
    "cue": "t21Bias",
    "note": "INGROUP BIAS sits tight under the card edge. Open a clearer gap beneath the gold rule.",
    "author": "author",
    "status": "open"
  },
  {
    "scene": "APAGroupDefinitions",
    "element": "allocation-tray",
    "ms": 263890,
    "cue": "t21Behavior",
    "note": "The derived-illustration tray crowds the source line. Lift the tray so the citation keeps its safe-area gap.",
    "author": "author",
    "status": "open"
  }
]
```

Cue times are the bare cues in `ov/stage_plan.json`. `definition-term` is the `data-text` id on the definition card. `allocation-tray` is the `data-shape` id on the derived-illustration panel.

## Files

- `video/src/stage/review/schema.ts`, `cue.ts`, `hit.ts`, `storage.ts`, `ReviewMode.tsx`
- `video/site/main.tsx`, `video/site/style.css`, `video/site/vite.config.ts`
- `video/scripts/check-review-determinism.tsx`, `video/scripts/run-check-review.mjs`, `video/scripts/fixtures/demo-title-ms2000.html`
- `video/package.json` (`check-review` script)
- `review/notes.json`, `review/screenshots/`

This session did not write a book-repo session log. The brief asks for one only when the harness requires it, and this Cursor session does not.
