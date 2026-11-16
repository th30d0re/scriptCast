#!/usr/bin/env python3
"""Sweep Stage critic S1 thresholds over a labelled R3 manifest, headlessly."""
from __future__ import annotations

import argparse
import csv
import json
import os
import tempfile
import time
from pathlib import Path
from typing import Iterable


def thresholds() -> list[float]:
    return [round(step / 10, 1) for step in range(1, 10)]


def read_manifest(path: Path, limit: int | None = None) -> list[dict[str, str]]:
    with path.open(newline="") as stream:
        rows = list(csv.DictReader(stream))
    required = {"still_path", "reference_path", "label", "shot", "source"}
    if not rows or not required.issubset(rows[0]):
        raise ValueError(f"{path} must contain columns {sorted(required)}")
    for row in rows:
        if row["label"] not in {"good", "bad"}:
            raise ValueError(f"invalid label {row['label']!r} for {row['still_path']}")
        if not Path(row["still_path"]).is_file() or not Path(row["reference_path"]).is_file():
            raise FileNotFoundError(f"missing still or reference for {row['still_path']}")
    return rows[:limit] if limit else rows


def metrics(rows: Iterable[dict[str, object]]) -> dict[str, float | int | None]:
    rows = list(rows)
    bad = [row for row in rows if row["label"] == "bad"]
    good = [row for row in rows if row["label"] == "good"]
    failed_bad = sum(row["verdict"] == "fail" for row in bad)
    failed_good = sum(row["verdict"] == "fail" for row in good)
    return {
        "n": len(rows), "bad_n": len(bad), "good_n": len(good),
        "recall": failed_bad / len(bad) if bad else None,
        "false_fail": failed_good / len(good) if good else None,
        "tier2_call_rate": sum(bool(row["tier2_ran"]) for row in rows) / len(rows) if rows else 0.0,
        "latency_ms_mean": sum(float(row["latency_ms"]) for row in rows) / len(rows) if rows else 0.0,
    }


def mock_runner(request: dict) -> dict:
    """Valid, deterministic Tier-2 response used by --dry-run; never loads MLX."""
    return {"results": [{"file": item["file"], "raw": '{"verdict":"pass","defects":[]}'} for item in request["items"]]}


def judge_one(row: dict[str, str], threshold: float, tier1_only: bool, dry_run: bool) -> dict[str, object]:
    """Stage one manifest row in an isolated directory and call the production API."""
    # Import lazily: collection and pure-metric tests do not require Pillow/NumPy.
    from scriptcast.tools.stage_critic import critic_stills
    with tempfile.TemporaryDirectory(prefix="r3-calibration-") as raw:
        root = Path(raw)
        stills, references = root / "stills", root / "references"
        stills.mkdir(); references.mkdir()
        # One row per directory avoids collisions between reference and rerender
        # files that intentionally have the same canonical filename.
        name = "frame.png"
        os.symlink(Path(row["still_path"]).resolve(), stills / name)
        os.symlink(Path(row["reference_path"]).resolve(), references / name)
        plan = root / "plan.json"
        plan.write_text('{"width":1920,"height":1080,"frames":[]}\n')
        started = time.perf_counter()
        if dry_run:
            report = critic_stills(stills, plan, reference_dir=references, s1_threshold=threshold,
                                   tier1_only=tier1_only, runner=mock_runner)
        else:
            report = critic_stills(stills, plan, reference_dir=references, s1_threshold=threshold,
                                   tier1_only=tier1_only)
        elapsed = (time.perf_counter() - started) * 1000
        frame = report["frames"][0]
    return {"still_path": row["still_path"], "label": row["label"], "shot": row["shot"],
            "verdict": frame.get("verdict", "error"), "tier2_ran": bool(frame.get("tier2_ran")),
            "latency_ms": elapsed, "status": frame.get("status")}


def run_sweep(rows: list[dict[str, str]], dry_run: bool) -> list[dict[str, object]]:
    output: list[dict[str, object]] = []
    for threshold in thresholds():
        for tier1_only in (True, False):
            results = [judge_one(row, threshold, tier1_only, dry_run) for row in rows]
            output.append({"threshold": threshold, "mode": "tier1_only" if tier1_only else "two_tier",
                           **metrics(results), "rows": results})
    return output


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=Path(__file__).with_name("manifest.csv"))
    parser.add_argument("--output", type=Path, default=Path(__file__).with_name("results.json"))
    parser.add_argument("--dry-run", action="store_true", help="mock Tier-2; suitable without a GPU")
    parser.add_argument("--limit", type=int, help="use first N rows (use 2 for the smoke test)")
    args = parser.parse_args()
    rows = read_manifest(args.manifest, args.limit)
    results = run_sweep(rows, args.dry_run)
    args.output.write_text(json.dumps({"dry_run": args.dry_run, "results": results}, indent=2) + "\n")
    print(f"wrote {args.output} ({len(rows)} rows × {len(results)} runs)")


if __name__ == "__main__":
    main()
