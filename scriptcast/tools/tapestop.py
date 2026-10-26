"""Tape-stop effect on a stitched episode.

Slows the last stretch of a chosen turn (usually an archive clip) to a stop, the way a tape
machine winding down: playback speed falls from 1 to 0, so the pitch drops with it. The effect
reads only audio the turn already contains, so it needs no sample library, and it stays inside
the turn plus the gap after it so the episode timeline does not move.
"""
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

import numpy as np

SAMPLE_RATE = 48000


def stop_positions(sample_rate: int, duration: float, curve: float) -> np.ndarray:
    """Source offsets (seconds) for each output sample while playback speed goes 1 -> 0."""
    t = np.arange(int(duration * sample_rate)) / sample_rate
    speed = np.clip(1.0 - (t / duration) ** curve, 0.0, 1.0)
    return np.cumsum(speed) / sample_rate


def apply_tapestop(samples: np.ndarray, sample_rate: int, end_s: float, duration: float = 0.6,
                   curve: float = 1.6, fade_ms: int = 40) -> tuple[np.ndarray, float]:
    """Return a copy of samples with a tape stop that consumes source audio up to end_s.

    The stop starts early enough that the source it plays ends exactly at end_s, and it
    finishes after end_s (into the gap that follows the turn). Returns (audio, start_s).
    """
    offsets = stop_positions(sample_rate, duration, curve)
    consumed = float(offsets[-1])
    start_s = end_s - consumed
    if start_s < 0:
        raise ValueError("Not enough audio before the end point for this duration")
    source_time = start_s + offsets
    index = source_time * sample_rate
    lo = np.floor(index).astype(int)
    hi = np.minimum(lo + 1, len(samples) - 1)
    frac = index - lo
    slowed = samples[lo] * (1 - frac) + samples[hi] * frac
    fade = min(int(fade_ms / 1000 * sample_rate), len(slowed))
    slowed[-fade:] *= np.linspace(1.0, 0.0, fade)
    out = samples.copy()
    begin = int(round(start_s * sample_rate))
    end = min(begin + len(slowed), len(out))
    out[begin:end] = slowed[: end - begin]
    return out, start_s


def _decode(path: Path) -> np.ndarray:
    result = subprocess.run(["ffmpeg", "-v", "error", "-i", str(path), "-f", "f32le", "-ac", "1", "-ar", str(SAMPLE_RATE), "-"],
                            capture_output=True, check=True)
    return np.frombuffer(result.stdout, dtype=np.float32).copy()


def _encode(samples: np.ndarray, out: Path) -> None:
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-f", "f32le", "-ac", "1", "-ar", str(SAMPLE_RATE), "-i", "-",
                    "-c:a", "libmp3lame", "-q:a", "2", str(out)],
                   input=np.clip(samples, -1.0, 1.0).astype(np.float32).tobytes(), check=True)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Add a tape-stop to the end of a turn in a stitched episode")
    parser.add_argument("audio", type=Path, help="stitched episode .mp3")
    parser.add_argument("--out", type=Path, required=True)
    where = parser.add_mutually_exclusive_group(required=True)
    where.add_argument("--at", type=float, help="seconds: the moment the source audio ends")
    where.add_argument("--turn", type=int, help="turn index whose end_ms is the end point")
    parser.add_argument("--manifest", type=Path, help="episode_manifest.json (default: next to the audio)")
    parser.add_argument("--duration", type=float, default=0.6, help="seconds the stop takes (default 0.6)")
    parser.add_argument("--curve", type=float, default=1.6, help="deceleration curve; higher stops later (default 1.6)")
    args = parser.parse_args(argv)
    if not shutil.which("ffmpeg"):
        parser.error("ffmpeg is required")
    end_s = args.at
    if args.turn is not None:
        manifest = json.loads((args.manifest or args.audio.parent / "episode_manifest.json").read_text())
        match = [t for t in manifest["turns"] if t["turn_index"] == args.turn]
        if not match:
            parser.error(f"Turn {args.turn} is not in the manifest")
        end_s = match[0]["end_ms"] / 1000
    samples = _decode(args.audio)
    out, start_s = apply_tapestop(samples, SAMPLE_RATE, end_s, args.duration, args.curve)
    _encode(out, args.out)
    print(f"tape stop {start_s:.2f}s to {start_s + args.duration:.2f}s written to {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
