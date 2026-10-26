# Brief: design scriptCast Phase N screens in FigCraft (Gemini)

## Rules (read first)

- **Never run git.**
- **Never start other agent CLIs.** Never use `osascript`, never activate, launch, quit or kill any app (including FigCraft), and never use `pkill`/`kill`.
- **Write only** under `/Users/emmanuel/Dev/Tools/scriptCast/plans/Native_UI/figcraft/`; scratch goes in `/tmp`.
- **Read-only:** everything else, including `ui/` and the real project.
- **No Figma.** This is a FigCraft-only task.

## FigCraft control channel (measured)

- FigCraft is **running now**, started by the operator with `FIGCRAFT_CONTROL_PORT=47831`, and answering on `127.0.0.1:47831`. A `ui.inspect` call returned ok on 2026-10-17.
- **Protocol:** JSON lines over TCP. Send `{"id": 1, "method": "...", "params": {...}}`, one line in and one line out.
- **A working client already exists:** `/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/yt02/figcraft/build_dashboard.py` (read-only), from an earlier Gemini run that built a full dashboard this way. Read it and its `NOTES.md` in the same folder before starting. Copy its socket client into your own script here.
- **Which call to use:**
  - `engine.execute {command, params}` returns results, such as created node ids and text bounds;
  - `ui.menu.invoke` only returns `ok`;
  - `engine.commands {filter}` lists commands with their parameter docs. Read a command's params before you use it.
- **If calls start timing out,** the FigCraft window is probably covered by another window. Stop, write "FigCraft window needs to be visible" in NOTES.md, and exit. Do **not** try to bring it forward yourself.

## Goal

Design four 1440×900 frames, N1–N4, for scriptCast, a dark-theme macOS app for rendering podcast episodes from a script. The new frames must look like **the same app** as the existing screens.

## Visual context

Look at these PNGs first. They are the existing, approved screens, and the new frames must match them:

- `/Users/emmanuel/Dev/Tools/scriptCast/ui/design/J1-new-episode.png`, `J4-pipeline.png`, `K3-tag-toolbar.png`, `L2-voices.png`, `L3-clips.png`, `M1-clip-maker.png`, `M3-audition.png` and `M4-calibrate-outliers.png`
- `ui/design/screens/` (screens 01–06)
- **Design tokens:** `ui/design/tokens.json` and `ui/design/DESIGN.md`. The colours below come from `ui/src/theme.rs`, which is the authority.
  - BG_WINDOW `#17191C`
  - BG_SURFACE `#1E2024`
  - BG_RAISED `#272A2F`
  - BG_CONTROL `#30343A`
  - BG_SELECTED `#2A3B56`
  - BORDER `#363A41`
  - TEXT_PRIMARY `#EDF0F4`
  - TEXT_MUTED `#A4ABB6`
  - TEXT_SUBTLE `#717985`
  - ACCENT `#8CAFF3`
  - ON_ACCENT `#151C29`
  - PASSED `#9ABEB0`
  - WARNING `#D4B070`
  - FAILED `#F08287`
  - DANGER_PILL `#C44852`
  - **Radii:** control 5, card 8, dialog 12. Title bar 38 px; episode header 76 px.
- **Font:** OpenDyslexic if FigCraft has it, otherwise the closest available. Record which one you used.
- **Every screen has:**
  - the app header: a logo mark, "scriptCast", the project name "TheOriginalPower" and its root path, with "Project settings" on the right;
  - bordered panels on `BG_SURFACE`;
  - a footer bar with status text on the left and Revert + a primary action on the right.
- **No tooltips.**
- **No Figma annotation strips** (the "J1 Validate…" bars at the bottom of the reference PNGs are annotations, not UI).

## The four frames

**N1 — Mix editor**
- **Loudness strip (top):** integrated LUFS, short-term LUFS and true peak against a −16 LUFS target.
- **Timeline strip:** a waveform, with overlay flags, gain bars, and per-turn loudness dots; outliers in the warning colour.
- **Left:**
  - an **Overlays** list. Each card has: the file `archive/sources/nice.wav` with Play; the anchor turn chosen by its text ("Turn 042 · …sixty-nine days after she'd already signed…"); an "after word" snippet with the matched word highlighted; delay ms; and a gain dB slider (−40 to 0, default −22). One card is in a red error state: "No recognised word matches ^(69|sixtynine)$ in turn 042".
  - a **Turn gains** list: turn, gain dB (−12 to +12), and Play.
- **Right:** "Render mix" primary with a progress line, the output `ATO_EP04_mixed.mp3`, and an A/B player (Stitched / Mixed buttons, a shared scrubber, and the current turn).
- **Footer:** "Placement follows the script text, so it survives re-timing. The stitched MP3 is never changed." Revert, Save mix.yaml.

**N2 — Captions**
- **Left settings:**
  - max characters per line (a slider, 20–60, default 42);
  - the Whisper model;
  - a "Reuse recognised words" checkbox showing the cached file name;
  - the output path (`ATO_EP04.captions.json` / `.srt`);
  - "Generate captions" with a progress line.
- **Main:** a waveform, and a large 16:9 dark caption preview box showing the current line.
- **Below it:** a caption list (index, start → end, text, character count). Rows over the maximum are red.

**N3 — Tape-stop and Craft export** (two panels)
- **Tape-stop:**
  - a turn picker (number plus first words) or an exact time;
  - a duration slider (0.6 s) and a curve slider (1.6), with a small speed-over-time curve drawn beside them;
  - the output file;
  - "Render tape-stop" and an A/B Play.
- **Craft export:** two cards, Cue sheet and Timeline, each with a one-line description, an output path and "Export". After an export, a result row shows the path with "Reveal in Finder" and "Copy path".

**N4 — Effects**
- **Left:** a speaker list (colour dot, name, engine chip): Toussaint, Aisha and Emmanuel Theodore on OmniVoice; Jala Chantelle, The Speech Prof and Nick Powers on archive. "Master" sits at the bottom.
- **Right, with Aisha selected:** a horizontal chain of effect cards, each with an on/off switch:
  - **Channel Strip:** trim, gate threshold, compressor threshold/ratio, low/mid/high EQ, HPF;
  - **De-Esser:** frequency, amount;
  - **Reverb:** size, mix.

  Plus "Bypass all" A/B and Play selected turn.
- **Master state as an inset:** Maximizer (target LUFS, ceiling), Dither, and the loudness readout.
- **Footer:** "Applied when stitching and mixing; Live and Logic sessions keep the dry audio." Revert, Save.

## Deliver (all in `plans/Native_UI/figcraft/`)

1. `build_phase_n.py`, the script that builds the frames over the control channel. Every layer gets a meaningful name.
2. `phase_n.figcraft`, the saved native file, and `phase_n.fig`, using `file.export fig`.
3. `N1.png` … `N4.png`, exported at 1440 wide.
4. `NOTES.md`, covering:
   - the commands you used;
   - the font used;
   - what you could not match from the reference PNGs, and why;
   - the time taken.

   Write it early and keep it updated.

## Self-check before finishing

Open each exported PNG next to `ui/design/L2-voices.png` and `M3-audition.png`. Then fix anything that looks like a different app: colours, panel borders, spacing, type sizes, the header, the footer.
