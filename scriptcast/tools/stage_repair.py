"""Turn grounded Stage critic failures into bounded builder repair briefs.

The default is deliberately report-only.  ``--dispatch`` is the opt-in that
contacts a worker and then re-critiques the single affected still.
"""
from __future__ import annotations

import argparse
import json
import shutil
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

from scriptcast.tools import stage_critic, stage_dispatch
from scriptcast.tools.stage import main as stage_main


def _frame_name(file: str) -> tuple[str, str]:
    scene, sep, cue = Path(file).stem.rpartition("_")
    return (scene, cue) if sep else (Path(file).stem, "end")


def _marks(path: Path) -> dict[str, Any]:
    try:
        raw = json.loads(path.read_text())
    except (OSError, ValueError, json.JSONDecodeError):
        return {}
    marks = raw.get("marks", raw) if isinstance(raw, dict) else raw
    if not isinstance(marks, list):
        return {}
    return {str(mark["id"]): mark.get("box") for mark in marks
            if isinstance(mark, dict) and mark.get("id") and mark.get("box") is not None}


def _shot(plan: Path, scene: str) -> str:
    """Resolve the Stage shot id for a scene; old plans use the scene as shot."""
    data = json.loads(plan.read_text())
    for segment in data.get("segments", []):
        if segment.get("scene", segment.get("instance")) == scene:
            shots = segment.get("shots", [])
            if shots and shots[0].get("shot_id"):
                return str(shots[0]["shot_id"])
    return scene


def stage_boxes(plan: Path, shot: str, output: Path) -> dict[str, Any]:
    """Render live Stage marks for a shot and return its id → box mapping."""
    code = stage_main(["boxes", str(plan), "--shot", shot, "-o", str(output)])
    if code:
        raise RuntimeError(f"stage boxes failed for {shot} (exit {code})")
    return _marks(output)


def repair_briefs(report: dict, stills: Path, plan: Path, boxes_dir: Path | None = None,
                  reference_dir: Path | None = None,
                  boxes: Callable[[Path, str, Path], dict[str, Any]] = stage_boxes) -> list[dict]:
    """Build one brief per grounded critic defect, resolving live Stage boxes."""
    out = []
    for frame in report.get("frames", []):
        if frame.get("verdict") != "fail" or frame.get("status") == "contract_error":
            continue
        file = str(frame.get("file", "")); scene, cue = _frame_name(file); shot = _shot(plan, scene)
        still = stills / file
        sidecar = (boxes_dir / f"{file}.boxes.json") if boxes_dir else still.with_name(f"{file}.boxes.json")
        marks = _marks(sidecar)
        for defect in frame.get("defects", []):
            element_id = defect.get("element_id")
            # Anonymous Content defects do not have a builder-addressable mark.
            if not element_id:
                continue
            if element_id not in marks:
                generated = still.with_name(f"{file}.{element_id}.boxes.json")
                marks.update(boxes(plan, shot, generated))
            box = marks.get(element_id)
            ref = reference_dir / file if reference_dir and (reference_dir / file).is_file() else None
            text = (f"Repair Stage shot {shot} (scene {scene}, cue {cue}).\n"
                    f"Element: {element_id}\nBox: {json.dumps(box)}\n"
                    f"Defect: {defect.get('evidence', '')}\n"
                    f"Reference still: {ref if ref else '(none)'}\n"
                    f"Current still: {still}\n"
                    "Change only what is needed, then preserve declared element ids.")
            out.append({"scene": scene, "shot": shot, "cue": cue, "file": file, "element_id": element_id,
                        "box": box, "defect": defect.get("evidence", ""), "reference": str(ref) if ref else None,
                        "brief": text})
    return out


def _attempts(path: Path) -> dict[tuple[str, str, str], int]:
    if not path.is_file(): return {}
    result = {}
    for line in path.read_text().splitlines():
        try: row = json.loads(line)
        except json.JSONDecodeError: continue
        key = (str(row.get("scene")), str(row.get("cue")), str(row.get("element_id")))
        result[key] = result.get(key, 0) + 1
    return result


def _recritic(stills: Path, plan: Path, brief: dict, reference_dir: Path | None,
              critic: Callable[..., dict]) -> dict | None:
    """Give critic exactly one still, rather than treating a whole review as fixed."""
    source = stills / brief["file"]
    with tempfile.TemporaryDirectory(prefix="stage-repair-") as temp:
        target = Path(temp); shutil.copy2(source, target / source.name)
        sidecar = source.with_name(f"{source.name}.boxes.json")
        if sidecar.is_file(): shutil.copy2(sidecar, target / sidecar.name)
        refs = None
        if reference_dir and (reference_dir / source.name).is_file():
            refs = target / "references"; refs.mkdir(); shutil.copy2(reference_dir / source.name, refs / source.name)
        report = critic(target, plan, reference_dir=refs)
        return next((f for f in report.get("frames", []) if f.get("file") == source.name), None)


def run_repairs(episode: Path, critic_report: Path, stills: Path, plan: Path, *, dispatch: bool = False,
                boxes_dir: Path | None = None, reference_dir: Path | None = None,
                agent: str = "codex", model: str = "gpt-5.6-sol",
                scenes: set[str] | None = None,
                dispatch_main: Callable[[list[str]], int] = stage_dispatch.main,
                critic: Callable[..., dict] = stage_critic.critic_stills,
                boxes: Callable[[Path, str, Path], dict[str, Any]] = stage_boxes) -> list[dict]:
    review = episode / "stage" / "review"; review.mkdir(parents=True, exist_ok=True)
    log = review / "repairs.jsonl"; attempts = _attempts(log)
    briefs = repair_briefs(json.loads(critic_report.read_text()), stills, plan, boxes_dir, reference_dir, boxes)
    records = []
    for brief in briefs:
        if scenes is not None and brief["scene"] not in scenes:
            continue
        key = (brief["scene"], brief["cue"], brief["element_id"]); prior = attempts.get(key, 0)
        record = {k: brief[k] for k in ("scene", "shot", "cue", "file", "element_id", "box", "defect", "reference")}
        record.update({"ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"), "before": "fail",
                       "attempt": prior + 1, "action": "report"})
        # Useful in report-only mode; it also makes each repair record auditable.
        record["brief"] = brief["brief"]
        if prior >= 2:
            record.update({"action": "operator_queue", "after": None})
        elif dispatch:
            message = review / f"repair-{brief['scene']}-{brief['cue']}-{brief['element_id']}.md"
            message.write_text(brief["brief"] + "\n")
            reg = stage_dispatch.Registry(stage_dispatch.registry_path(episode)).get(
                stage_dispatch.registry_key(episode, brief["scene"], "builder"))
            cmd = (["resume", "--message", str(message), "--episode", str(episode), "--scene", brief["scene"]]
                   if reg else ["start", "--agent", agent, "--model", model, "--brief", str(message), "--episode", str(episode), "--scene", brief["scene"]])
            code = dispatch_main(cmd); record.update({"action": "resume" if reg else "start", "dispatch_exit": code})
            after = _recritic(stills, plan, brief, reference_dir, critic) if code == 0 else None
            record["after"] = after.get("verdict") if after else None
            record["after_frame"] = after
        else:
            record["after"] = None
        with log.open("a") as f: f.write(json.dumps(record) + "\n")
        records.append(record)
    return records


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--episode", type=Path, required=True); p.add_argument("--critic", type=Path, required=True)
    p.add_argument("--stills", type=Path, required=True); p.add_argument("--plan", type=Path, required=True)
    p.add_argument("--boxes-dir", type=Path); p.add_argument("--reference-dir", type=Path)
    p.add_argument("--scenes", nargs="+", help="limit repairs to named scenes")
    p.add_argument("--dispatch", action="store_true"); p.add_argument("--report-only", action="store_true")
    p.add_argument("--agent", default="codex"); p.add_argument("--model", default="gpt-5.6-sol")
    args = p.parse_args(argv)
    records = run_repairs(args.episode, args.critic, args.stills, args.plan, dispatch=args.dispatch,
                          boxes_dir=args.boxes_dir, reference_dir=args.reference_dir, agent=args.agent, model=args.model,
                          scenes=set(args.scenes) if args.scenes else None)
    # Explicit --report-only and the default both print the actual builder brief.
    for row in records:
        if not args.dispatch:
            print(row["brief"] + "\n")
        print(f"{row['action']}: {row['scene']}/{row['cue']}/{row['element_id']} ({row['before']} → {row['after']})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
