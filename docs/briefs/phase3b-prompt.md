You are implementing **Phase 3b** of `docs/briefs/v0.2-video-tooling.md` (scriptCast, branch
`v0.2-video-tooling`). Read the whole brief, then `docs/briefs/FINDINGS-v0.2.md` (Phases 1, 2,
3a are committed; append a `## Phase 3b` section). Phase 3b is the **episode assembly**: an
`Episode` Remotion composition and the `scriptcast-video render` subcommand. Cards already
exist (`video/src/cards/`, Phase 3a). Do not start Phase 4.

Rules:

1. Do NOT run `git commit`, `git add`, `git checkout`, `git stash`, or any history command.
   Leave changes uncommitted; the orchestrator reviews and commits.
2. Do not modify `/Users/emmanuel/Documents/Theory/TheOriginalPower`. Read-only. Its real
   inputs (read them to design against, never write there):
   - `outputs/chapter135_reply/episode_manifest.json` (turns have `turn_index`, `speaker_id`,
     `start_ms`, `end_ms`; archive-clip turns use speaker `the_reel` and carry NO clip id)
   - `outputs/chapter135_reply/chapter135_reply.mp3`
   - `Architecting_the_operation/podcasts/chapter135_rebuttal_reply.md` (archive turns are
     written `[clip:reel_q1] text...`; the clip id is in the script, resolved to a turn
     through script turn order exactly as `scriptcast/tools/shotspec.py` does)
   - `Architecting_the_operation/archive/clips.yaml` (`clips: <id>: {source, start, end}`;
     `source` is repo-relative to that project; sources are 720x1280 HEVC mp4 with AAC)
   - `Architecting_the_operation/video/specs/chapter135_rebuttal.json` (shot specs)
3. Python tests: `/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q`.
   Baseline **148 passed**; must stay green plus new tests.
4. No network, except none needed: do not `npm install` anything new. Use only the installed
   packages (`remotion`, `@remotion/cli`, react, typescript). If you need `@remotion/media-utils`
   or similar, it ships as a separate package: do not add it; use core `remotion` primitives
   (`Sequence`, `Audio`, `OffthreadVideo`, `Img`, `staticFile`) and pass durations in as props.
5. Headless Chrome cannot launch in your sandbox. Do not attempt renders beyond a single
   probe; state plainly that renders were not run. The orchestrator runs them.
6. Do not edit `README.md`. Add to the `[0.2.0]` entry in `CHANGELOG.md`.
7. Test fixtures must be neutral and synthetic (invent a tiny fake episode; no manuscript
   content in `tests/`).

## Design

**Timeline model (compute in Python, hand to Remotion as one JSON "plan").** The Python
side does all resolution; TypeScript only renders a plan. Add
`scriptcast/tools/video_plan.py` with a pure function `build_plan(...)` and a CLI-less API,
producing `plan.json`:

```
{
  "fps": 30, "width": 1080, "height": 1920,
  "duration_ms": <manifest total>,
  "audio": "<path to mp3>",
  "clips":  [{"turn_index", "clip_id", "src", "in_ms", "out_ms", "start_ms", "end_ms"}],
  "cards":  [{"shot_id", "start_ms", "end_ms", "card": {…card json…} }],
  "warnings": [ ... ]
}
```

- **Clips:** parse the script for `[clip:ID]` turns in order; the k-th archive turn in the
  script is the k-th archive turn in the manifest (match by turn order, same principle as
  `shotspec._manifest_lookup`; do NOT match on timestamps, they drift after retime). Look
  up `ID` in `clips.yaml` for `source/start/end`. `start_ms/end_ms` come from the manifest
  turn; `in_ms/out_ms` from clips.yaml seconds. If the manifest turn is shorter than the
  clip (`end-start`), trim the tail; if longer, hold the last frame (warn). Video muted:
  audio always comes from the single mp3.
- **Cards:** a shot appears on screen only if a card JSON exists for it. Accept
  `--cards DIR` containing `<shot-id>.json` (e.g. `g10.json` for `G-10`, case-insensitive,
  hyphen optional) in the Phase 3a card schema (`component` field). Window =
  shot `anchor.start_ms` to `anchor.start_ms + hold.script_ms` if a hold exists, else to the
  turn's `end_ms`. Shots with no card file are skipped with a warning, not an error. Two
  cards may not overlap: on overlap, shorten the earlier one to the later one's start and
  warn.
- **Layering** in `Episode`: black/navy background (theme navy) full frame; clip video
  scaled to cover the safe-zone usable box's width, centred (720x1280 source into 1080
  wide is a 1.5x upscale; use `objectFit: contain`, do not crop faces); cards drawn full
  frame above; no burned-in captions (the edit adds captions). 30 fps; frame numbers from
  ms via `Math.round(ms*fps/1000)`, and no gaps or negative durations.
- **`src/Episode.tsx`** takes the plan as props (`calculateMetadata` sets
  `durationInFrames` from `plan.duration_ms`), registered in `Root.tsx` as composition id
  `Episode`. Media paths in the plan are absolute filesystem paths; make the Python side
  copy or symlink them under `video/public/episode/` for the run (Remotion cannot read
  arbitrary absolute paths) and rewrite plan paths to `staticFile(...)`-relative ones.
  Clean nothing by `rm -rf`; use a per-run `video/public/episode/<run>/` directory listed
  in `.gitignore`.

**CLI.** Extend `scriptcast/tools/video.py` with:

```
scriptcast-video render <episode_dir> --script S.md --specs specs.json --clips clips.yaml \
    --cards DIR --out out.mp4 [--project-root ROOT] [--plan-only] [--dry-run]
```

- `<episode_dir>` contains `episode_manifest.json` and the `.mp3` (the single `*.mp3` in it;
  error if zero or several unless `--audio` is given).
- `--plan-only` writes `<out>.plan.json` and stops (this is what the sandbox can verify).
- `--dry-run` prints the `npx remotion render` argv and exits.
- Otherwise shells out to `npx remotion render src/index.ts Episode <out> --props=<plan>`
  with the same `--browser-executable` resolution as `still`, `--codec h264`, and the audio
  taken from the mp3 via the composition.
- Reuse the existing `still` helpers for Chrome resolution and workspace discovery; do not
  duplicate.

## Acceptance (run these, paste raw output into the findings file)

```bash
/Users/emmanuel/Documents/Theory/TheOriginalPower/.venv-voice/bin/python -m pytest tests -q
cd video && npx tsc --noEmit && npm run check-copy
```

Synthetic tests (`tests/test_video_plan.py`): clip resolution by turn order when manifest
timestamps have drifted from the script's; clip longer/shorter than its turn (trim vs hold
+ warning); a shot without a card file is skipped with a warning; overlap shortening; frame
conversion has no gaps; `--plan-only` and `--dry-run` argv with a mocked `subprocess.run`
(no Node in Python tests); missing clip id in clips.yaml is an error naming the id.

**Real-data plan check (read-only, required).** From
`/Users/emmanuel/Documents/Theory/TheOriginalPower`, with a scratch cards dir under `/tmp`
holding copies of `Architecting_the_operation/video/cards_json`-style files only for G-10
and G-11 (copy `/Users/emmanuel/Documents/Theory/scriptCast/video/examples/g10.json` and
`g11.json`), run `render --plan-only` against the real episode and paste the resulting
plan's `clips` array and `cards` array raw. Expected: 5 clips (reel_q1, reel_a1, reel_q2,
reel_a2, reel_close) at manifest turns 0, 9, and so on, with `start_ms` equal to the
manifest's; 2 cards. Report any mismatch, do not paper over it.

**Typecheck the Episode composition** with a plan built from a tiny synthetic fixture in
`video/examples/plan.sample.json` (neutral; two clips, one card, paths that need not exist).

## Findings and final message

Update `docs/briefs/FINDINGS-v0.2.md` early and as you go (brief defects, decisions, what
you could not verify). End your final message with: files changed, raw outputs of pytest /
tsc / check-copy / the real-data plan check, and every unverified item: renders not run,
HEVC decode by Remotion's `OffthreadVideo` untested, whether the installed
`scriptcast-video render` console script was smoke-tested.
