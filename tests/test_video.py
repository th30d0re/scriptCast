import json
from types import SimpleNamespace

import pytest

from scriptcast.tools import video


@pytest.fixture
def workspace(tmp_path, monkeypatch):
    root = tmp_path / "video"
    (root / "src").mkdir(parents=True)
    (root / "src/index.ts").touch()
    (root / "node_modules/.bin").mkdir(parents=True)
    (root / "node_modules/.bin/remotion").touch()
    monkeypatch.setattr(video, "VIDEO_DIR", root)
    monkeypatch.setattr(video, "DEFAULT_CHROME", tmp_path / "Chrome")
    monkeypatch.delenv("SCRIPTCAST_CHROME", raising=False)
    monkeypatch.chdir(tmp_path)
    card = tmp_path / "card data.json"
    card.write_text(json.dumps({"component": "TitleCard", "headline": "Shapes", "items": [], "sources": "Example"}))
    return root, card


@pytest.mark.parametrize("browser", ["env", "default", "none"])
def test_still_command(workspace, monkeypatch, browser):
    root, card = workspace
    if browser == "env":
        monkeypatch.setenv("SCRIPTCAST_CHROME", "/custom browser/Chrome")
    elif browser == "default":
        video.DEFAULT_CHROME.touch()
    calls = []
    monkeypatch.setattr(video.subprocess, "run", lambda cmd, **kw: calls.append((cmd, kw)) or SimpleNamespace(returncode=7))
    assert video.main(["still", card.name, "out image.png"]) == 7
    expected = ["npx", "--no-install", "remotion", "still", "src/index.ts", "TitleCard",
                str(card.parent / "out image.png"), f"--props={card}"]
    if browser != "none":
        expected.append("--browser-executable=" + ("/custom browser/Chrome" if browser == "env" else str(video.DEFAULT_CHROME)))
    assert calls == [(expected, {"cwd": root, "check": False})]


def test_component_override(workspace, monkeypatch):
    _, card = workspace
    card.write_text('{"headline": "Shapes"}')
    calls = []
    monkeypatch.setattr(video.subprocess, "run", lambda cmd, **kw: calls.append(cmd) or SimpleNamespace(returncode=0))
    assert video.main(["still", str(card), "out.png", "--component", "CompareCard"]) == 0
    assert calls[0][5] == "CompareCard"


@pytest.mark.parametrize("data", ["bad json", "[]", "{}", '{"component": ["TitleCard"]}'])
def test_bad_card(workspace, monkeypatch, data):
    _, card = workspace
    card.write_text(data)
    monkeypatch.setattr(video.subprocess, "run", lambda *a, **kw: pytest.fail("must not run"))
    with pytest.raises(SystemExit) as exc:
        video.main(["still", str(card), "out.png"])
    assert exc.value.code == 2


def test_missing_dependencies(workspace):
    root, card = workspace
    (root / "node_modules/.bin/remotion").unlink()
    with pytest.raises(SystemExit):
        video.main(["still", str(card), "out.png"])


@pytest.mark.parametrize("argv, expected", [
    (["generate", "A circle"], ["generate", "A circle", "--model", "arrow-2", "--n", "1"]),
    (["generate", "A square", "--model", "arrow-2", "--instructions", "Flat shape", "--n", "2", "--dry-run"],
     ["generate", "A square", "--model", "arrow-2", "--n", "2", "--instructions", "Flat shape", "--dry-run"]),
    (["animate", "shape art.svg", "--prompt", "Rotate", "--dry-run"],
     ["animate", "ABSOLUTE", "--prompt", "Rotate", "--dry-run"]),
    (["models"], ["models"]),
    (["models", "--dry-run"], ["models", "--dry-run"]),
])
def test_svg_command(workspace, monkeypatch, argv, expected):
    root, card = workspace
    (root / "node_modules/.bin/tsc").touch()
    expected = [str(card.parent / "shape art.svg") if x == "ABSOLUTE" else x for x in expected]
    calls = []
    monkeypatch.setattr(video.subprocess, "run", lambda cmd, **kw: calls.append((cmd, kw)) or SimpleNamespace(returncode=7))
    assert video.main(["svg", *argv]) == 7
    assert calls == [(["npm", "run", "--silent", "quiver", "--", *expected], {"cwd": root, "check": False})]


def test_svg_compile_dependency_missing(workspace):
    with pytest.raises(SystemExit) as exc:
        video.main(["svg", "models", "--dry-run"])
    assert exc.value.code == 2

def test_missing_assets(workspace, monkeypatch, capsys):
    root, card = workspace
    card.write_text(json.dumps({"component": "TitleCard", "headline": "Shapes", "svgAsset": "assets/missing.svg"}))
    with pytest.raises(SystemExit) as exc:
        video.main(["still", str(card), "out.png"])
    assert "Missing assets" in capsys.readouterr().err

    # With allow missing assets
    calls = []
    monkeypatch.setattr(video.subprocess, "run", lambda cmd, **kw: calls.append(cmd) or SimpleNamespace(returncode=0))
    video.main(["still", str(card), "out.png", "--allow-missing-assets"])
    assert len(calls) == 1

def test_missing_hero_assets(workspace, monkeypatch, capsys):
    root, card = workspace
    card.write_text(json.dumps({"component": "TitleCard", "headline": "Shapes", "art": {"src": "assets/missing.svg"}}))
    with pytest.raises(SystemExit) as exc:
        video.main(["still", str(card), "out.png"])
    assert "Missing assets" in capsys.readouterr().err

    # Create the asset and it should pass
    public = root / "public"
    (public / "assets").mkdir(parents=True, exist_ok=True)
    (public / "assets/missing.svg").touch()
    calls = []
    monkeypatch.setattr(video.subprocess, "run", lambda cmd, **kw: calls.append(cmd) or SimpleNamespace(returncode=0))
    video.main(["still", str(card), "out.png"])
    assert len(calls) == 1


def test_chunk_ranges_cover_every_frame_once():
    from scriptcast.tools.video import chunk_ranges
    ranges = chunk_ranges(0, 99, 3)
    assert ranges == [(0, 33), (34, 67), (68, 99)] or sum(b - a + 1 for a, b in ranges) == 100
    assert ranges[0][0] == 0 and ranges[-1][1] == 99
    assert all(ranges[i][1] + 1 == ranges[i + 1][0] for i in range(len(ranges) - 1))
    assert chunk_ranges(5, 6, 8) == [(5, 5), (6, 6)]
