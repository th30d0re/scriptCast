# JSON timeline

ScriptCast emits a JSON timeline and the Remotion `Timeline` composition renders it.
The video is regenerated from data, not from design exports.

## Format

Contract: `video/src/timeline/timeline.schema.json` (TypeScript: `video/src/timeline/types.ts`).

```json
{"fps": 30, "width": 1080, "height": 1920, "durationSec": 4,
 "items": [{"card": "TitleCard", "props": {"headline": "…"}, "startSec": 0, "endSec": 2}]}
```

`card` is a name in `video/src/timeline/registry.ts`. `props` go to that card unchanged.
An unknown card name stops the render with an error.

## Emit

```python
from scriptcast.timeline_export import write_timeline
write_timeline(plan, "timeline.json")  # plan = scriptcast.tools.video_plan.build_plan(...)
write_timeline(plan, "timeline.json", size="youtube-4k")  # or "shorts", "youtube"
```

## Sizes

Cards pick their layout from the aspect: taller than wide is the Shorts portrait layout
(1080x1920, kept clear of the platform overlays), wider than tall is the YouTube landscape
layout (1920x1080, 5% title-safe margin). Any other size of the same aspect draws that layout
scaled to fit, so `youtube-4k` (3840x2160) is the 1080p layout at 2x, rendered sharp.

## Render

Run from `video/`:

```bash
npx remotion render Timeline out.mp4 --props=examples/timeline.example.json
```

The width and height in the JSON set the output size. `Timeline-YouTube` and
`Timeline-YouTube-4K` preview the example at those sizes in the Studio.
