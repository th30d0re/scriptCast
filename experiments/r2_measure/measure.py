#!/usr/bin/env python3
"""R2 step 17 measurement: fresh starts vs resumed sessions in dispatch data.

Reads every dispatch log and session registry the Stage dispatcher can have
written (see scriptcast/tools/stage_dispatch.py), classifies each dispatch as
a fresh start or a resume, and compares tokens, wall time, exit status /
acceptance, and the number of follow-up resumes, by agent and kind.

Sources:
  - sessions.jsonl under SESSION_LOG_DIR (env SCRIPTCAST_SESSION_LOGS):
    dispatch records and registry_* events share one file.
  - the older vault dispatch-log.jsonl (pre-registry schema, no session ids).
  - session registries at <episode>/stage/.router/sessions.json.

Classification is flags-based: a dispatch is a resume when its recorded
command flags carry the agent's resume switch (codex `exec resume`, kimi
`-S`, cursor-agent `--resume`, agy `--conversation`); otherwise it is a
start. Task/brief text is never inspected, so a task named "resume ..." in a
fresh dispatch still counts as a start.
"""
from __future__ import annotations

import argparse
import glob
import json
import os
import re
import statistics
import sys
from collections import Counter
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple

HOME = Path.home()
DEFAULT_SESSION_LOG_DIR = (
    HOME
    / "Library/Mobile Documents/iCloud~md~obsidian/Documents/Root/AI Session Logs/workers"
)
DEFAULT_VAULT_DISPATCH_LOG = (
    HOME
    / "Library/Mobile Documents/iCloud~md~obsidian/Documents/Root/Orchestration/dispatch-log.jsonl"
)
DEFAULT_REGISTRY_GLOBS = (
    "/Users/emmanuel/Dev/Tools/*/stage/.router/sessions.json",
    "/Users/emmanuel/Documents/Theory/*/Architecting_the_operation/video/*/stage/.router/sessions.json",
)

# Resume switches per agent, matched against the logged `flags` field only.
RESUME_PATTERNS = {
    "codex": re.compile(r"\bexec\s+resume\b"),
    "kimi": re.compile(r"(?:^|\s)-S\s+\S"),
    "cursor-agent": re.compile(r"--resume\b"),
    "cursor": re.compile(r"--resume\b"),
    "agy": re.compile(r"--conversation\b"),
}
GENERIC_RESUME = re.compile(
    r"\bexec\s+resume\b|--resume\b|--conversation\b|(?:^|\s)-S\s+\S"
)


# ---------------------------------------------------------------------------
# Loading


def read_jsonl(path: Path) -> Tuple[List[Dict[str, Any]], int]:
    """Parse a JSONL file tolerantly. Returns (records, bad_line_count)."""
    records: List[Dict[str, Any]] = []
    bad = 0
    for line in Path(path).read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            bad += 1
            continue
        if isinstance(obj, dict):
            records.append(obj)
        else:
            bad += 1
    return records, bad


def is_registry_event(rec: Dict[str, Any]) -> bool:
    return str(rec.get("event", "")).startswith("registry_")


def split_sessions_jsonl(
    records: Iterable[Dict[str, Any]],
) -> Tuple[List[Dict[str, Any]], List[Dict[str, Any]]]:
    """sessions.jsonl mixes dispatch records and registry_* events."""
    dispatches, events = [], []
    for rec in records:
        (events if is_registry_event(rec) else dispatches).append(rec)
    return dispatches, events


def load_registry(path: Path) -> List[Dict[str, Any]]:
    """Flatten a sessions.json registry into per-session rows (live + archive)."""
    data = json.loads(Path(path).read_text())
    rows = []
    for key, e in (data.get("sessions") or {}).items():
        rows.append(_registry_row(key, e, "live"))
    for e in data.get("archive") or []:
        rows.append(_registry_row(e.get("key"), e, "archive"))
    return rows


def _registry_row(key: Optional[str], e: Dict[str, Any], where: str) -> Dict[str, Any]:
    turns = int(e.get("turns") or 1)
    return {
        "registry": None,  # filled by caller when known
        "key": key,
        "where": where,
        "agent": e.get("agent"),
        "model": e.get("model"),
        "session_id": e.get("session_id"),
        "turns": turns,
        "follow_up_resumes": max(0, turns - 1),
        "tokens": e.get("tokens") or 0,
        "tokens_kind": e.get("tokens_kind"),
        "brief_chars": e.get("brief_chars") or 0,
        "context_limit": e.get("context_limit"),
        "status": e.get("status"),
        "started": e.get("started"),
        "last_active": e.get("last_active"),
    }


# ---------------------------------------------------------------------------
# Classification


def classify_action(rec: Dict[str, Any]) -> str:
    """'resume' when the logged flags carry the agent's resume switch, else 'start'."""
    agent = str(rec.get("agent") or "")
    flags = str(rec.get("flags") or "")
    pattern = RESUME_PATTERNS.get(agent, GENERIC_RESUME)
    return "resume" if pattern.search(flags) else "start"


def tag_source(recs: Iterable[Dict[str, Any]], source: str) -> List[Dict[str, Any]]:
    out = []
    for r in recs:
        rec = dict(r)
        rec["_source"] = source
        out.append(rec)
    return out


# ---------------------------------------------------------------------------
# Analysis


def _num(v: Any) -> Optional[float]:
    return float(v) if isinstance(v, (int, float)) and not isinstance(v, bool) else None


def _stats(values: List[float]) -> Optional[Dict[str, float]]:
    if not values:
        return None
    return {
        "n": len(values),
        "mean": round(statistics.fmean(values), 2),
        "median": round(statistics.median(values), 2),
        "min": round(min(values), 2),
        "max": round(max(values), 2),
    }


def summarize(dispatches: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Per (source, agent, kind, action) aggregates."""
    groups: Dict[Tuple[str, str, str, str], List[Dict[str, Any]]] = {}
    for d in dispatches:
        key = (
            str(d.get("_source") or "?"),
            str(d.get("agent") or "?"),
            str(d.get("kind") or "?"),
            str(d.get("action") or "?"),
        )
        groups.setdefault(key, []).append(d)
    rows = []
    for (source, agent, kind, action), ds in sorted(groups.items()):
        walls = [w for w in (_num(d.get("wall_s")) for d in ds) if w is not None]
        toks = [t for t in (_num(d.get("tokens")) for d in ds) if t]
        briefs = [b for b in (_num(d.get("brief_chars")) for d in ds) if b is not None]
        rows.append(
            {
                "source": source,
                "agent": agent,
                "kind": kind,
                "action": action,
                "n": len(ds),
                "wall_s": _stats(walls),
                "tokens_reported_n": len(toks),
                "tokens": _stats(toks),
                "brief_chars": _stats(briefs),
                "verdicts": dict(Counter(str(d.get("verdict") or "?") for d in ds)),
                "acceptance": dict(Counter(str(d.get("acceptance") or "?") for d in ds)),
                "wrote_files": sum(1 for d in ds if d.get("wrote_files")),
                "iterations_total": sum(int(d.get("iterations") or 1) for d in ds),
            }
        )
    return rows


def registry_turn_deltas(events: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Per-turn injected text/tokens from cumulative registry events.

    Registry entries carry cumulative brief_chars/tokens; the delta between
    consecutive events for one key is what that turn added. A registry_start
    for an already-seen key (supersede) resets the baseline.
    """
    by_key: Dict[str, List[Dict[str, Any]]] = {}
    for e in events:
        key = e.get("key")
        if key:
            by_key.setdefault(str(key), []).append(e)
    rows = []
    for key, evs in sorted(by_key.items()):
        evs.sort(key=lambda e: str(e.get("ts") or ""))
        prev_chars = prev_tokens = 0
        for e in evs:
            event = str(e.get("event"))
            if event not in ("registry_start", "registry_resume"):
                continue
            chars = int(e.get("brief_chars") or 0)
            tokens = int(e.get("tokens") or 0)
            if event == "registry_start":
                d_chars, d_tokens = chars, tokens
            else:
                d_chars, d_tokens = chars - prev_chars, tokens - prev_tokens
            prev_chars, prev_tokens = chars, tokens
            rows.append(
                {
                    "key": key,
                    "ts": e.get("ts"),
                    "action": "start" if event == "registry_start" else "resume",
                    "agent": e.get("agent"),
                    "model": e.get("model"),
                    "turn": e.get("turns"),
                    "brief_chars_delta": d_chars,
                    "tokens_delta": d_tokens,
                    "tokens_kind": e.get("tokens_kind"),
                    "status": e.get("status"),
                }
            )
    return rows


def detect_restarts(dispatches: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Fresh starts that follow an earlier dispatch for the same (project, task).

    Each is a follow-up handled as a cold start — the behaviour the session
    registry exists to remove.
    """
    groups: Dict[Tuple[str, str], List[Dict[str, Any]]] = {}
    for d in dispatches:
        groups.setdefault((str(d.get("project") or "?"), str(d.get("task") or "?")), []).append(d)
    out = []
    for (project, task), ds in sorted(groups.items()):
        ds.sort(key=lambda d: str(d.get("ts") or ""))
        starts = [d for d in ds if d.get("action") == "start"]
        resumes = [d for d in ds if d.get("action") == "resume"]
        if len(starts) > 1:
            out.append(
                {
                    "project": project,
                    "task": task,
                    "starts": len(starts),
                    "resumes": len(resumes),
                    "restart_count": len(starts) - 1,
                    "wasted_wall_s": round(
                        sum(
                            w
                            for w in (_num(d.get("wall_s")) for d in starts[:-1])
                            if w is not None
                        ),
                        2,
                    ),
                    "records": [
                        {
                            "ts": d.get("ts"),
                            "agent": d.get("agent"),
                            "model": d.get("model"),
                            "wall_s": d.get("wall_s"),
                            "verdict": d.get("verdict"),
                            "wrote_files": d.get("wrote_files"),
                        }
                        for d in ds
                    ],
                }
            )
    return out


def session_id_collisions(dispatches: List[Dict[str, Any]]) -> Dict[str, List[str]]:
    """Session ids shared by more than one (agent, task) pair — join hazard."""
    by_id: Dict[str, List[Dict[str, Any]]] = {}
    for d in dispatches:
        sid = d.get("session_id")
        if sid:
            by_id.setdefault(str(sid), []).append(d)
    return {
        sid: sorted({f"{d.get('agent')}:{d.get('task')}" for d in ds})
        for sid, ds in by_id.items()
        if len({(d.get("agent"), d.get("task")) for d in ds}) > 1
    }


def build_report(
    dispatches: List[Dict[str, Any]],
    registry_events: List[Dict[str, Any]],
    registries: List[Dict[str, Any]],
    bad_lines: Dict[str, int],
) -> Dict[str, Any]:
    for d in dispatches:
        d["action"] = classify_action(d)
    starts = [d for d in dispatches if d["action"] == "start"]
    resumes = [d for d in dispatches if d["action"] == "resume"]
    deltas = registry_turn_deltas(registry_events)
    resume_events = [r for r in deltas if r["action"] == "resume"]
    return {
        "inputs": {
            "dispatch_records": len(dispatches),
            "registry_events": len(registry_events),
            "registry_rows": len(registries),
            "bad_lines": bad_lines,
        },
        "actions": {
            "starts": len(starts),
            "resumes_from_dispatch_flags": len(resumes),
            "resume_events_without_dispatch_record": len(resume_events),
        },
        "by_agent_kind": summarize(dispatches),
        "turn_deltas": deltas,
        "registries": registries,
        "restarts": detect_restarts(dispatches),
        "session_id_collisions": session_id_collisions(dispatches),
    }


# ---------------------------------------------------------------------------
# Rendering


def _fmt_stats(s: Optional[Dict[str, float]], unit: str = "") -> str:
    if not s:
        return "—"
    if s["n"] == 1:
        return f"{s['median']:g}{unit} (n=1)"
    return f"med {s['median']:g}{unit}, mean {s['mean']:g}{unit}, [{s['min']:g}–{s['max']:g}] (n={s['n']:.0f})"


def render_text(report: Dict[str, Any]) -> str:
    lines: List[str] = []
    inp = report["inputs"]
    act = report["actions"]
    lines.append("R2 step 17 — fresh start vs resume measurement")
    lines.append("=" * 48)
    lines.append(
        "Inputs: {dispatch_records} dispatch records, {registry_events} registry events, "
        "{registry_rows} registry rows".format(**inp)
    )
    if any(inp["bad_lines"].values()):
        lines.append(f"Skipped malformed lines: {inp['bad_lines']}")
    lines.append(
        "Actions: {starts} fresh starts, {resumes_from_dispatch_flags} resumes in dispatch flags, "
        "{resume_events_without_dispatch_record} registry resume events without a dispatch record".format(
            **act
        )
    )
    lines.append("")
    lines.append("By source / agent / kind / action")
    lines.append("-" * 48)
    for r in report["by_agent_kind"]:
        lines.append(
            f"[{r['source']}] {r['agent']} / {r['kind']} / {r['action']}: n={r['n']}"
        )
        lines.append(f"  wall_s:      {_fmt_stats(r['wall_s'], 's')}")
        lines.append(
            f"  tokens:      {_fmt_stats(r['tokens'])} "
            f"(reported on {r['tokens_reported_n']}/{r['n']})"
        )
        lines.append(f"  brief_chars: {_fmt_stats(r['brief_chars'])}")
        lines.append(
            f"  verdicts: {r['verdicts']}; acceptance: {r['acceptance']}; "
            f"wrote_files {r['wrote_files']}/{r['n']}; iterations {r['iterations_total']}"
        )
    if report["turn_deltas"]:
        lines.append("")
        lines.append("Per-turn injected text (registry events; cumulative deltas)")
        lines.append("-" * 48)
        for r in report["turn_deltas"]:
            lines.append(
                f"  {r['key']} turn {r['turn']} ({r['action']}): "
                f"+{r['brief_chars_delta']} chars, +{r['tokens_delta']} tokens "
                f"({r['tokens_kind']}), status={r['status']}"
            )
    if report["registries"]:
        lines.append("")
        lines.append("Session registry rows")
        lines.append("-" * 48)
        for r in report["registries"]:
            lines.append(
                f"  [{r['where']}] {r['key']}: {r['agent']}/{r['model']}, turns={r['turns']} "
                f"(follow-up resumes {r['follow_up_resumes']}), tokens={r['tokens']} "
                f"({r['tokens_kind']}), status={r['status']}"
            )
    if report["restarts"]:
        lines.append("")
        lines.append("Follow-ups dispatched as fresh starts (restarts)")
        lines.append("-" * 48)
        for r in report["restarts"]:
            lines.append(
                f"  {r['project']}/{r['task']}: {r['starts']} starts, {r['resumes']} resumes, "
                f"wasted wall {r['wasted_wall_s']:g}s on superseded starts"
            )
            for rec in r["records"]:
                lines.append(
                    f"    {rec['ts']} {rec['agent']}/{rec['model']} wall={rec['wall_s']}s "
                    f"verdict={rec['verdict']} wrote_files={rec['wrote_files']}"
                )
    if report["session_id_collisions"]:
        lines.append("")
        lines.append("Session-id collisions (same id across tasks — joins unreliable)")
        lines.append("-" * 48)
        for sid, pairs in report["session_id_collisions"].items():
            lines.append(f"  {sid}: {', '.join(pairs)}")
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# CLI


def default_sessions_jsonl() -> Optional[Path]:
    log_dir = Path(os.environ.get("SCRIPTCAST_SESSION_LOGS", DEFAULT_SESSION_LOG_DIR))
    path = log_dir / "sessions.jsonl"
    return path if path.is_file() else None


def default_registries() -> List[Path]:
    paths: List[Path] = []
    for pattern in DEFAULT_REGISTRY_GLOBS:
        for match in sorted(glob.glob(pattern)):
            p = Path(match)
            if p not in paths:
                paths.append(p)
    return paths


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--sessions-jsonl",
        type=Path,
        action="append",
        default=None,
        help="sessions.jsonl path (repeatable); default: $SCRIPTCAST_SESSION_LOGS or the iCloud workers dir",
    )
    parser.add_argument(
        "--dispatch-log",
        type=Path,
        action="append",
        default=None,
        help="extra dispatch-log JSONL (repeatable); default: vault Orchestration/dispatch-log.jsonl when present",
    )
    parser.add_argument(
        "--registry",
        type=Path,
        action="append",
        default=None,
        help="sessions.json registry path (repeatable); default: auto-discovered under Dev/Tools and the book repo",
    )
    parser.add_argument("--json", action="store_true", help="emit the full report as JSON")
    args = parser.parse_args(argv)

    sessions_paths = args.sessions_jsonl
    if sessions_paths is None:
        default = default_sessions_jsonl()
        sessions_paths = [default] if default else []
    dispatch_paths = args.dispatch_log
    if dispatch_paths is None:
        dispatch_paths = [DEFAULT_VAULT_DISPATCH_LOG] if DEFAULT_VAULT_DISPATCH_LOG.is_file() else []
    registry_paths = args.registry if args.registry is not None else default_registries()

    dispatches: List[Dict[str, Any]] = []
    events: List[Dict[str, Any]] = []
    registries: List[Dict[str, Any]] = []
    bad_lines: Dict[str, int] = {}

    for path in sessions_paths:
        records, bad = read_jsonl(path)
        bad_lines[str(path)] = bad
        d, e = split_sessions_jsonl(records)
        dispatches.extend(tag_source(d, Path(path).name))
        events.extend(tag_source(e, Path(path).name))
    for path in dispatch_paths:
        records, bad = read_jsonl(path)
        bad_lines[str(path)] = bad
        d, e = split_sessions_jsonl(records)  # tolerate registry events here too
        dispatches.extend(tag_source(d, Path(path).name))
        events.extend(tag_source(e, Path(path).name))
    for path in registry_paths:
        for row in load_registry(path):
            row["registry"] = str(path)
            registries.append(row)

    report = build_report(dispatches, events, registries, bad_lines)
    if args.json:
        print(json.dumps(report, indent=1))
    else:
        print(render_text(report))
    return 0


if __name__ == "__main__":
    sys.exit(main())
