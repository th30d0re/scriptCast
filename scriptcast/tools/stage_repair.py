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

from PIL import Image

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


def _sidecar_path(path: Path, boxes_dir: Path | None = None) -> Path:
    """Resolve exact-still sidecars first, then the documented stem form."""
    base = boxes_dir if boxes_dir is not None else path.parent
    exact = base / f"{path.name}.boxes.json"
    return exact if exact.is_file() else base / f"{path.stem}.boxes.json"


def _scene_details(plan: Path, scene: str) -> tuple[str, str]:
    """Resolve a scene's shot and its builder source from a Stage plan."""
    data = json.loads(plan.read_text())
    for segment in data.get("segments", []):
        if segment.get("scene", segment.get("instance")) == scene:
            shots = segment.get("shots", [])
            shot = str(shots[0].get("shot_id", scene)) if shots else scene
            # Keep this permissive for plans which record the source explicitly,
            # while retaining the documented Stage convention as the fallback.
            source = next((value for item in (segment, *shots)
                           for key in ("source_file", "source", "file", "tsx")
                           if isinstance(item, dict) and (value := item.get(key))), None)
            if source:
                return shot, str(source)
            root = Path(data.get("episode_dir", plan.parent))
            return shot, str(root / f"{scene}.tsx")
    return scene, str(Path(data.get("episode_dir", plan.parent)) / f"{scene}.tsx")


def _box_values(box: Any) -> tuple[float, float, float, float] | None:
    if not isinstance(box, dict):
        return None
    try:
        return (float(box["x"]), float(box["y"]),
                float(box.get("w", box.get("width"))), float(box.get("h", box.get("height"))))
    except (KeyError, TypeError, ValueError):
        return None


def _defect_class(still_box: Any, reference_box: Any, frame_size: tuple[int, int]) -> str:
    """Classify a marked difference using the supplied rendered boxes."""
    current, approved = _box_values(still_box), _box_values(reference_box)
    if approved and not current:
        return "missing"
    if current and not approved:
        return "extra"
    if not current or not approved:
        return "appearance"
    sx, sy, sw, sh = current; rx, ry, rw, rh = approved
    # Marks may be normalized (0..1) or be in rendered pixels.
    normalized = max(abs(sx), abs(sy), abs(sw), abs(sh), abs(rx), abs(ry), abs(rw), abs(rh)) <= 1.0
    width, height = (1.0, 1.0) if normalized else frame_size
    centre_shift = max(abs((sx + sw / 2) - (rx + rw / 2)) / width,
                       abs((sy + sh / 2) - (ry + rh / 2)) / height)
    if centre_shift > .04:
        return "moved"
    if rw <= 0 or rh <= 0 or sw <= 0 or sh <= 0:
        return "appearance"
    ratio = (sw * sh) / (rw * rh)
    return "resized" if ratio < .75 or ratio > 1.33 else "appearance"


def _write_crop(source: Path, box: Any, destination: Path) -> str | None:
    values = _box_values(box)
    if not values or not source.is_file():
        return None
    try:
        with Image.open(source) as image:
            x, y, width, height = values
            if max(abs(x), abs(y), abs(width), abs(height)) <= 1.0:
                x, width = x * image.width, width * image.width
                y, height = y * image.height, height * image.height
            padding_x, padding_y = width * .1, height * .1
            crop = image.crop((max(0, int(x - padding_x)), max(0, int(y - padding_y)),
                               min(image.width, int(x + width + padding_x)),
                               min(image.height, int(y + height + padding_y))))
            crop.save(destination)
    except (OSError, ValueError):
        return None
    return str(destination)


def stage_boxes(plan: Path, shot: str, output: Path) -> dict[str, Any]:
    """Render live Stage marks for a shot and return its id → box mapping."""
    code = stage_main(["boxes", str(plan), "--shot", shot, "-o", str(output)])
    if code:
        raise RuntimeError(f"stage boxes failed for {shot} (exit {code})")
    return _marks(output)


def repair_briefs(report: dict, stills: Path, plan: Path, boxes_dir: Path | None = None,
                  reference_dir: Path | None = None,
                  boxes: Callable[[Path, str, Path], dict[str, Any]] = stage_boxes,
                  s1_threshold: float | None = None, report_dir: Path | None = None) -> list[dict]:
    """Build one brief per grounded critic defect, resolving live Stage boxes."""
    out = []
    for frame in report.get("frames", []):
        if frame.get("verdict") != "fail" or frame.get("status") == "contract_error":
            continue
        file = str(frame.get("file", "")); scene, cue = _frame_name(file); shot, source_file = _scene_details(plan, scene)
        still = stills / file
        sidecar = _sidecar_path(still, boxes_dir)
        marks = _marks(sidecar)
        defects = list(frame.get("defects", []))
        if not defects and frame.get("tier1", {}).get("flagged"):
            threshold = s1_threshold
            if threshold is None:
                threshold = float(report.get("s1_threshold", 0.2))
            candidates = [element for element in frame.get("tier1", {}).get("elements", [])
                          if element.get("p_defect", 0) >= threshold and not str(element.get("id", "")).startswith("grid_")]
            defects = [{"element_id": element["id"],
                        "evidence": (f"System 1 flagged this element (p={element['p_defect']:.2f}) "
                                     "as differing from the approved reference"),
                        "_p_defect": element["p_defect"]}
                       for element in sorted(candidates, key=lambda item: item.get("p_defect", 0), reverse=True)[:3]]
        for defect in defects:
            element_id = defect.get("element_id")
            # Anonymous Content defects do not have a builder-addressable mark.
            if not element_id or str(element_id).startswith("grid_"):
                continue
            ref = reference_dir / file if reference_dir and (reference_dir / file).is_file() else None
            reference_marks = _marks(_sidecar_path(ref)) if ref else {}
            if element_id not in marks and element_id not in reference_marks:
                generated = still.with_name(f"{file}.{element_id}.boxes.json")
                marks.update(boxes(plan, shot, generated))
            box, reference_box = marks.get(element_id), reference_marks.get(element_id)
            try:
                with Image.open(still) as image:
                    frame_size = image.size
            except (OSError, ValueError):
                frame_size = (1, 1)
            defect_class = _defect_class(box, reference_box, frame_size)
            crop_dir = report_dir or stills
            crop_dir.mkdir(parents=True, exist_ok=True)
            crop_box = box or reference_box
            still_crop = _write_crop(still, crop_box, crop_dir / f"{element_id}-still.png")
            reference_crop = _write_crop(ref, crop_box, crop_dir / f"{element_id}-ref.png") if ref else None
            text = (f"In {source_file} (scene {scene}), element {element_id} is {defect_class}: "
                    f"{defect.get('evidence', 'it differs from the approved reference')}.\n"
                    f"Stage shot {shot}, cue {cue}.\nBox: {json.dumps(box)}\nReference box: {json.dumps(reference_box)}\n"
                    f"Still crop: {still_crop or '(unavailable)'}\nReference crop: {reference_crop or '(unavailable)'}\n"
                    f"Reference still: {ref if ref else '(none)'}\n"
                    f"Current still: {still}\n"
                    "Change only what is needed, then preserve declared element ids.")
            out.append({"scene": scene, "shot": shot, "cue": cue, "file": file, "source_file": source_file,
                        "element_id": element_id, "box": box, "reference_box": reference_box,
                        "defect_class": defect_class, "still_crop": still_crop, "reference_crop": reference_crop,
                        "defect": defect.get("evidence", ""), "reference": str(ref) if ref else None,
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
        sidecar = _sidecar_path(source)
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
    briefs = repair_briefs(json.loads(critic_report.read_text()), stills, plan, boxes_dir, reference_dir, boxes,
                           report_dir=critic_report.parent)
    records = []
    for brief in briefs:
        if scenes is not None and brief["scene"] not in scenes:
            continue
        key = (brief["scene"], brief["cue"], brief["element_id"]); prior = attempts.get(key, 0)
        record = {k: brief[k] for k in ("scene", "shot", "cue", "file", "source_file", "element_id", "box",
                                        "reference_box", "defect_class", "still_crop", "reference_crop", "defect",
                                        "reference")}
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
