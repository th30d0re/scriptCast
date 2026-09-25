"""Tape-machine transitions cut from a source video: a stop, then a rewind and replay.

A tape stop slows the last stretch of an excerpt to a halt (speed falls from 1 to 0, so the pitch
falls with it and the picture slows to a freeze). A rewind plays an earlier stretch backwards at
high speed (audio pitch rises with it), and the replay plays a chosen phrase again at normal speed.
Both outputs are ordinary video files, so they register as archive clips like any other.
"""
from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np

SR = 48000
FPS = 30


def stop_offsets(duration: float, power: float, rate: int = SR) -> np.ndarray:
    """Source seconds consumed at each output sample while speed goes (1 - t/D) ** power -> 0."""
    t = np.arange(int(duration * rate)) / rate
    speed = np.clip(1.0 - t / duration, 0.0, 1.0) ** power
    return np.cumsum(speed) / rate


def stop_audio(samples: np.ndarray, end_s: float, duration: float, power: float, fade_ms: int = 60):
    """Slowed copy of the source that consumes audio up to end_s. Returns (audio, start_s)."""
    offsets = stop_offsets(duration, power)
    start_s = end_s - float(offsets[-1])
    if start_s < 0:
        raise ValueError("Not enough audio before the end point for this stop")
    index = (start_s + offsets) * SR
    lo = np.floor(index).astype(int)
    hi = np.minimum(lo + 1, len(samples) - 1)
    frac = (index - lo).astype(np.float32)
    out = samples[lo] * (1 - frac) + samples[hi] * frac
    fade = min(int(fade_ms / 1000 * SR), len(out))
    out[-fade:] *= np.linspace(1.0, 0.0, fade, dtype=np.float32)
    return out.astype(np.float32), start_s


def _run(cmd, **kwargs):
    return subprocess.run(cmd, check=True, capture_output=True, **kwargs)


def _decode_audio(source: Path) -> np.ndarray:
    result = _run(["ffmpeg", "-v", "error", "-i", str(source), "-vn", "-f", "f32le", "-ac", "1", "-ar", str(SR), "-"])
    return np.frombuffer(result.stdout, dtype=np.float32).copy()


def _write_wav(path: Path, samples: np.ndarray) -> None:
    _run(["ffmpeg", "-v", "error", "-y", "-f", "f32le", "-ac", "1", "-ar", str(SR), "-i", "-", str(path)],
         input=np.clip(samples, -1, 1).astype(np.float32).tobytes())


X264 = ["-c:v", "libx264", "-preset", "veryfast", "-crf", "18", "-pix_fmt", "yuv420p"]


def make_tapestop(source: Path, out: Path, clip_start: float, clip_end: float, duration: float, power: float,
                  tail: float, work: Path) -> float:
    audio = _decode_audio(source)
    stop, start_s = stop_audio(audio, clip_end, duration, power)
    normal = audio[int(clip_start * SR): int(start_s * SR)]
    full = np.concatenate([normal, stop, np.zeros(int(tail * SR), dtype=np.float32)])
    _write_wav(work / "stop.wav", full)
    lead = work / "lead.mp4"
    _run(["ffmpeg", "-v", "error", "-y", "-ss", f"{clip_start:.3f}", "-to", f"{start_s:.3f}", "-i", str(source),
          "-an", "-vf", f"fps={FPS}", *X264, str(lead)])
    frames = work / "frames"
    frames.mkdir(exist_ok=True)
    offsets = stop_offsets(duration, power, rate=FPS)
    count = int(round((duration + tail) * FPS))
    for n in range(count):
        src_t = start_s + (offsets[n] if n < len(offsets) else offsets[-1])
        _run(["ffmpeg", "-v", "error", "-y", "-ss", f"{src_t:.3f}", "-i", str(source), "-frames:v", "1",
              str(frames / f"{n:04d}.png")])
    hold = work / "hold.mp4"
    _run(["ffmpeg", "-v", "error", "-y", "-framerate", str(FPS), "-i", str(frames / "%04d.png"), *X264, str(hold)])
    listing = work / "concat.txt"
    listing.write_text(f"file '{lead}'\nfile '{hold}'\n")
    silent = work / "silent.mp4"
    _run(["ffmpeg", "-v", "error", "-y", "-f", "concat", "-safe", "0", "-i", str(listing), "-c", "copy", str(silent)])
    _run(["ffmpeg", "-v", "error", "-y", "-i", str(silent), "-i", str(work / "stop.wav"), "-c:v", "copy",
          "-c:a", "aac", "-b:a", "192k", "-shortest", str(out)])
    return start_s


def make_rewind_replay(source: Path, out: Path, rewind_from: float, rewind_to: float, speed: float,
                       phrase: tuple[float, float], work: Path) -> None:
    span = rewind_from - rewind_to
    rewound = work / "rewind.mp4"
    _run(["ffmpeg", "-v", "error", "-y", "-ss", f"{rewind_to:.3f}", "-to", f"{rewind_from:.3f}", "-i", str(source),
          "-vf", f"reverse,setpts=PTS/{speed},fps={FPS}",
          "-af", f"areverse,asetrate={int(SR * speed)},aresample={SR},lowpass=f=9000,volume=0.8",
          *X264, "-c:a", "aac", "-ar", str(SR), "-ac", "1", "-b:a", "192k", str(rewound)])
    replay = work / "replay.mp4"
    _run(["ffmpeg", "-v", "error", "-y", "-ss", f"{phrase[0]:.3f}", "-to", f"{phrase[1]:.3f}", "-i", str(source),
          "-vf", f"fps={FPS}", "-af", f"aresample={SR}", *X264, "-c:a", "aac", "-ar", str(SR), "-ac", "1", "-b:a", "192k", str(replay)])
    _run(["ffmpeg", "-v", "error", "-y", "-i", str(rewound), "-i", str(replay), "-filter_complex",
          "[0:v][0:a][1:v][1:a]concat=n=2:v=1:a=1[v][a]", "-map", "[v]", "-map", "[a]", *X264,
          "-c:a", "aac", "-ar", str(SR), "-b:a", "192k", str(out)])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Cut a tape stop and a rewind-and-replay from a source video")
    parser.add_argument("source", type=Path)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--clip-start", type=float, required=True, help="excerpt start in the source, seconds")
    parser.add_argument("--clip-end", type=float, required=True, help="where the excerpt ends and the tape stops")
    parser.add_argument("--phrase", type=float, nargs=2, metavar=("START", "END"), required=True,
                        help="the phrase to replay after the rewind")
    parser.add_argument("--stop-duration", type=float, default=1.2)
    parser.add_argument("--stop-power", type=float, default=2.0, help="higher drops the pitch harder and sooner")
    parser.add_argument("--stop-tail", type=float, default=0.2, help="frozen silent seconds after the stop")
    parser.add_argument("--rewind-to", type=float, help="source time the rewind runs back to (default: phrase start - 0.5)")
    parser.add_argument("--rewind-speed", type=float, default=6.0)
    args = parser.parse_args(argv)
    if not shutil.which("ffmpeg"):
        parser.error("ffmpeg is required")
    args.out_dir.mkdir(parents=True, exist_ok=True)
    rewind_to = args.rewind_to if args.rewind_to is not None else args.phrase[0] - 0.5
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        start = make_tapestop(args.source, args.out_dir / "tapestop.mp4", args.clip_start, args.clip_end,
                              args.stop_duration, args.stop_power, args.stop_tail, work)
        (work / "frames2").mkdir()
        make_rewind_replay(args.source, args.out_dir / "rewind_replay.mp4", args.clip_end, rewind_to,
                           args.rewind_speed, tuple(args.phrase), work)
    print(f"tape stop begins at source {start:.2f}s; wrote tapestop.mp4 and rewind_replay.mp4 in {args.out_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
