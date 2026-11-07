#!/usr/bin/env python3
"""Collect *only documented* R3 Stage-critic labels into a CSV manifest.

The current YT02 Stage tree has four author-approved dashboard handoff frames
and byte-identical logged renders. Rejections recorded by the Stage UI or CLI
are explicit bad labels. This collector deliberately does not turn names such
as ``fix1`` or ``review`` into labels: a calibration set with guessed negatives
is worse than an incomplete one.
"""
from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path
from typing import Iterable

IMAGE_SUFFIXES = {".png", ".jpg", ".jpeg", ".webp"}
FIELDS = ("still_path", "reference_path", "label", "shot", "source")


def shot_name(path: Path) -> str:
    """Stable grouping key when an explicit shot id was not recorded."""
    return path.stem


def approved_dashboard_rows(stage: Path) -> list[dict[str, str]]:
    """Return the documented author-approved handoff frames and logged renders."""
    reference_dir = stage / "handoff" / "png"
    rerender_dir = stage.parent / "logs" / "stage_dashboard"
    readme = stage / "handoff" / "README.md"
    if not (reference_dir.is_dir() and rerender_dir.is_dir() and readme.is_file()):
        return []
    # The README explicitly calls these references "author-approved".  Require
    # matching logged filenames, rather than treating arbitrary log images as good.
    rows: list[dict[str, str]] = []
    for reference in sorted(reference_dir.iterdir()):
        if reference.suffix.lower() not in IMAGE_SUFFIXES:
            continue
        rerender = rerender_dir / reference.name
        if not rerender.is_file():
            continue
        common = {"reference_path": str(reference), "label": "good", "shot": shot_name(reference)}
        rows.append({"still_path": str(reference), **common, "source": "stage/handoff/README.md: author-approved reference"})
        rows.append({"still_path": str(rerender), **common, "source": "logs/stage_dashboard: filename-paired genuine re-render"})
    return rows


def _records(value: object) -> Iterable[dict]:
    """Yield dict records from common label-file wrappers without guessing fields."""
    if isinstance(value, list):
        for item in value:
            yield from _records(item)
    elif isinstance(value, dict):
        if any(key in value for key in ("label", "verdict", "decision", "status")):
            yield value
        for item in value.values():
            if isinstance(item, (dict, list)):
                yield from _records(item)


def explicit_label_rows(stage: Path) -> list[dict[str, str]]:
    """Read explicit bad/good records if later review exports add them.

    Accepted labels are exact values, and the still path must exist.  This is
    intentionally conservative so prose JSON and layout data cannot leak in.
    """
    rows: list[dict[str, str]] = []
    for labels in stage.rglob("*.json"):
        if "site-dist" in labels.parts:
            continue
        try:
            payload = json.loads(labels.read_text())
        except (OSError, ValueError):
            continue
        for record in _records(payload):
            raw_label = next((record.get(k) for k in ("label", "verdict", "decision", "status") if k in record), None)
            label = str(raw_label).lower()
            if label in {"pass", "approved", "accept", "accepted", "good"}:
                label = "good"
            elif label in {"fail", "rejected", "reject", "bad"}:
                label = "bad"
            else:
                continue
            raw_path = next((record.get(k) for k in ("still_path", "still", "file", "path", "image") if record.get(k)), None)
            if not isinstance(raw_path, str):
                continue
            still = Path(raw_path)
            if not still.is_absolute():
                still = labels.parent / still
            if not still.is_file() or still.suffix.lower() not in IMAGE_SUFFIXES:
                continue
            raw_ref = record.get("reference_path") or record.get("reference")
            reference = Path(raw_ref) if isinstance(raw_ref, str) else still
            if not reference.is_absolute():
                reference = labels.parent / reference
            if not reference.is_file():
                reference = still
            rows.append({"still_path": str(still), "reference_path": str(reference), "label": label,
                         "shot": str(record.get("shot") or shot_name(still)), "source": str(labels)})
    return rows


def rejection_rows(stage: Path) -> list[dict[str, str]]:
    """Read append-only Stage still rejections as documented bad labels."""
    labels = stage / "review" / "rejections.jsonl"
    if not labels.is_file():
        return []
    rows: list[dict[str, str]] = []
    for line_number, raw in enumerate(labels.read_text(encoding="utf-8").splitlines(), 1):
        if not raw.strip():
            continue
        try:
            record = json.loads(raw)
        except ValueError:
            continue
        if not isinstance(record, dict):
            continue
        raw_path = record.get("still_path")
        if not isinstance(raw_path, str) or not raw_path:
            continue
        still = Path(raw_path)
        if not still.is_absolute():
            still = (labels.parent / still).resolve()
        if not still.is_file() or still.suffix.lower() not in IMAGE_SUFFIXES:
            continue
        rows.append({
            "still_path": str(still), "reference_path": str(still), "label": "bad",
            "shot": str(record.get("shot") or shot_name(still)),
            "source": f"{labels}:{line_number}",
        })
    return rows


def deduplicate(rows: Iterable[dict[str, str]]) -> list[dict[str, str]]:
    """Keep first provenance for a labelled still; disagreeing labels are unsafe."""
    result: dict[str, dict[str, str]] = {}
    for row in rows:
        key = row["still_path"]
        if key not in result:
            result[key] = row
        elif result[key]["label"] != row["label"]:
            raise ValueError(f"conflicting labels for {key}: {result[key]['label']} vs {row['label']}")
    return sorted(result.values(), key=lambda row: (row["label"], row["shot"], row["still_path"]))


def write_manifest(rows: Iterable[dict[str, str]], output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=FIELDS)
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stage", type=Path, default=Path("/Users/emmanuel/Documents/Theory/TheOriginalPower/Architecting_the_operation/video/yt02/stage"))
    parser.add_argument("--output", type=Path, default=Path(__file__).with_name("manifest.csv"))
    args = parser.parse_args()
    rows = deduplicate([
        *approved_dashboard_rows(args.stage), *explicit_label_rows(args.stage),
        *rejection_rows(args.stage),
    ])
    write_manifest(rows, args.output)
    bad = sum(row["label"] == "bad" for row in rows)
    print(f"wrote {args.output} ({len(rows)} rows: {len(rows) - bad} good, {bad} bad)")
    if bad < 20:
        print("WARNING: fewer than 20 documented bad stills; no inferred negatives were added.")


if __name__ == "__main__":
    main()
