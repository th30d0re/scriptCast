import base64
import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

from scriptcast.project import Project
from scriptcast.tools import youtube as cli
from scriptcast.youtube import (
    card_commands,
    frame_index,
    frame_marks,
    hold_durations,
    load_shots,
    write_beats,
)

PNG = base64.b64decode(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg=="
)


def _need_ffmpeg() -> None:
    if shutil.which("ffmpeg") is None or shutil.which("ffprobe") is None:
        pytest.skip("ffmpeg is not installed")


def _probe(path: Path, entry: str) -> str:
    return subprocess.check_output(
        [
            "ffprobe", "-v", "error", "-select_streams", "v:0",
            "-count_frames", "-show_entries", entry, "-of", "csv=p=0", str(path),
        ],
        text=True,
    ).strip()


def _episode(tmp_path: Path) -> Path:
    episode = tmp_path / "episode"
    episode.mkdir()
    manifest = {
        "episode_id": "episode",
        "turns": [
            {"start_ms": 0, "end_ms": 1000},
            {"start_ms": 1000, "end_ms": 2000},
        ],
    }
    (episode / "episode_manifest.json").write_text(json.dumps(manifest))
    shots = tmp_path / "shots.md"
    shots.write_text(
        "## S00-01 — Title\n"
        "- **Type:** card\n"
        "- **Render:** `figcraft:cards/title.json`\n"
        "\n"
        "## S00-02 — Hold\n"
        "- **Type:** hold\n"
        "- **Render:** `figcraft:cards/title.json`\n"
        "- **Holds:** `S00-01`\n"
    )
    cards = tmp_path / "cards"
    cards.mkdir()
    (cards / "title.json").write_text(json.dumps({
        "kind": "title",
        "headline": "What it is",
        "items": [
            {"title": "First", "detail": "One"},
            {"title": "Second", "detail": "Two"},
        ],
        "sources": "The book",
    }))
    return episode


def test_frame_index_rounds_half_up() -> None:
    assert frame_index(0, 30) == 0
    assert frame_index(1, 30) == 30
    assert frame_index(1 / 30, 30) == 1


def test_shot_boundary_is_the_manifest_frame() -> None:
    turns = [{"start_ms": 0, "end_ms": 1000}, {"start_ms": 1000, "end_ms": 2000}]
    marks = frame_marks(turns, 15)
    assert marks[1] == frame_index(1.0, 15)
    assert marks[1] - marks[0] == 15


def test_beats_reveal_a_quarter_second_after_the_turn(tmp_path: Path) -> None:
    episode = _episode(tmp_path)
    assert cli.main(["beats", str(episode), "--shots", str(tmp_path / "shots.md")]) == 0
    sheet = json.loads((episode / "video" / "beats.json").read_text())
    run = sheet["runs"][0]
    assert run["duration"] == 2.0
    assert [beat["t"] for beat in run["beats"]] == [0.25, 1.25]
    assert hold_durations(run) == [1.25, 0.75]


def test_second_frame_keeps_the_first_item(tmp_path: Path) -> None:
    episode = _episode(tmp_path)
    shots = load_shots(tmp_path / "shots.md")
    beats = write_beats(shots, [
        {"start_ms": 0, "end_ms": 1000},
        {"start_ms": 1000, "end_ms": 2000},
    ], tmp_path / "beats.json")
    run = json.loads(beats.read_text())["runs"][0]
    card = json.loads((tmp_path / "cards" / "title.json").read_text())
    commands, exports = card_commands(card, run["beats"], tmp_path / "frames")
    texts = [step["params"].get("text") for step in commands]
    assert "First" in texts and texts.index("Second") > texts.index("First")
    first_frame = [step for step in commands if step["params"].get("name") == "beat-0"]
    assert first_frame and first_frame[0]["params"]["width"] == 1920
    assert len(exports) == 2
    assert "Second" not in [
        step["params"].get("text") for step in commands
        if step["params"].get("parent", {}).get("$ref") == "beat-0"
    ]


def test_check_requires_the_card_file(tmp_path: Path) -> None:
    episode = _episode(tmp_path)
    assert cli.main(["check", str(episode), "--shots", str(tmp_path / "shots.md")]) == 0
    (tmp_path / "cards" / "title.json").unlink()
    assert cli.main(["check", str(episode), "--shots", str(tmp_path / "shots.md")]) == 1


def test_youtube_table_in_project_config(tmp_path: Path) -> None:
    (tmp_path / "scriptcast.toml").write_text(
        '[youtube]\nshots = "video/shots.md"\nrender = "echo render"\n',
        encoding="utf-8",
    )
    project = Project(tmp_path)
    assert project.youtube_shots == tmp_path / "video" / "shots.md"
    assert project.youtube_render == "echo render"


def test_cut_duration_matches_the_manifest_window(tmp_path: Path) -> None:
    _need_ffmpeg()
    episode = _episode(tmp_path)
    run = episode / "video" / "runs" / "960x540@15"
    run.mkdir(parents=True)
    subprocess.check_call([
        "ffmpeg", "-v", "error", "-y", "-f", "lavfi",
        "-i", "color=c=0x0F1B33:s=960x540:r=15",
        "-frames:v", "45", "-pix_fmt", "yuv420p", str(run / "S00-01.mp4"),
    ])
    assert cli.main([
        "cut", str(episode), "--shots", str(tmp_path / "shots.md"),
        "--seg", "00", "--res", "960x540", "--fps", "15", "--no-audio",
    ]) == 0
    out = episode / "video" / "preview" / "960x540" / "SEG00.mp4"
    assert int(_probe(out, "stream=nb_read_frames")) == 45


def test_conform_pads_to_the_run_length(tmp_path: Path) -> None:
    _need_ffmpeg()
    src = tmp_path / "src.mp4"
    subprocess.check_call([
        "ffmpeg", "-v", "error", "-y", "-f", "lavfi",
        "-i", "color=c=red:s=320x240:r=30",
        "-frames:v", "10", "-pix_fmt", "yuv420p", str(src),
    ])
    out = tmp_path / "run.mp4"
    assert cli.main([
        "conform", "--src", str(src), "--out", str(out),
        "--frames", "8", "--res", "960x540", "--fps", "15",
    ]) == 0
    assert _probe(out, "stream=width,height") == "960,540"
    assert int(_probe(out, "stream=nb_read_frames")) == 8


def test_figcraft_exports_one_png_per_beat(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    _need_ffmpeg()
    episode = _episode(tmp_path)
    fake = tmp_path / "figcraft-cli"
    log = tmp_path / "log.jsonl"
    png = tmp_path / "pixel.png"
    png.write_bytes(PNG)
    fake.write_text(
        "#!/usr/bin/env python3\n"
        "import json, os, sys\n"
        "from pathlib import Path\n"
        "log = Path(os.environ['FIGCRAFT_LOG'])\n"
        "png = Path(os.environ['FIGCRAFT_PNG']).read_bytes()\n"
        "def reply(obj):\n"
        "    sys.stdout.write(json.dumps(obj) + '\\n')\n"
        "    sys.stdout.flush()\n"
        "for line in sys.stdin:\n"
        "    msg = json.loads(line)\n"
        "    method = msg.get('method')\n"
        "    if method == 'notifications/initialized' or 'id' not in msg:\n"
        "        continue\n"
        "    if method == 'initialize':\n"
        "        reply({'jsonrpc':'2.0','id':msg['id'],'result':{'protocolVersion':'2025-06-18','capabilities':{}}})\n"
        "        continue\n"
        "    name = msg['params']['name']\n"
        "    args = msg['params'].get('arguments') or {}\n"
        "    with log.open('a') as handle:\n"
        "        handle.write(json.dumps({'tool': name}) + '\\n')\n"
        "    if name == 'run_command':\n"
        "        body = {'id': 2}\n"
        "    elif name == 'export':\n"
        "        Path(args['path']).parent.mkdir(parents=True, exist_ok=True)\n"
        "        Path(args['path']).write_bytes(png)\n"
        "        body = {'ok': True}\n"
        "    else:\n"
        "        body = {'ok': True}\n"
        "    reply({'jsonrpc':'2.0','id':msg['id'],'result':{'content':[{'type':'text','text':json.dumps(body)}]}})\n"
    )
    fake.chmod(0o755)
    monkeypatch.setenv("FIGCRAFT_LOG", str(log))
    monkeypatch.setenv("FIGCRAFT_PNG", str(png))
    monkeypatch.setenv("SCRIPTCAST_PROJECT", str(tmp_path))
    assert cli.main([
        "figcraft", str(episode), "--shots", str(tmp_path / "shots.md"),
        "--primary", "S00-01", "--figcraft", str(fake),
        "--res", "960x540", "--fps", "15",
    ]) == 0
    tools = [json.loads(line)["tool"] for line in log.read_text().splitlines()]
    assert tools[0] == "run_command"
    assert tools[-2:] == ["export", "export"]
    assert "batch" not in tools
    run = episode / "video" / "runs" / "960x540@15" / "S00-01.mp4"
    assert run.is_file()
    assert int(_probe(run, "stream=nb_read_frames")) == 30


def test_render_hook_receives_the_primary(tmp_path: Path) -> None:
    episode = _episode(tmp_path)
    marker = tmp_path / "marker.txt"
    hook = tmp_path / "hook.py"
    hook.write_text(
        "import os\n"
        f"from pathlib import Path\n"
        f"Path({str(marker)!r}).write_text(os.environ['SCRIPTCAST_PRIMARY'])\n"
    )
    assert cli.main([
        "render", str(episode), "--shots", str(tmp_path / "shots.md"),
        "--primary", "S00-01", "--command", f"{sys.executable} {hook}",
    ]) == 0
    assert marker.read_text() == "S00-01"
