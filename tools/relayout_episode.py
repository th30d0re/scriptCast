"""Re-time a rendered episode without re-synthesizing any audio.

The renderer preserves clip positions across runs via render_state.json so an
edit to one turn does not shift the rest of the timeline. That preservation is
wrong when the audio itself changes length — switching voices or engines, or
changing --tail-ms, leaves every clip sitting at a position computed for the
previous render. The result is an Ableton set whose gaps range from several
seconds of overlap to several seconds of silence.

This tool recomputes every position sequentially from the audio that actually
exists on disk, then rewrites the manifest, the render state, and the .als. It
takes seconds and touches no WAV file, so gaps are a dial rather than a
14-minute re-render.

    python3 tools/relayout_episode.py outputs/<episode_id> --gap-ms 350
"""
from __future__ import annotations

import argparse
import json
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from voice_pipeline.als_generator import generate_als
from voice_pipeline.models import SegmentResult


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("episode_dir", type=Path)
    ap.add_argument("--gap-ms", type=int, required=True,
                    help="Uniform silence between consecutive clips, in ms.")
    args = ap.parse_args()

    ep = args.episode_dir
    manifest_path = ep / "episode_manifest.json"
    manifest = json.loads(manifest_path.read_text())

    ordered = sorted(
        ((turn, seg) for turn in manifest["turns"] for seg in turn["segments"]),
        key=lambda pair: (pair[0]["turn_index"], pair[1]["chunk_index"]),
    )

    cursor = 0
    results: list[SegmentResult] = []
    for turn, seg in ordered:
        wav = ep / seg["segment_wav"]
        if not wav.exists():
            raise SystemExit(
                f"missing sample: {seg['segment_wav']}\n"
                "Re-render before re-timing; a set cannot be laid out around a hole."
            )
        seg["start_ms"] = cursor
        seg["end_ms"] = cursor + seg["speech_duration_ms"]
        seg["gap_after_ms"] = args.gap_ms
        seg.setdefault("turn_id", turn["turn_id"])
        cursor = seg["end_ms"] + args.gap_ms

        results.append(SegmentResult(
            turn_index=turn["turn_index"], turn_id=turn["turn_id"],
            chunk_index=seg["chunk_index"], speaker_id=turn["speaker_id"],
            wav_path=wav, duration_ms=seg["duration_ms"],
            speech_duration_ms=seg["speech_duration_ms"],
            sample_rate=manifest["sample_rate"], gap_after_ms=args.gap_ms,
            checksum=seg["checksum"],
        ))

    for turn in manifest["turns"]:
        if turn["segments"]:
            turn["start_ms"] = turn["segments"][0]["start_ms"]
            turn["end_ms"] = turn["segments"][-1]["end_ms"]

    shutil.copy(manifest_path, manifest_path.with_suffix(".json.bak"))
    manifest_path.write_text(json.dumps(manifest, indent=2))

    # render_state carries the positions a later --precision-insert restores for
    # unchanged turns. Renaming it away would force the next edit into a full
    # re-render, so update its positions in place to match what we just laid out.
    state_path = ep / "render_state.json"
    if state_path.exists():
        state = json.loads(state_path.read_text())
        new_positions = {
            (seg["turn_id"], seg["chunk_index"]): seg
            for turn in manifest["turns"] for seg in turn["segments"]
            if "turn_id" in seg
        }
        for seg in state.get("segments", []):
            key = (seg.get("turn_id"), seg.get("chunk_index"))
            updated = new_positions.get(key)
            if updated:
                seg["start_ms"] = updated["start_ms"]
                seg["gap_after_ms"] = updated["gap_after_ms"]
        state_path.write_text(json.dumps(state, indent=2) + "\n")
        print(f"updated {len(state.get('segments', []))} positions in render_state.json")

    # Ableton caches each sample's waveform in a sibling .asd. Re-rendering a
    # turn keeps the filename (the turn_id is unchanged) but replaces the audio,
    # so a stale .asd makes Live draw the PREVIOUS render's waveform: speech that
    # stops early followed by phantom silence, inside a clip whose geometry is
    # actually correct. Drop them so Live re-analyses on open.
    stale = list(ep.rglob("*.asd"))
    for asd in stale:
        asd.unlink()

    als = generate_als(results, ep / f"{manifest['episode_id']}.als")
    if stale:
        print(f"cleared {len(stale)} stale Ableton waveform caches (.asd)")
    print(f"re-timed {len(results)} clips at {args.gap_ms}ms gaps -> {als}")
    print(f"timeline: {cursor // 60000}m{cursor // 1000 % 60}s")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
