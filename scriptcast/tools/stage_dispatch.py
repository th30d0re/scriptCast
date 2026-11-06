"""Wrapper for worker CLIs: codex, cursor-agent, kimi, agy."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from typing import Any, Dict, List, Optional, Tuple

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
        cmd = ["kimi", "-p", full_brief, "-m", model, "--auto"]
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
        if worktree:
            cmd.extend(["-C", str(worktree.resolve())])
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
    worktree: Optional[Path] = None,
) -> Tuple[Optional[str], int]:
    """Capture session id (cursor-agent --output-format json; codex/kimi/agy from session listing)."""
    session_id: Optional[str] = None
    tokens: int = 0

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
                        tokens = u.get("inputTokens", 0) + u.get("outputTokens", 0)
            except Exception:
                pass
        if not session_id and stdout.strip():
            try:
                data = json.loads(stdout.strip())
                if isinstance(data, dict):
                    session_id = data.get("session_id")
                    if "usage" in data and isinstance(data["usage"], dict):
                        u = data["usage"]
                        tokens = u.get("inputTokens", 0) + u.get("outputTokens", 0)
            except Exception:
                pass

    elif agent == "codex":
        idx_path = Path.home() / ".codex" / "session_index.jsonl"
        if idx_path.is_file():
            try:
                lines = [l.strip() for l in idx_path.read_text().splitlines() if l.strip()]
                if lines:
                    entry = json.loads(lines[-1])
                    session_id = entry.get("id")
            except Exception:
                pass

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


def cmd_start(args: argparse.Namespace) -> int:
    worktree = Path(args.worktree).resolve() if args.worktree else Path.cwd().resolve()
    brief_path = Path(args.brief).resolve()
    if not brief_path.is_file():
        print(f"Brief file not found: {brief_path}", file=sys.stderr)
        return 1

    brief_raw = brief_path.read_text()
    header = make_brief_header(worktree, args.owns)
    full_brief = header + brief_raw
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

    _, dirty_before = get_git_info(worktree)
    t0 = time.monotonic()
    cp = subprocess.run(cmd, cwd=worktree, capture_output=True, text=True)
    wall_s = time.monotonic() - t0
    _, dirty_after = get_git_info(worktree)

    session_id, tokens = capture_session_id(args.agent, stdout=cp.stdout, worktree=worktree)

    verdict = "success" if cp.returncode == 0 else "failure"
    failure_mode = None if verdict == "success" else f"exit_{cp.returncode}"
    wrote_files = bool(dirty_after != dirty_before)

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
        "tokens": tokens,
        "wrote_files": wrote_files,
        "acceptance": getattr(args, "acceptance", "not_run"),
        "iterations": 1,
        "verdict": verdict,
        "failure_mode": failure_mode,
        "note": args.note or "",
    }

    if args.log:
        append_dispatch_log(Path(args.log), record)
    stored = collect_session_log(worktree, record, cp.stdout)
    print(f"Session log: {stored}")

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

    _, dirty_before = get_git_info(worktree)
    t0 = time.monotonic()
    cp = subprocess.run(cmd, cwd=worktree, capture_output=True, text=True)
    wall_s = time.monotonic() - t0
    _, dirty_after = get_git_info(worktree)

    _, tokens = capture_session_id(args.agent, stdout=cp.stdout, worktree=worktree)
    verdict = "success" if cp.returncode == 0 else "failure"
    failure_mode = None if verdict == "success" else f"exit_{cp.returncode}"
    wrote_files = bool(dirty_after != dirty_before)

    if args.agent == "agy":
        flags_list = [c for c in cmd[1:] if not c.startswith("--print=")]
    else:
        flags_list = cmd[1:-1]
    flags_str = " ".join(flags_list)

    if args.log:
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
            "tokens": tokens,
            "wrote_files": wrote_files,
            "acceptance": getattr(args, "acceptance", "not_run"),
            "iterations": 1,
            "verdict": verdict,
            "failure_mode": failure_mode,
            "note": getattr(args, "note", ""),
        }
        append_dispatch_log(Path(args.log), record)
        print(f"Session log: {collect_session_log(worktree, record, cp.stdout)}")

    print(f"Resumed {args.agent} session {args.session}.")
    if cp.stdout:
        print(cp.stdout)
    if cp.stderr and cp.returncode != 0:
        print(cp.stderr, file=sys.stderr)

    return cp.returncode


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

    # Subcommand: resume
    p_resume = sub.add_parser("resume", help="Resume an existing session")
    p_resume.add_argument("--session", required=True, help="Session / conversation ID")
    p_resume.add_argument("--agent", required=True, choices=AGENTS, help="Worker CLI agent")
    p_resume.add_argument("--message", type=Path, required=True, help="Path to message file")
    p_resume.add_argument("--worktree", type=Path, help="Worktree directory (defaults to cwd)")
    p_resume.add_argument("--log", type=Path, help="Path to append dispatch log JSONL")
    p_resume.add_argument("--project", help="Project name for dispatch log")
    p_resume.add_argument("--task", help="Task name for dispatch log")
    p_resume.add_argument("--kind", default="fix", help="Task kind")
    p_resume.add_argument("--note", default="", help="Note for dispatch log")
    p_resume.add_argument("--model", default="resume", help="Model name")
    p_resume.add_argument("--dry-run", action="store_true", help="Print command without executing")

    args = parser.parse_args(argv)

    if args.command == "start":
        return cmd_start(args)
    elif args.command == "resume":
        return cmd_resume(args)
    return 0


if __name__ == "__main__":
    sys.exit(main())
