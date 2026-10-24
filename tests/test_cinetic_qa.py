from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from scriptcast.tools import cinetic_qa


def _write_executable(path: Path, text: str) -> None:
    path.write_text(text)
    path.chmod(0o755)


def _fake_cinetic(tmp_path: Path, *, probe_exit: int = 0) -> Path:
    root = tmp_path / "cinetic"
    scripts = root / "scripts"
    scripts.mkdir(parents=True)
    python_stub = """\
from __future__ import annotations
import sys
from pathlib import Path
if '--help' in sys.argv:
    print('usage: check VIDEO [--json FILE]')
    raise SystemExit(0)
print(Path(__file__).name, 'ran', ' '.join(sys.argv[1:]))
raise SystemExit(EXIT)
"""
    for name in ("probe.py", "forensics.py", "sheet.py", "check-sync.py"):
        exit_code = probe_exit if name == "probe.py" else 0
        _write_executable(scripts / name, python_stub.replace("EXIT", str(exit_code)))
    _write_executable(
        scripts / "lint-film.mjs",
        """\
#!/usr/bin/env node
if (process.argv.includes('--help')) {
  console.log('Usage: lint-film.mjs [src]\\n  src file or directory to scan');
  process.exit(0);
}
console.log('lint ran', process.argv.slice(2).join(' '));
""",
    )
    return root


def _media(tmp_path: Path) -> tuple[Path, Path]:
    run_dir = tmp_path / "run"
    run_dir.mkdir()
    video = run_dir / "clip001.mp4"
    audio = run_dir / "000.mp3"
    video.write_bytes(b"not real video")
    audio.write_bytes(b"not real audio")
    return video, audio


def test_combines_statuses_and_returns_failure(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    root = _fake_cinetic(tmp_path, probe_exit=7)
    video, audio = _media(tmp_path)
    out = tmp_path / "qa"
    monkeypatch.setattr(cinetic_qa, "_has_cv2", lambda: True)

    report = cinetic_qa.run(video, root, out)

    assert report["probe"]["status"] == "fail"
    assert report["probe"]["exit_code"] == 7
    assert report["forensics"]["status"] == "pass"
    assert report["sheet"]["status"] == "pass"
    assert report["check-sync"]["status"] == "pass"
    assert str(audio) in (out / "check-sync.log").read_text()
    assert report["lint-film"]["status"] == "pass"
    assert report["av-audit"]["status"] == "n/a"
    assert json.loads((out / "report.json").read_text()) == report
    assert "--spec @30" in (out / "probe.log").read_text()


def test_missing_root_is_a_clear_cli_error(tmp_path: Path) -> None:
    video, _ = _media(tmp_path)
    missing = tmp_path / "does-not-exist"

    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "scriptcast.tools.cinetic_qa",
            str(video),
            "--cinetic-root",
            str(missing),
        ],
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 2
    assert f"cinetic root does not exist: {missing}" in result.stderr
    assert "pass --cinetic-root or set CINETIC_ROOT" in result.stderr


def test_missing_cv2_skips_dependent_checks(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    root = _fake_cinetic(tmp_path)
    video, _ = _media(tmp_path)
    out = tmp_path / "qa"
    monkeypatch.setattr(cinetic_qa, "_has_cv2", lambda: False)

    report = cinetic_qa.run(video, root, out)

    expected = {"status": "skipped", "exit_code": None, "summary": "missing dependency cv2"}
    assert report["forensics"] == expected
    assert report["sheet"] == expected
    assert (out / "forensics.log").read_text() == "missing dependency cv2\n"
    assert (out / "sheet.log").read_text() == "missing dependency cv2\n"
