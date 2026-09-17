"""Stitch a rendered episode into one MP3 that matches the manifest timeline.

The renderer trims each clip's trailing non-speech and schedules the next clip at
the previous one's SPEECH end (`end_ms`), so the intended timeline is tighter than
a naive concatenation of the WAV files. Concatenating full WAVs re-introduces every
clip's trailing silence, which is audible as long, eerie gaps between turns.

This script lays clips out at their manifest `start_ms` and cuts each one at its
`speech_duration_ms`, so the MP3 matches what the Ableton set plays.

Usage: python3 tools/stitch_episode.py outputs/<episode_id> [--pad-ms N]
"""
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("episode_dir", type=Path)
    ap.add_argument("--pad-ms", type=int, default=None,
                    help="Silence between consecutive clips in ms. Defaults to the "
                         "gap the renderer already scheduled, so the MP3 matches the "
                         "Ableton set. Pass a value to override it.")
    ap.add_argument("--timeline", dest="sequential", action="store_false",
                    help="Place clips at their manifest start_ms instead of laying "
                         "them out back to back. The manifest can schedule clips to "
                         "overlap, so this can produce speakers talking over each "
                         "other; sequential layout is the default.")
    ap.set_defaults(sequential=True)
    ap.add_argument("--bitrate", default="128k")
    args = ap.parse_args()

    ep = args.episode_dir
    manifest = json.loads((ep / "episode_manifest.json").read_text())
    sample_rate = manifest["sample_rate"]

    segments = sorted(
        (seg for turn in manifest["turns"] for seg in turn["segments"]),
        key=lambda s: s["start_ms"],
    )
    if not segments:
        raise SystemExit("no segments in manifest")

    # Build one filter_complex: trim each clip to its speech duration, delay it to
    # its scheduled start, then mix. Silence falls out of the scheduling for free.
    inputs: list[str] = []
    filters: list[str] = []
    cursor = 0
    for i, seg in enumerate(segments):
        wav = (ep / seg["segment_wav"]).resolve()
        inputs += ["-i", str(wav)]
        speech_ms = seg.get("speech_duration_ms") or seg["duration_ms"]
        if args.sequential:
            pad = args.pad_ms if args.pad_ms is not None else seg.get("gap_after_ms", 0)
            start_ms = cursor
            cursor += speech_ms + pad
        else:
            start_ms = seg["start_ms"]
        filters.append(
            f"[{i}:a]atrim=end={speech_ms / 1000:.4f},"
            f"asetpts=PTS-STARTPTS,"
            f"adelay={start_ms}|{start_ms}[a{i}]"
        )
    mix = "".join(f"[a{i}]" for i in range(len(segments)))
    filters.append(f"{mix}amix=inputs={len(segments)}:normalize=0[out]")

    out = ep / f"{manifest['episode_id']}.mp3"
    cmd = (["ffmpeg", "-y", *inputs, "-filter_complex", ";".join(filters),
            "-map", "[out]", "-c:a", "libmp3lame", "-b:a", args.bitrate,
            "-ar", str(sample_rate), str(out)])
    subprocess.run(cmd, check=True, capture_output=True)

    end_ms = cursor if args.sequential else max(s["start_ms"] for s in segments)
    print(f"wrote {out}  ({len(segments)} clips, timeline ends ~{end_ms // 60000}m{end_ms // 1000 % 60}s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
