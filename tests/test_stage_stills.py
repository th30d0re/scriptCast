"""Tests for scriptcast-stage-stills."""
import json
import math
from pathlib import Path
from unittest.mock import MagicMock, patch
from PIL import Image
import pytest

from scriptcast.tools.stage_stills import (
    make_contact_sheet,
    resolve_scene_stills,
    render_stills,
    main,
)


@pytest.fixture
def sample_plan():
    return {
        "fps": 30,
        "range": {"start_ms": 0, "end_ms": 10000},
        "cues": {
            "S01.c1": 1000,
            "S01.c2": 2000,
            "S02.c3": 5000,
        },
        "segments": [
            {
                "scene": "SceneA",
                "start_ms": 0,
                "end_ms": 4000,
                "shots": [
                    {"shot_id": "S01", "start_ms": 0, "end_ms": 4000}
                ],
            },
            {
                "scene": "SceneB",
                "start_ms": 4000,
                "end_ms": 9000,
                "shots": [
                    {"shot_id": "S02", "start_ms": 4000, "end_ms": 9000}
                ],
            },
        ],
    }


def test_resolve_scene_stills(sample_plan):
    stills = resolve_scene_stills(sample_plan)
    # SceneA has 2 cues (c1, c2) + 1 end frame = 3 stills
    # SceneB has 1 cue (c3) + 1 end frame = 2 stills
    assert len(stills) == 5

    # Check SceneA stills
    a_stills = [s for s in stills if s["scene"] == "SceneA"]
    assert len(a_stills) == 3
    assert a_stills[0]["cue"] == "c1"
    assert a_stills[0]["ms"] == 1700  # 1000 + 700
    assert a_stills[0]["frame"] == math.ceil(1700 * 30 / 1000)

    assert a_stills[1]["cue"] == "c2"
    assert a_stills[1]["ms"] == 2700  # 2000 + 700
    assert a_stills[1]["frame"] == math.ceil(2700 * 30 / 1000)

    assert a_stills[2]["cue"] == "end"
    end_frame_a = math.ceil(4000 * 30 / 1000) - 1
    assert a_stills[2]["frame"] == end_frame_a


def test_resolve_scene_stills_filter(sample_plan):
    stills = resolve_scene_stills(sample_plan, scenes_filter=["SceneA"])
    assert len(stills) == 3
    assert all(s["scene"] == "SceneA" for s in stills)


def test_make_contact_sheet(tmp_path):
    out_dir = tmp_path / "contact"
    out_dir.mkdir()

    stills = []
    for i in range(3):
        fn = f"Test_{i}.png"
        fp = out_dir / fn
        im = Image.new("RGB", (1920, 1080), color=(i * 50, 100, 150))
        im.save(fp)
        stills.append({
            "scene": "Test",
            "cue": f"cue_{i}",
            "ms": 1000 + i * 500,
            "frame": 30 + i * 15,
            "file": fn,
            "path": str(fp),
        })

    sheet_path = out_dir / "Test_sheet.png"
    make_contact_sheet("Test", stills, sheet_path)

    assert sheet_path.is_file()
    with Image.open(sheet_path) as sheet_im:
        # 3 stills in 2 columns: 2 rows of tiles
        # tile_w = 960, tile_h = 540 + 36 = 576
        # sheet_w = 2 * 960 = 1920
        # sheet_h = 2 * 576 = 1152
        assert sheet_im.size == (1920, 1152)


def test_render_stills_flow(sample_plan, tmp_path):
    plan_file = tmp_path / "plan.json"
    plan_file.write_text(json.dumps(sample_plan))

    out_dir = tmp_path / "rendered_stills"
    ep_dir = tmp_path / "ep"
    ep_dir.mkdir()

    with patch("scriptcast.tools.stage_stills.subprocess.run") as mock_run:
        mock_run.return_value = MagicMock(returncode=0, stdout="", stderr="")

        # Also create fake still files so contact sheet / files exist
        def side_effect(cmd, **kwargs):
            if "bundle" in cmd:
                # create fake bundle index.html
                out_dir_flag = [c for c in cmd if c.startswith("--out-dir=")][0]
                b_dir = Path(out_dir_flag.split("=", 1)[1])
                b_dir.mkdir(parents=True, exist_ok=True)
                (b_dir / "index.html").write_text("fake bundle")
            elif "still" in cmd:
                out_png = Path(cmd[cmd.index("StageStill") + 1])
                out_png.parent.mkdir(parents=True, exist_ok=True)
                Image.new("RGB", (1920, 1080), color=(10, 20, 30)).save(out_png)
            return MagicMock(returncode=0, stdout="", stderr="")

        mock_run.side_effect = side_effect

        ret = render_stills(
            plan_path=plan_file,
            episode_dir=ep_dir,
            out_dir=out_dir,
            scenes=["SceneA"],
            concurrency=2,
            sheet=True,
        )
        assert ret == 0

    stills_json = out_dir / "stills.json"
    assert stills_json.is_file()
    records = json.loads(stills_json.read_text())
    assert len(records) == 3
    assert all("file" in r and "scene" in r and "cue" in r and "ms" in r for r in records)

    sheet_png = out_dir / "SceneA_sheet.png"
    assert sheet_png.is_file()


def test_real_plan_tajfel_citation():
    real_plan_path = Path("/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/yt02/stage/ov/stage_plan.json")
    if not real_plan_path.is_file():
        pytest.skip("Real stage_plan.json not present")

    plan = json.loads(real_plan_path.read_text())
    stills = resolve_scene_stills(plan, scenes_filter=["TajfelCitation"])

    # TajfelCitation has 7 cues + 1 end frame = 8 stills
    assert len(stills) == 8
    cues = [s["cue"] for s in stills]
    assert cues == [
        "t22Henri",
        "t22Turner",
        "t22Formalized",
        "t22SeventyNine",
        "t22Social",
        "t22Theory",
        "t22Relations",
        "end",
    ]
    assert stills[-1]["frame"] == 8267
