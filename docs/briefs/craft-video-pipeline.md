# scriptCast — hand a finished episode to the Craft CLIs

Work in this repo (`/Users/emmanuel/Dev/Tools/scriptCast`). Read
`/Users/emmanuel/Documents/Theory/TheOriginalPower` only if a shot spec cites it.
Do not modify that repo.

## Prohibitions

- Do not clone, patch, or vendor EffectCraft, FilmCraft, VectorCraft, or DesignCraft.
  They stay release binaries. The installed versions on this machine are recorded below.
- Do not change speech synthesis, turn timing, gaps, or the bytes of existing wavs.
  The episode manifest is the clock. Picture is placed on that clock.
- Do not replace `scriptcast-video` or the Remotion workspace in `video/`. That path
  keeps rendering the card episode. This work adds a second delivery that consumes
  the same manifest.
- Do not rewrite `README.md`. Document the command in `CHANGELOG.md` and in the
  findings file.
- Do not write API keys, cookies, or account tokens into any file.

A well-supported "this interchange does not work" is an acceptable result. Report
it in the findings file and stop at the condition named below.

## Goal

A scriptCast episode becomes a video podcast without a second timing system.

scriptCast already owns the script, the per-turn speech, the manifest
(`start_ms` / `end_ms`), the captions, and the card plan. The Craft apps own
picture that scriptCast does not draw:

- VectorCraft draws still art (logos, plates, diagrams) and exports PNG or SVG.
- EffectCraft animates titles and motion graphics and renders a silent movie
  (ProRes or PNG with alpha, or H.264).
- DesignCraft lays out a page (a quote card, a chapter opener, a one-sheet) and
  exports PDF or PNG. It does not typeset the book. The manuscript stays in TeX.
- FilmCraft assembles those pictures with the episode's dialogue and captions
  and exports one H.264 file.

The intent is one watchable episode: dialogue from scriptCast, picture from the
Craft CLIs, both on the manifest's timeline. An agent, or a person, can produce
the picture files with the CLIs already installed. scriptCast's job is to say
which file covers which span, to write an interchange FilmCraft can open, and
to write captions in a format FilmCraft imports.

## Established facts

Measured on this machine, 2026-10-06, after installing the release zips into
`~/.local/bin` and checking each archive against the release `SHA256SUMS.txt`:

```
effectcraft-cli 0.3.1
filmcraft-cli 0.2.1
vectorcraft-cli 0.3.1
designcraft-cli 0.2.1
```

`which` resolved all four to `/Users/emmanuel/.local/bin/`. Each `--version`
printed the version above. Help text, quoted from those binaries:

- `effectcraft-cli render [--comp C] --out FILE` and `effectcraft-cli mcp`
- `filmcraft-cli import <file>...`, `filmcraft-cli export <out>`,
  `filmcraft-cli run <script.jsonl>`, `filmcraft-cli mcp`
- `vectorcraft-cli run [--in FILE] [--export FILE]` (png, svg, pdf, …)
- `designcraft-cli run [--in FILE] [--export OUT]` (pdf and per-page png)

### What scriptCast already writes

Audio timeline, landscape, dialogue only. `scriptcast/fcpxml_generator.py`:

- line 13: `_FCPXML_VERSION = "1.10"`
- lines 18–20: format `FFVideoFormat1080p30`, 30 fps
- lines 165–174: the format resource is `width="1920"` `height="1080"`
- lines 43–67: `_build_asset` sets `hasVideo="0"` and `hasAudio="1"` on each wav

Captions are JSON, not SRT or WebVTT. `scriptcast/tools/captions.py` `build_captions`
returns dicts `{turn_index, start_ms, end_ms, text}` (lines 115–117) and `main`
writes that list with `json.dumps` (line 162).

The card episode is portrait. `scriptcast/tools/video_plan.py` line 146 returns
`fps=30, width=1080, height=1920`. `video/src/safeZone.ts` lines 1–2 set
`WIDTH = 1080` and `HEIGHT = 1920`. `scriptcast-video render` burns
`captions.json` inside Remotion (`scriptcast/tools/video.py` lines 146–147 and
258).

Entry points live in `pyproject.toml` lines 42–49: `scriptcast-video` and
`scriptcast-captions` among them. There is no Craft command.

### What is a claim, not a measurement

FilmCraft's README says it imports FCPXML 1.9–1.11. That was read from the
upstream README. Nobody in this session has imported a scriptCast `.fcpxml`
with `filmcraft-cli import`. Treat the import as unverified until the probe
below succeeds.

EffectCraft's README says it renders ProRes with alpha and H.264. That was read
from the upstream README and from `effectcraft-cli --help` listing
`prores` and `h264`. No render was run in this session.

## Authorities

- Episode clock: `episode_manifest.json` turn fields `start_ms` and `end_ms`,
  as `video_plan.py` lines 38–48 already require (`end` strictly after `start`,
  duration at least the last turn's end).
- Card frame: `video_plan.py` line 146, `width=1080`, `height=1920`, `fps=30`.
  The FCPXML generator's 1920×1080 is the audio-session format. The video
  delivery uses the card frame.
- Caption record: `captions.py` lines 115–117.
- FCPXML version this repo writes: `fcpxml_generator.py` line 13, `"1.10"`.

## Files you own

- `scriptcast/tools/craft_export.py` (new)
- `scriptcast/tools/captions.py` (add an SRT/WebVTT writer; keep the JSON)
- `scriptcast/fcpxml_generator.py` (picture assets on the existing timeline)
- `tests/` files you add for the above
- `pyproject.toml` (one new console script)
- `CHANGELOG.md` (one entry)
- `docs/briefs/FINDINGS-craft-video.md` (write this first)

Everything else is read-only, including `video/`, `scriptcast/tools/video.py`,
and `scriptcast/tools/video_plan.py`.

## Contract

One new command, `scriptcast-craft`, with two subcommands.

`scriptcast-craft cue EPISODE_DIR --out cue.json`

Reads `episode_manifest.json`. Writes a cue sheet. Does not call a Craft binary.
Default frame is the card frame: 1080×1920, 30 fps. Schema:

```json
{
  "fps": 30,
  "width": 1080,
  "height": 1920,
  "duration_ms": 0,
  "cues": [
    {
      "start_ms": 0,
      "end_ms": 0,
      "src": "/absolute/path/to/picture.png",
      "kind": "still"
    }
  ]
}
```

`kind` is `still`, `motion`, or `page`. `src` is an absolute path the operator
filled in after VectorCraft, EffectCraft, or DesignCraft exported the file.
A cue with an empty `src` is allowed in the sheet and is omitted from the
timeline. Cue times are milliseconds on the manifest clock. Overlaps shorten
the earlier cue to the later cue's start, the same rule as card windows in
`video_plan.py` lines 114–117. A cue may cover many turns. The command does
not invent times that the manifest does not contain.

`scriptcast-craft timeline EPISODE_DIR --cue cue.json --captions captions.json --out episode.fcpxml`

Writes FCPXML 1.10 whose format width, height, and frame duration come from
the cue sheet. Existing dialogue wavs stay on their speaker lanes with
`hasVideo="0"`. Each cue with a `src` becomes a video asset (`hasVideo="1"`,
`hasAudio="0"`) placed from `start_ms` for `end_ms - start_ms`. Motion files
are picture only, so their audio cannot double the dialogue. Also writes
`episode.srt` and `episode.vtt` beside the FCPXML from the captions JSON.
Times in the captions stay the `start_ms` / `end_ms` already stored.

Do not shell out to `effectcraft-cli`, `vectorcraft-cli`, or `designcraft-cli`
inside these commands. Those tools produce the files named by `src`. FilmCraft
is the consumer of the FCPXML, invoked by the operator or by a later probe,
not by this command's default path.

## Acceptance (run these)

From the repo root, with the project venv:

```bash
python -m pytest tests
```

Existing tests stay green. New tests, with no network and no Craft binary:

1. A fixture manifest with two turns produces a cue sheet whose `duration_ms`
   equals the manifest duration and whose default frame is 1080×1920 at 30 fps.
2. Two overlapping cues shorten the earlier one to the later one's `start_ms`.
3. `timeline` on a fixture with one wav and one PNG writes FCPXML 1.10 that
   contains `hasVideo="0"` for the wav, `hasVideo="1"` and `hasAudio="0"` for
   the PNG, and offsets matching the cue's `start_ms` at 30 fps.
4. The same run writes SRT and WebVTT whose first cue text and times match the
   fixture captions JSON.
5. A cue with an empty `src` contributes no video asset.

## Not yours to claim

Do not claim that FilmCraft opened the FCPXML, that a Craft app rendered a
real episode, or that the H.264 export matches the Remotion episode. Those
runs belong to a follow-up on this machine, after the unit tests pass. If you
do run `filmcraft-cli import` while probing the stop condition, paste the
command and the full output into the findings file and label it a probe.

## Stop conditions

Before changing `fcpxml_generator.py`, run this probe and record it in the
findings file:

```bash
filmcraft-cli --project /tmp/craft-probe.fcproj import PATH_TO_AN_EXISTING_SCRIPTCAST.fcpxml
```

Use any `.fcpxml` this repo already generates from a test or a tiny fixture.
If `filmcraft-cli` is missing, or the import fails, stop. Leave the generator
unchanged. Write the failure in the findings file. The fallback, if a later
brief asks for one, is `filmcraft-cli run` with a JSONL script of timeline
commands built from the cue sheet. Do not invent that fallback in this pass.

If the probe succeeds, continue with the contract above.

## Findings file

Write `docs/briefs/FINDINGS-craft-video.md` before the first code edit. Put
the probe output there, plus any place this brief disagrees with the code.
A loop that stops on the probe has finished if that file contains the command,
the output, and the conclusion.
