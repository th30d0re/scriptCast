"""Export an already-rendered episode using its manifest and WAV headers.

Clip times are rounded to the nearest 1/30 s frame (FCPXML timeline), at most 16.7 ms from the manifest.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile

import soundfile
from yaml import YAMLError

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


def script_json(episode_dir: Path, script: Path | None, changes: bool) -> dict:
    """Read the project's script and render state without publishing any files."""
    from scriptcast.markup import tokenize_markup
    from scriptcast.parser import parse_transcript
    from scriptcast.project import current
    from scriptcast.render_state import load_render_state_with_migration, plan_precision_insert
    from scriptcast.voices import load_voices

    episode_dir = episode_dir.resolve()
    project = current(episode_dir)
    if script is None:
        state = json.loads((episode_dir / "render_state.json").read_text(encoding="utf-8"))
        script = project.resolve(_field(state, "source_file", str, "render_state"))
    script = script.resolve()
    original = script.read_bytes()
    if b"\r" in original.replace(b"\r\n", b""):
        raise ValueError("Unsupported bare CR line endings; use LF or CRLF")
    # Preserve raw line endings for the editor; parse_transcript owns header recognition.
    text = original.decode("utf-8")
    # Split physical LF lines only: Unicode paragraph separators belong to the body.
    lines = text.split("\n")
    lines = [line + "\n" for line in lines[:-1]] + ([lines[-1]] if lines[-1] else [])
    turns = parse_transcript(script)
    if script.read_bytes() != original:
        raise ValueError("The script changed on disk. Reload to edit.")
    if changes:
        # Fingerprints include speech chunk counts, so match the render CLI's tokenization.
        turns = tokenize_markup(turns)
        previous = load_render_state_with_migration(episode_dir, turns)
        if previous is None:
            raise ValueError(f"Missing render state: {episode_dir / 'render_state.json'}")
        # Fingerprinting also resolves pronunciation and archive data through current().
        # Anchor those lookups at this episode's project, even when invoked elsewhere.
        old_project = os.environ.get("SCRIPTCAST_PROJECT")
        os.environ["SCRIPTCAST_PROJECT"] = str(project.root)
        try:
            plan = plan_precision_insert(turns, previous, load_voices(project.voices))
        finally:
            if old_project is None:
                os.environ.pop("SCRIPTCAST_PROJECT", None)
            else:
                os.environ["SCRIPTCAST_PROJECT"] = old_project
        return {
            "inserted": [turn.turn_index for turn in plan.inserted],
            "modified": [turn.turn_index for turn in plan.modified],
            "deleted": plan.deleted_ids,
            "unchanged": len(plan.unchanged),
        }
    result = []
    for turn in turns:
        header, end = turn.line_span
        start = header
        while start < end and not lines[start].strip():
            start += 1
        while end > start and not lines[end - 1].strip():
            end -= 1
        result.append({
            "index": turn.turn_index,
            "turn_id": turn.turn_id,
            "speaker_id": turn.speaker_id,
            "display_name": turn.display_name,
            "header_line": header,
            "body_start": start + 1,
            "body_end": end,
            "body": "".join(lines[start:end]).removesuffix("\n").removesuffix("\r"),
        })
    return {
        "script_path": str(script),
        "script_sha256": hashlib.sha256(original).hexdigest(),
        "line_ending": "crlf" if b"\r\n" in original else "lf",
        "turns": result,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Export an already-rendered episode without synthesis.",
        epilog=_TIMING_HELP,
    )
    parser.add_argument("episode_dir", type=Path)
    formats = parser.add_mutually_exclusive_group(required=True)
    formats.add_argument("--fcpxml", dest="format", action="store_const", const="fcpxml",
                         help="write a Final Cut Pro XML session")
    for mode in ("turns", "changes"):
        formats.add_argument(f"--{mode}-json", dest="format", action="store_const", const=mode,
                             help=f"print read-only script {mode} JSON")
    parser.add_argument("--script", type=Path, help="override the transcript path for JSON modes")
    parser.add_argument("--out", type=Path, help="output path (default: EPISODE_DIR/EPISODE_ID.fcpxml)")
    args = parser.parse_args(argv)
    try:
        if args.format == "fcpxml":
            if args.script is not None:
                raise ValueError("--script is only supported with JSON modes")
            output = str(export_fcpxml(args.episode_dir, args.out))
        else:
            if args.out is not None:
                raise ValueError("--out is only supported with --fcpxml")
            output = json.dumps(script_json(args.episode_dir, args.script, args.format == "changes"))
    except (OSError, ValueError, RuntimeError, TypeError, KeyError, AttributeError, ImportError, YAMLError) as exc:
        print(f"scriptcast-export: {exc}", file=sys.stderr)
        return 1
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
