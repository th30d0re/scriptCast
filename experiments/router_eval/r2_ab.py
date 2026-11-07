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
import re
import tomllib
import sys


ROOT = Path(__file__).resolve().parents[2]
FIXES = Path(__file__).with_name("r2_fixes.json")
CSV = Path(__file__).with_name("r2_tokens.csv")
DEFAULT_BOOK = Path("/Users/emmanuel/Documents/Theory/TheOriginalPower")
NOTEBOOKS = ROOT / "scriptcast/router/notebooks.toml"
ARTIFACTS = ROOT / "experiments/router_eval/r2_runs"
DISPATCH = ROOT / "scriptcast/tools/stage_dispatch.py"
FIELDS = ["fix", "condition", "brief_chars", "tokens", "tokens_kind", "wall_s", "accepted", "exit_code", "stderr_tail"]


def load_fixes(path: Path = FIXES) -> list[dict]:
    rows = json.loads(path.read_text())
    if not isinstance(rows, list) or len(rows) != 3:
        raise ValueError("r2_fixes.json must contain exactly three fixes")
    required = {"id", "scene", "file", "description", "acceptance_check"}
    for row in rows:
        if required - set(row):
            raise ValueError(f"fix is missing fields: {row}")
    return rows


def _episode_number(episode: Path) -> int:
    match = re.search(r"yt(\d+)$", Path(episode).name, re.I)
    if not match:
        raise ValueError(f"cannot resolve notebook episode from {episode}")
    # YT02 is the premiere (episode 0) in the book's episode map.
    return int(match.group(1)) - 2


def notebook_id_for_episode(episode: Path, notebooks: Path = NOTEBOOKS) -> str:
    """Resolve the NotebookLM id from the episode number, never its title."""
    data = tomllib.loads(Path(notebooks).read_text())
    wanted = _episode_number(episode)
    for entry in (data.get("notebooks") or {}).values():
        if entry.get("episode") == wanted and entry.get("id"):
            return entry["id"]
    raise KeyError(f"no notebook configured for episode {wanted}")


def acceptance_command(episode: Path, fix: dict) -> list[str]:
    """Return the real stage check and require a worker diff."""
    script = fix["acceptance_check"]
    if not script.startswith("node tools/"):
        raise ValueError(f"acceptance_check must be a node stage check: {script}")
    return ["/bin/zsh", "-c", f"{script} && ! git diff --quiet"]


def _create_worktree(book: Path, fix: dict, condition: str) -> Path:
    worktree = Path(tempfile.mkdtemp(prefix=f"r2-{fix['id']}-{condition}-"))
    try:
        subprocess.run(["git", "-C", str(book.resolve()), "worktree", "add",
                        "--detach", str(worktree), fix.get("base", "HEAD")],
                       stdin=subprocess.DEVNULL, check=True)
    except Exception:
        shutil.rmtree(worktree, ignore_errors=True)
        raise
    return worktree


def _remove_worktree(book: Path, worktree: Path) -> None:
    subprocess.run(["git", "-C", str(book.resolve()), "worktree", "remove",
                    "--force", str(worktree)], stdin=subprocess.DEVNULL, check=True)


def _save_run_artifacts(worktree: Path, fix: dict, condition: str, stdout: str,
                        artifacts: Path = ARTIFACTS) -> None:
    dest = artifacts / f"{fix['id']}-{condition}"
    dest.mkdir(parents=True, exist_ok=True)
    diff = subprocess.run(["git", "diff"], cwd=worktree, capture_output=True, text=True, check=False)
    (dest / "git.diff").write_text(diff.stdout)
    (dest / "worker.stdout.tail").write_text("\n".join(stdout.splitlines()[-80:]) + ("\n" if stdout else ""))


def _codex_tokens_from_stdout(stdout: str) -> int | None:
    matches = re.findall(r"tokens?\s+used\s*[:=]\s*([\d,]+)", stdout, re.I)
    return int(matches[-1].replace(",", "")) if matches else None


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
                        "--context", f"{notebook_id_for_episode(episode)}:retrieve the relevant YT02 scene fix guidance",
                        "--task", fix["id"], "--log", str(log)]
    if dry_run:
        cmd.append("--dry-run")
    return [cmd]


def _accept_cmd(log: Path, fix: dict, passed: bool) -> list[str]:
    return ["python3", str(DISPATCH), "accept", "--task", fix["id"],
            "--status", "pass" if passed else "fail", "--log", str(log)]


def _dispatch_record(log: Path) -> dict:
    if not log.is_file():
        return {}
    records = []
    for line in log.read_text().splitlines():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(record, dict):
            records.append(record)
    return records[-1] if records else {}


def _session_id_from_run(fix_id: str, runs: Path = ARTIFACTS) -> str:
    """Read the original Codex session from stage_dispatch's saved output."""
    path = runs / f"{fix_id}-A" / "worker.stdout.tail"
    matches = re.findall(
        r"^Dispatched codex \([^\r\n]*\)\. Session ID: "
        r"([0-9a-fA-F]{8}(?:-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12})\s*$",
        path.read_text(), re.MULTILINE,
    )
    if not matches:
        raise ValueError(f"no Codex session id in {path}")
    return matches[-1]


def _followup_commands(worktree: Path, fix: dict, condition: str, session_id: str,
                       followup: Path, brief: Path, dry_run: bool) -> list[str]:
    common = [sys.executable, str(DISPATCH)]
    log = worktree / f".r2-{fix['id']}-{condition}.dispatch.jsonl"
    if condition == "resume":
        cmd = common + ["resume", "--session", session_id, "--agent", "codex", "--message", str(followup),
                        "--worktree", str(worktree), "--task", fix["id"], "--log", str(log)]
    else:
        cmd = common + ["start", "--agent", "codex", "--model", "gpt-5.6-luna",
                        "--brief", str(brief), "--worktree", str(worktree),
                        "--task", fix["id"], "--log", str(log)]
    if dry_run:
        cmd.append("--dry-run")
    return cmd


def run(args: argparse.Namespace) -> int:
    fixes = load_fixes(args.fixes)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    rows = []
    if args.resume_followup:
        for fix in fixes:
            session_id = _session_id_from_run(fix["id"], args.runs)
            with tempfile.TemporaryDirectory(prefix=f"r2-followup-{fix['id']}-") as temp:
                temp_dir = Path(temp)
                followup = temp_dir / "followup.txt"
                followup.write_text(fix["followup"] + "\n")
                brief = temp_dir / "followup-brief.txt"
                brief.write_text(f"{fix['followup']}\n\nFile: {fix['file']}\n")
                for condition in ("resume", "fresh"):
                    worktree = temp_dir / condition
                    cmd = _followup_commands(worktree, fix, condition, session_id,
                                             followup, brief, dry_run=not args.execute)
                    print(shlex.join(cmd))
                    if not args.execute:
                        rows.append({"fix": fix["id"], "condition": condition,
                                     "brief_chars": "", "tokens": "", "tokens_kind": "dry-run",
                                     "wall_s": "", "accepted": "not_run", "exit_code": "",
                                     "stderr_tail": ""})
                        continue
                    worktree = _create_worktree(args.book_repo, fix, f"followup-{condition}")
                    stdout = ""
                    try:
                        subprocess.run(
                            ["git", "apply", str((args.runs / f"{fix['id']}-A" / "git.diff").resolve())],
                            cwd=worktree, stdin=subprocess.DEVNULL, check=True,
                        )
                        cmd = _followup_commands(worktree, fix, condition, session_id,
                                                 followup, brief, dry_run=False)
                        t0 = time.monotonic()
                        env = os.environ.copy()
                        env["PYTHONPATH"] = str(ROOT.resolve())
                        proc = subprocess.run(cmd, cwd=worktree, stdin=subprocess.DEVNULL,
                                              capture_output=True, text=True, env=env)
                        stdout = proc.stdout
                        wall = time.monotonic() - t0
                        record = _dispatch_record(worktree / f".r2-{fix['id']}-{condition}.dispatch.jsonl")
                    finally:
                        try:
                            _save_run_artifacts(worktree, fix, f"followup-{condition}", stdout, args.runs)
                        finally:
                            _remove_worktree(args.book_repo, worktree)
                    tokens = record.get("tokens", "") or _codex_tokens_from_stdout(proc.stdout) or ""
                    rows.append({"fix": fix["id"], "condition": condition,
                                 "brief_chars": record.get("brief_chars", ""), "tokens": tokens,
                                 "tokens_kind": record.get("tokens_kind", "") or ("reported" if tokens else ""),
                                 "wall_s": f"{wall:.3f}", "accepted": "not_run",
                                 "exit_code": proc.returncode,
                                 "stderr_tail": "\n".join(proc.stderr.splitlines()[-10:])})
        with args.output.open("w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=FIELDS)
            writer.writeheader()
            writer.writerows(rows)
        return 0
    for fix in fixes:
        for condition in ("A", "B"):
            if args.execute:
                worktree = _create_worktree(args.book_repo, fix, condition)
            else:
                worktree = args.book_repo
            try:
                cmd = commands(args.book_repo, worktree, fix, condition, dry_run=not args.execute)[0]
                print(shlex.join(cmd))
                if not args.execute:
                    rows.append({"fix": fix["id"], "condition": condition, "brief_chars": "", "tokens": "", "tokens_kind": "dry-run", "wall_s": "", "accepted": "not_run", "exit_code": "", "stderr_tail": ""})
                    continue
                t0 = time.monotonic()
                env = os.environ.copy()
                env["PYTHONPATH"] = str(ROOT) + (os.pathsep + env["PYTHONPATH"] if env.get("PYTHONPATH") else "")
                proc = subprocess.run(cmd, cwd=worktree, stdin=subprocess.DEVNULL,
                                      capture_output=True, text=True, env=env)
                wall = time.monotonic() - t0
                stderr_tail = "\n".join(proc.stderr.splitlines()[-10:])
                if proc.returncode:
                    print(stderr_tail, file=os.sys.stderr)
                log = worktree / f".r2-{fix['id']}-{condition}.dispatch.jsonl"
                record = _dispatch_record(log)
                check = subprocess.run(acceptance_command(_episode(worktree), fix), cwd=_episode(worktree) / "stage").returncode == 0
                subprocess.run(_accept_cmd(log, fix, check), cwd=worktree, stdin=subprocess.DEVNULL, check=False)
                _save_run_artifacts(worktree, fix, condition, proc.stdout)
                tokens = record.get("tokens", "")
                if not tokens and "--agent codex" in shlex.join(cmd):
                    tokens = _codex_tokens_from_stdout(proc.stdout) or ""
                rows.append({"fix": fix["id"], "condition": condition,
                             "brief_chars": record.get("brief_chars", ""),
                             "tokens": tokens,
                             "tokens_kind": record.get("tokens_kind", "") or ("reported" if tokens else ""),
                             "wall_s": f"{wall:.3f}", "accepted": str(check).lower(),
                             "exit_code": proc.returncode, "stderr_tail": stderr_tail})
            finally:
                if args.execute:
                    _save_run_artifacts(worktree, fix, condition, locals().get("proc").stdout if "proc" in locals() else "")
                    _remove_worktree(args.book_repo, worktree)
    with args.output.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=FIELDS)
        writer.writeheader()
        writer.writerows(rows)
    return 0


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--book-repo", type=Path, default=DEFAULT_BOOK)
    ap.add_argument("--fixes", type=Path, default=FIXES)
    ap.add_argument("--output", type=Path)
    ap.add_argument("--execute", action="store_true", help="run workers; omitted by design in this sandbox")
    ap.add_argument("--resume-followup", action="store_true", help="run true resume/fresh follow-up arms")
    ap.add_argument("--runs", type=Path, default=ARTIFACTS, help="prior run artifacts containing dispatch logs")
    args = ap.parse_args(argv)
    if args.output is None:
        args.output = CSV.with_name("r2_resume.csv") if args.resume_followup else CSV
    return run(args)


if __name__ == "__main__":
    raise SystemExit(main())
