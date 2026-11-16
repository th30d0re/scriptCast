"""R2 measure parsing tests: JSONL loading, start/resume classification,
registry turn deltas, restart and session-collision detection, aggregation."""
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MEASURE_PY = ROOT / "experiments" / "r2_measure" / "measure.py"

spec = importlib.util.spec_from_file_location("r2_measure", MEASURE_PY)
measure = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = measure
spec.loader.exec_module(measure)


def _write_jsonl(path, records):
    path.write_text("\n".join(json.dumps(r) for r in records) + "\n")
    return path


def _dispatch(**over):
    rec = {
        "ts": "2026-11-06T20:00:00Z",
        "session_id": "s1",
        "project": "p",
        "task": "t",
        "kind": "infra",
        "agent": "codex",
        "model": "gpt-5.6-terra",
        "flags": "exec -m gpt-5.6-terra -s workspace-write -C /wt",
        "brief_chars": 1000,
        "wall_s": 100.0,
        "tokens": 0,
        "wrote_files": True,
        "acceptance": "not_run",
        "iterations": 1,
        "verdict": "success",
        "failure_mode": None,
        "note": "",
    }
    rec.update(over)
    return rec


# --- read_jsonl / split ----------------------------------------------------


def test_read_jsonl_skips_blank_and_malformed(tmp_path):
    path = tmp_path / "log.jsonl"
    path.write_text(
        '{"a": 1}\n\n   \nnot json\n[1, 2]\n{"b": 2}\n'
    )
    records, bad = measure.read_jsonl(path)
    assert records == [{"a": 1}, {"b": 2}]
    assert bad == 2  # the non-JSON line and the non-dict line


def test_split_sessions_jsonl_separates_registry_events():
    records = [
        {"ts": "t0", "task": "x"},
        {"ts": "t1", "event": "registry_start", "key": "k"},
        {"ts": "t2", "event": "registry_resume", "key": "k"},
    ]
    dispatches, events = measure.split_sessions_jsonl(records)
    assert dispatches == [{"ts": "t0", "task": "x"}]
    assert [e["event"] for e in events] == ["registry_start", "registry_resume"]


# --- classification --------------------------------------------------------


def test_classify_codex_start_and_resume():
    start = _dispatch(agent="codex", flags="exec -m gpt-5.6-terra -s workspace-write -C /wt")
    resume = _dispatch(agent="codex", flags="exec resume 01a1b3c0-69a2-7611-b854-d26ec5fbb3b4")
    assert measure.classify_action(start) == "start"
    assert measure.classify_action(resume) == "resume"


def test_classify_kimi_agy_cursor():
    kimi_start = _dispatch(agent="kimi", flags="-p fix the scene -m kimi-code/k3 --add-dir /wt")
    kimi_resume = _dispatch(agent="kimi", flags="-S session_abc-123 -p")
    agy_start = _dispatch(agent="agy", flags="--model gemini-3.8-flash-low --print-timeout 40m --dangerously-skip-permissions --add-dir /wt")
    agy_resume = _dispatch(agent="agy", flags="--conversation d57b441e --print-timeout 40m --dangerously-skip-permissions")
    cursor_resume = _dispatch(agent="cursor-agent", flags="-p --output-format json --trust --resume 0987adf2 --workspace /wt")
    assert measure.classify_action(kimi_start) == "start"
    assert measure.classify_action(kimi_resume) == "resume"
    assert measure.classify_action(agy_start) == "start"
    assert measure.classify_action(agy_resume) == "resume"
    assert measure.classify_action(cursor_resume) == "resume"


def test_task_text_saying_resume_does_not_make_a_resume():
    # A fresh dispatch whose task description contains "resume" is still a start:
    # classification reads flags only.
    rec = _dispatch(
        task="Phase 1 resume — fix split collision",
        note="resume the halted pipeline",
        flags="exec --skip-git-repo-check -s workspace-write -c developer_instructions=''",
    )
    assert measure.classify_action(rec) == "start"


def test_freeform_flags_note_via_resume():
    # Vault L4-style record: the resume is documented in the flags text.
    rec = _dispatch(agent="cursor", flags="-p --trust --force --output-format json; fix round via --resume")
    assert measure.classify_action(rec) == "resume"


# --- registry turn deltas --------------------------------------------------


def test_registry_turn_deltas_subtract_cumulative():
    events = [
        {"event": "registry_start", "key": "k/boxes/builder", "ts": "t0", "turns": 1,
         "brief_chars": 2996, "tokens": 749, "tokens_kind": "proxy", "status": "live"},
        {"event": "registry_resume", "key": "k/boxes/builder", "ts": "t1", "turns": 2,
         "brief_chars": 3858, "tokens": 964, "tokens_kind": "proxy", "status": "live"},
        {"event": "registry_resume", "key": "k/boxes/builder", "ts": "t2", "turns": 3,
         "brief_chars": 5127, "tokens": 1281, "tokens_kind": "proxy", "status": "live"},
    ]
    rows = measure.registry_turn_deltas(events)
    assert [r["brief_chars_delta"] for r in rows] == [2996, 862, 1269]
    assert [r["tokens_delta"] for r in rows] == [749, 215, 317]
    assert [r["action"] for r in rows] == ["start", "resume", "resume"]


def test_registry_turn_deltas_reset_on_restart():
    # A second registry_start for the same key (supersede) resets the baseline;
    # its delta is the full value, not zero.
    events = [
        {"event": "registry_start", "key": "k/t/builder", "ts": "t0", "turns": 1,
         "brief_chars": 1674, "tokens": 418},
        {"event": "registry_start", "key": "k/t/builder", "ts": "t1", "turns": 1,
         "brief_chars": 1674, "tokens": 418},
        {"event": "registry_resume", "key": "k/t/builder", "ts": "t2", "turns": 2,
         "brief_chars": 2000, "tokens": 500},
    ]
    rows = measure.registry_turn_deltas(events)
    assert [r["brief_chars_delta"] for r in rows] == [1674, 1674, 326]


def test_registry_turn_deltas_skip_close_events():
    events = [
        {"event": "registry_start", "key": "k", "ts": "t0", "turns": 1, "brief_chars": 100, "tokens": 25},
        {"event": "registry_close", "key": "k", "ts": "t1", "brief_chars": 100, "tokens": 25},
    ]
    rows = measure.registry_turn_deltas(events)
    assert len(rows) == 1 and rows[0]["action"] == "start"


# --- restarts and collisions -----------------------------------------------


def test_detect_restarts_flags_repeated_starts_for_one_task():
    ds = [
        _dispatch(ts="t0", task="fix", wall_s=500.0, verdict="success", wrote_files=False),
        _dispatch(ts="t1", task="fix", wall_s=100.0, verdict="success", wrote_files=True),
        _dispatch(ts="t2", task="other", wall_s=50.0),
    ]
    for d in ds:
        d["action"] = measure.classify_action(d)
    restarts = measure.detect_restarts(ds)
    assert len(restarts) == 1
    r = restarts[0]
    assert r["task"] == "fix"
    assert r["starts"] == 2 and r["restart_count"] == 1
    assert r["wasted_wall_s"] == 500.0  # only the superseded start counts


def test_detect_restarts_ignores_single_dispatch_and_resume_continuation():
    ds = [
        _dispatch(ts="t0", task="a", flags="exec -m x"),
        _dispatch(ts="t1", task="a", agent="codex", flags="exec resume abc"),
    ]
    for d in ds:
        d["action"] = measure.classify_action(d)
    assert measure.detect_restarts(ds) == []


def test_session_id_collisions():
    ds = [
        _dispatch(session_id="shared", task="a"),
        _dispatch(session_id="shared", task="b", agent="codex"),
        _dispatch(session_id="unique", task="c"),
        _dispatch(session_id=None, task="d"),
    ]
    collisions = measure.session_id_collisions(ds)
    assert set(collisions) == {"shared"}
    assert set(collisions["shared"]) == {"codex:a", "codex:b"}


# --- registry loading and summary ------------------------------------------


def test_load_registry_flattens_live_and_archive(tmp_path):
    reg = {
        "sessions": {
            "ep/s1/builder": {"agent": "codex", "model": "m", "session_id": "x",
                              "turns": 6, "tokens": 1917, "tokens_kind": "proxy",
                              "brief_chars": 7669, "context_limit": 400000,
                              "status": "live", "started": "t0", "last_active": "t6"},
        },
        "archive": [
            {"key": "ep/s0/builder", "agent": "kimi", "model": "k3", "turns": 1,
             "tokens": 591, "tokens_kind": "proxy", "brief_chars": 2367,
             "status": "superseded", "started": "t0", "last_active": "t0"},
        ],
    }
    path = tmp_path / "sessions.json"
    path.write_text(json.dumps(reg))
    rows = measure.load_registry(path)
    assert len(rows) == 2
    live = next(r for r in rows if r["where"] == "live")
    assert live["follow_up_resumes"] == 5
    assert live["tokens_kind"] == "proxy"
    archived = next(r for r in rows if r["where"] == "archive")
    assert archived["follow_up_resumes"] == 0
    assert archived["status"] == "superseded"


def test_summarize_groups_by_agent_kind_action():
    ds = [
        _dispatch(agent="codex", kind="infra", wall_s=100.0, tokens=0, verdict="success"),
        _dispatch(agent="codex", kind="infra", wall_s=300.0, tokens=5000, verdict="failure"),
        _dispatch(agent="agy", kind="experiment", wall_s=200.0, tokens=0, verdict="success"),
    ]
    for d in ds:
        d["action"] = "start"
    rows = measure.summarize(ds)
    assert len(rows) == 2
    codex = next(r for r in rows if r["agent"] == "codex")
    assert codex["n"] == 2
    assert codex["wall_s"]["median"] == 200.0
    assert codex["tokens_reported_n"] == 1  # tokens=0 counts as unreported
    assert codex["verdicts"] == {"success": 1, "failure": 1}
    assert codex["wrote_files"] == 2


def test_build_report_counts(tmp_path):
    sessions = _write_jsonl(tmp_path / "sessions.jsonl", [
        _dispatch(ts="t0", task="a"),
        {"ts": "t0", "event": "registry_start", "key": "ep/a/builder", "turns": 1,
         "brief_chars": 1000, "tokens": 250},
        {"ts": "t1", "event": "registry_resume", "key": "ep/a/builder", "turns": 2,
         "brief_chars": 1400, "tokens": 350},
    ])
    registry = tmp_path / "sessions.json"
    registry.write_text(json.dumps({"sessions": {}, "archive": []}))

    records, bad = measure.read_jsonl(sessions)
    dispatches, events = measure.split_sessions_jsonl(records)
    report = measure.build_report(
        measure.tag_source(dispatches, sessions.name),
        measure.tag_source(events, sessions.name),
        measure.load_registry(registry),
        {str(sessions): bad},
    )
    assert report["actions"]["starts"] == 1
    assert report["actions"]["resumes_from_dispatch_flags"] == 0
    assert report["actions"]["resume_events_without_dispatch_record"] == 1
    assert len(report["turn_deltas"]) == 2
    assert report["turn_deltas"][1]["brief_chars_delta"] == 400
