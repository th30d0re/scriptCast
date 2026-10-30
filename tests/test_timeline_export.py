import json
from copy import deepcopy
from pathlib import Path

import pytest

from scriptcast.timeline_export import build_timeline, schema, write_timeline

VIDEO = Path(__file__).resolve().parent.parent / "video"


@pytest.fixture
def plan():
    return json.loads((VIDEO / "examples" / "plan.sample.json").read_text())


def test_cards_become_items(plan):
    before = deepcopy(plan)
    timeline = build_timeline(plan)
    assert plan == before
    assert timeline["fps"] == 30 and timeline["width"] == 1080 and timeline["height"] == 1920
    assert timeline["durationSec"] == 3.0
    assert timeline["items"] == [{
        "card": "TitleCard",
        "props": {"headline": "Two shapes", "items": [{"title": "Circle and square"}], "sources": "Synthetic example"},
        "startSec": 1.0, "endSec": 3.0,
    }]


def test_items_sorted_and_ms_rounded(plan):
    first = deepcopy(plan["cards"][0])
    first.update(shot_id="S-0", start_ms=0, end_ms=1033)
    plan["cards"].append(first)
    items = build_timeline(plan)["items"]
    assert [(i["startSec"], i["endSec"]) for i in items] == [(0.0, 1.033), (1.0, 3.0)]


def test_bad_window(plan):
    plan["cards"][0]["end_ms"] = 0
    with pytest.raises(ValueError, match="S-1"):
        build_timeline(plan)


def test_matches_schema(plan, tmp_path):
    jsonschema = pytest.importorskip("jsonschema")
    path = write_timeline(plan, tmp_path / "timeline.json")
    jsonschema.validate(json.loads(path.read_text()), schema())
    jsonschema.validate(json.loads((VIDEO / "examples" / "timeline.example.json").read_text()), schema())


@pytest.mark.parametrize("size, expected", [("shorts", (1080, 1920)), ("youtube", (1920, 1080)), ("youtube-4k", (3840, 2160))])
def test_size_overrides_plan(plan, size, expected):
    timeline = build_timeline(plan, size)
    assert (timeline["width"], timeline["height"]) == expected
    assert (plan["width"], plan["height"]) == (1080, 1920)


def test_unknown_size(plan):
    with pytest.raises(ValueError, match="youtube-4k"):
        build_timeline(plan, "imax")
