"""Wrapper for worker CLIs: codex, cursor-agent, kimi, agy (+ session registry, nlm context)."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
from typing import Any, Dict, List, Optional, Tuple
import fcntl
import tomllib

from scriptcast.tools import stage_context

AGENTS = ("codex", "cursor-agent", "kimi", "agy")

# Worker session logs. Workers write WORKER_LOG in their worktree (always
# writable from their sandbox); the dispatcher, running on the host, moves it
# to SESSION_LOG_DIR and indexes it in sessions.jsonl for later evaluation.
WORKER_LOG = ".session-log.md"
DEFAULT_SESSION_LOG_DIR = (
    Path.home()
    / "Library/Mobile Documents/iCloud~md~obsidian/Documents/Root/AI Session Logs/workers"
)
LOG_INSTRUCTIONS = (
    f"Session log (required): before you finish, write `{WORKER_LOG}` in the worktree root "
    "with four sections: 1. What Was Requested, 2. What I Did (steps, files changed), "
    "3. Challenges Encountered (obstacles, failed approaches), 4. Next Ideas. "
    "Do not commit it; the dispatcher collects it.\n"
)


def session_log_dir() -> Path:
    return Path(os.environ.get("SCRIPTCAST_SESSION_LOGS", DEFAULT_SESSION_LOG_DIR))


def collect_session_log(worktree: Path, record: Dict[str, Any], stdout: str = "",
                        log_dir: Optional[Path] = None) -> Path:
    """Move the worker's session log to the shared dir and index it.

    Writes a stub from the tail of stdout when the worker wrote no log, so
    every dispatch leaves a record. Returns the stored path.
    """
    log_dir = Path(log_dir) if log_dir else session_log_dir()
    log_dir.mkdir(parents=True, exist_ok=True)
    src = worktree / WORKER_LOG
    slug = "".join(c if c.isalnum() else "-" for c in str(record.get("task", "task")))[:48].strip("-")
    stamp = record["ts"].replace(":", "").replace("-", "")
    dest = log_dir / f"{stamp}-{record['agent']}-{record['model']}-{slug}.md".replace("/", "_")
    front = "---\n" + "".join(f"{k}: {json.dumps(v)}\n" for k, v in record.items()) + "---\n\n"
    if src.is_file():
        body, written = src.read_text(), True
        src.unlink()
    else:
        tail = "\n".join(stdout.strip().splitlines()[-40:])
        body, written = f"(worker wrote no session log; last output lines)\n\n```\n{tail}\n```\n", False
    dest.write_text(front + body)
    with open(log_dir / "sessions.jsonl", "a", encoding="utf-8") as f:
        f.write(json.dumps({**record, "session_log": str(dest), "worker_wrote_log": written}) + "\n")
    return dest


# ---------------------------------------------------------------------------
# Session registry (Laya Router R2, plan steps 13 and 15).
# <episode>/stage/.router/sessions.json, keyed "<episode>/<scene>/<role>".
# Every transition also lands in the vault sessions.jsonl (event=registry_*).

TIERS_TOML = Path(__file__).resolve().parents[1] / "router" / "tiers.toml"
# Fallback context sizes (tokens) by model-name prefix; tiers.toml wins.
DEFAULT_CONTEXT = {
    "gpt-5": 400_000,
    "gemini": 1_000_000,
    "kimi": 256_000,
    "claude": 200_000,
    "grok": 256_000,
    "composer": 200_000,
}
DEFAULT_CONTEXT_FALLBACK = 128_000
COMPACT_AT = 0.60
CLOSE_AT = 0.80
CHARS_PER_TOKEN = 4
COMPACT_MESSAGE = (
    "\n\nCompaction (context {pct:.0%} used): before acting, summarise in five bullets "
    "the agreed props, element ids and open defects; keep later replies short.\n"
)
CLOSE_MESSAGE = (
    "\n\nContext {pct:.0%} used: after this turn, write your final state summary "
    "(agreed props, element ids, open defects, files touched) to `.session-summary.md`. "
    "The dispatcher will close this session and seed a fresh one with it.\n"
)


def _utcnow() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def load_context_sizes(path: Optional[Path] = None) -> Dict[str, int]:
    """Per-model context sizes. Reads tiers.toml (`[context]` name = int, or
    `[models.<name>] context = int`) when present, over the default table."""
    sizes = dict(DEFAULT_CONTEXT)
    path = Path(path) if path else TIERS_TOML
    if path.is_file():
        data = tomllib.loads(path.read_text())
        for name, val in (data.get("context") or {}).items():
            if isinstance(val, int):
                sizes[name] = val
        for name, m in (data.get("models") or {}).items():
            if isinstance(m, dict):
                val = m.get("context") or m.get("context_limit")
                if isinstance(val, int):
                    sizes[name] = val
    return sizes


def context_limit(model: str, sizes: Optional[Dict[str, int]] = None) -> int:
    sizes = load_context_sizes() if sizes is None else sizes
    if model in sizes:
        return sizes[model]
    best = max((k for k in sizes if model.startswith(k)), key=len, default=None)
    return sizes[best] if best else DEFAULT_CONTEXT_FALLBACK


def registry_path(episode: Path) -> Path:
    return Path(episode) / "stage" / ".router" / "sessions.json"


def registry_key(episode: Path, scene: str, role: str) -> str:
    return f"{Path(episode).resolve().name}/{scene}/{role}"


class Registry:
    """sessions.json with an flock held for the read-modify-write."""

    def __init__(self, path: Path):
        self.path = Path(path)

    def __enter__(self) -> Dict[str, Any]:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self._lock = open(self.path.with_suffix(".lock"), "w")
        fcntl.flock(self._lock, fcntl.LOCK_EX)
        self.data = json.loads(self.path.read_text()) if self.path.is_file() else {}
        self.data.setdefault("sessions", {})
        self.data.setdefault("archive", [])
        return self.data

    def __exit__(self, exc_type, *_):
        if exc_type is None:
            tmp = self.path.with_suffix(".tmp")
            tmp.write_text(json.dumps(self.data, indent=1))
            tmp.replace(self.path)
        fcntl.flock(self._lock, fcntl.LOCK_UN)
        self._lock.close()

    def get(self, key: str) -> Optional[Dict[str, Any]]:
        if not self.path.is_file():
            return None
        return json.loads(self.path.read_text()).get("sessions", {}).get(key)


def log_registry_event(event: str, key: str, entry: Dict[str, Any],
                       log_dir: Optional[Path] = None, **extra: Any) -> None:
    log_dir = Path(log_dir) if log_dir else session_log_dir()
    log_dir.mkdir(parents=True, exist_ok=True)
    rec = {"ts": _utcnow(), "event": f"registry_{event}", "key": key, **entry, **extra}
    with open(log_dir / "sessions.jsonl", "a", encoding="utf-8") as f:
        f.write(json.dumps(rec) + "\n")


def load_fraction(entry: Dict[str, Any]) -> float:
    return entry.get("tokens", 0) / max(1, entry.get("context_limit") or DEFAULT_CONTEXT_FALLBACK)


def compaction_message(entry: Dict[str, Any]) -> str:
    """Message to append to the next resume: '' under 60%, compact 60-80%, close over 80%."""
    pct = load_fraction(entry)
    if pct >= CLOSE_AT:
        return CLOSE_MESSAGE.format(pct=pct)
    if pct >= COMPACT_AT:
        return COMPACT_MESSAGE.format(pct=pct)
    return ""


def _add_usage(entry: Dict[str, Any], tokens: int, chars: int) -> None:
    entry["brief_chars"] = entry.get("brief_chars", 0) + chars
    if tokens and entry.get("tokens_kind") != "proxy":
        entry["tokens"] = entry.get("tokens", 0) + tokens
        entry["tokens_kind"] = "reported"
    else:
        # No reported usage: brief-character sum is the load proxy.
        entry["tokens_kind"] = "proxy"
        entry["tokens"] = entry["brief_chars"] // CHARS_PER_TOKEN


def registry_start(path: Path, key: str, agent: str, model: str, session_id: Optional[str],
                   tokens: int, chars: int, sizes: Optional[Dict[str, int]] = None,
                   log_dir: Optional[Path] = None, ok: bool = True) -> Dict[str, Any]:
    with Registry(path) as data:
        old = data["sessions"].get(key)
        if old:
            data["archive"].append({"key": key, **old, "status": old.get("status") if old.get(
                "status") in ("closed", "dead") else "superseded", "archived": _utcnow()})
        entry = {"agent": agent, "model": model, "session_id": session_id, "turns": 1,
                 "tokens": 0, "tokens_kind": "reported", "brief_chars": 0,
                 "context_limit": context_limit(model, sizes),
                 "status": "failed" if not ok else ("live" if session_id else "dead"),
                 "summary_path": None,
                 "started": _utcnow(), "last_active": _utcnow()}
        _add_usage(entry, tokens, chars)
        data["sessions"][key] = entry
    log_registry_event("start", key, entry, log_dir)
    return entry


def registry_resume(path: Path, key: str, tokens: int, chars: int, ok: bool,
                    compacted: bool, log_dir: Optional[Path] = None) -> Dict[str, Any]:
    with Registry(path) as data:
        entry = data["sessions"][key]
        entry["turns"] += 1
        _add_usage(entry, tokens, chars)
        entry["last_active"] = _utcnow()
        if not ok:
            entry["status"] = "failed"
        elif load_fraction(entry) >= CLOSE_AT:
            entry["status"] = "close_due"
        elif compacted:
            entry["status"] = "compacted"
    log_registry_event("resume", key, entry, log_dir)
    return entry


def registry_close(path: Path, key: str, summary_path: Optional[str],
                   log_dir: Optional[Path] = None) -> Dict[str, Any]:
    with Registry(path) as data:
        entry = data["sessions"].pop(key)
        entry.update({"status": "closed", "summary_path": summary_path, "closed": _utcnow()})
        data["archive"].append({"key": key, **entry})
    log_registry_event("close", key, entry, log_dir)
    return entry


def _registry_args(args: argparse.Namespace) -> Optional[Tuple[Path, str]]:
    ep = getattr(args, "episode", None)
    scene = getattr(args, "scene", None)
    if not ep and not scene:
        return None
    if not (ep and scene):
        raise SystemExit("--episode and --scene go together")
    return registry_path(Path(ep)), registry_key(Path(ep), scene, args.role)


def context_section(args: argparse.Namespace) -> Tuple[Optional[str], int]:
    """Fetch `--context notebook:query` sections. Returns (text, exit_code)."""
    specs = getattr(args, "context", None) or []
    if not specs:
        return "", 0
    cache = None
    if getattr(args, "episode", None):
        cache = registry_path(Path(args.episode)).parent / "context_cache"
    parts = []
    for spec in specs:
        notebook, sep, query = spec.partition(":")
        if not sep or not query.strip():
            print(f"--context expects <notebook>:<query>, got {spec!r}", file=sys.stderr)
            return None, 1
        try:
            parts.append(stage_context.get_context(notebook, query.strip(), cache)["section"])
        except stage_context.ContextError as e:
            print(str(e), file=sys.stderr)
            return None, e.code
    return "\n".join(parts) + "\n", 0


def get_git_info(worktree: Path) -> Tuple[str, str]:
    """Retrieve checkpoint hash and dirty paths via git status."""
    checkpoint = "unknown"
    dirty = ""
    try:
        cp = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            cwd=worktree,
            capture_output=True,
            text=True,
            check=True,
        )
        checkpoint = cp.stdout.strip()
    except Exception:
        pass

    try:
        sp = subprocess.run(
            ["git", "status", "--short"],
            cwd=worktree,
            capture_output=True,
            text=True,
            check=True,
        )
        dirty = sp.stdout.strip()
    except Exception:
        pass

    return checkpoint, dirty


def make_brief_header(worktree: Path, owns: Optional[List[str]] = None) -> str:
    """Prepend standard brief header (checkpoint, owns, dirty paths)."""
    checkpoint, dirty = get_git_info(worktree)
    owned_str = ", ".join(owns) if owns else "all"
    dirty_str = dirty if dirty else "(clean worktree)"
    return (
        f"Checkpoint: {checkpoint}\n"
        f"Owned files: {owned_str}\n"
        f"Dirty paths: {dirty_str}\n"
        f"{LOG_INSTRUCTIONS}\n"
    )


def build_start_cmd(
    agent: str,
    model: str,
    full_brief: str,
    worktree: Optional[Path] = None,
    images: Optional[List[str]] = None,
) -> List[str]:
    """Build exact argument list safely without shell string splitting."""
    if agent == "cursor-agent":
        cmd = ["cursor-agent", "-p", "--output-format", "json", "--trust", "--model", model]
        if worktree:
            cmd.extend(["--workspace", str(worktree.resolve())])
        cmd.append(full_brief)
        return cmd
    elif agent == "codex":
        cmd = ["codex", "exec", "-m", model, "-s", "workspace-write"]
        if worktree:
            cmd.extend(["-C", str(worktree.resolve())])
        if images:
            for img in images:
                cmd.extend(["-i", str(img)])
        cmd.append(full_brief)
        return cmd
    elif agent == "kimi":
        cmd = ["kimi", "-p", full_brief, "-m", model]  # -p cannot combine with --auto/--yolo (kimi 2.1)
        if worktree:
            cmd.extend(["--add-dir", str(worktree.resolve())])
        return cmd
    elif agent == "agy":
        cmd = ["agy", "--model", model, "--print-timeout", "40m", "--dangerously-skip-permissions"]
        if worktree:
            cmd.extend(["--add-dir", str(worktree.resolve())])
        cmd.append(f"--print={full_brief}")
        return cmd
    else:
        raise ValueError(f"Unknown agent: {agent}")


def build_resume_cmd(
    agent: str,
    session_id: str,
    message: str,
    worktree: Optional[Path] = None,
) -> List[str]:
    """Build exact resume argument list safely without shell string splitting."""
    if agent == "cursor-agent":
        cmd = ["cursor-agent", "-p", "--output-format", "json", "--trust", "--resume", session_id]
        if worktree:
            cmd.extend(["--workspace", str(worktree.resolve())])
        cmd.append(message)
        return cmd
    elif agent == "codex":
        cmd = ["codex", "exec", "resume", session_id]
        cmd.append(message)
        return cmd
    elif agent == "kimi":
        cmd = ["kimi", "-S", session_id, "-p", message]
        if worktree:
            cmd.extend(["--add-dir", str(worktree.resolve())])
        return cmd
    elif agent == "agy":
        cmd = ["agy", "--conversation", session_id, "--print-timeout", "40m", "--dangerously-skip-permissions"]
        if worktree:
            cmd.extend(["--add-dir", str(worktree.resolve())])
        cmd.append(f"--print={message}")
        return cmd
    else:
        raise ValueError(f"Unknown agent: {agent}")


def capture_session_id(
    agent: str,
    stdout: str = "",
    stderr: str = "",
    worktree: Optional[Path] = None,
) -> Tuple[Optional[str], int]:
    """Capture this run's session id and reported token usage where available."""
    session_id: Optional[str] = None
    tokens: int = 0

    # All supported workers may emit JSON usage or a human-readable "tokens
    # used" line.  Parse only the supplied invocation output for usage.
    output = stdout + "\n" + stderr
    for line in output.splitlines():
        try:
            data = json.loads(line)
        except Exception:
            data = None
        if isinstance(data, dict):
            usage = data.get("usage")
            if isinstance(usage, dict):
                tokens += sum(int(usage.get(k, 0) or 0) for k in
                              ("inputTokens", "outputTokens", "input_tokens", "output_tokens",
                               "prompt_tokens", "completion_tokens", "total_tokens"))
            for key in ("tokens_used", "tokensUsed", "total_tokens"):
                if isinstance(data.get(key), (int, float)):
                    tokens = max(tokens, int(data[key]))
        match = re.search(r"tokens?\s+used\s*[:=]\s*([\d,]+)", line, re.I)
        if match:
            tokens = max(tokens, int(match.group(1).replace(",", "")))

    if agent == "cursor-agent":
        for line in stdout.splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                data = json.loads(line)
                if isinstance(data, dict):
                    if "session_id" in data:
                        session_id = data["session_id"]
                    if "usage" in data and isinstance(data["usage"], dict):
                        u = data["usage"]
                        tokens = max(tokens, u.get("inputTokens", 0) + u.get("outputTokens", 0))
            except Exception:
                pass
        if not session_id and stdout.strip():
            try:
                data = json.loads(stdout.strip())
                if isinstance(data, dict):
                    session_id = data.get("session_id")
                    if "usage" in data and isinstance(data["usage"], dict):
                        u = data["usage"]
                        tokens = max(tokens, u.get("inputTokens", 0) + u.get("outputTokens", 0))
            except Exception:
                pass

    elif agent == "codex":
        # Codex prints the ID for this invocation near the top of its output.
        # Do not consult session_index.jsonl: it can still point at another run.
        match = re.search(
            r"\bsession\s+id\s*:\s*([0-9a-f]{8}-[0-9a-f-]{27,})\b",
            stdout + "\n" + stderr,
            flags=re.IGNORECASE,
        )
        if match:
            session_id = match.group(1)

    elif agent == "kimi":
        try:
            cmd = ["kimi", "session", "list", "--all", "--json", "--limit", "1"]
            res = subprocess.run(cmd, capture_output=True, text=True, check=True)
            items = json.loads(res.stdout)
            if items and isinstance(items, list) and isinstance(items[0], dict):
                session_id = items[0].get("id")
        except Exception:
            pass
        if not session_id:
            sessions_root = Path.home() / ".kimi-code" / "sessions"
            if sessions_root.is_dir():
                found_dirs = list(sessions_root.glob("*/session_*"))
                if found_dirs:
                    latest = max(found_dirs, key=lambda p: p.stat().st_mtime)
                    session_id = latest.name

    elif agent == "agy":
        brain_dir = Path.home() / ".gemini" / "antigravity-cli" / "brain"
        if brain_dir.is_dir():
            dirs = [p for p in brain_dir.iterdir() if p.is_dir() and not p.name.startswith(".")]
            if dirs:
                latest = max(dirs, key=lambda p: p.stat().st_mtime)
                session_id = latest.name

    return session_id, tokens


def append_dispatch_log(log_path: Path, record: Dict[str, Any]) -> None:
    """Append one record to dispatch log following standard schema."""
    log_path = Path(log_path).resolve()
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with open(log_path, "a", encoding="utf-8") as f:
        f.write(json.dumps(record) + "\n")


def default_dispatch_log() -> Path:
    return session_log_dir() / "dispatch-log.jsonl"


def files_changed_since(worktree: Path, before_dirty: str, after_dirty: str,
                        checkpoint: str, checkpoint_after: str) -> bool:
    if before_dirty != after_dirty:
        return True
    if checkpoint_after != checkpoint:
        if checkpoint and checkpoint != "unknown":
            return True
    return False


def cmd_start(args: argparse.Namespace) -> int:
    worktree = Path(args.worktree).resolve() if args.worktree else Path.cwd().resolve()
    brief_path = Path(args.brief).resolve()
    if not brief_path.is_file():
        print(f"Brief file not found: {brief_path}", file=sys.stderr)
        return 1

    brief_raw = brief_path.read_text()
    ctx, code = context_section(args)
    if ctx is None:
        return code
    reg = _registry_args(args)
    header = make_brief_header(worktree, args.owns)
    full_brief = header + ctx + brief_raw
    brief_chars = len(full_brief)

    cmd = build_start_cmd(
        agent=args.agent,
        model=args.model,
        full_brief=full_brief,
        worktree=worktree,
        images=args.images,
    )

    if args.dry_run:
        import shlex
        print("Dry run command:", shlex.join(cmd))
        return 0

    checkpoint, dirty_before = get_git_info(worktree)
    t0 = time.monotonic()
    cp = subprocess.run(cmd, cwd=worktree, capture_output=True, text=True, stdin=subprocess.DEVNULL)
    wall_s = time.monotonic() - t0
    checkpoint_after, dirty_after = get_git_info(worktree)

    session_id, tokens = capture_session_id(
        args.agent, stdout=cp.stdout, stderr=cp.stderr, worktree=worktree
    )

    verdict = "success" if cp.returncode == 0 else "failure"
    failure_mode = None if verdict == "success" else f"exit_{cp.returncode}"
    wrote_files = files_changed_since(worktree, dirty_before, dirty_after, checkpoint, checkpoint_after)

    # Flags string for log: omit prompt body
    if args.agent == "agy":
        flags_list = [c for c in cmd[1:] if not c.startswith("--print=")]
    else:
        flags_list = cmd[1:-1]
    flags_str = " ".join(flags_list)

    record = {
        "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "session_id": session_id,
        "project": args.project or worktree.name or "scriptcast",
        "task": args.task or brief_path.stem,
        "kind": args.kind or "implementation",
        "agent": args.agent,
        "model": args.model,
        "flags": flags_str,
        "brief_chars": brief_chars,
        "wall_s": round(wall_s, 2),
        "tokens": tokens or brief_chars // CHARS_PER_TOKEN,
        "tokens_kind": "reported" if tokens else "proxy",
        "checkpoint": checkpoint,
        "wrote_files": wrote_files,
        "acceptance": getattr(args, "acceptance", "not_run"),
        "iterations": 1,
        "verdict": verdict,
        "failure_mode": failure_mode,
        "note": args.note or "",
    }

    append_dispatch_log(Path(args.log) if args.log else default_dispatch_log(), record)
    stored = collect_session_log(worktree, record, cp.stdout)
    print(f"Session log: {stored}")
    if reg:
        entry = registry_start(reg[0], reg[1], args.agent, args.model, session_id,
                               tokens, brief_chars, ok=cp.returncode == 0)
        print(f"Registry: {reg[1]} {entry['status']} ({entry['tokens']} {entry['tokens_kind']} tokens)")

    print(f"Dispatched {args.agent} ({args.model}). Session ID: {session_id}")
    if cp.stdout:
        print(cp.stdout)
    if cp.stderr and cp.returncode != 0:
        print(cp.stderr, file=sys.stderr)

    return cp.returncode


def cmd_resume(args: argparse.Namespace) -> int:
    worktree = Path(args.worktree).resolve() if args.worktree else Path.cwd().resolve()
    msg_path = Path(args.message).resolve()
    if not msg_path.is_file():
        print(f"Message file not found: {msg_path}", file=sys.stderr)
        return 1

    message_content = msg_path.read_text()
    ctx, code = context_section(args)
    if ctx is None:
        return code
    reg = _registry_args(args)
    entry = None
    if reg:
        entry = Registry(reg[0]).get(reg[1])
        if entry is None:
            print(f"No registry entry for {reg[1]}; use start", file=sys.stderr)
            return 1
        if entry["status"] in ("closed", "dead"):
            print(f"Session {reg[1]} is {entry['status']}; start a fresh one", file=sys.stderr)
            return 1
        args.session = args.session or entry["session_id"]
        args.agent = args.agent or entry["agent"]
        if args.model == "resume":
            args.model = entry["model"]
    if not args.session or not args.agent:
        print("resume needs --session and --agent, or --episode/--scene with a registry entry",
              file=sys.stderr)
        return 1
    compact = compaction_message(entry) if entry else ""
    message_content = ctx + message_content + compact
    cmd = build_resume_cmd(
        agent=args.agent,
        session_id=args.session,
        message=message_content,
        worktree=worktree,
    )

    if args.dry_run:
        import shlex
        print("Dry run resume command:", shlex.join(cmd))
        return 0

    checkpoint, dirty_before = get_git_info(worktree)
    t0 = time.monotonic()
    cp = subprocess.run(cmd, cwd=worktree, capture_output=True, text=True, stdin=subprocess.DEVNULL)
    wall_s = time.monotonic() - t0
    checkpoint_after, dirty_after = get_git_info(worktree)

    _, tokens = capture_session_id(
        args.agent, stdout=cp.stdout, stderr=cp.stderr, worktree=worktree
    )
    verdict = "success" if cp.returncode == 0 else "failure"
    failure_mode = None if verdict == "success" else f"exit_{cp.returncode}"
    wrote_files = files_changed_since(worktree, dirty_before, dirty_after, checkpoint, checkpoint_after)

    if args.agent == "agy":
        flags_list = [c for c in cmd[1:] if not c.startswith("--print=")]
    else:
        flags_list = cmd[1:-1]
    flags_str = " ".join(flags_list)

    record = {
            "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "session_id": args.session,
            "project": getattr(args, "project", None) or worktree.name or "scriptcast",
            "task": getattr(args, "task", None) or msg_path.stem,
            "kind": getattr(args, "kind", "fix"),
            "agent": args.agent,
            "model": getattr(args, "model", "resume"),
            "flags": flags_str,
            "brief_chars": len(message_content),
            "wall_s": round(wall_s, 2),
            "tokens": tokens or len(message_content) // CHARS_PER_TOKEN,
            "tokens_kind": "reported" if tokens else "proxy",
            "checkpoint": checkpoint,
            "wrote_files": wrote_files,
            "acceptance": getattr(args, "acceptance", "not_run"),
            "iterations": 1,
            "verdict": verdict,
            "failure_mode": failure_mode,
            "note": getattr(args, "note", ""),
        }
    append_dispatch_log(Path(args.log) if args.log else default_dispatch_log(), record)
    print(f"Session log: {collect_session_log(worktree, record, cp.stdout)}")
    if reg:
        entry = registry_resume(reg[0], reg[1], tokens, len(message_content),
                                cp.returncode == 0, bool(compact))
        print(f"Registry: {reg[1]} turn {entry['turns']} {entry['status']} "
              f"({load_fraction(entry):.0%} of context)")

    print(f"Resumed {args.agent} session {args.session}.")
    if cp.stdout:
        print(cp.stdout)
    if cp.stderr and cp.returncode != 0:
        print(cp.stderr, file=sys.stderr)

    return cp.returncode


def cmd_close(args: argparse.Namespace) -> int:
    path, key = registry_path(args.episode), registry_key(args.episode, args.scene, args.role)
    if Registry(path).get(key) is None:
        print(f"No registry entry for {key}", file=sys.stderr)
        return 1
    summary = str(Path(args.summary).resolve()) if args.summary else None
    entry = registry_close(path, key, summary)
    print(f"Closed {key} after {entry['turns']} turns; summary: {summary}")
    return 0


def cmd_accept(args: argparse.Namespace) -> int:
    path = Path(args.log) if args.log else default_dispatch_log()
    if not path.is_file():
        print(f"Dispatch log not found: {path}", file=sys.stderr)
        return 1
    records = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    for record in reversed(records):
        if record.get("task") == args.task:
            record["acceptance"] = args.status
            if args.note is not None:
                record["note"] = args.note
            path.write_text("\n".join(json.dumps(r) for r in records) + "\n")
            return 0
    print(f"No dispatch record for task {args.task}", file=sys.stderr)
    return 1


def _add_registry_flags(p: argparse.ArgumentParser, context: bool = True) -> None:
    p.add_argument("--episode", type=Path, help="Episode dir; registry at <episode>/stage/.router/sessions.json")
    p.add_argument("--scene", help="Scene name (registry key part)")
    p.add_argument("--role", default="builder", help="builder | checker | ... (registry key part)")
    if context:
        p.add_argument("--context", action="append", metavar="NOTEBOOK:QUERY",
                       help="Prepend an nlm context section (alias from router/notebooks.toml or id)")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    # Subcommand: start
    p_start = sub.add_parser("start", help="Start a worker session with a brief")
    p_start.add_argument("--agent", required=True, choices=AGENTS, help="Worker CLI agent")
    p_start.add_argument("--model", required=True, help="Model identifier")
    p_start.add_argument("--brief", type=Path, required=True, help="Path to brief file")
    p_start.add_argument("--worktree", type=Path, help="Worktree directory (defaults to cwd)")
    p_start.add_argument("--images", nargs="*", help="Optional image files to attach")
    p_start.add_argument("--owns", nargs="*", help="Owned files listed in header")
    p_start.add_argument("--log", type=Path, help="Path to append dispatch log JSONL")
    p_start.add_argument("--project", help="Project name for dispatch log")
    p_start.add_argument("--task", help="Task name for dispatch log")
    p_start.add_argument("--kind", default="implementation", help="Task kind")
    p_start.add_argument("--note", default="", help="Note for dispatch log")
    p_start.add_argument("--acceptance", default="not_run", help="Acceptance status")
    p_start.add_argument("--dry-run", action="store_true", help="Print command without executing")
    _add_registry_flags(p_start)

    # Subcommand: resume
    p_resume = sub.add_parser("resume", help="Resume an existing session")
    p_resume.add_argument("--session", help="Session / conversation ID (else from registry)")
    p_resume.add_argument("--agent", choices=AGENTS, help="Worker CLI agent (else from registry)")
    p_resume.add_argument("--message", type=Path, required=True, help="Path to message file")
    p_resume.add_argument("--worktree", type=Path, help="Worktree directory (defaults to cwd)")
    p_resume.add_argument("--log", type=Path, help="Path to append dispatch log JSONL")
    p_resume.add_argument("--project", help="Project name for dispatch log")
    p_resume.add_argument("--task", help="Task name for dispatch log")
    p_resume.add_argument("--kind", default="fix", help="Task kind")
    p_resume.add_argument("--note", default="", help="Note for dispatch log")
    p_resume.add_argument("--model", default="resume", help="Model name")
    p_resume.add_argument("--dry-run", action="store_true", help="Print command without executing")
    _add_registry_flags(p_resume)

    p_close = sub.add_parser("close", help="Close a registry session and archive it")
    _add_registry_flags(p_close, context=False)
    p_close.add_argument("--summary", type=Path, help="Final state summary file")

    p_accept = sub.add_parser("accept", help="Set acceptance on the latest record for a task")
    p_accept.add_argument("--task", required=True)
    p_accept.add_argument("--status", required=True, choices=("pass", "fail"))
    p_accept.add_argument("--note")
    p_accept.add_argument("--log", type=Path)

    args = parser.parse_args(argv)

    if args.command == "start":
        return cmd_start(args)
    elif args.command == "resume":
        return cmd_resume(args)
    elif args.command == "close":
        if not (args.episode and args.scene):
            parser.error("close needs --episode and --scene")
        return cmd_close(args)
    elif args.command == "accept":
        return cmd_accept(args)
    return 0


if __name__ == "__main__":
    sys.exit(main())
