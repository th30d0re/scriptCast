"""R2 A/B harness for three real YT02 fixes.

The default mode is deliberately dry-run: this environment cannot launch
workers.  ``--execute`` is the operator-only path that creates one detached
book-repo worktree per condition, runs stage_dispatch with stdin closed, runs
the fix's acceptance command, and marks the matching dispatch record accepted.
"""
from __future__ import annotations

import argparse
import csv
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile
import time


ROOT = Path(__file__).resolve().parents[2]
FIXES = Path(__file__).with_name("r2_fixes.json")
CSV = Path(__file__).with_name("r2_tokens.csv")
DEFAULT_BOOK = Path("/Users/emmanuel/Documents/Theory/TheOriginalPower")
DISPATCH = ROOT / "scriptcast/tools/stage_dispatch.py"
FIELDS = ["fix", "condition", "brief_chars", "tokens", "tokens_kind", "wall_s", "accepted"]


def load_fixes(path: Path = FIXES) -> list[dict]:
    rows = json.loads(path.read_text())
    if not isinstance(rows, list) or len(rows) != 3:
        raise ValueError("r2_fixes.json must contain exactly three fixes")
    required = {"id", "scene", "file", "description", "acceptance_check"}
    for row in rows:
        if required - set(row):
            raise ValueError(f"fix is missing fields: {row}")
    return rows


def _brief(book: Path, fix: dict) -> Path:
    candidates = {
        "minute-two-figure-centering": "stage/BRIEF_MINUTE_TWO_FIX2.md",
        "dashboard-tray-wage-plane": "stage/BRIEF_DASHBOARD_TRAY_FIX2.md",
        "first-minute-dictionary-gutter": "stage/BRIEF_FIRST_MINUTE_FIX1.md",
    }
    path = book / "Architecting_the_operation/video/yt02" / candidates[fix["id"]]
    if not path.is_file():
        raise FileNotFoundError(path)
    return path


def _episode(worktree: Path) -> Path:
    return worktree / "Architecting_the_operation/video/yt02"


def _registry_exists(episode: Path, scene: str) -> bool:
    path = episode / "stage/.router/sessions.json"
    if not path.is_file():
        return False
    try:
        data = json.loads(path.read_text())
        key = f"{episode.name}/{scene}/builder"
        entry = data.get("sessions", {}).get(key)
        return bool(entry and entry.get("status") not in {"closed", "dead"})
    except (OSError, json.JSONDecodeError):
        return False


def commands(book: Path, worktree: Path, fix: dict, condition: str, dry_run: bool = True) -> list[list[str]]:
    episode = _episode(worktree)
    log = worktree / f".r2-{fix['id']}-{condition}.dispatch.jsonl"
    brief = _brief(worktree, fix)
    common = ["python3", str(DISPATCH)]
    if condition == "A":
        cmd = common + ["start", "--agent", "codex", "--model", "gpt-5.6-luna",
                        "--brief", str(brief), "--worktree", str(worktree),
                        "--episode", str(episode), "--scene", fix["scene"],
                        "--task", fix["id"], "--log", str(log)]
        if dry_run:
            cmd.append("--dry-run")
        return [cmd]
    if _registry_exists(episode, fix["scene"]):
        cmd = common + ["resume", "--message", str(brief), "--worktree", str(worktree),
                        "--episode", str(episode), "--scene", fix["scene"],
                        "--task", fix["id"], "--log", str(log)]
    else:
        cmd = common + ["start", "--agent", "codex", "--model", "gpt-5.6-luna",
                        "--brief", str(brief), "--worktree", str(worktree),
                        "--episode", str(episode), "--scene", fix["scene"],
                        "--context", "NOTEBOOK:The Original Power:retrieve the relevant YT02 scene fix guidance",
                        "--task", fix["id"], "--log", str(log)]
    if dry_run:
        cmd.append("--dry-run")
    return [cmd]


def _accept_cmd(log: Path, fix: dict, passed: bool) -> list[str]:
    return ["python3", str(DISPATCH), "accept", "--task", fix["id"],
            "--status", "pass" if passed else "fail", "--log", str(log)]


def run(args: argparse.Namespace) -> int:
    fixes = load_fixes(args.fixes)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    rows = []
    for fix in fixes:
        for condition in ("A", "B"):
            if args.execute:
                worktree = Path(tempfile.mkdtemp(prefix=f"r2-{fix['id']}-{condition}-"))
                subprocess.run(["git", "-C", str(args.book_repo), "worktree", "add", "--detach", str(worktree), "HEAD"], check=True)
            else:
                worktree = args.book_repo
            try:
                cmd = commands(args.book_repo, worktree, fix, condition, dry_run=not args.execute)[0]
                print(shlex.join(cmd))
                if not args.execute:
                    rows.append({"fix": fix["id"], "condition": condition, "brief_chars": "", "tokens": "", "tokens_kind": "dry-run", "wall_s": "", "accepted": "not_run"})
                    continue
                t0 = time.monotonic()
                proc = subprocess.run(cmd[:-1], cwd=worktree, stdin=subprocess.DEVNULL, capture_output=True, text=True)
                wall = time.monotonic() - t0
                check = subprocess.run(fix["acceptance_check"], cwd=_episode(worktree) / "stage", shell=True, executable="/bin/zsh").returncode == 0
                log = worktree / f".r2-{fix['id']}-{condition}.dispatch.jsonl"
                subprocess.run(_accept_cmd(log, fix, check), cwd=worktree, stdin=subprocess.DEVNULL, check=False)
                rows.append({"fix": fix["id"], "condition": condition, "brief_chars": "", "tokens": "", "tokens_kind": "reported_or_proxy", "wall_s": f"{wall:.3f}", "accepted": str(check).lower()})
                if proc.returncode:
                    return proc.returncode
            finally:
                if args.execute:
                    subprocess.run(["git", "-C", str(args.book_repo), "worktree", "remove", "--force", str(worktree)], check=False)
                    shutil.rmtree(worktree, ignore_errors=True)
    with args.output.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=FIELDS)
        writer.writeheader()
        writer.writerows(rows)
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--book-repo", type=Path, default=DEFAULT_BOOK)
    ap.add_argument("--fixes", type=Path, default=FIXES)
    ap.add_argument("--output", type=Path, default=CSV)
    ap.add_argument("--execute", action="store_true", help="run workers; omitted by design in this sandbox")
    return run(ap.parse_args(argv))


if __name__ == "__main__":
    raise SystemExit(main())
