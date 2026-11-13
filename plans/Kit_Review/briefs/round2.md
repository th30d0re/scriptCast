# Kit Review round 2: TermTriad, TrainingTextStrip, player sharpness

Suggested worker: Codex gpt-5.6-terra, medium effort, in a scriptCast worktree off `dev`.

## Prohibitions
- Do not touch `review/notes.json`, `.claude/launch.json`, `experiments/` or `review/kit-review.json`. The repo has uncommitted work in them that is not yours.
- Commit only the files listed under "Files you own". No pushes, no rebases, no branch switching outside your worktree.
- Do not change any other kit component. Scenes and other kit items are read-only.

## Goal
Clear the three open notes from the author's second review pass. A good outcome is each note either fixed and visible in a re-rendered review video, or answered with evidence in the findings file. "This is already correct, here is why" is an acceptable answer for item 2a if the evidence supports it.

## Established facts
Round-1 notes were fixed in commits `1da303a`, `0483206`, `8d400e9`, `4e6f7d0` and `0cdda1c` (2026-11-08). Note that `review/kit-review.md` predates these commits and is stale. The authority is `review/kit-review.json`. These are the only open notes in it, quoted verbatim:

**TermTriad** (status `change`):
> [Fixed 2026-11-08, recheck] should be re drawn with a custom version of FitBox
>
> Okay, for one, I don't know why it's so low res. I know you changed the viewer to make it more responsive, which I appreciate, but that's kind of annoying. Second of all, for this view, excuse me, term triad, we have the text pattern underneath, which I don't really get. Was that part of the change that we had before?

**TrainingTextStrip** (status `keep`, but has a new defect):
> Also, for training text strip, there is a yellow line underneath that kind of glitches out a little bit, and it like makes the pattern rectangle double in size first, which kind of looks weird instead of like moving with the text as it's typing in. So we need to fix that.

What I checked (orchestrator, artifact-backed):
- `video/public/kit-review/TermTriad.mp4` is 1920x1080, rendered with `crf: 18` (`video/scripts/render-kit-videos.mjs:42`). So the render's resolution is not the cause of the low-res look. The cause is unknown. Candidates are the size of `ItemPlayer`'s box in the page (`site/src/kit/ItemPlayer.tsx:21`, `width:100%`, 16:9) and the CSS layout of `site/src/kit/`. Measure it; don't guess.
- The player opens in `rendered` mode (the mp4) and has a `live` mode (Remotion Player, 1920x1080 composition). Check both.
- TermTriad already draws each title through `FitBox` (`video/src/stage/kit/components/TermTriad.tsx`, the `<FitBox text={term.title} ...>` call). Batch C changed it.
- TrainingTextStrip (`video/src/stage/kit/components/TrainingTextStrip.tsx`):
  - It sizes the FitBox once with the full text and types the text inside it.
  - It draws the gold underline through an invisible text copy positioned absolutely at `left:30, top:42`. The underline animates over the last 15% of `progress` (`(progress-.85)/.15`).
  - The "pattern rectangle doubles in size first" likely comes from FitBox's own box draw-in running ahead of the typing. That is unverified.
- Specimen props for both are in `video/src/stage/kit/KitSheet.tsx` (`case 'TermTriad'`, `case 'TrainingTextStrip'`).

## Tasks
1. **Player sharpness.** Find out why kit items look low-res in the `#kit` page and fix it in the site. Record the measured displayed size and devicePixelRatio, before and after, in the findings file.
2. **TermTriad.**
   a. Screenshot the specimen at progress 1 and identify what the author calls "the text pattern underneath". Say which element it is and which commit introduced it (`git log -L` or blame). If it came from batch C and has no purpose in the specimen, remove it. If it is the bracket and its label ("RELATED TERMS"), don't remove it. Report it and stop on that sub-item.
   b. Confirm that the titles render as a FitBox in the review video (the round-1 note) and say so in findings.
3. **TrainingTextStrip.** The box and the gold underline must grow with the typed text, not appear at full or double size first. The underline must not jitter. Keep the props API unchanged.
4. Re-render only these two items:
   ```bash
   cd video && node scripts/render-kit-videos.mjs public/kit-review --only TermTriad,TrainingTextStrip
   ```
   Usage is in `video/scripts/render-kit-videos.mjs:13`.

## Files you own
- `video/src/stage/kit/components/TermTriad.tsx`
- `video/src/stage/kit/components/TrainingTextStrip.tsx`
- `video/src/stage/kit/KitSheet.tsx`, only the two specimen `case` lines
- `site/src/kit/*` and the site's kit CSS, for task 1 only
- `video/public/kit-review/TermTriad.mp4` and `video/public/kit-review/TrainingTextStrip.mp4` (re-rendered)
- `plans/Kit_Review/briefs/round2.findings.md`

## Acceptance (run these)
- `npm run build -w site` passes.
- `npx tsc --noEmit -p video` passes, or the repo's equivalent video typecheck.
- `node --test site/kit-review-middleware.test.mjs` passes, if it exists.
- For TrainingTextStrip, take frame stills at progress .1, .3, .5, .7, .9 and 1. Save them under `plans/Kit_Review/briefs/round2-evidence/`. They must show the box width non-decreasing and never wider than the final box. No underline before typing ends.

## Not yours to claim
- Whether it "looks right" to the author: the orchestrator and the author recheck in the `#kit` page.
- The native UI button.

## Stop conditions
- If fixing TrainingTextStrip needs a change to `FitBox` or any motion variant, stop and report. That is a shared primitive used in every scene.
- If the low-res cause turns out to be outside `site/src/kit/` (for example the global site layout from commits `54e03f2` and `34cf65f`), report it and stop on task 1.

## Findings file
Write `plans/Kit_Review/briefs/round2.findings.md` first, before any code change. Put your plan and the item 2a identification there, then update it as you go. Report any wrong fact in this brief there.
