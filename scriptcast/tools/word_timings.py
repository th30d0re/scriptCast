"""Turn-relative read-along timing from the manifest's processed WAVs."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import sys
import tempfile

from scriptcast.parser import parse_transcript
from scriptcast.project import current
from scriptcast.tools.captions import DEFAULT_MODEL, _assign_times, _fill_gaps, script_words, transcribe


def turn_selection(value: str) -> set[int]:
    """Parse manifest turn indices, including inclusive ranges."""
    selected = set()
    try:
        for part in value.split(","):
            bounds = [int(n) for n in part.split("-")]
            if len(bounds) not in (1, 2) or min(bounds) < 0 or bounds[-1] < bounds[0]:
                raise ValueError
            selected.update(range(bounds[0], bounds[-1] + 1))
    except ValueError as exc:
        raise argparse.ArgumentTypeError("Expected turn indices such as 5,10-12") from exc
    return selected


def transcribe_with_prompt(
    audio: Path, model: str, *, initial_prompt: str | None = None
) -> list[dict]:
    """Whisper word timings; optional ``initial_prompt`` for vocabulary hints."""
    try:
        import mlx_whisper
    except ImportError as exc:  # pragma: no cover - depends on the environment
        raise SystemExit(
            "scriptcast-word-timings needs mlx-whisper (pip install mlx-whisper)"
        ) from exc
    kwargs = {
        "path_or_hf_repo": model,
        "language": "en",
        "word_timestamps": True,
        "condition_on_previous_text": False,
    }
    if initial_prompt:
        kwargs["initial_prompt"] = initial_prompt
    result = mlx_whisper.transcribe(str(audio), **kwargs)
    return [
        {"word": w["word"].strip(), "start": w["start"], "end": w["end"]}
        for segment in result["segments"]
        for w in segment.get("words", [])
    ]


def build_timings(episode_dir: Path, manifest: dict, script_turns, model: str,
                  turns: set[int] | None = None, *, transcribe_fn=transcribe) -> dict:
    """Align each turn independently; no stitched episode audio is read."""
    by_id = {turn.turn_id: turn for turn in script_turns}
    entries = manifest["turns"]
    missing = (turns or set()) - {turn["turn_index"] for turn in entries}
    if missing:
        raise ValueError(f"Turns absent from manifest: {sorted(missing)}")
    result = {}
    for turn in entries:
        if turns is not None and turn["turn_index"] not in turns:
            continue
        script_turn = by_id.get(turn["turn_id"])
        if script_turn is None:
            raise ValueError(f"Turn {turn['turn_index']} no longer matches the script; re-render first")
        words = script_words([script_turn])
        if not words or any(word["clip"] for word in words):
            continue
        start = turn["start_ms"]
        duration = (turn["end_ms"] - start) / 1000
        if duration < 0 or not turn["segments"]:
            raise ValueError(f"Invalid audio window for turn {turn['turn_index']}")
        heard = []
        for segment in sorted(turn["segments"], key=lambda s: s["start_ms"]):
            audio = (episode_dir / segment["segment_wav"]).resolve()
            if not audio.is_relative_to(episode_dir.resolve()):
                raise ValueError(f"Segment outside episode: {audio}")
            if not audio.is_file():
                raise ValueError(f"Missing WAV: {audio}")
            offset = (segment["start_ms"] - start) / 1000
            if offset < 0:
                raise ValueError("Segment starts before its turn")
            for word in transcribe_fn(audio, model):
                lo, hi = float(word["start"]), float(word["end"])
                if not math.isfinite(lo) or not math.isfinite(hi) or hi < lo:
                    raise ValueError(f"Invalid transcribed word time in {audio}")
                limit = segment["duration_ms"] / 1000
                heard.append({"word": word["word"], "start": offset + max(0, min(lo, limit)),
                              "end": offset + max(0, min(hi, limit))})
        _assign_times(words, heard)
        _fill_gaps(words, {script_turn.turn_index: (0, duration)})
        result[turn["turn_id"]] = [
            {"text": word["text"], "start_ms": round(word["start"] * 1000),
             "end_ms": round(word["end"] * 1000)} for word in words
        ]
        print(f"Turn {turn['turn_index']}: {len(words)} words", flush=True)
    return result


def atomic_write(path: Path, data: dict) -> None:
    """Publish only a complete JSON file; keep the previous output on failure."""
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent,
                                         prefix=f".{path.name}.", suffix=".tmp", delete=False) as handle:
            temporary = Path(handle.name)
            json.dump(data, handle, ensure_ascii=False, indent=2, allow_nan=False)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("episode_dir", type=Path)
    parser.add_argument("--script", type=Path)
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--turns", type=turn_selection)
    parser.add_argument("--out", type=Path)
    parser.add_argument(
        "--no-feedback",
        action="store_true",
        help="Ignore vocabulary hints from the project review log.",
    )
    args = parser.parse_args(argv)
    try:
        episode_dir = args.episode_dir.resolve()
        manifest_path = episode_dir / "episode_manifest.json"
        original = manifest_path.read_bytes()
        manifest = json.loads(original)
        script = args.script
        project = current(episode_dir)
        if script is None:
            source = manifest.get("source_file")
            if not source:
                source = json.loads((episode_dir / "render_state.json").read_text())["source_file"]
            script = project.resolve(source)
        output = args.out or episode_dir / "word_timings.json"
        inputs = {manifest_path, script.resolve(), episode_dir / "render_state.json"}
        inputs.update((episode_dir / segment["segment_wav"]).resolve()
                      for turn in manifest["turns"] for segment in turn["segments"])
        if output.resolve() in inputs or output.suffix.lower() != ".json":
            raise ValueError("Output must be a .json file and cannot overwrite an input")
        hints = ""
        if not args.no_feedback:
            from scriptcast.review import project_dir_from_voices, vocabulary_hints
            try:
                hints = vocabulary_hints(project_dir_from_voices(project.voices))
            except Exception as exc:  # noqa: BLE001 - hints are optional
                print(f"vocabulary hints unavailable ({exc})", file=sys.stderr)
                hints = ""
        if hints:
            def transcribe_fn(audio, model):
                return transcribe_with_prompt(audio, model, initial_prompt=hints)
        else:
            # Same default path as before H5 when no hints exist.
            transcribe_fn = transcribe
        data = {"manifest_sha256": hashlib.sha256(original).hexdigest(), "model": args.model,
                "turns": build_timings(episode_dir, manifest, parse_transcript(script), args.model,
                                       args.turns, transcribe_fn=transcribe_fn)}
        if manifest_path.read_bytes() != original:
            raise ValueError("Manifest changed during transcription; rerun timings")
        atomic_write(output, data)
        print(f"Wrote {len(data['turns'])} turns to {output}")
        return 0
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print(f"scriptcast-word-timings: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
