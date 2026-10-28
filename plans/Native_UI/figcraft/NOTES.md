# FigCraft Build Notes — scriptCast Phase N (N1–N4)

## Status

- **Session:** 2026-10-28 17:02–17:05 EDT
- FigCraft.app was running and was not listening on `127.0.0.1:47831`. The frames were built with `figcraft-cli run`, which does not use the window or the control port.
- A one-frame smoke (`file.new`, `frame.create`, `text.create`, `file.export`) wrote `/tmp/figcraft-cli-smoke.png` before the four screens.

## Commands

`plans/Native_UI/figcraft/build_phase_n.py` writes a JSON command list and runs `figcraft-cli run`.

- `file.new`, `page.rename`, `page.setBackground`
- `frame.create` for N1 Mix, N2 Captions, N3 Tape-stop and Craft, N4 Effects, each 1440×900
- `command.batch` of `node.create` / `text.create` for the layers, parented to that frame
- `file.saveAs` → `phase_n.figcraft`
- `file.export` format `fig` → `phase_n.fig`
- `file.export` format `png` → `N1.png` … `N4.png`

## Font

`text.fonts` lists `OpenDyslexic3` (Regular, Bold) and `OpenDyslexicMono` (Regular). The family name `OpenDyslexic` is not in the list. Body text is OpenDyslexic3. Times, paths, and character counts are OpenDyslexicMono.

## What the frames do not match

- The waveform bars are drawn rectangles, not peaks from an episode.
- Buttons are text on a rounded rectangle. There are no Lucide icons.
- The title bar is a 38 px strip with the window title. It does not draw traffic lights.
- Channel Strip has trim, gate, compressor threshold, ratio, HPF, and one EQ line (`Low / Mid / High`). It does not have three separate EQ sliders.
- The caption preview is a 16:9 dark well with the current line. It is not a video picture.

Colours, panel borders, the header, and the footer follow `ui/src/theme.rs` and the L2 voices screen.

## Earlier attempt (2026-10-17)

TCP to `127.0.0.1:47831` connected, and `ui.inspect` timed out. That run stopped without activating the window.
