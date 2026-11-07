"""Grounded VLM criticism of rendered Stage stills.

The main environment remains MLX-free; :mod:`scriptcast.critic.runner` is
executed with ``.venv-critic`` only when actual model inference is requested.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Callable

from scriptcast.critic.grounding import ground_defects
from scriptcast.critic.s1 import judge_frame

DEFAULT_MODEL = "mlx-community/MiniCPM-V-4.7-35B-A3B-4bit"
ROOT = Path(__file__).resolve().parents[2]
PROMPT = (ROOT / "scriptcast" / "critic" / "prompt.md").read_text()


def _json_object(raw: str) -> dict | None:
    match = re.search(r"\{.*\}", raw, re.S)
    if not match:
        return None
    try:
        value = json.loads(match.group(0))
    except json.JSONDecodeError:
        return None
    return value if isinstance(value, dict) else None


def validate_reply(value: object) -> dict | None:
    """Small dependency-free implementation of the public critic contract."""
    if not isinstance(value, dict) or set(value) != {"verdict", "defects"}:
        return None
    if value["verdict"] not in ("pass", "fail") or not isinstance(value["defects"], list):
        return None
    clean = {"verdict": value["verdict"], "defects": []}
    for defect in value["defects"]:
        if not isinstance(defect, dict) or set(defect) != {"category", "element_id", "evidence", "severity", "region"}:
            return None
        if defect["category"] not in {"Layout", "Rendering", "Appearance", "State", "Content"}:
            return None
        if defect["element_id"] is not None and not isinstance(defect["element_id"], str):
            return None
        if not isinstance(defect["evidence"], str) or not defect["evidence"]:
            return None
        if defect["severity"] not in {"low", "medium", "high"}:
            return None
        region = defect["region"]
        if not (isinstance(region, list) and len(region) == 4 and all(isinstance(x, (int, float)) and 0 <= x <= 1 for x in region)):
            return None
        clean["defects"].append(dict(defect, region=[float(x) for x in region]))
    return clean


def _box(raw: dict, width: float, height: float) -> list[float] | None:
    value = raw.get("box", raw.get("region"))
    if isinstance(value, dict) and all(k in value for k in ("x", "y", "w", "h")):
        value = [value["x"], value["y"], value["w"], value["h"]]
    if value is None and all(k in raw for k in ("x", "y", "w", "h")):
        value = [raw["x"], raw["y"], raw["w"], raw["h"]]
    if not isinstance(value, (list, tuple)) or len(value) != 4:
        return None
    try:
        box = [float(x) for x in value]
    except (TypeError, ValueError):
        return None
    # Stage layout boxes are normally pixels; preserve explicitly normalised ones.
    if max(abs(x) for x in box) > 1:
        box = [box[0] / width, box[1] / height, box[2] / width, box[3] / height]
    return box if all(0 <= x <= 1 for x in box) else None


def declared_elements(plan: dict, scene: str, cue: str) -> dict[str, list[float]]:
    """Read common plan declarations, accepting either frame-local or global ids."""
    width, height = float(plan.get("width", 1920)), float(plan.get("height", 1080))
    frames = plan.get("frames", plan.get("critic_frames", []))
    candidates = [f for f in frames if f.get("scene") == scene and str(f.get("cue")) == str(cue)] if isinstance(frames, list) else []
    source = candidates[0].get("elements", []) if candidates else plan.get("elements", [])
    result = {}
    if isinstance(source, dict):
        source = [dict(v, id=k) if isinstance(v, dict) else {"id": k, "box": v} for k, v in source.items()]
    for item in source if isinstance(source, list) else []:
        if isinstance(item, dict) and isinstance(item.get("id"), str):
            box = _box(item, width, height)
            if box:
                result[item["id"]] = box
    return result


def grid_elements() -> dict[str, list[float]]:
    """A stable whole-frame fallback for plans which declare no elements."""
    return {f"grid_{row}_{column}": [column / 3, row / 3, 1 / 3, 1 / 3]
            for row in range(3) for column in range(3)}


def marked_elements(path: Path, plan: dict) -> dict[str, list[float]] | None:
    """Read a StageMarks sidecar, returning ``None`` only when it is absent.

    A valid empty mark list is meaningful: it says the actual frame has no
    tagged elements, and must not quietly turn into the synthetic 3x3 grid.
    """
    if not path.is_file():
        return None
    try:
        raw = json.loads(path.read_text())
    except (OSError, ValueError, json.JSONDecodeError):
        return None
    marks = raw.get("marks", raw) if isinstance(raw, dict) else raw
    if isinstance(marks, dict):
        marks = marks.get("marks", [])
    if not isinstance(marks, list):
        return None
    width, height = float(plan.get("width", 1920)), float(plan.get("height", 1080))
    result = {}
    for mark in marks:
        if isinstance(mark, dict) and isinstance(mark.get("id"), str):
            box = _box(mark, width, height)
            if box:
                result[mark["id"]] = box
    return result


def _sidecar_path(path: Path, boxes_dir: Path | None = None) -> Path:
    """Resolve both documented StageMarks sidecar naming conventions."""
    base = boxes_dir if boxes_dir is not None else path.parent
    exact = base / f"{path.name}.boxes.json"
    return exact if exact.is_file() else base / f"{path.stem}.boxes.json"


def default_s1_threshold() -> float:
    """Read the S1 cutoff without making its training data part of this tool."""
    weights = Path(__file__).resolve().parents[1] / "critic" / "s1_weights.json"
    try:
        return float(json.loads(weights.read_text()).get("threshold", .5))
    except (OSError, ValueError, json.JSONDecodeError):
        return .5


def _s1_boxes(elements: dict[str, list[float]]) -> list[dict[str, float | str]]:
    return [{"element_id": ident, "x": box[0], "y": box[1], "w": box[2], "h": box[3]}
            for ident, box in elements.items()]


def scene_spec(plan: dict, scene: str, cue: str) -> str:
    frames = plan.get("frames", plan.get("critic_frames", []))
    if isinstance(frames, list):
        for frame in frames:
            if frame.get("scene") == scene and str(frame.get("cue")) == str(cue):
                return str(frame.get("storyboard", frame.get("spec", "(not supplied)")))
    return str(plan.get("storyboard", plan.get("scene_spec", "(not supplied)")))


def _ocr(path: Path) -> str:
    # Existing Stage OCR is used only after a model makes an anonymous Content claim.
    from scriptcast.tools.stage_verify import TesseractBackend, load_frame
    return " ".join(word.text for word in TesseractBackend().read(load_frame(path)))


def run_runner(request: dict) -> dict:
    python = ROOT / ".venv-critic" / "bin" / "python"
    proc = subprocess.run([str(python), str(ROOT / "scriptcast" / "critic" / "runner.py")], input=json.dumps(request), text=True, capture_output=True)
    if proc.returncode:
        raise RuntimeError(f"critic runner failed ({proc.returncode}): {proc.stderr.strip()}")
    return json.loads(proc.stdout)


def critic_stills(stills: Path, plan_path: Path, model: str = DEFAULT_MODEL, reference_dir: Path | None = None,
                  only_passed: Path | None = None, runner: Callable[[dict], dict] = run_runner,
                  ocr: Callable[[Path], str] = _ocr, s1_threshold: float | None = None,
                  tier1_only: bool = False, boxes_dir: Path | None = None) -> dict:
    plan = json.loads(Path(plan_path).read_text())
    allowed = None
    output_path = Path(stills) / "critic.json"
    if only_passed:
        verify = json.loads(Path(only_passed).read_text())
        allowed = {str(x.get("file")) for x in verify.get("frames", []) if x.get("pass") is True}
        output_path = Path(only_passed).parent / "critic.json"
    images = sorted(p for p in Path(stills).iterdir() if p.suffix.lower() in {".png", ".jpg", ".jpeg", ".webp"})
    threshold = default_s1_threshold() if s1_threshold is None else s1_threshold
    items, metadata, tier1_by_file = [], {}, {}
    for image in images:
        if allowed is not None and image.name not in allowed:
            continue
        # Stage-stills names are Scene_cue.png. Split only at the final underscore.
        stem = image.stem
        scene, sep, cue = stem.rpartition("_")
        if not sep:
            scene, cue = stem, "end"
        sidecar = _sidecar_path(image, boxes_dir)
        elements = marked_elements(sidecar, plan)
        if elements is None:
            elements = declared_elements(plan, scene, cue)
        if not elements and not sidecar.is_file():
            elements = grid_elements()
        # ``prompt.md`` deliberately contains a literal JSON example, so avoid
        # ``str.format`` interpreting its braces as replacement fields.
        prompt = (PROMPT.replace("{scene_spec}", scene_spec(plan, scene, cue))
                        .replace("{elements}", json.dumps(sorted(elements))))
        reference = reference_dir / image.name if reference_dir and (reference_dir / image.name).is_file() else None
        tier1 = {"elements": [], "flagged": False, "ms": 0.0}
        if reference:
            started = time.perf_counter()
            judged = judge_frame(reference, image, _s1_boxes(elements), threshold=threshold)
            tier1 = {"elements": [{"id": item["element_id"], "p_defect": item["p_defect"]}
                                 for item in judged["elements"]],
                     "flagged": judged["verdict"] == "fail",
                     "ms": (time.perf_counter() - started) * 1000}
        flagged = {item["id"]: elements[item["id"]] for item in tier1["elements"]
                   if item["p_defect"] >= threshold}
        if not tier1_only and (reference is None or tier1["flagged"]):
            region_note = json.dumps(flagged) if flagged else "the full frame (no reference still is available)"
            items.append({"file": image.name, "still": str(image), "reference": str(reference) if reference else None,
                          "prompt": f"{prompt}\n\nCheck these regions: {region_note}."})
        metadata[image.name] = (image, elements, reference)
        tier1_by_file[image.name] = tier1
    answer = runner({"model": model, "items": items}) if items else {"results": []}
    raw_by_file = {str(x.get("file")): x.get("raw", "") for x in answer.get("results", [])}
    records = []
    for image in images:
        if allowed is not None and image.name not in allowed:
            continue
        tier1 = tier1_by_file[image.name]
        image_path, elements, reference = metadata[image.name]
        if image.name not in raw_by_file:
            # A reference-backed S1 pass needs no slow-model confirmation.
            s1_verdict = "fail" if tier1["flagged"] else "pass"
            records.append({"file": image.name, "status": "s1_only" if tier1_only else "ok", "verdict": s1_verdict,
                            "defects": [], "ungrounded": 0, "tier1": tier1, "tier2_ran": False})
            continue
        raw = raw_by_file.get(image.name, "")
        item = next(x for x in items if x["file"] == image.name)
        parsed = validate_reply(_json_object(raw))
        if parsed is None:
            # One retry for a schema failure, as required by the contract.
            retry = runner({"model": model, "items": [item]}).get("results", [])
            raw = retry[0].get("raw", "") if retry else ""
            parsed = validate_reply(_json_object(raw))
            if parsed is None:
                if reference:
                    records.append({"file": item["file"], "status": "s1_only",
                                    "verdict": "fail" if tier1["flagged"] else "pass", "defects": [],
                                    "ungrounded": 0, "tier1": tier1, "tier2_ran": True, "raw": raw})
                else:
                    records.append({"file": item["file"], "status": "contract_error", "raw": raw,
                                    "tier1": tier1, "tier2_ran": True})
                continue
        image = image_path
        needs_ocr = any(d["element_id"] is None and d["category"] == "Content" for d in parsed["defects"])
        kept, ungrounded = ground_defects(
            parsed["defects"], elements, ocr(image) if needs_ocr else "", reference, image,
            {entry["id"] for entry in tier1["elements"] if entry["p_defect"] >= threshold},
        )
        records.append({"file": item["file"], "status": "ok", "verdict": "fail" if kept else "pass",
                        "defects": kept, "ungrounded": ungrounded, "tier1": tier1, "tier2_ran": True})
    report = {"model": model, "s1_threshold": threshold, "frames": records}
    output_path.write_text(json.dumps(report, indent=2) + "\n")
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description="Grounded VLM critic for Stage stills")
    parser.add_argument("--stills", type=Path, required=True)
    parser.add_argument("--plan", type=Path, required=True, help="stage_plan.json")
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--reference-dir", type=Path)
    parser.add_argument("--only-passed", type=Path, metavar="verify.json")
    parser.add_argument("--s1-threshold", type=float, help="S1 defect cutoff (defaults to s1_weights.json)")
    parser.add_argument("--tier1-only", action="store_true", help="skip MiniCPM after System 1")
    parser.add_argument("--boxes-dir", type=Path, help="directory containing <still>.boxes.json StageMarks sidecars")
    args = parser.parse_args()
    report = critic_stills(args.stills, args.plan, args.model, args.reference_dir, args.only_passed,
                           s1_threshold=args.s1_threshold, tier1_only=args.tier1_only, boxes_dir=args.boxes_dir)
    print(f"wrote {(args.only_passed.parent if args.only_passed else args.stills) / 'critic.json'} ({len(report['frames'])} stills)")


if __name__ == "__main__":
    main()
