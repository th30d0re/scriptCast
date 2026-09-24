import json
from pathlib import Path
import sys

import pytest

from scriptcast.tools import shotspec

FIXTURES = Path(__file__).parent / "fixtures"
SHOTLIST = FIXTURES / "shotspec_shotlist.md"
SCRIPT = FIXTURES / "shotspec_episode.md"
MANIFEST = FIXTURES / "shotspec_manifest.json"


def shot(note="[design]", anchor="`Guest (00:04)`"):
    return shotspec.Shot("G-01", "Shapes", {"Anchor": anchor, "Note": note})


def test_turn_order_ignores_drift_and_manifest_list_order():
    doc = shotspec.build(SHOTLIST, MANIFEST, SCRIPT)
    assert doc["schema_version"] == 1
    assert doc["shots"][0]["anchor"] == {
        "speaker": "Guest", "timestamp": "00:04", "speaker_id": "guest",
        "turn_index": 1, "start_ms": 6000, "end_ms": 11000,
    }
    assert doc["shots"][1]["anchor"]["turn_index"] == 2
    names, turns = shotspec._manifest_lookup(MANIFEST, SCRIPT)
    assert shotspec.spec_for(shot(anchor="Host (00:01)"), names, turns)["anchor"]["start_ms"] == 0
    assert ("guest", "00:09") not in turns


def test_fallback_without_script_and_missing_script(tmp_path):
    for script in (None, tmp_path / "missing.md"):
        names, turns = shotspec._manifest_lookup(MANIFEST, script)
        spec = shotspec.spec_for(shot(anchor="Guest (00:09)"), names, turns)
        assert spec["anchor"]["start_ms"] == 6000
        assert spec["anchor"]["end_ms"] == 11000
        assert spec["anchor"]["turn_index"] == 1


def test_continued_fields_hold_prompt_and_tracks():
    doc = shotspec.build(SHOTLIST, MANIFEST, SCRIPT)
    first, second = doc["shots"]
    assert first["described"] == {"Type": "diagram", "Visual": "A blue square beside a green triangle."}
    assert first["note"] == "[source] Cards 15 and 16; Card 15. See https://example.invalid/shapes."
    assert first["hold"] == {"until": "00:08", "script_ms": 4000}
    assert second["hold"]["script_ms"] == 0
    assert first["prompt_seed"] == "Square study. Type: diagram; Visual: A blue square beside a green triangle."
    assert [s["suggested_track"] for s in doc["shots"]] == ["vector", "archival"]
    assert first["provenance"]["citations"] == [
        {"url": "https://example.invalid/shapes"}, {"card": "Card 15"}, {"card": "Card 16"}]
    assert shotspec.validate(doc["shots"]) == []


@pytest.mark.parametrize("note,valid", [
    ("[source] https://example.invalid/info", True),
    ("[source] Cards 2 and 3", True),
    ("[source] Card 15", True),
    ("[source] no reference", False),
    ("[source] Paper/x.tex:12", False),
])
def test_source_validation(note, valid):
    errors = shotspec.validate([shotspec.spec_for(shot(note))])
    assert errors == ([] if valid else [
        ("error", "G-01", "tagged [source] with no URL or card reference")])


def test_no_tag_warns():
    assert shotspec.validate([shotspec.spec_for(shot(""))]) == [("warn", "G-01", "no provenance tag")]


def test_manuscript_off_by_default(tmp_path):
    spec = shotspec.spec_for(shot('[book] [data] Paper/x.tex:12 `:9` `"a missing quoted phrase"` `sec:missing`'))
    assert spec["provenance"] == {"tags": ["book", "data"], "citations": [], "labels": [], "quotes": []}
    shotspec.resolve_quotes([spec])
    assert shotspec.validate([spec]) == []


def test_book_opt_in_and_repo_checks(tmp_path):
    paper = tmp_path / "Paper"
    paper.mkdir()
    book = paper / "x.tex"
    book.write_text('A tiny square has four sides.\n\\label{sec:shapes}\n')
    note = '[book] Paper/x.tex:12, 1 `:2` `"A tiny square has four sides"` `sec:shapes`'
    spec = shotspec.spec_for(shot(note), book=Path("Paper/x.tex"), repo=tmp_path)
    assert spec["provenance"]["citations"] == [
        {"path": "Paper/x.tex", "line": 12}, {"path": "Paper/x.tex", "line": 1},
        {"path": "Paper/x.tex", "line": 2}]
    shotspec.resolve_quotes([spec], tmp_path, Path("Paper/x.tex"))
    assert spec["provenance"]["resolved_quotes"][0]["line"] == 1
    assert shotspec.validate([spec], tmp_path, Path("Paper/x.tex")) == [
        ("error", "G-01", "Paper/x.tex:12 is past the end of the file (2 lines)")]
    spec = shotspec.spec_for(shot('[book] `:1`'), book=book)
    assert spec["provenance"]["citations"] == [{"path": str(book), "line": 1}]
    assert shotspec.validate([spec], book=book) == []


def test_repo_only_and_manuscript_errors(tmp_path):
    paper = tmp_path / "Paper"
    (paper / "data").mkdir(parents=True)
    (paper / "The_Original_Power.tex").write_text('A synthetic manuscript.\n')
    (paper / "data" / "shapes.csv").write_text('shape,count\nsquare,2\n')
    good = shotspec.spec_for(shot('[book] [data] `:1` Paper/data/shapes.csv https://example.invalid'), repo=tmp_path)
    assert shotspec.validate([good], tmp_path) == []
    bad = shotspec.spec_for(shot('[book] [data] `sec:missing` `"a missing quoted phrase"` Paper/missing.tex:1'), repo=tmp_path)
    shotspec.resolve_quotes([bad], tmp_path)
    messages = [p[2] for p in shotspec.validate([bad], tmp_path)]
    assert len(messages) == 4
    assert "cited label does not exist: sec:missing" in messages
    assert "cited file is missing: Paper/missing.tex" in messages
    empty = shotspec.spec_for(shot('[book]'), repo=tmp_path)
    assert shotspec.validate([empty], tmp_path)[0][2] == 'tagged [book] with no line or label'


@pytest.mark.parametrize("strict,expected", [(False, 0), (True, 1)])
def test_cli_strict_and_book_flag(tmp_path, monkeypatch, capsys, strict, expected):
    source = tmp_path / "shots.md"
    source.write_text('## G-01 — Shapes\n- **Anchor:** Guest (00:04)\n- **Note:** [book] Paper/x.tex:12\n')
    out = tmp_path / "nested" / "spec.json"
    args = ["scriptcast-shotspec", str(source), "--out", str(out), "--book", str(tmp_path / "book.tex")]
    monkeypatch.setattr(sys, "argv", args + (["--strict"] if strict else []))
    assert shotspec.main() == expected
    assert json.loads(out.read_text())["validation"][0]["level"] == "error"
    assert "shots.md: 1 shots, 0 anchored to real time, 1 vector" in capsys.readouterr().out
    monkeypatch.setattr(sys, "argv", args[:4] + ["--strict"])
    assert shotspec.main() == 0


def test_strict_allows_warnings(tmp_path, monkeypatch):
    source = tmp_path / "shots.md"
    source.write_text('## G-01 — Shapes\n- **Anchor:** `Host` after the greeting\n')
    monkeypatch.setattr(sys, "argv", ["scriptcast-shotspec", str(source), "--strict"])
    assert shotspec.main() == 0
