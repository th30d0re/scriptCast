"""Tests for scriptcast-stage-context with an nlm stub on PATH."""
import json
import os
from pathlib import Path
import stat

import pytest

from scriptcast.tools import stage_context as sc

BOOK = "d3f54f50-61a5-465e-a94a-55d17326c0e2"
PERSUASION = "28147d00-0394-4241-ac8f-7b35891c351b"

QUERY_JSON = {
    "value": {
        "answer": "Racism is psycho-legal social software [1].\n\n💡 Would you like more?",
        "references": [
            {"source_id": "s1", "citation_number": 2, "cited_text": "Second passage."},
            {"source_id": "s1", "citation_number": 1, "cited_text": "First   passage\nwith breaks."},
            {"source_id": "s1", "citation_number": 3, "cited_text": "First passage with breaks."},
        ],
    }
}
GET_JSON = {"value": {"sources": [{"id": "s1", "title": "The_Original_Power.pdf"}]}}


@pytest.fixture
def nlm_stub(tmp_path, monkeypatch):
    """Install a fake `nlm` that records calls; mode file switches behaviour."""
    bindir = tmp_path / "bin"
    bindir.mkdir()
    calls = tmp_path / "calls.log"
    mode = tmp_path / "mode"
    mode.write_text("ok")
    (tmp_path / "query.json").write_text(json.dumps(QUERY_JSON))
    (tmp_path / "get.json").write_text(json.dumps(GET_JSON))
    script = bindir / "nlm"
    script.write_text(f"""#!/bin/sh
echo "$@" >> {calls}
m=$(cat {mode})
if [ "$m" = auth ]; then echo "Authentication expired. Run nlm login" >&2; exit 1; fi
if [ "$m" = boom ]; then echo "server error" >&2; exit 3; fi
if [ "$2" = get ]; then cat {tmp_path / 'get.json'}; else cat {tmp_path / 'query.json'}; fi
""")
    script.chmod(script.stat().st_mode | stat.S_IEXEC)
    monkeypatch.setenv("PATH", f"{bindir}{os.pathsep}{os.environ['PATH']}")
    return {"calls": calls, "mode": mode}


def test_notebooks_toml_covers_book_and_episodes():
    nbs = sc.load_notebooks()
    assert nbs["book"]["id"] == BOOK and nbs["book"]["kind"] == "book"
    assert nbs["persuasion"]["id"] == PERSUASION and nbs["persuasion"]["kind"] == "public_research"
    eps = sorted(v["episode"] for k, v in nbs.items() if k.startswith("ep"))
    assert eps == list(range(1, 28))
    assert all(v["kind"] in ("book", "public_research") for v in nbs.values())


def test_resolve_alias_id_and_unknown():
    assert sc.resolve_notebook("book")["id"] == BOOK
    assert sc.resolve_notebook(PERSUASION)["alias"] == "persuasion"
    assert sc.resolve_notebook("unknown-id")["kind"] == "public_research"


def test_book_section_unfenced_and_trimmed(nlm_stub, tmp_path):
    out = sc.get_context("book", "define racism", tmp_path / "cache", max_chars=1200)
    sec = out["section"]
    assert sec.startswith("Context (nlm, The Original Power")
    assert "```" not in sec and sc.FENCE_HEAD not in sec
    assert "💡" not in sec
    # citation order, whitespace collapsed, duplicate dropped, source title mapped
    assert sec.index("First passage with breaks.") < sec.index("Second passage.")
    assert sec.count("First passage with breaks.") == 1
    assert "[The_Original_Power.pdf]" in sec


def test_public_research_fenced(nlm_stub, tmp_path):
    sec = sc.get_context("persuasion", "q", tmp_path / "cache")["section"]
    assert sc.FENCE_HEAD in sec and "```text" in sec


def test_max_chars_caps_passages(nlm_stub, tmp_path):
    out = sc.get_context("book", "q", None, max_chars=60)
    total = sum(len(l) for l in out["section"].splitlines() if l.startswith("- ["))
    assert total < 120


def test_cache_hit_skips_nlm(nlm_stub, tmp_path):
    cache = tmp_path / "cache"
    first = sc.get_context("book", "same q", cache)
    n = len(nlm_stub["calls"].read_text().splitlines())
    second = sc.get_context("book", "same q", cache)
    assert not first["cached"] and second["cached"]
    assert first["section"] == second["section"]
    assert len(nlm_stub["calls"].read_text().splitlines()) == n
    assert (cache / f"{sc.cache_key(BOOK, 'same q')}.json").is_file()
    assert sc.cache_key(BOOK, "a") != sc.cache_key(BOOK, "b")


def test_auth_failure_exits_2_with_hint(nlm_stub, capsys):
    nlm_stub["mode"].write_text("auth")
    assert sc.main(["--notebook", "book", "--query", "q"]) == 2
    assert "nlm login" in capsys.readouterr().err


def test_other_failure_exits_1(nlm_stub, capsys):
    nlm_stub["mode"].write_text("boom")
    assert sc.main(["--notebook", "book", "--query", "q"]) == 1


def test_cli_prints_section(nlm_stub, tmp_path, capsys):
    assert sc.main(["--notebook", "ep4", "--query", "q", "--cache-dir", str(tmp_path / "c")]) == 0
    assert capsys.readouterr().out.startswith("Context (nlm, ATO Ep 4")


def test_section_body_within_budget(nlm_stub, tmp_path):
    long = {"value": {"answer": "A" * 5000, "references": [
        {"source_id": "s1", "citation_number": i, "cited_text": f"{i} " + "p" * 900} for i in range(5)]}}
    (tmp_path / "query.json").write_text(json.dumps(long))
    out = sc.get_context("book", "q", None, max_chars=1200)
    body = out["section"].split("\n", 2)[2]
    assert len(body) < 1200 + 100  # bullets/source labels add a little framing
