"""Export an episode plan as the JSON timeline the Remotion `Timeline` composition renders.

The source is the plan from ``scriptcast.tools.video_plan.build_plan``: each
resolved card window becomes one item. The contract is
``video/src/timeline/timeline.schema.json``.
"""

from __future__ import annotations

import json
from pathlib import Path

# Output sizes by name. Cards lay out portrait or landscape from the aspect; 4K draws the
# landscape layout at 2x.
SIZES = {
    "shorts": (1080, 1920),
    "youtube": (1920, 1080),
    "youtube-4k": (3840, 2160),
}

_SCHEMA_PATH = Path(__file__).resolve().parent.parent / "video" / "src" / "timeline" / "timeline.schema.json"


def _sec(ms: float) -> float:
    return round(ms / 1000, 3)


def build_timeline(plan: dict, size: str | None = None) -> dict:
    """Return the timeline dict for a resolved plan. The plan is not changed.

    ``size`` names an entry in ``SIZES`` and overrides the plan's width and height.
    """
    if size is not None and size not in SIZES:
        raise ValueError(f"unknown size {size!r}; expected one of {', '.join(SIZES)}")
    width, height = SIZES[size] if size else (int(plan["width"]), int(plan["height"]))
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
        "width": width,
        "height": height,
        "durationSec": _sec(plan["duration_ms"]),
        "items": items,
    }


def write_timeline(plan: dict, path: str | Path, size: str | None = None) -> Path:
    """Write the timeline JSON for ``plan`` to ``path`` and return the path."""
    target = Path(path)
    target.write_text(json.dumps(build_timeline(plan, size), indent=2) + "\n", encoding="utf-8")
    return target


def schema() -> dict:
    return json.loads(_SCHEMA_PATH.read_text(encoding="utf-8"))
