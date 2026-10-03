"""Export an already-rendered episode using its manifest and WAV headers.

Clip times are rounded to the nearest 1/30 s frame (FCPXML timeline), at most 16.7 ms from the manifest.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import sys
import tempfile

import soundfile

from scriptcast.fcpxml_generator import generate_fcpxml
from scriptcast.models import SegmentResult


_TIMING_HELP = (
    "Clip times are rounded to the nearest 1/30 s frame (FCPXML timeline), "
    "at most 16.7 ms from the manifest."
)


def _field(obj: object, key: str, kind: type, context: str):
    if not isinstance(obj, dict) or key not in obj:
        raise ValueError(f"{context}: missing {key}")
    value = obj[key]
    if type(value) is not kind:
        raise ValueError(f"{context}.{key}: expected {kind.__name__}")
    if kind is int and value < 0:
        raise ValueError(f"{context}.{key}: must be nonnegative")
    if kind is str and (not value.strip() or "\0" in value):
        raise ValueError(f"{context}.{key}: expected a nonempty string without NUL")
    return value


def _load_segments(
    episode_dir: Path,
) -> tuple[str, list[SegmentResult], dict[tuple[str, int], int]]:
    manifest_path = episode_dir / "episode_manifest.json"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise ValueError(f"Cannot read manifest {manifest_path}: {exc}") from exc
    except (ValueError, UnicodeError) as exc:
        raise ValueError(f"Invalid manifest {manifest_path}: {exc}") from exc

    segments: list[SegmentResult] = []
    positions: dict[tuple[str, int], int] = {}
    try:
        episode_id = _field(manifest, "episode_id", str, "manifest")
        if episode_id in {".", ".."} or Path(episode_id).name != episode_id:
            raise ValueError("episode_id must be a single filename component")
        sample_rate = _field(manifest, "sample_rate", int, "manifest")
        if sample_rate == 0:
            raise ValueError("sample_rate must be positive")
        for i, turn in enumerate(_field(manifest, "turns", list, "manifest")):
            context = f"turns[{i}]"
            turn_index = _field(turn, "turn_index", int, context)
            turn_id = _field(turn, "turn_id", str, context)
            speaker_id = _field(turn, "speaker_id", str, context)
            for j, entry in enumerate(_field(turn, "segments", list, context)):
                location = f"{context}.segments[{j}]"
                chunk_index = _field(entry, "chunk_index", int, location)
                relative_path = Path(_field(entry, "segment_wav", str, location))
                wav_path = episode_dir / relative_path
                if relative_path.is_absolute() or not wav_path.resolve().is_relative_to(episode_dir):
                    raise ValueError(f"{location}.segment_wav must stay inside the episode directory")
                key = (turn_id, chunk_index)
                if key in positions:
                    raise ValueError(f"{location}: duplicate segment key {key}")
                positions[key] = _field(entry, "start_ms", int, location)
                segments.append(SegmentResult(
                    turn_index=turn_index,
                    turn_id=turn_id,
                    chunk_index=chunk_index,
                    speaker_id=speaker_id,
                    wav_path=wav_path,
                    duration_ms=_field(entry, "duration_ms", int, location),
                    speech_duration_ms=_field(entry, "speech_duration_ms", int, location),
                    sample_rate=sample_rate,
                    gap_after_ms=_field(entry, "gap_after_ms", int, location),
                    checksum=_field(entry, "checksum", str, location),
                ))
    except ValueError as exc:
        raise ValueError(f"Invalid manifest {manifest_path}: {exc}") from exc

    missing = [segment.wav_path for segment in segments if not segment.wav_path.is_file()]
    if missing:
        raise ValueError("Missing WAV files:\n" + "\n".join(str(path) for path in missing))
    for segment in segments:
        try:
            segment.audio_channels = soundfile.info(segment.wav_path).channels
        except (OSError, RuntimeError) as exc:
            raise ValueError(f"Cannot read WAV header {segment.wav_path}: {exc}") from exc
    return episode_id, segments, positions


def export_fcpxml(episode_dir: Path, output_path: Path | None = None) -> Path:
    """Validate all inputs, then atomically publish one FCPXML file."""
    episode_dir = episode_dir.resolve()
    episode_id, segments, positions = _load_segments(episode_dir)
    output_path = (output_path if output_path is not None else episode_dir / f"{episode_id}.fcpxml").absolute()
    input_paths = {episode_dir / "episode_manifest.json"}
    input_paths.update(segment.wav_path.resolve() for segment in segments)
    if output_path.resolve() in input_paths:
        raise ValueError(f"Output would overwrite an input file: {output_path}")

    # The destination directory must exist; this command creates only its output.
    with tempfile.NamedTemporaryFile(
        dir=output_path.parent, prefix=f".{output_path.name}.", suffix=".tmp", delete=False,
    ) as temporary:
        temporary_path = Path(temporary.name)
    try:
        generate_fcpxml(segments, temporary_path, position_map=positions, episode_id=episode_id)
        os.replace(temporary_path, output_path)
    finally:
        temporary_path.unlink(missing_ok=True)
    return output_path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Export an already-rendered episode without synthesis.",
        epilog=_TIMING_HELP,
    )
    parser.add_argument("episode_dir", type=Path)
    formats = parser.add_mutually_exclusive_group(required=True)
    formats.add_argument("--fcpxml", dest="format", action="store_const", const="fcpxml",
                         help="write a Final Cut Pro XML session")
    parser.add_argument("--out", type=Path, help="output path (default: EPISODE_DIR/EPISODE_ID.fcpxml)")
    args = parser.parse_args(argv)
    try:
        output_path = export_fcpxml(args.episode_dir, args.out)
    except (OSError, ValueError, RuntimeError) as exc:
        print(f"scriptcast-export: {exc}", file=sys.stderr)
        return 1
    print(output_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
