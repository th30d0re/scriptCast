"""Turn a raw recording into a voice reference clip and its exact transcript.

OmniVoice conditions on the clip and its text together, so the two must agree
word for word. A window cut by loudness slices mid-phrase and hands the model
two signals that disagree; early references cut that way failed one generation
in three. This picks a window that starts and ends on sentence
boundaries instead, then transcribes it and writes both halves.

    scriptcast-reference ~/raw_host.m4a --name host
    scriptcast-reference ~/raw_guest.m4a --name guest --seconds 8

Writes <references>/<name>.wav and adds the transcript to
<references>/reference_texts.json, where <references> is the project's
`references` directory. Print the transcript and read it against
the clip before trusting it: a wrong transcript is worse than a shorter clip.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy
import soundfile

from scriptcast.project import current

_TARGET_SR = 24000


def _to_wav(source: Path, dest: Path) -> None:
    """Decode anything ffmpeg reads into mono at the engine's sample rate."""
    subprocess.run(
        ["ffmpeg", "-v", "error", "-y", "-i", str(source),
         "-ac", "1", "-ar", str(_TARGET_SR), str(dest)],
        check=True,
    )


def _best_window(segments: list[dict], seconds: float) -> tuple[float, float, str]:
    """The run of whole sentences closest to `seconds`, by Whisper's own splits.

    Whisper segments end on sentence boundaries, so building the window out of
    consecutive segments keeps the clip and its transcript aligned.
    """
    best = None
    for start_index in range(len(segments)):
        for end_index in range(start_index, len(segments)):
            start = segments[start_index]["start"]
            end = segments[end_index]["end"]
            span = end - start
            if span < 3.0:
                continue
            if span > 12.0:
                break
            text = " ".join(
                s["text"].strip() for s in segments[start_index : end_index + 1]
            ).strip()
            if not text.endswith((".", "!", "?")):
                continue
            score = abs(span - seconds)
            if best is None or score < best[0]:
                best = (score, start, end, text)
    if best is None:
        raise SystemExit(
            "No run of complete sentences between 3 and 12 seconds was found. "
            "Ask for a longer recording, or one where sentences are finished."
        )
    return best[1], best[2], best[3]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("recording", type=Path)
    ap.add_argument("--name", required=True, help="Reference name, e.g. host")
    ap.add_argument("--seconds", type=float, default=9.0,
                    help="Preferred clip length; the nearest sentence run wins.")
    ap.add_argument("--model", default="mlx-community/whisper-medium.en-mlx")
    ap.add_argument("--peak", type=float, default=0.92,
                    help="Normalise to this peak. Recordings are usually quiet.")
    args = ap.parse_args()

    import mlx_whisper

    if not args.recording.exists():
        raise SystemExit(f"No such recording: {args.recording}")

    with tempfile.TemporaryDirectory() as tmp:
        decoded = Path(tmp) / "decoded.wav"
        _to_wav(args.recording, decoded)
        result = mlx_whisper.transcribe(
            str(decoded), path_or_hf_repo=args.model, verbose=False
        )
        segments = [
            {"start": s["start"], "end": s["end"], "text": s["text"]}
            for s in result.get("segments", [])
        ]
        if not segments:
            raise SystemExit("No speech found in that recording.")

        start, end, text = _best_window(segments, args.seconds)
        audio, rate = soundfile.read(str(decoded), dtype="float32", always_2d=False)

    clip = audio[int(start * rate) : int(end * rate)]
    peak = float(numpy.abs(clip).max())
    if peak > 0:
        clip = clip / peak * args.peak

    out_dir = current().references
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / f"{args.name}.wav"
    soundfile.write(str(out_path), clip, _TARGET_SR)

    texts_path = out_dir / "reference_texts.json"
    texts = json.loads(texts_path.read_text()) if texts_path.exists() else {}
    texts[args.name] = text
    texts_path.write_text(json.dumps(texts, indent=1, sort_keys=True) + "\n")

    print(f"wrote {out_path}  {len(clip)/_TARGET_SR:.2f}s  "
          f"from {start:.1f}s to {end:.1f}s of the recording")
    print(f"peak was {peak:.3f}, normalised to {args.peak}")
    print(f"transcript: {text}")
    print("\nRead that against the clip before rendering with it. "
          "The text has to match the audio word for word.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
