"""Cue sheet and FilmCraft timeline for an episode that is already rendered.

Picture times stay on the manifest clock. This command does not call a Craft binary.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import sys
import tempfile

from scriptcast.fcpxml_generator import generate_fcpxml
from scriptcast.tools.captions import render_srt, render_webvtt
from scriptcast.tools.export_session import _load_segments

CARD_FPS = 30
CARD_WIDTH = 1080
CARD_HEIGHT = 1920
KINDS = {"still", "motion", "page"}


def _ms(value: object, label: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise ValueError(f"{label} must be a nonnegative integer")
    return value


def _read_json(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise ValueError(f"Cannot read {path}: {exc}") from exc
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"Invalid JSON {path}: {exc}") from exc


def manifest_duration_ms(manifest: dict) -> tuple[int, list[tuple[int, int]]]:
    """Return the episode duration and each turn window.

    Duration follows video_plan.py: duration_ms, else total_duration_ms, else
    the latest turn end. A duration that ends before that turn is rejected.
    """
    if not isinstance(manifest, dict):
        raise ValueError("Manifest must be an object")
    turns = manifest.get("turns")
    if not isinstance(turns, list) or not turns:
        raise ValueError("Manifest has no turns")
    indexes: list[int] = []
    windows: list[tuple[int, int]] = []
    for i, turn in enumerate(turns):
        if not isinstance(turn, dict):
            raise ValueError(f"turns[{i}] must be an object")
        context = f"turns[{i}]"
        turn_index = turn.get("turn_index")
        if isinstance(turn_index, bool) or not isinstance(turn_index, int):
            raise ValueError(f"{context}: missing turn_index")
        indexes.append(turn_index)
        start = _ms(turn.get("start_ms"), f"{context} start")
        end = _ms(turn.get("end_ms"), f"{context} end")
        if end <= start:
            raise ValueError(f"Invalid window for turn {turn_index}")
        windows.append((start, end))
    if len(set(indexes)) != len(indexes):
        raise ValueError("Duplicate manifest turn_index")
    last = max(end for _, end in windows)
    if "duration_ms" in manifest:
        duration = _ms(manifest["duration_ms"], "duration_ms")
    elif "total_duration_ms" in manifest:
        duration = _ms(manifest["total_duration_ms"], "total_duration_ms")
    else:
        duration = last
    if duration < last:
        raise ValueError("Manifest duration ends before its turns")
    return duration, windows


def shorten_overlaps(cues: list[dict]) -> list[dict]:
    """Shorten an earlier cue so it ends where the next cue starts.

    Same pairwise rule as card windows in video_plan.py lines 114–117.
    """
    ordered = [dict(cue) for cue in sorted(cues, key=lambda cue: cue["start_ms"])]
    for earlier, later in zip(ordered, ordered[1:]):
        if earlier["end_ms"] > later["start_ms"]:
            earlier["end_ms"] = later["start_ms"]
    return ordered


def build_cue_sheet(manifest: dict) -> dict:
    """One placeholder cue per turn. src stays empty for the operator to fill."""
    duration, windows = manifest_duration_ms(manifest)
    cues = [
        {"start_ms": start, "end_ms": end, "src": "", "kind": "still"}
        for start, end in windows
    ]
    return {
        "fps": CARD_FPS,
        "width": CARD_WIDTH,
        "height": CARD_HEIGHT,
        "duration_ms": duration,
        "cues": shorten_overlaps(cues),
    }


def _cue(raw: object, index: int) -> dict:
    if not isinstance(raw, dict):
        raise ValueError(f"cues[{index}] must be an object")
    label = f"cues[{index}]"
    start = _ms(raw.get("start_ms"), f"{label} start")
    end = _ms(raw.get("end_ms"), f"{label} end")
    if end <= start:
        raise ValueError(f"{label}: end_ms must be after start_ms")
    kind = raw.get("kind")
    if kind not in KINDS:
        raise ValueError(f"{label}.kind must be still, motion, or page")
    src = raw.get("src")
    if not isinstance(src, str):
        raise ValueError(f"{label}.src must be a string")
    src = src.strip()
    if src and not Path(src).is_absolute():
        raise ValueError(f"{label}.src must be an absolute path")
    return {"start_ms": start, "end_ms": end, "src": src, "kind": kind}


def load_cue_sheet(path: Path) -> dict:
    raw = _read_json(path)
    if not isinstance(raw, dict):
        raise ValueError(f"Invalid cue sheet {path}: expected an object")
    for key in ("fps", "width", "height", "duration_ms"):
        value = _ms(raw.get(key), key)
        if value <= 0:
            raise ValueError(f"{key} must be positive")
        raw[key] = value
    cues = raw.get("cues")
    if not isinstance(cues, list):
        raise ValueError("cues must be a list")
    raw["cues"] = shorten_overlaps([_cue(cue, i) for i, cue in enumerate(cues)])
    return raw


def load_captions(path: Path) -> list[dict]:
    raw = _read_json(path)
    if not isinstance(raw, list):
        raise ValueError(f"Invalid captions {path}: expected a list")
    captions = []
    for i, item in enumerate(raw):
        if not isinstance(item, dict):
            raise ValueError(f"caption {i} must be an object")
        text = item.get("text")
        if not isinstance(text, str) or not text.strip():
            raise ValueError(f"caption {i} has no text")
        start = _ms(item.get("start_ms"), f"caption {i} start")
        end = _ms(item.get("end_ms"), f"caption {i} end")
        if end <= start:
            raise ValueError(f"caption {i}: end_ms must be after start_ms")
        captions.append({"start_ms": start, "end_ms": end, "text": text})
    return captions


def _atomic_bytes(path: Path, data: bytes) -> None:
    if not path.parent.is_dir():
        raise ValueError(f"Output directory does not exist: {path.parent}")
    with tempfile.NamedTemporaryFile(
        dir=path.parent, prefix=f".{path.name}.", suffix=".tmp", delete=False,
    ) as temporary:
        temporary_path = Path(temporary.name)
        temporary.write(data)
    try:
        os.replace(temporary_path, path)
    finally:
        temporary_path.unlink(missing_ok=True)


def write_cue(episode_dir: Path, output_path: Path) -> Path:
    manifest_path = episode_dir / "episode_manifest.json"
    manifest = _read_json(manifest_path)
    sheet = build_cue_sheet(manifest)
    output_path = output_path.absolute()
    if output_path.resolve() == manifest_path.resolve():
        raise ValueError(f"Output would overwrite an input file: {output_path}")
    _atomic_bytes(output_path, (json.dumps(sheet, indent=2) + "\n").encode("utf-8"))
    return output_path


def _pictures(cues: list[dict]) -> list[dict]:
    pictures = []
    for cue in cues:
        if not cue["src"] or cue["end_ms"] <= cue["start_ms"]:
            continue
        src = Path(cue["src"])
        if not src.is_file():
            raise ValueError(f"Missing picture file: {src}")
        pictures.append(cue)
    return pictures


def write_timeline(
    episode_dir: Path,
    cue_path: Path,
    captions_path: Path,
    output_path: Path,
) -> Path:
    episode_dir = episode_dir.resolve()
    episode_id, segments, positions = _load_segments(episode_dir)
    sheet = load_cue_sheet(cue_path)
    captions = load_captions(captions_path)
    output_path = output_path.absolute()
    srt_path = output_path.with_suffix(".srt")
    vtt_path = output_path.with_suffix(".vtt")
    protected = {episode_dir / "episode_manifest.json", cue_path.resolve(), captions_path.resolve()}
    protected.update(segment.wav_path.resolve() for segment in segments)
    for path in (output_path, srt_path, vtt_path):
        if path.resolve() in protected:
            raise ValueError(f"Output would overwrite an input file: {path}")
    pictures = _pictures(sheet["cues"])
    if not output_path.parent.is_dir():
        raise ValueError(f"Output directory does not exist: {output_path.parent}")
    with tempfile.NamedTemporaryFile(
        dir=output_path.parent, prefix=f".{output_path.name}.", suffix=".tmp", delete=False,
    ) as temporary:
        temporary_path = Path(temporary.name)
    try:
        generate_fcpxml(
            segments,
            temporary_path,
            position_map=positions,
            episode_id=episode_id,
            width=sheet["width"],
            height=sheet["height"],
            fps=sheet["fps"],
            pictures=pictures,
        )
        os.replace(temporary_path, output_path)
    finally:
        temporary_path.unlink(missing_ok=True)
    _atomic_bytes(srt_path, render_srt(captions).encode("utf-8"))
    _atomic_bytes(vtt_path, render_webvtt(captions).encode("utf-8"))
    return output_path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="scriptcast-craft",
        description="Write a Craft cue sheet, or an FCPXML timeline with caption sidecars.",
    )
    sub = parser.add_subparsers(dest="command", required=True)
    cue = sub.add_parser("cue", help="write a cue sheet from the episode manifest")
    cue.add_argument("episode_dir", type=Path)
    cue.add_argument("--out", type=Path, required=True)
    timeline = sub.add_parser("timeline", help="write FCPXML plus SRT and WebVTT")
    timeline.add_argument("episode_dir", type=Path)
    timeline.add_argument("--cue", type=Path, required=True)
    timeline.add_argument("--captions", type=Path, required=True)
    timeline.add_argument("--out", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        if args.command == "cue":
            output = write_cue(args.episode_dir, args.out)
        else:
            output = write_timeline(args.episode_dir, args.cue, args.captions, args.out)
    except (OSError, ValueError, RuntimeError, UnicodeError) as exc:
        print(f"scriptcast-craft: {exc}", file=sys.stderr)
        return 1
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
