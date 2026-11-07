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
