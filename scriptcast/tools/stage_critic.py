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
from pathlib import Path
from typing import Callable

from scriptcast.critic.grounding import ground_defects

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
                  ocr: Callable[[Path], str] = _ocr) -> dict:
    plan = json.loads(Path(plan_path).read_text())
    allowed = None
    output_path = Path(stills) / "critic.json"
    if only_passed:
        verify = json.loads(Path(only_passed).read_text())
        allowed = {str(x.get("file")) for x in verify.get("frames", []) if x.get("pass") is True}
        output_path = Path(only_passed).parent / "critic.json"
    images = sorted(p for p in Path(stills).iterdir() if p.suffix.lower() in {".png", ".jpg", ".jpeg", ".webp"})
    items, metadata = [], {}
    for image in images:
        if allowed is not None and image.name not in allowed:
            continue
        # Stage-stills names are Scene_cue.png. Split only at the final underscore.
        stem = image.stem
        scene, sep, cue = stem.rpartition("_")
        if not sep:
            scene, cue = stem, "end"
        elements = declared_elements(plan, scene, cue)
        # ``prompt.md`` deliberately contains a literal JSON example, so avoid
        # ``str.format`` interpreting its braces as replacement fields.
        prompt = (PROMPT.replace("{scene_spec}", scene_spec(plan, scene, cue))
                        .replace("{elements}", json.dumps(sorted(elements))))
        reference = reference_dir / image.name if reference_dir and (reference_dir / image.name).is_file() else None
        items.append({"file": image.name, "still": str(image), "reference": str(reference) if reference else None, "prompt": prompt})
        metadata[image.name] = (image, elements, reference)
    answer = runner({"model": model, "items": items})
    raw_by_file = {str(x.get("file")): x.get("raw", "") for x in answer.get("results", [])}
    records = []
    for item in items:
        raw = raw_by_file.get(item["file"], "")
        parsed = validate_reply(_json_object(raw))
        if parsed is None:
            # One retry for a schema failure, as required by the contract.
            retry = runner({"model": model, "items": [item]}).get("results", [])
            raw = retry[0].get("raw", "") if retry else ""
            parsed = validate_reply(_json_object(raw))
            if parsed is None:
                records.append({"file": item["file"], "status": "contract_error", "raw": raw})
                continue
        image, elements, reference = metadata[item["file"]]
        needs_ocr = any(d["element_id"] is None and d["category"] == "Content" for d in parsed["defects"])
        kept, ungrounded = ground_defects(
            parsed["defects"], elements, ocr(image) if needs_ocr else "", reference, image,
        )
        records.append({"file": item["file"], "status": "ok", "verdict": "fail" if kept else "pass",
                        "defects": kept, "ungrounded": ungrounded})
    report = {"model": model, "frames": records}
    output_path.write_text(json.dumps(report, indent=2) + "\n")
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description="Grounded VLM critic for Stage stills")
    parser.add_argument("--stills", type=Path, required=True)
    parser.add_argument("--plan", type=Path, required=True, help="stage_plan.json")
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--reference-dir", type=Path)
    parser.add_argument("--only-passed", type=Path, metavar="verify.json")
    args = parser.parse_args()
    report = critic_stills(args.stills, args.plan, args.model, args.reference_dir, args.only_passed)
    print(f"wrote {(args.only_passed.parent if args.only_passed else args.stills) / 'critic.json'} ({len(report['frames'])} stills)")


if __name__ == "__main__":
    main()
