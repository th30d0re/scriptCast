"""Ensemble ASR for archive clip turns; writes clip_transcripts.json."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import sys
import tempfile

from scriptcast.archive import clip_id_in
from scriptcast.asr.engines import ensemble
from scriptcast.parser import parse_transcript
from scriptcast.project import current
from scriptcast.tools.word_timings import turn_selection


LISTEN_MANUALLY_THRESHOLD = 15.0


def atomic_write(path: Path, data: dict) -> None:
    """Publish only a complete JSON file; keep the previous output on failure."""
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            dir=path.parent,
            prefix=f".{path.name}.",
            suffix=".tmp",
            delete=False,
        ) as handle:
            temporary = Path(handle.name)
            json.dump(data, handle, ensure_ascii=False, indent=2, allow_nan=False)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def _offset_words(words: list[dict], offset: float, limit: float) -> list[dict]:
    """Shift engine words into turn-relative seconds and clamp to the segment."""
    shifted: list[dict] = []
    for word in words:
        lo, hi = float(word["start"]), float(word["end"])
        if not math.isfinite(lo) or not math.isfinite(hi) or hi < lo:
            raise ValueError("Invalid transcribed word time")
        shifted.append(
            {
                "text": str(word["text"]),
                "start_ms": round((offset + max(0.0, min(lo, limit))) * 1000),
                "end_ms": round((offset + max(0.0, min(hi, limit))) * 1000),
                "engine": str(word.get("engine", "unknown")),
            }
        )
    return shifted


def build_clip_transcripts(
    episode_dir: Path,
    manifest: dict,
    script_turns,
    turns: set[int] | None = None,
    *,
    ensemble_fn=ensemble,
) -> dict:
    """Run the ensemble on each clip turn's processed WAVs; ignore non-clips."""
    by_id = {turn.turn_id: turn for turn in script_turns}
    entries = manifest["turns"]
    missing = (turns or set()) - {turn["turn_index"] for turn in entries}
    if missing:
        raise ValueError(f"Turns absent from manifest: {sorted(missing)}")

    result: dict[str, dict] = {}
    for turn in entries:
        if turns is not None and turn["turn_index"] not in turns:
            continue
        script_turn = by_id.get(turn["turn_id"])
        if script_turn is None:
            raise ValueError(
                f"Turn {turn['turn_index']} no longer matches the script; re-render first"
            )
        clip_id = clip_id_in(script_turn.clean_text)
        if clip_id is None:
            continue

        start = turn["start_ms"]
        duration = (turn["end_ms"] - start) / 1000
        if duration < 0 or not turn["segments"]:
            raise ValueError(f"Invalid audio window for turn {turn['turn_index']}")

        words_ms: list[dict] = []
        warnings: list[dict] = []
        disagreement_values: list[float] = []
        engines: list[str] = []
        for segment in sorted(turn["segments"], key=lambda item: item["start_ms"]):
            audio = (episode_dir / segment["segment_wav"]).resolve()
            if not audio.is_relative_to(episode_dir.resolve()):
                raise ValueError(f"Segment outside episode: {audio}")
            if not audio.is_file():
                raise ValueError(f"Missing WAV: {audio}")
            offset = (segment["start_ms"] - start) / 1000
            if offset < 0:
                raise ValueError("Segment starts before its turn")
            limit = segment["duration_ms"] / 1000
            payload = ensemble_fn(audio)
            words_ms.extend(_offset_words(payload.get("words", []), offset, limit))
            warnings.extend(payload.get("coverage_warnings") or [])
            disagreement_values.append(float(payload.get("whisper_disagreement_pct") or 0.0))
            for name in payload.get("engines_used") or []:
                if name not in engines:
                    engines.append(name)

        disagreement = max(disagreement_values) if disagreement_values else 0.0
        result[turn["turn_id"]] = {
            "clip_id": clip_id,
            "words": words_ms,
            "coverage_warnings": warnings,
            "whisper_disagreement_pct": disagreement,
            "listen_manually": disagreement > LISTEN_MANUALLY_THRESHOLD,
            "engines_used": engines,
        }
        print(
            f"Turn {turn['turn_index']} ({clip_id}): {len(words_ms)} words, "
            f"disagree {disagreement}%",
            flush=True,
        )
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("episode_dir", type=Path)
    parser.add_argument("--script", type=Path)
    parser.add_argument("--turns", type=turn_selection)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args(argv)
    try:
        episode_dir = args.episode_dir.resolve()
        manifest_path = episode_dir / "episode_manifest.json"
        original = manifest_path.read_bytes()
        manifest = json.loads(original)
        script = args.script
        if script is None:
            source = manifest.get("source_file")
            if not source:
                source = json.loads((episode_dir / "render_state.json").read_text())[
                    "source_file"
                ]
            script = current(episode_dir).resolve(source)
        output = args.out or episode_dir / "clip_transcripts.json"
        inputs = {manifest_path, script.resolve(), episode_dir / "render_state.json"}
        inputs.update(
            (episode_dir / segment["segment_wav"]).resolve()
            for turn in manifest["turns"]
            for segment in turn["segments"]
        )
        if output.resolve() in inputs or output.suffix.lower() != ".json":
            raise ValueError("Output must be a .json file and cannot overwrite an input")
        data = {
            "manifest_sha256": hashlib.sha256(original).hexdigest(),
            "turns": build_clip_transcripts(
                episode_dir,
                manifest,
                parse_transcript(script),
                args.turns,
                ensemble_fn=ensemble,
            ),
        }
        if manifest_path.read_bytes() != original:
            raise ValueError("Manifest changed during transcription; rerun")
        atomic_write(output, data)
        print(f"Wrote {len(data['turns'])} clip turns to {output}")
        return 0
    except (OSError, ValueError, KeyError, TypeError, FileNotFoundError) as exc:
        print(f"scriptcast-clip-transcribe: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
