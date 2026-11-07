import csv
import json
from pathlib import Path

from experiments.router_eval import r2_ab


def test_fixture_has_three_real_fixes_and_checks():
    rows = r2_ab.load_fixes()
    assert len(rows) == 3
    assert {r["id"] for r in rows} == {
        "minute-two-figure-centering", "dashboard-tray-wage-plane", "first-minute-dictionary-gutter"
    }
    for row in rows:
        assert row["file"].endswith((".tsx", ".ts", ".mjs"))
        assert row["acceptance_check"].startswith("test -f")


def test_dry_run_emits_six_rows(tmp_path, capsys):
    out = tmp_path / "tokens.csv"
    assert r2_ab.main(["--book-repo", "/Users/emmanuel/Documents/Theory/TheOriginalPower", "--output", str(out)]) == 0
    lines = list(csv.DictReader(out.open()))
    assert len(lines) == 6
    assert {r["condition"] for r in lines} == {"A", "B"}
    assert all(r["tokens_kind"] == "dry-run" for r in lines)
    assert "gpt-5.6-luna" in capsys.readouterr().out


def test_dispatch_accept_command_shape():
    cmd = r2_ab._accept_cmd(Path("dispatch.jsonl"), {"id": "x"}, True)
    assert cmd[-4:] == ["--status", "pass", "--log", "dispatch.jsonl"]


def test_execute_dispatch_command_keeps_log_argument(tmp_path, monkeypatch):
    fix = r2_ab.load_fixes()[0]
    worktree = tmp_path / "worktree"
    episode = worktree / "Architecting_the_operation/video/yt02"
    brief = episode / "stage/BRIEF_MINUTE_TWO_FIX2.md"
    brief.parent.mkdir(parents=True)
    brief.write_text("brief")
    monkeypatch.setattr(r2_ab, "_registry_exists", lambda *_: False)

    cmd = r2_ab.commands(tmp_path, worktree, fix, "A", dry_run=False)[0]

    assert "--dry-run" not in cmd
    assert cmd[-2:] == ["--log", str(worktree / ".r2-minute-two-figure-centering-A.dispatch.jsonl")]
