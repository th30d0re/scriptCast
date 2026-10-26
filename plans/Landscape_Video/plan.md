# Plan: 1920×1080 YouTube picture, on the episode clock

Host: `scriptCast`, branch `dev`. Do not start until the open pull request is on `main`. Build here, and open a pull request to `main` only when a fixture episode cuts cleanly.

The Original Power already makes a standard YouTube frame. ScriptCast renders the speech, the manifest, the stitch, and the word timings. A show-local editor, `edit16x9.py` beside `common16x9.py` in that repo, cuts silent picture runs onto that clock and lays the stitched MP3 underneath. The scenes themselves (the manim files, the Figma frames, the episode's shot list) stay in the show. What moves here is the clock and the cut, so the next show does not copy that editor.

This does not replace the vertical pipeline. `scriptcast-video` stays 1080×1920. A YouTube cut is a second target on the same manifest.

## What The Original Power does today

`edit16x9.py` treats one shot as one manifest turn. A run is a primary shot plus the holds that point at it. Each run is rendered once, silent. Shot `i` then occupies frames `[F(i), F(i+1))`, where `F(t) = round(t × fps)` and `t` is the turn's `start_ms` from `episode_manifest.json`. A cut taken that way cannot drift against the voiceover.

Picture for a shot comes from one of three places: a manimgx scene, an `ext:` video file conformed with ffmpeg, or a Figma card. A missing run becomes a flat placeholder so the segment still cuts. Preview is 960×540 at 15 fps. The deliverable is 1920×1080 at 30 fps. The voice is the episode MP3, trimmed to the segment, not a second mix.

Most of YT02v2 is the third kind. The shot list says `figma-card:cards/….json`. An agent writes that JSON from the narration. A second agent builds the frame in Figma through the Figma MCP, in the house style, and exports a silent MP4. `card_beats_v2.py` turns each manifest turn into a reveal time (`start_ms` relative to the run, plus a quarter second). `edit16x9.py` then cuts that export onto the clock like any other run. Equations stay in manim, because Figma does not set math. Figma's own video export is the fragile step: a server render, a handful at a time, links that expire, and a word-by-word text reveal that comes back as one fade.

Word cues (`cue(seg, turn, word)`) read `words.json` from `scriptcast-word-timings`. Those cues belong to the scene, which stays in the show.

## FigCraft is the design tool in the loop

Figma stays a file a person can open. It is no longer the path a card has to take. The YT02 dashboard was already built in FigCraft through its command API: 968 calls, seven 1920×1080 frames, PNG export, in under a second of command time. That API is now an MCP server in the open-source tree, [storytold/figcraft](https://github.com/storytold/figcraft) (`crates/mcp`, MIT OR Apache-2.0). `figcraft-cli mcp` runs it headless. `figcraft-cli mcp --attach <port>` drives a running app so the operator sees each change. Both dispatch the same command ids as the TCP control channel (`create_node`, `batch`, `run_command`, `export`, `screenshot`).

FigCraft does not export a video. It exports a PNG, an SVG, a PDF, or the native `.figcraft` file. That is the right grain for this clock. One frame per beat, held until the next turn, is a silent run the cutter already knows how to build. It avoids Figma's server render, the expiring links, and the word-by-word reveal that comes back as a single fade. A beat that has to move, rather than hold, is still a manim scene or an `ext:` file.

The installed app is 0.6.0. Its control channel stalls when the window is covered. The current docs say a later build keeps answering while covered, and the headless CLI never needs the window. ScriptCast talks to `figcraft-cli`, not to port 47831 on the old app.

What ScriptCast owns:

- The card JSON (kind, items, sources) and the rule that item `i` is visible from beat `i`.
- Beats computed from the manifest, written next to the episode.
- A `figcraft` action. It starts a headless session, builds one 1920×1080 frame per beat from the card JSON, and writes a PNG per beat into the episode. The same JSON builds the same frames. An agent may design a new kind through the MCP and save a `.figcraft` file. The action then opens that file and exports it. It does not invent a layout.
- Conform turns those PNGs into the silent MP4, each held for its beat. The cutter treats that file like any other run.

FigCraft is an external binary, the way ffmpeg is. ScriptCast does not vendor the engine. A show that still has a finished Figma MP4 can conform that file. HyperFrames is not in this path.

## What lands in ScriptCast

A new command, `scriptcast-youtube`, with five actions:

- `check` confirms every shot names a render target that exists.
- `render` is a hook the show fills in. ScriptCast does not vendor manimgx, and it does not render a scene it was not given. The default hook shells out to a command the project config names.
- `conform` scales an external video to the frame, sets the frame rate, and pads or trims it to the run's frame count. Silent H.264.
- `cut` builds one segment MP4: constant frame rate, trim by frame index, concatenate, then the episode MP3 underneath. A missing run is a placeholder of the right length.
- `sheet` writes one still per shot, at the frame the viewer sees.

Frame sizes are only `960x540` (preview) and `1920x1080` (deliverable), at 15 or 30 fps. The frame index uses the same rounding as `edit16x9.py`. The shot list is the one `scriptcast-shotspec` already resolves onto manifest time. One shot per turn is the rule for this cutter. A list that does not match the manifest turn count is an error, not a best effort.

Project config gains an optional `youtube` table: the shot-list path, the render command, and the output directory under the episode (`video/preview/<WxH>/`). Relative paths stay relative to the project root.

Tests use a fixture manifest, a fixture shot list, and ffmpeg color sources. No manimgx, no network, no Original Power checkout. Two assertions matter: the cut's duration matches the manifest window for that segment, and a shot boundary falls on the same frame as `round(start_ms / 1000 × fps)`.

## What stays in the show

Every manim scene, every shot list, and the decision of which turn holds on which primary. A `.figcraft` file an agent designed for one episode stays in the show. The driver that turns card JSON into frames does not. The Original Power keeps `edit16x9.py` until `scriptcast-youtube cut` reproduces one of its segments. Then that editor becomes a thin caller, not a second implementation.

## Order

1. Frame index and the cut, with ffmpeg fixtures. No render hook yet.
2. Conform an external file onto a run.
3. The render hook. Beats are a JSON file computed from the manifest. A `figcraft` shot exports one PNG per beat through `figcraft-cli` and conforms them into the silent run. A shot that already has an MP4 is conformed as it is.
4. Point one Original Power card run at the new command and compare duration and frame boundaries. Do not copy the scenes across.

Out of scope: replacing the 1080×1920 Remotion cards, generating manim, and showing the picture in the desktop app. The window is a separate plan, and it waits on Figma.
