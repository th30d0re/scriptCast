# Plan: See the YouTube cut in the window

Host: `scriptCast` `ui/`. Draw the frames in Figma before any Rust. No implementer starts from a live Figma call. The account is on the Starter plan. When a frame is agreed, snapshot it into `ui/design/screens/` the same way screens 01–06 were snapshotted, and build from that copy.

This plan starts after `scriptcast-youtube` can cut a segment. Until then there is no picture file for the window to open. The vertical Remotion render is a different shape and is not what these frames show.

## What the window is for

The operator is already looking at a turn, hearing it, and reading the transcript. The missing piece is the frame that will be on YouTube at that moment: which shot, whether the picture exists, and whether it stays locked to the voice. Playback stays on the clock the app already has. The video follows the playhead. It does not keep a second clock.

## Frames to draw

Design at 1440×900, dark, on top of the current episode window (header, timeline, table, transport). OpenDyslexic is the face the app will use. Draw the layout in the existing type if that is faster, and do not invent a second chrome.

1. **Picture present.** A 16:9 frame in the inspector, above the selected turn. It shows the shot that covers the playhead. The transport is the one already on screen. Scrubbing the seek bar moves the frame.
2. **Picture missing.** The same slot, with the shot id and the length, and a clear empty state. The turn still plays. A missing frame is not an error banner over the whole window.
3. **Shot against the turn.** The selected turn names its shot. A hold shows the primary it belongs to. The table does not grow a second timeline.
4. **Format.** A control that says YouTube (1920×1080) or Reel (1080×1920). Switching it changes which rendered files the pane reads. It does not re-render.
5. **Cut in progress.** The existing job runner, with a preset that runs `scriptcast-youtube cut` for the selected segment. Log, cancel, and completion reload the pane. No new progress widget.

Skip, until those five exist as snapshots: drawing on the frame, trimming a shot in the window, uploading, captions burned in by the app, and a preview that mixes a new voiceover.

## After the snapshots

Build in the order of the frames. The pane reads files the cutter already wrote (`video/preview/<WxH>/`). It does not call ffmpeg itself. A turn with no still and no segment shows frame 2. Space, stop, and auto-advance behave as they do now. Adding the pane does not change how a turn is selected or how a job is confirmed.

Acceptance is a snapshot test on a fixture episode that has one real still and one missing shot, plus a manual pass: play across a shot boundary and watch the frame change with the voice.
