#!/usr/bin/env python3
"""Render and judge the SVG-offload experiment candidates.

This intentionally does no work at import time.  It is meant to be run on the
GPU host after ``run.py`` has produced its diffs.
"""
from __future__ import annotations

import argparse
import csv
import json
import os
import shutil
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Callable

EXP_DIR = Path(__file__).resolve().parent
SCRIPTCAST_REPO = Path(__file__).resolve().parents[2]
BOOK = Path("/Users/emmanuel/Documents/Theory/TheOriginalPower")
STAGE_REL = Path("Architecting_the_operation/video/yt02/stage")
LIVE_PLAN = BOOK / STAGE_REL / "ov/stage_plan.json"
TIMEOUT = 300
JUDGE_HEADER = ["cell", "worker", "cue", "status", "render_ok", "critic_verdict", "n_defects", "pixel_diff"]


def cue_fraction(plan: dict, shot: str, cue_id: str, settle_seconds: float = .9) -> float:
    """Return a cue's position in ``shot``, shifted past its animation settle."""
    cue = plan.get("cues", {}).get(f"{shot}.{cue_id}", plan.get("cues", {}).get(cue_id))
    if isinstance(cue, dict):
        cue = cue.get("at_ms", cue.get("time_ms", cue.get("start_ms")))
    if cue is None:
        raise KeyError(f"cue not found: {shot}.{cue_id}")
    for segment in plan.get("segments", []):
        for item in segment.get("shots", []):
            if item.get("shot_id", item.get("id")) == shot:
                start, end = float(item["start_ms"]), float(item["end_ms"])
                if end <= start:
                    raise ValueError(f"invalid shot range for {shot}")
                return min(.98, max(0.0, (float(cue) + settle_seconds * 1000 - start) / (end - start)))
    raise KeyError(f"shot not found: {shot}")


def rewrite_plan(live_plan: Path, destination: Path, episode_dir: Path) -> dict:
    plan = json.loads(live_plan.read_text())
    plan["episode_dir"] = str(episode_dir)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(plan, indent=2) + "\n")
    return plan


def _run(command: list[str], *, cwd: Path | None = None, dry_run: bool = False,
         runner: Callable = subprocess.run) -> subprocess.CompletedProcess:
    if dry_run:
        print("[dry-run]", " ".join(map(str, command)))
        return subprocess.CompletedProcess(command, 0, "", "")
    return runner(command, cwd=cwd, stdin=subprocess.DEVNULL, text=True, capture_output=True, timeout=TIMEOUT)


def _render(plan: Path, shot: str, fraction: float, output: Path, repo: Path, *, dry_run: bool, runner: Callable) -> bool:
    output.parent.mkdir(parents=True, exist_ok=True)
    result = _run([sys.executable, "-m", "scriptcast.tools.stage", "still", str(plan), "--shot", shot,
                   "--at", f"{fraction:.6f}", "-o", str(output)], cwd=repo, dry_run=dry_run, runner=runner)
    return result.returncode == 0 and (dry_run or output.is_file())


def _cell_name(row: dict) -> str:
    return f"{row['cue']}-{row.get('condition', 'B')}-{row['worker']}"


def judge_row(row: dict, exp_dir: Path, live_plan: Path, *, dry_run: bool, runner: Callable) -> dict:
    cell = _cell_name(row)
    record = {"cell": cell, "worker": row.get("worker", ""), "cue": row.get("cue", ""), "status": "pending",
              "render_ok": False, "critic_verdict": "", "n_defects": "", "pixel_diff": ""}
    diff = exp_dir / "out" / cell / "tsx.diff"
    if not diff.is_file() or not diff.read_text().strip():
        record["status"] = "no_change"
        return record
    root = exp_dir / "judge"
    worktree = root / "wt" / cell
    candidate_dir = root / cell
    try:
        added = _run(["git", "-C", str(BOOK), "worktree", "add", "--detach", str(worktree), "HEAD"], dry_run=dry_run, runner=runner)
        if added.returncode:
            record["status"] = "apply_failed"
            return record
        applied = _run(["git", "-C", str(worktree), "apply", str(diff)], dry_run=dry_run, runner=runner)
        if applied.returncode:
            record["status"] = "apply_failed"
            return record
        plan = rewrite_plan(live_plan, candidate_dir / "plan.json", worktree / STAGE_REL)
        fraction = cue_fraction(plan, row["shot"], row["cue_id"])
        new = candidate_dir / "new.png"
        record["render_ok"] = _render(candidate_dir / "plan.json", row["shot"], fraction, new, SCRIPTCAST_REPO,
                                      dry_run=dry_run, runner=runner)
        record["status"] = "ok" if record["render_ok"] else "render_failed"
        return record
    except (OSError, subprocess.TimeoutExpired, KeyError, ValueError, json.JSONDecodeError):
        record["status"] = "render_failed"
        return record
    finally:
        if worktree.exists() or dry_run:
            _run(["git", "-C", str(BOOK), "worktree", "remove", "--force", str(worktree)], dry_run=dry_run, runner=runner)


def _reference(row: dict, exp_dir: Path, plan: dict, live_plan: Path, *, dry_run: bool, runner: Callable) -> Path:
    target = exp_dir / "judge/ref" / f"{row['cue']}.png"
    if target.is_file() or dry_run:
        return target
    target.parent.mkdir(parents=True, exist_ok=True)
    _render(live_plan, row["shot"], cue_fraction(plan, row["shot"], row["cue_id"]), target, SCRIPTCAST_REPO,
            dry_run=dry_run, runner=runner)
    return target


def _critic(records: list[dict], rows: dict[str, dict], exp_dir: Path, live_plan: Path, *, dry_run: bool, runner: Callable) -> None:
    # stage_critic derives scene/cue from the filename, so four workers cannot
    # share one flat directory.  Give each invocation the exact Stage layout.
    for record in records:
        if not record["render_ok"]:
            continue
        row = rows[record["cell"]]
        name = f"{row['scene']}_{row['cue_id']}.png"
        source = exp_dir / "judge" / record["cell"] / "new.png"
        stills = exp_dir / "judge" / record["cell"] / "critic_stills"
        references = exp_dir / "judge" / record["cell"] / "critic_ref"
        if not dry_run:
            shutil.rmtree(stills, ignore_errors=True); shutil.rmtree(references, ignore_errors=True)
            stills.mkdir(); references.mkdir()
            os.symlink(source, stills / name)
            os.symlink(exp_dir / "judge/ref" / f"{row['cue']}.png", references / name)
        _run([sys.executable, "-m", "scriptcast.tools.stage_critic", "--stills", str(stills), "--plan", str(live_plan),
              "--reference-dir", str(references)], cwd=SCRIPTCAST_REPO, dry_run=dry_run, runner=runner)
        report_path = stills / "critic.json"
        report = json.loads(report_path.read_text()) if report_path.is_file() else {"frames": []}
        frame = next((f for f in report.get("frames", []) if f.get("file") == name), {})
        record["critic_verdict"] = frame.get("verdict", frame.get("status", ""))
        record["n_defects"] = len(frame.get("defects", []))
        from scriptcast.critic.grounding import pixel_diff
        record["pixel_diff"] = pixel_diff(exp_dir / "judge/ref" / f"{row['cue']}.png", source, [0, 0, 1, 1])


def contact_sheets(records: list[dict], rows: dict[str, dict], exp_dir: Path) -> None:
    from PIL import Image, ImageDraw
    for cue in sorted({r["cue"] for r in records}):
        group = [r for r in records if r["cue"] == cue]
        paths = [("reference", exp_dir / "judge/ref" / f"{cue}.png")] + [(r["worker"], exp_dir / "judge" / r["cell"] / "new.png") for r in group]
        available = [(label, path) for label, path in paths if path.is_file()]
        if not available:
            continue
        images = [Image.open(path).convert("RGB") for _, path in available]
        width, height = images[0].size
        sheet = Image.new("RGB", (width * len(images), height + 28), "white")
        draw = ImageDraw.Draw(sheet)
        for index, ((label, _), image) in enumerate(zip(available, images)):
            sheet.paste(image, (index * width, 28)); draw.text((index * width + 4, 6), label, fill="black")
        sheet.save(exp_dir / "judge" / f"contact_sheet_{cue}.png")


def main(argv: list[str] | None = None, *, runner: Callable = subprocess.run) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--exp-dir", type=Path, default=EXP_DIR)
    parser.add_argument("--cells", nargs="*")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--skip-critic", action="store_true")
    parser.add_argument("--concurrency", type=int, default=2)
    args = parser.parse_args(argv)
    exp = args.exp_dir.resolve(); plan_path = LIVE_PLAN
    rows = list(csv.DictReader((exp / "results.csv").open(newline="")))
    if args.cells:
        wanted = set(args.cells); rows = [r for r in rows if _cell_name(r) in wanted or r["cue"] in wanted]
    plan = json.loads(plan_path.read_text())
    records: list[dict] = []
    with ThreadPoolExecutor(max_workers=args.concurrency) as pool:
        futures = [pool.submit(judge_row, row, exp, plan_path, dry_run=args.dry_run, runner=runner) for row in rows]
        for future in as_completed(futures): records.append(future.result())
    records.sort(key=lambda r: r["cell"]); by_cell = {_cell_name(row): row for row in rows}
    for row in rows: _reference(row, exp, plan, plan_path, dry_run=args.dry_run, runner=runner)
    if not args.skip_critic: _critic(records, by_cell, exp, plan_path, dry_run=args.dry_run, runner=runner)
    if not args.dry_run:
        out = exp / "judge/judge.csv"; out.parent.mkdir(parents=True, exist_ok=True)
        with out.open("w", newline="") as handle:
            writer = csv.DictWriter(handle, fieldnames=JUDGE_HEADER); writer.writeheader(); writer.writerows(records)
        contact_sheets(records, by_cell, exp)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
