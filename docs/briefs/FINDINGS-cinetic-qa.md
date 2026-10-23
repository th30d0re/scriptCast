# Cinetic QA baseline: `run-uw1w370f/clip001.mp4`

Baseline run on 2026-10-22:

```text
python3 -m scriptcast.tools.cinetic_qa \
  /Users/emmanuel/Dev/Tools/scriptCast/video/public/episode/run-uw1w370f/clip001.mp4 \
  --out /tmp/cinetic-qa-baseline
```

The combined result was a failure because three checks that ran failed. `forensics.py` and
`sheet.py` were skipped because this environment does not have `cv2`; no package was installed.

## Measured facts

| Check | Result | Key result |
| --- | --- | --- |
| `probe.py` | fail (exit 1) | 720x1280, 30.0 fps, 443 frames, 14.7667 s, H.264 High, `yuv420p`, 4.573 MB, 2,478 kb/s. The BT.709 space/primaries/transfer and TV range all passed. The required audio check failed: expected `present`, got `none`. |
| `forensics.py` | skipped | `missing dependency cv2` |
| `sheet.py` | skipped | `missing dependency cv2` |
| `check-sync.py` | fail (exit 1) | It found `000.mp3` beside the video, but reported `no decodable audio stream in the video`; lag, correlation, true peak, and loudness therefore could not be measured. |
| `lint-film.mjs` | fail (exit 1) | Scanned 16 files under `video/src`: 7 errors and 7 warnings. |
| `av-audit.py` | n/a | scriptCast does not produce the cinetic cues file this check requires. |

The probe was explicitly run with `--spec @30`; it did not apply cinetic's 60 fps or 1080p
deliverable assumptions. The render passed the 30 fps, codec, pixel-format, and colour-tag checks.

The seven lint errors were three banned palette colours in `theme.ts` (`#f4ead2`, `#d9a441`,
and `#e2573f`), a beige light shadow and a five-pixel light glow in `PartsCard.tsx`, italic type
in `StatBarsCard.tsx`, and wall-clock nondeterminism (`new Date()`) in `quiver/cache.ts`. The seven
warnings were non-token `rgba(...)` colours: one in `Episode.tsx`, five in `FreezeCallout.tsx`,
and one in `PartsCard.tsx`.

## Manual observation

A separate six-frame, evenly spaced visual sample (not the skipped cinetic sheet check) showed a
late caption frame at approximately 12.3 s reading `I THINK THAT'S THE THE`; the red active-word
overlay appears beside rather than exactly over the white `THE`, so the word reads twice in that
frame.

## Relevance to podcast episodes

- `av-audit.py` is not applicable. It audits a cinetic cues file, and scriptCast has no equivalent
  artifact to give it.
- The sync check is applicable only to a muxed audiovisual deliverable. It cannot assess this MP4
  because the MP4 has no audio stream, even though source audio is present beside it.
- Probe's audio-presence rule is conditional. It is a valid failure if `clip001.mp4` is meant to be
  watched as delivered; it is irrelevant if scriptCast intentionally emits silent visual clips and
  muxes audio in a later step.
- Cinetic's motion-forensics thresholds are only partly transferable. Long holds and low-motion
  cards can be deliberate in a spoken podcast, so stall/hold findings would need editorial review
  rather than automatic rejection. The check did not run in this baseline.
- Cinetic's palette, italic, and glow bans express its house style, not basic podcast correctness.
  They are relevant only if scriptCast cards are meant to adopt that art direction. Determinism and
  render-safe animation lint rules remain technically relevant.
- The contact sheet is relevant as a review aid, but it is not itself a pass/fail podcast spec.

## Top five concrete issues

1. The MP4 has no audio stream. This is the direct cause of both the probe and sync failures.
2. A sampled late frame visibly duplicates `THE` in the caption (`THE THE`), apparently because the
   active-word overlay is horizontally displaced.
3. `video/src/quiver/cache.ts` calls `new Date()`, making generated cache metadata depend on wall-clock
   time and preventing fully repeatable output.
4. `PartsCard.tsx` uses a blurred light shadow (`5 px`) that lint classifies as a glow and separately
   introduces a non-token beige shadow colour.
5. `StatBarsCard.tsx` renders the quote in italic, while the theme contains three colours rejected by
   cinetic's palette rules. These are concrete source findings, though their severity depends on
   whether cinetic's house style is intended for scriptCast.

## Interpretation and open questions (not measured facts)

- The missing audio may be intentional if `clip001.mp4` is an intermediate visual asset. The file
  name and directory layout alone do not establish that workflow.
- The doubled `THE` looks like active-word positioning rather than duplicated source dialogue, but
  the baseline did not inspect caption timing data to prove the cause.
- The lint palette/italic/glow failures may represent an intentional scriptCast visual identity,
  not defects. They should not be changed solely to make cinetic lint green without an art-direction
  decision.

## Rerun with cv2 (reviewer, 2026-10-22)

Rerun with `uv sync --all-extras` in this worktree (so the `cinetic` extra supplies cv2), same clip:

- `sheet.py`: pass, 45 frames (f0-f440, every 10).
- `forensics.py`: fail, 43 findings. 20 shown are `stall` (frame N repeats N-1), occurring on a strict
  5-frame cadence (f156, f161, f166 ... f231, f266, f286 ...); the rest are `spike` (one-frame jumps at
  f137, f158, f234, f283 ...).
- `hermetic tests`: 3 passed.

Interpretation (not proven):
- The 1-in-5 duplicated frames match a 25 fps source conformed to 30 fps without frame blending. Worth
  checking the source fps in `make_clip.py` / `video.py`; if confirmed it is real judder, and the most
  useful finding of this run.
- The spikes line up with B-roll cuts on the contact sheet (f100, f158, f290 ...). They are undeclared
  cuts, not defects; passing cut frames via `--cuts` would silence them.

Correction to "Manual observation" above: the contact sheet shows the caption building
`I THINK THAT'S THE` then `I THINK THAT'S THE THE NUMBER` across f400-f430, so the doubled `THE` is very
likely the speaker's own repetition in the transcript, not an overlay offset.
