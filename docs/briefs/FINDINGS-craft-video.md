# Findings — craft video pipeline

Model: Grok 4.7

Written before any code edit. The stop-condition probe is below.

## Probe

Fixture: `/tmp/scriptcast-craft-probe/probe.fcpxml`, produced by the current
`generate_fcpxml` (unchanged). One mono 48 kHz WAV, one second of silence,
FCPXML 1.10, format `FFVideoFormat1080p30` at 1920×1080.

`filmcraft-cli` is `/Users/emmanuel/.local/bin/filmcraft-cli`, version 0.2.1.

### Attempt 1 — project path from the brief, file absent

```bash
filmcraft-cli --project /tmp/craft-probe.fcproj import /tmp/scriptcast-craft-probe/probe.fcpxml
```

```
filmcraft-cli: /tmp/craft-probe.fcproj: No such file or directory (os error 2)
```

Exit status 1. `--project` opens an existing project. It does not create one.

An empty project was then created and saved, so the import itself could be measured:

```bash
filmcraft-cli --save-as /tmp/craft-probe.fcproj exec file.newProject name=craft-probe
```

That command printed `null` and exited 0. `/tmp/craft-probe.fcproj` was 673 bytes.

### Attempt 2 — the brief's command, project present

```bash
filmcraft-cli --project /tmp/craft-probe.fcproj import /tmp/scriptcast-craft-probe/probe.fcpxml
```

```json
{
  "documents": [
    {
      "format": "FCPXML",
      "linkedMedia": 1,
      "offlineMedia": [],
      "report": [],
      "sequences": [
        1
      ]
    }
  ],
  "errors": [],
  "items": [],
  "sequences": [
    1
  ]
}
```

Exit status 0. FilmCraft parsed the file as FCPXML, linked the one WAV, and
reported one sequence and no errors.

Conclusion: the interchange works for a scriptCast FCPXML 1.10 audio timeline
when `--project` points at a project that already exists. The generator may be
changed. This does not show that a portrait timeline, a picture asset, or an
H.264 export works.

### Extra inspect, not the stop condition

The command above does not pass `--save`, and a later `inspect` on a fresh
open reported `no active sequence`. Re-running the import with `--save` exited
0 with the same JSON. `inspect` then showed sequence `craft_probe`: 1920×1080,
30 fps, 30 frames, the WAV on Audio 2, Video 1 empty. The clip sits on lane 1
because the generator hangs it off the primary gap. One second is
254016000000 ticks.

## Where the brief and the tree differ

- `filmcraft-cli help` describes `import <file>...` as "import media into the
  project". There is no `file.importFcpxml` among the 666 commands.
  `file.exportFcpxml` exists (`params: {"path": str}`). The `import` subcommand
  nonetheless accepted this `.fcpxml` and returned `format: FCPXML`.
- `video_plan.py` line 146 is the card frame (1080×1920, 30 fps).
  `fcpxml_generator.py` lines 18–20 and 165–174 hardcode
  `FFVideoFormat1080p30` at 1920×1080. The audio export keeps that default.
  The craft timeline passes width, height, and frame duration from the cue
  sheet. A portrait format is named `FFVideoFormat{width}x{height}p{fps}` so
  it does not claim the landscape preset name.
- `captions.py` `main` writes JSON only. SRT and WebVTT are new functions.
  `scriptcast-captions` still writes JSON.
- Episode manifests in this repo (`ui/tests/fixtures/episode_small/episode_manifest.json`,
  `tests/fixtures/shotspec_manifest.json`) have turn `start_ms` / `end_ms` and
  no top-level `duration_ms`. Cue-sheet duration uses `duration_ms`, then
  `total_duration_ms`, then the latest turn `end_ms`, and rejects a duration
  shorter than that end. That is `video_plan.py` lines 47–49.
- `cue` writes one placeholder cue per turn (`kind` `still`, `src` `""`) from
  that turn's `start_ms` and `end_ms`, sorted by `start_ms`, then
  overlap-shortened. The brief requires those times to be manifest times and
  allows an empty `src`. It does not name the placeholder `kind`. A cue that
  covers many turns is an edit to this sheet, not a second clock.
- Picture clips are further connected clips on the next lane after the speaker
  lanes, on the same gap. `hasAudio` is `0` for `still`, `motion`, and `page`.

## Follow-up probe — portrait timeline with a still

After the unit tests passed. Not the stop condition. The stop-condition import
had already exited 0.

`scriptcast-craft timeline` wrote `/tmp/scriptcast-craft-portrait/episode.fcpxml`:
1080×1920, `frameDuration` `1/30s`, one mono WAV (`hasVideo="0"`) and
`plate.png` (`hasVideo="1"` `hasAudio="0"`).

```bash
filmcraft-cli --save-as /tmp/craft-probe-portrait.fcproj exec file.newProject name=portrait
filmcraft-cli --project /tmp/craft-probe-portrait.fcproj --save import /tmp/scriptcast-craft-portrait/episode.fcpxml
```

Import exited 0:

```json
{
  "documents": [
    {
      "format": "FCPXML",
      "linkedMedia": 2,
      "offlineMedia": [],
      "report": [],
      "sequences": [1]
    }
  ],
  "errors": [],
  "items": [],
  "sequences": [1]
}
```

Opening that saved project failed:

```
filmcraft-cli: /tmp/craft-probe-portrait.fcproj: project file is damaged: invalid type: null, expected f64 at line 1 column 2090
```

The saved JSON has `"Vec2":{"x":null,"y":null}` on the picture clip's motion
anchor and position. The same damage happened for a landscape still, for the
still placed on the spine, and for a 1080×1920 H.264 substituted for the PNG.
An explicit `adjust-transform` did not change it.

In one process, before save, the sequence is intact. `file.import` of the same
FCPXML then `sequence.inspect` showed preset `Custom 1080x1920 30`, the WAV
`chunk0` on Audio 2 for 30 frames at start 0, and `plate` on Video 3 for 30
frames at start 0. Motion position and anchor were `Vec2 { x: NaN, y: NaN }`.
Those NaNs are what `--save` writes as null.

A portrait timeline with only the WAV, no picture asset, imports and reopens:
1080×1920, preset `Custom 1080x1920 30`. Importing the PNG alone, not on a
sequence, also leaves a project that opens.

Conclusion: FilmCraft 0.2.1 accepts this FCPXML and builds the portrait
sequence, including the still. Saving a sequence that contains a picture clip
writes NaN motion components, and the project will not open again. No JSONL
fallback was added.

## Follow-up probe — portrait timeline with a still

Not the stop condition. Run after `scriptcast-craft timeline` existed. The
FCPXML is `/tmp/scriptcast-craft-portrait/episode.fcpxml`: 1080×1920, 30 fps,
one mono WAV, one PNG (`plate`), both one second.

Portrait audio with the picture asset removed imports and reopens:

```bash
filmcraft-cli --project /tmp/scriptcast-craft-trials/portrait-audio-only/probe.fcproj --save import /tmp/scriptcast-craft-trials/portrait-audio-only/episode.fcpxml
filmcraft-cli --project /tmp/scriptcast-craft-trials/portrait-audio-only/probe.fcproj inspect
```

Inspect exited 0. Sequence settings: width 1080, height 1920, preset
`Custom 1080x1920 30`.

The same timeline with the PNG, saved, will not reopen:

```bash
filmcraft-cli --project /tmp/craft-probe-portrait.fcproj --save import /tmp/scriptcast-craft-portrait/episode.fcpxml
```

Import exited 0:

```json
{
  "documents": [
    {
      "format": "FCPXML",
      "linkedMedia": 2,
      "offlineMedia": [],
      "report": [],
      "sequences": [1]
    }
  ],
  "errors": [],
  "items": [],
  "sequences": [1]
}
```

The next `inspect` exited 1:

```
filmcraft-cli: /tmp/craft-probe-portrait.fcproj: project file is damaged: invalid type: null, expected f64 at line 1 column 2090
```

The saved project has `"Vec2":{"x":null,"y":null}` on the picture clip's motion
anchor and position. An in-memory `sequence.inspect` in the same process, before
save, shows the clip is there: Video 3 `plate`, 30 frames, start frame 0;
Audio 2 `chunk0`, 30 frames, start frame 0; frame `1080×1920`. Motion position
and anchor are `Vec2 { x: NaN, y: NaN }`. Those NaNs become JSON null on save.

The same save failure happened for a 1920×1080 still, for the still placed on
the spine instead of a lane, for an `adjust-transform` on the clip, and for a
1080×1920 H.264 in place of the PNG. Importing the PNG alone, with no sequence,
does not damage the project. Audio-only FCPXML, landscape or portrait, saves
and reopens.

Conclusion: FilmCraft 0.2.1 accepts this FCPXML and builds the portrait
sequence, then writes NaN motion vectors for any picture clip. The project file
cannot be opened again. No JSONL fallback was added.
