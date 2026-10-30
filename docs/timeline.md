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
```

## Render

Run from `video/`:

```bash
npx remotion render Timeline out.mp4 --props=examples/timeline.example.json
```
