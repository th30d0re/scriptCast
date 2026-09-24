"""Refit speech tails from existing WAVs, optionally re-timing the episode.

No audio is synthesized or modified. Re-stitch the episode to refresh its MP3.
"""
from __future__ import annotations

import argparse
import json
import math
import shutil
from pathlib import Path

from scriptcast.__main__ import _segment_from_wav
from scriptcast.als_generator import generate_als
from scriptcast.project import current


def refit_episode(
    ep: Path, tail_ms: int, speech_threshold: float, gap_ms: int | None = None,
) -> None:
    if tail_ms < 0 or (gap_ms is not None and gap_ms < 0):
        raise ValueError("tail and gap must be nonnegative")
    if not math.isfinite(speech_threshold) or speech_threshold < 0:
        raise ValueError("speech threshold must be finite and nonnegative")
    manifest_path = ep / "episode_manifest.json"
    manifest = json.loads(manifest_path.read_text())
    state_path = ep / "render_state.json"
    state = json.loads(state_path.read_text()) if state_path.exists() else None
    if state is not None and state.get("schema_version") != "2.0":
        raise SystemExit("Legacy render state: run scriptcast --migrate before refitting.")
    ordered = sorted(
        ((turn, seg) for turn in manifest["turns"] for seg in turn["segments"]),
        key=lambda pair: (pair[0]["turn_index"], pair[1]["chunk_index"]),
    )
    results = []
    updated = {}
    cursor = grew = 0
    for turn, seg in ordered:
        wav = ep / seg["segment_wav"]
        if not wav.is_file():
            raise SystemExit(f"missing sample: {seg['segment_wav']}\nRe-render before refitting.")
        result = _segment_from_wav(
            wav, turn["turn_index"], turn["turn_id"], seg["chunk_index"],
            turn["speaker_id"], seg["gap_after_ms"] if gap_ms is None else gap_ms,
            speech_threshold, tail_ms,
        )
        grew += result.speech_duration_ms > seg["speech_duration_ms"]
        seg.update(duration_ms=result.duration_ms,
                   speech_duration_ms=result.speech_duration_ms,
                   checksum=result.checksum, gap_after_ms=result.gap_after_ms)
        if gap_ms is not None:
            seg["start_ms"] = cursor
        seg["end_ms"] = seg["start_ms"] + result.speech_duration_ms
        cursor = seg["end_ms"] + result.gap_after_ms
        updated[(turn["turn_id"], seg["chunk_index"])] = seg
        results.append(result)
    for turn in manifest["turns"]:
        if turn["segments"]:
            turn["start_ms"] = min(s["start_ms"] for s in turn["segments"])
            turn["end_ms"] = max(s["end_ms"] for s in turn["segments"])
    settings = {"tail_ms": tail_ms, "speech_threshold": speech_threshold}
    manifest["refit_settings"] = settings
    if state is not None:
        state["refit_settings"] = settings
        for seg in state.get("segments", []):
            match = updated.get((seg.get("turn_id"), seg.get("chunk_index")))
            if match is not None:
                for key in ("start_ms", "duration_ms", "speech_duration_ms", "gap_after_ms"):
                    seg[key] = match[key]
    # Finish reads and validation before replacing either JSON file.
    if gap_ms is not None:
        als = generate_als(results, ep / f"{manifest['episode_id']}.als")
    shutil.copy(manifest_path, manifest_path.with_suffix(".json.bak"))
    manifest_path.write_text(json.dumps(manifest, indent=2))
    if state is not None:
        state_path.write_text(json.dumps(state, indent=2) + "\n")
        print(f"updated {len(state.get('segments', []))} segments in render_state.json")
    else:
        print("no render_state.json; future renders must use the same tail settings")
    print(f"refit {len(results)} segments, {grew} grew")
    if gap_ms is not None:
        print(f"re-timed {len(results)} clips at {gap_ms}ms gaps -> {als}")
        print(f"timeline: {cursor // 60000}m{cursor // 1000 % 60}s")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("episode_dir", type=Path)
    ap.add_argument("--tail-ms", type=int)
    ap.add_argument("--speech-threshold", type=float)
    ap.add_argument("--gap-ms", type=int)
    args = ap.parse_args()
    project = current(args.episode_dir)
    refit_episode(args.episode_dir,
                  project.tail_ms if args.tail_ms is None else args.tail_ms,
                  project.speech_threshold if args.speech_threshold is None else args.speech_threshold,
                  args.gap_ms)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
