"""Tests for scriptcast-stage-dispatch CLI wrapper."""
import json
from pathlib import Path
from unittest.mock import MagicMock, patch
import pytest

from scriptcast.tools.stage_dispatch import (
    build_start_cmd,
    build_resume_cmd,
    capture_session_id,
    main,
    make_brief_header,
    collect_session_log,
    WORKER_LOG,
)


@pytest.fixture(autouse=True)
def _session_logs_in_tmp(tmp_path, monkeypatch):
    monkeypatch.setenv("SCRIPTCAST_SESSION_LOGS", str(tmp_path / "session-logs"))



def test_argument_lists_exact(tmp_path):
    wt = tmp_path / "worktree"
    wt.mkdir()
    brief = "Test brief instructions"

    # 1. cursor-agent
    cmd_cursor = build_start_cmd("cursor-agent", "grok-4.7-low", brief, worktree=wt)
    assert cmd_cursor == [
        "cursor-agent",
        "-p",
        "--output-format",
        "json",
        "--trust",
        "--model",
        "grok-4.7-low",
        "--workspace",
        str(wt.resolve()),
        brief,
    ]

    # 2. codex
    cmd_codex = build_start_cmd("codex", "gpt-5.6-sol", brief, worktree=wt, images=["img1.png"])
    assert cmd_codex == [
        "codex",
        "exec",
        "-m",
        "gpt-5.6-sol",
        "-s",
        "workspace-write",
        "-C",
        str(wt.resolve()),
        "-i",
        "img1.png",
        brief,
    ]

    # 3. kimi
    cmd_kimi = build_start_cmd("kimi", "moonshot-v1-auto", brief, worktree=wt)
    assert cmd_kimi == [
        "kimi",
        "-p",
        brief,
        "-m",
        "moonshot-v1-auto",
        "--add-dir",
        str(wt.resolve()),
    ]

    # 4. agy
    cmd_agy = build_start_cmd("agy", "gemini-3.8-flash-high", brief, worktree=wt)
    assert cmd_agy == [
        "agy",
        "--model",
        "gemini-3.8-flash-high",
        "--print-timeout",
        "40m",
        "--dangerously-skip-permissions",
        "--add-dir",
        str(wt.resolve()),
        f"--print={brief}",
    ]


def test_brief_spaces_quotes_survive(tmp_path):
    brief_content = (
        'He said: "Quotes inside \'single\' and "double" must survive."\n'
        'Variables like $PATH, backticks `code`, and symbols: \n'
        '  - line 1 with spaces   \n'
        '  - line 2 with tabs\t\n'
    )
    brief_file = tmp_path / "complex_brief.md"
    brief_file.write_text(brief_content)

    log_file = tmp_path / "dispatch.jsonl"
    fake_json = json.dumps({"session_id": "test-chat-quotes", "usage": {"inputTokens": 10, "outputTokens": 5}})

    with patch("scriptcast.tools.stage_dispatch.subprocess.run") as mock_run:
        # For git calls and main agent call
        mock_run.return_value = MagicMock(returncode=0, stdout=fake_json, stderr="")
        ret = main([
            "start",
            "--agent", "cursor-agent",
            "--model", "grok-4.7-low",
            "--brief", str(brief_file),
            "--worktree", str(tmp_path),
            "--log", str(log_file),
        ])
        assert ret == 0

        # Verify the command received the exact string without shell mangling
        agent_calls = [c[0][0] for c in mock_run.call_args_list if c[0][0][0] == "cursor-agent"]
        assert len(agent_calls) == 1
        agent_cmd = agent_calls[0]
        full_passed_brief = agent_cmd[-1]
        assert brief_content in full_passed_brief
        assert '"Quotes inside \'single\' and "double"' in full_passed_brief
        assert "$PATH" in full_passed_brief
        assert "`code`" in full_passed_brief


def test_session_id_captured_and_logged(tmp_path):
    log_file = tmp_path / "dispatch-log.jsonl"
    brief_file = tmp_path / "brief.md"
    brief_file.write_text("Do the task.")

    cursor_output = (
        '{"type":"progress","text":"working"}\n'
        '{"type":"result","session_id":"sess-cursor-987","usage":{"inputTokens":120,"outputTokens":30}}\n'
    )

    with patch("scriptcast.tools.stage_dispatch.subprocess.run") as mock_run:
        mock_run.return_value = MagicMock(returncode=0, stdout=cursor_output, stderr="")
        ret = main([
            "start",
            "--agent", "cursor-agent",
            "--model", "grok-4.7-low",
            "--brief", str(brief_file),
            "--worktree", str(tmp_path),
            "--log", str(log_file),
            "--project", "TestProj",
            "--task", "TestTask",
            "--note", "Dispatch note",
        ])
        assert ret == 0

    assert log_file.is_file()
    lines = [json.loads(l) for l in log_file.read_text().splitlines() if l.strip()]
    assert len(lines) == 1
    record = lines[0]

    # Check all schema keys
    expected_schema = [
        "ts", "project", "task", "kind", "agent", "model", "flags",
        "brief_chars", "wall_s", "tokens", "wrote_files", "acceptance",
        "iterations", "verdict", "failure_mode", "note",
    ]
    for key in expected_schema:
        assert key in record, f"Missing key {key} in dispatch log record"

    assert record["session_id"] == "sess-cursor-987"
    assert record["agent"] == "cursor-agent"
    assert record["model"] == "grok-4.7-low"
    assert record["tokens"] == 150
    assert record["verdict"] == "success"
    assert record["project"] == "TestProj"
    assert record["task"] == "TestTask"
    assert record["note"] == "Dispatch note"


def test_session_id_capture_codex_kimi_agy(tmp_path):
    # Codex from index
    fake_codex_index = tmp_path / "session_index.jsonl"
    fake_codex_index.write_text('{"id":"codex-uuid-111","thread_name":"foo"}\n')
    with patch("pathlib.Path.home", return_value=tmp_path):
        (tmp_path / ".codex").mkdir()
        (tmp_path / ".codex" / "session_index.jsonl").write_text('{"id":"codex-uuid-111"}\n')
        sid, _ = capture_session_id("codex")
        assert sid == "codex-uuid-111"

    # Kimi from session list
    with patch("scriptcast.tools.stage_dispatch.subprocess.run") as mock_kimi:
        mock_kimi.return_value = MagicMock(
            returncode=0,
            stdout=json.dumps([{"id": "kimi-session-222"}]),
        )
        sid, _ = capture_session_id("kimi")
        assert sid == "kimi-session-222"

    # Agy from brain directory
    brain_dir = tmp_path / ".gemini" / "antigravity-cli" / "brain"
    brain_dir.mkdir(parents=True)
    (brain_dir / "agy-uuid-333").mkdir()
    with patch("pathlib.Path.home", return_value=tmp_path):
        sid, _ = capture_session_id("agy")
        assert sid == "agy-uuid-333"


def test_resume_uses_right_flag_per_agent(tmp_path):
    msg = "Please fix the failing test."
    msg_file = tmp_path / "fix_message.md"
    msg_file.write_text(msg)

    # 1. cursor-agent --resume
    cmd_cursor = build_resume_cmd("cursor-agent", "sess-1", msg, worktree=tmp_path)
    assert "--resume" in cmd_cursor
    idx = cmd_cursor.index("--resume")
    assert cmd_cursor[idx + 1] == "sess-1"

    # 2. codex exec resume
    cmd_codex = build_resume_cmd("codex", "sess-2", msg, worktree=tmp_path)
    assert cmd_codex[:3] == ["codex", "exec", "resume"]
    assert cmd_codex[3] == "sess-2"

    # 3. kimi -S
    cmd_kimi = build_resume_cmd("kimi", "sess-3", msg, worktree=tmp_path)
    assert "-S" in cmd_kimi
    idx = cmd_kimi.index("-S")
    assert cmd_kimi[idx + 1] == "sess-3"

    # 4. agy --conversation
    cmd_agy = build_resume_cmd("agy", "sess-4", msg, worktree=tmp_path)
    assert "--conversation" in cmd_agy
    idx = cmd_agy.index("--conversation")
    assert cmd_agy[idx + 1] == "sess-4"


REC = {"ts": "2026-11-06T17:00:00Z", "agent": "codex", "model": "gpt-5.6-sol", "task": "Fix tray gap", "verdict": "success"}


def test_brief_header_asks_for_session_log(tmp_path):
    assert WORKER_LOG in make_brief_header(tmp_path)


def test_collect_moves_worker_log_and_indexes(tmp_path):
    (tmp_path / WORKER_LOG).write_text("## What Was Requested\nfix\n")
    out = tmp_path / "logs"
    dest = collect_session_log(tmp_path, REC, "", log_dir=out)
    assert not (tmp_path / WORKER_LOG).exists()
    text = dest.read_text()
    assert text.startswith("---\n") and 'model: "gpt-5.6-sol"' in text and "What Was Requested" in text
    idx = [json.loads(l) for l in (out / "sessions.jsonl").read_text().splitlines()]
    assert idx[0]["worker_wrote_log"] is True and idx[0]["session_log"] == str(dest)


def test_collect_writes_stub_when_worker_skipped_log(tmp_path):
    out = tmp_path / "logs"
    dest = collect_session_log(tmp_path, REC, "line1\nfinal answer", log_dir=out)
    assert "final answer" in dest.read_text()
    assert json.loads((out / "sessions.jsonl").read_text())["worker_wrote_log"] is False
