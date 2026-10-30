"""Export an episode plan as the JSON timeline the Remotion `Timeline` composition renders.

The source is the plan from ``scriptcast.tools.video_plan.build_plan``: each
resolved card window becomes one item. The contract is
``video/src/timeline/timeline.schema.json``.
"""

from __future__ import annotations

import json
from pathlib import Path

_SCHEMA_PATH = Path(__file__).resolve().parent.parent / "video" / "src" / "timeline" / "timeline.schema.json"


def _sec(ms: float) -> float:
    return round(ms / 1000, 3)


def build_timeline(plan: dict) -> dict:
    """Return the timeline dict for a resolved plan. The plan is not changed."""
    items = []
    for entry in sorted(plan["cards"], key=lambda c: c["start_ms"]):
        if entry["end_ms"] < entry["start_ms"]:
            raise ValueError(f"{entry.get('shot_id', '?')}: end_ms is before start_ms")
        props = {key: value for key, value in entry["card"].items() if key != "component"}
        items.append({
            "card": entry["card"]["component"],
            "props": json.loads(json.dumps(props)),
            "startSec": _sec(entry["start_ms"]),
            "endSec": _sec(entry["end_ms"]),
        })
    return {
        "fps": int(plan["fps"]),
        "width": int(plan["width"]),
        "height": int(plan["height"]),
        "durationSec": _sec(plan["duration_ms"]),
        "items": items,
    }


def write_timeline(plan: dict, path: str | Path) -> Path:
    """Write the timeline JSON for ``plan`` to ``path`` and return the path."""
    target = Path(path)
    target.write_text(json.dumps(build_timeline(plan), indent=2) + "\n", encoding="utf-8")
    return target


def schema() -> dict:
    return json.loads(_SCHEMA_PATH.read_text(encoding="utf-8"))
