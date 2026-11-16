# Kit review: round 2 findings

## Plan

1. Measure the rendered and live player boxes and browser `devicePixelRatio` on `#kit`, then keep the player at a pixel-dense review size if the scoped kit layout is the cause.
2. Inspect the final TermTriad specimen and its history; retain the bracket only if it is the intended RELATED TERMS annotation.
3. Make TrainingTextStrip geometry derive from the currently typed text, capture the prescribed progress frames, then render only the two reviewed items.

## Initial findings

### TermTriad 2a

At progress 1, the text pattern beneath the three titles is the gold bracket and its `RELATED TERMS` label (`bracketLabel` in the TermTriad specimen). It was introduced with the original TermTriad implementation in `05ef4db` (`[kit0] Stage kit phase 0: harvest approved YT02 components, gallery sheets`), not batch C. It is the documented relationship annotation, so it is retained. A fresh specimen screenshot could not be produced because the Remotion bundler cannot bind a local port in this sandbox; this identification is source- and history-backed.

The title implementation is already a `FitBox` per term (`<FitBox text={term.title} ...>` in `TermTriad.tsx`); re-render validation will confirm it remains so in the review artifact.

## Changes made

### Player sharpness

The scoped kit layout imposed a `max-width: 1100px` cap on `.kit-player`, while both rendered MP4s are 1920×1080. I removed that cap and made the player consume the available kit-stage width. This applies identically to rendered video and the live Remotion Player, which both have `width: 100%` and a 16:9 aspect ratio.

I could not record the requested browser measurements (displayed rendered/live box size and `devicePixelRatio`) in this sandbox: Vite initially failed because `site/node_modules` is a symlink outside the writable worktree, and the native-loader workaround could build but this environment rejects local listener creation (`listen EPERM` on port 3000). The pre-change measured CSS constraint is 1100 CSS px maximum; post-change it is the available kit-stage width. No DPR value is asserted without a browser measurement.

### TrainingTextStrip

The strip passes the growing character prefix into its existing FitBox. Following review, typing completes at progress .85, FitBox entrance completes at .25, and the underline draws over .85–1 using the complete text copy. No shared primitive or motion variant was changed.

## Verification and blocked artifacts

- `npx tsc --noEmit -p video`: passed.
- Site build: `npm run build -w site` reaches a Vite temp-cache permission error caused by the external `site/node_modules` symlink. Equivalent native-loader build passed: `npx vite build --configLoader native` from `site/`.
- `node --test site/kit-review-middleware.test.mjs`: passed.
- The mandated render command was attempted exactly for `TermTriad,TrainingTextStrip`, but Remotion cannot start its required local bundler server in this environment (`listen EPERM :::3000`). Therefore neither MP4 was replaced and the required still evidence directory could not be populated. This is an environment limitation, not a claim of visual review completion.

## Review fixes

- Commit handoff: explicit staging of the four requested files failed creating the parent repository's worktree index.lock (Operation not permitted). No commit was created; dispatcher must commit on w-kitr2 with the requested message.
- Restored `.status.cut { background: #e66; }`. The only CSS change relative to dev is replacing the kit-player 1100px max-width with width: 100%. The requested grep of the diff prints the cut rule twice, once on each side.
- Prefix length is floor(character count × min(1, progress/.85)). FitBox receives min(1, progress/.25), preserving the early draw and flicker and remaining at rest after .25. Reading FitBox.opus, DrawRect.opus, and their timing shows no text-dependent seed or animation reset: seed stays 31 and timing is driven by explicit progress. FitBox's own early text reveal also runs during that entrance.
- Underline progress is clamped (progress-.85)/.15; the overlay exists only when that value is positive and uses the complete tokens throughout. Background width grows from 0 to 100% without changing the overlay's text.
- For the specimen's monospace text, prefixes have non-decreasing measured width, bounded by the final text width. Visual confirmation remains assigned to the orchestrator.
- Requested video typecheck passed (exit 0). Output was redirected to /private/tmp/kit-round2-typecheck.log and its tail read; the log was empty.
- No renders or browser measurements were attempted during this review-fix turn, per the user's instruction. The width cap is a source observation, not a measured cause of low resolution.
- Historical clarification for TermTriad: commit 14ec3d0 already removed a literal sample line reading "pattern" from the BIAS specimen. Thus the author's original wording may refer to that removed line. The current specimen retains only the RELATED TERMS annotation underneath; fresh visual confirmation is pending.
