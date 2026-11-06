"""Session registry, compaction thresholds and --context for stage_dispatch."""
import json
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

from scriptcast.tools import stage_dispatch as sd
from scriptcast.tools.stage_context import ContextError


@pytest.fixture(autouse=True)
def _logs(tmp_path, monkeypatch):
    monkeypatch.setenv("SCRIPTCAST_SESSION_LOGS", str(tmp_path / "logs"))


@pytest.fixture
def ep(tmp_path):
    d = tmp_path / "yt02"
    d.mkdir()
    return d


def events(tmp_path):
    p = tmp_path / "logs" / "sessions.jsonl"
    return [json.loads(l) for l in p.read_text().splitlines()] if p.is_file() else []


def reg(ep):
    return json.loads(sd.registry_path(ep).read_text())


def test_context_sizes_default_and_tiers(tmp_path):
    assert sd.context_limit("gpt-5.6-sol", sd.DEFAULT_CONTEXT) == 400_000
    assert sd.context_limit("mystery", sd.DEFAULT_CONTEXT) == sd.DEFAULT_CONTEXT_FALLBACK
    t = tmp_path / "tiers.toml"
    t.write_text('[context]\n"gpt-5.6-sol" = 272000\n[models."kimi-k3"]\ncontext = 1000\n')
    sizes = sd.load_context_sizes(t)
    assert sd.context_limit("gpt-5.6-sol", sizes) == 272_000
    assert sd.context_limit("kimi-k3", sizes) == 1000
    assert sd.load_context_sizes(tmp_path / "absent.toml") == sd.DEFAULT_CONTEXT


def test_transitions_start_resume_close(ep, tmp_path):
    path, key = sd.registry_path(ep), sd.registry_key(ep, "APA", "builder")
    assert key == "yt02/APA/builder"
    e = sd.registry_start(path, key, "codex", "gpt-5.6-sol", "sid1", 1000, 4000)
    assert e["status"] == "live" and e["turns"] == 1 and e["tokens_kind"] == "reported"
    e = sd.registry_resume(path, key, 500, 100, ok=True, compacted=False)
    assert e["turns"] == 2 and e["tokens"] == 1500 and e["brief_chars"] == 4100
    summary = tmp_path / "s.md"
    sd.registry_close(path, key, str(summary))
    data = reg(ep)
    assert key not in data["sessions"]
    assert data["archive"][-1]["status"] == "closed"
    assert data["archive"][-1]["summary_path"] == str(summary)
    assert [x["event"] for x in events(tmp_path)] == ["registry_start", "registry_resume", "registry_close"]
    assert all(x["key"] == key for x in events(tmp_path))


def test_proxy_tokens_when_unreported(ep):
    path, key = sd.registry_path(ep), "yt02/S/builder"
    e = sd.registry_start(path, key, "agy", "gemini-3-pro", "c1", 0, 8000)
    assert e["tokens_kind"] == "proxy" and e["tokens"] == 2000
    e = sd.registry_resume(path, key, 0, 4000, True, False)
    assert e["tokens"] == 3000


def test_restart_supersedes_old_entry(ep):
    path, key = sd.registry_path(ep), "yt02/S/builder"
    sd.registry_start(path, key, "codex", "gpt-5.6-sol", "a", 1, 1)
    sd.registry_start(path, key, "codex", "gpt-5.6-sol", "b", 1, 1)
    data = reg(ep)
    assert data["sessions"][key]["session_id"] == "b"
    assert data["archive"][0]["status"] == "superseded"


def test_failed_resume_marks_dead(ep):
    path, key = sd.registry_path(ep), "yt02/S/builder"
    sd.registry_start(path, key, "codex", "gpt-5.6-sol", "a", 1, 1)
    assert sd.registry_resume(path, key, 0, 1, ok=False, compacted=False)["status"] == "failed"


@pytest.mark.parametrize("tokens,expect", [(59, ""), (60, "Compaction"), (79, "Compaction"),
                                           (80, "final state summary"), (95, "final state summary")])
def test_compaction_thresholds(tokens, expect):
    msg = sd.compaction_message({"tokens": tokens, "context_limit": 100})
    assert (expect in msg) if expect else msg == ""


def _cp(stdout="", rc=0):
    m = MagicMock()
    m.returncode, m.stdout, m.stderr = rc, stdout, ""
    return m


def test_cli_start_then_resume_from_registry(ep, tmp_path):
    wt = tmp_path / "wt"
    wt.mkdir()
    brief = tmp_path / "b.md"
    brief.write_text("fix overlap")
    msg = tmp_path / "m.md"
    msg.write_text("next defect")
    out = json.dumps({"session_id": "cur-1", "usage": {"inputTokens": 70, "outputTokens": 30}})
    with patch("scriptcast.tools.stage_dispatch.subprocess.run", return_value=_cp(out)), \
         patch("scriptcast.tools.stage_dispatch.get_git_info", return_value=("abc", "")):
        assert sd.main(["start", "--agent", "cursor-agent", "--model", "composer-2", "--brief", str(brief),
                        "--worktree", str(wt), "--episode", str(ep), "--scene", "APA"]) == 0
    e = reg(ep)["sessions"]["yt02/APA/builder"]
    assert e["session_id"] == "cur-1" and e["tokens"] == 100 and e["context_limit"] == 200_000

    # Push load into the compaction band, then resume without --session/--agent.
    data = reg(ep)
    data["sessions"]["yt02/APA/builder"]["tokens"] = 130_000
    sd.registry_path(ep).write_text(json.dumps(data))
    with patch("scriptcast.tools.stage_dispatch.subprocess.run", return_value=_cp(out)) as run, \
         patch("scriptcast.tools.stage_dispatch.get_git_info", return_value=("abc", "")):
        assert sd.main(["resume", "--message", str(msg), "--worktree", str(wt),
                        "--episode", str(ep), "--scene", "APA"]) == 0
    cmd = run.call_args_list[0][0][0]
    assert cmd[:6] == ["cursor-agent", "-p", "--output-format", "json", "--trust", "--resume"]
    assert cmd[6] == "cur-1" and "Compaction" in cmd[-1] and cmd[-1].startswith("next defect")
    e = reg(ep)["sessions"]["yt02/APA/builder"]
    assert e["turns"] == 2 and e["status"] == "compacted"

    assert sd.main(["close", "--episode", str(ep), "--scene", "APA", "--summary", str(msg)]) == 0
    assert sd.main(["resume", "--message", str(msg), "--episode", str(ep), "--scene", "APA"]) == 1


def test_cli_context_prepended(ep, tmp_path):
    brief = tmp_path / "b.md"
    brief.write_text("BRIEF BODY")
    fake = {"section": "Context (nlm, Book)\n\npassage\n", "cached": False}
    with patch("scriptcast.tools.stage_context.get_context", return_value=fake) as gc, \
         patch("scriptcast.tools.stage_dispatch.subprocess.run", return_value=_cp()) as run, \
         patch("scriptcast.tools.stage_dispatch.get_git_info", return_value=("abc", "")), \
         patch("scriptcast.tools.stage_dispatch.capture_session_id", return_value=("x", 0)):
        assert sd.main(["start", "--agent", "codex", "--model", "gpt-5.6-terra", "--brief", str(brief),
                        "--worktree", str(tmp_path), "--episode", str(ep), "--scene", "S",
                        "--context", "book:what is the wage field?"]) == 0
    nb, q, cache = gc.call_args[0]
    assert (nb, q) == ("book", "what is the wage field?")
    assert cache == sd.registry_path(ep).parent / "context_cache"
    brief_arg = run.call_args[0][0][-1]
    assert brief_arg.index("Context (nlm, Book)") < brief_arg.index("BRIEF BODY")


def test_cli_context_auth_failure_fails_closed(ep, tmp_path):
    brief = tmp_path / "b.md"
    brief.write_text("x")
    with patch("scriptcast.tools.stage_context.get_context", side_effect=ContextError("login", 2)), \
         patch("scriptcast.tools.stage_dispatch.subprocess.run") as run:
        assert sd.main(["start", "--agent", "codex", "--model", "m", "--brief", str(brief),
                        "--context", "book:q"]) == 2
    run.assert_not_called()


def test_cli_context_bad_spec(tmp_path):
    brief = tmp_path / "b.md"
    brief.write_text("x")
    assert sd.main(["start", "--agent", "codex", "--model", "m", "--brief", str(brief),
                    "--context", "book"]) == 1
