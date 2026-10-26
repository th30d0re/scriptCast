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
from scriptcast.tool_json import JsonArgumentParser, emit, fail, flag_present, run_json

_TARGET_SR = 24000


def _to_wav(source: Path, dest: Path) -> None:
    """Decode anything ffmpeg reads into mono at the engine's sample rate."""
    subprocess.run(
        ["ffmpeg", "-v", "error", "-y", "-i", str(source),
         "-ac", "1", "-ar", str(_TARGET_SR), str(dest)],
        check=True,
    )


def _ranked_windows(segments: list[dict], seconds: float) -> list[dict]:
    """Sentence runs `_best_window` chooses from, closest to `seconds` first.

    Discovery order breaks ties, matching the original strict ``<`` comparison.
    ``score`` is ``1 / (1 + abs(span - seconds))`` so a nearer run ranks higher.
    """
    found = []
    order = 0
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
            distance = abs(span - seconds)
            found.append({
                "order": order,
                "start": start,
                "end": end,
                "text": text,
                "distance": distance,
            })
            order += 1
    found.sort(key=lambda item: (item["distance"], item["order"]))
    return found


def _bounded_time(value: float, duration: float) -> float:
    """``value`` clamped to ``[0, duration]``, rounded to 2 decimals when that stays inside."""
    clamped = min(max(float(value), 0.0), float(duration))
    rounded = round(clamped, 2)
    if rounded < 0.0 or rounded > duration:
        return clamped
    return rounded


def _clamp_ranked(ranked: list[dict], duration: float, seconds: float) -> list[dict]:
    """Clamp each run to the file and score that clamped span.

    Rank follows the clamped length, closest to ``seconds`` first. Discovery
    order still breaks ties. The legacy cutter keeps Whisper's raw ends.
    """
    clamped = []
    for item in ranked:
        start = _bounded_time(float(item["start"]), duration)
        end = _bounded_time(float(item["end"]), duration)
        if end < start:
            end = start
        clamped.append({
            "order": item["order"],
            "start": start,
            "end": end,
            "text": item["text"],
            "distance": abs((end - start) - seconds),
        })
    clamped.sort(key=lambda item: (item["distance"], item["order"]))
    return clamped


def _file_duration(path: Path) -> float:
    info = soundfile.info(str(path))
    return float(info.frames) / float(info.samplerate)


def _best_window(segments: list[dict], seconds: float) -> tuple[float, float, str]:
    """The run of whole sentences closest to `seconds`, by Whisper's own splits.

    Whisper segments end on sentence boundaries, so building the window out of
    consecutive segments keeps the clip and its transcript aligned.
    """
    ranked = _ranked_windows(segments, seconds)
    if not ranked:
        raise SystemExit(
            "No run of complete sentences between 3 and 12 seconds was found. "
            "Ask for a longer recording, or one where sentences are finished."
        )
    best = ranked[0]
    return best["start"], best["end"], best["text"]


def _text_in_span(result: dict, start: float, end: float) -> str:
    words = [w for seg in result.get("segments", []) for w in (seg.get("words") or [])]
    chosen = []
    for word in words:
        if float(word["end"]) <= start or float(word["start"]) >= end:
            continue
        token = str(word["word"]).strip()
        if token:
            chosen.append(token)
    if chosen:
        return " ".join(chosen)
    parts = []
    for seg in result.get("segments", []):
        if float(seg["end"]) <= start or float(seg["start"]) >= end:
            continue
        text = str(seg["text"]).strip()
        if text:
            parts.append(text)
    return " ".join(parts)


def main() -> int:
    candidates_mode = flag_present("--candidates-json")
    ap = JsonArgumentParser(json_flags=("--json", "--candidates-json"))
    ap.add_argument("recording", type=Path)
    ap.add_argument("--name", required=not candidates_mode, help="Reference name, e.g. host")
    ap.add_argument("--seconds", type=float, default=9.0,
                    help="Preferred clip length; the nearest sentence run wins.")
    ap.add_argument("--model", default="mlx-community/whisper-medium.en-mlx")
    ap.add_argument("--peak", type=float, default=0.92,
                    help="Normalise to this peak. Recordings are usually quiet.")
    ap.add_argument("--candidates-json", action="store_true",
                    help="List the sentence runs that would be chosen. Writes nothing.")
    ap.add_argument("--json", action="store_true",
                    help="Write --start/--end and print one JSON object.")
    ap.add_argument("--start", type=float, default=None, help="Exact span start, in seconds.")
    ap.add_argument("--end", type=float, default=None, help="Exact span end, in seconds.")
    args = ap.parse_args()
    json_mode = bool(args.json or args.candidates_json)
    return run_json(json_mode, lambda: _run(args))


def _run(args: argparse.Namespace) -> int:
    import mlx_whisper

    if args.candidates_json and args.json:
        return fail("--candidates-json and --json are separate modes")
    if not args.recording.exists():
        raise SystemExit(f"No such recording: {args.recording}")

    explicit = args.start is not None or args.end is not None
    if args.candidates_json and explicit:
        return fail("--candidates-json does not take --start or --end")
    if args.json and (args.start is None or args.end is None):
        return fail("--json requires --start and --end")
    if explicit and (args.start is None or args.end is None):
        raise SystemExit("--start and --end are both required")
    if explicit and args.end <= args.start:
        message = "--end must be greater than --start"
        if args.json:
            return fail(message)
        raise SystemExit(message)

    with tempfile.TemporaryDirectory() as tmp:
        decoded = Path(tmp) / "decoded.wav"
        _to_wav(args.recording, decoded)
        if explicit:
            result = mlx_whisper.transcribe(
                str(decoded), path_or_hf_repo=args.model, verbose=False, word_timestamps=True,
            )
        elif args.candidates_json:
            result = mlx_whisper.transcribe(
                str(decoded), path_or_hf_repo=args.model, verbose=False,
            )
        else:
            result = mlx_whisper.transcribe(
                str(decoded), path_or_hf_repo=args.model, verbose=False
            )
        segments = [
            {"start": s["start"], "end": s["end"], "text": s["text"]}
            for s in result.get("segments", [])
        ]
        if not segments:
            raise SystemExit("No speech found in that recording.")
        if args.candidates_json:
            ranked = _ranked_windows(segments, args.seconds)
            if not ranked:
                raise SystemExit(
                    "No run of complete sentences between 3 and 12 seconds was found. "
                    "Ask for a longer recording, or one where sentences are finished."
                )
            duration = _file_duration(decoded)
            ranked = _clamp_ranked(ranked, duration, args.seconds)
            emit({
                "source": str(args.recording.resolve()),
                "candidates": [
                    {
                        "rank": index,
                        "start": item["start"],
                        "end": item["end"],
                        "text": item["text"],
                        "score": round(1.0 / (1.0 + item["distance"]), 4),
                    }
                    for index, item in enumerate(ranked, start=1)
                ],
            })
            return 0
        if explicit:
            start, end = float(args.start), float(args.end)
            text = _text_in_span(result, start, end)
        else:
            start, end, text = _best_window(segments, args.seconds)
        audio, rate = soundfile.read(str(decoded), dtype="float32", always_2d=False)
        file_duration = len(audio) / float(rate)

    if explicit and (start < 0 or end > file_duration):
        message = "span extends outside the recording"
        if args.json:
            return fail(message)
        raise SystemExit(message)
    if explicit:
        begin = int(start * rate)
        stop = int(end * rate)
        clip = audio[begin:stop]
    else:
        clip = audio[int(start * rate) : int(end * rate)]
    if explicit:
        peak = float(numpy.abs(clip).max()) if len(clip) else 0.0
    else:
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

    duration = len(clip) / _TARGET_SR
    if args.json:
        print(
            f"wrote {out_path}  {duration:.2f}s  from {start:.1f}s to {end:.1f}s of the recording",
            file=sys.stderr,
        )
        emit({
            "wav": str(out_path.resolve()),
            "text": text,
            "duration": round(duration, 2),
            "peak": round(peak, 3),
        })
        return 0

    print(f"wrote {out_path}  {len(clip)/_TARGET_SR:.2f}s  "
          f"from {start:.1f}s to {end:.1f}s of the recording")
    print(f"peak was {peak:.3f}, normalised to {args.peak}")
    print(f"transcript: {text}")
    print("\nRead that against the clip before rendering with it. "
          "The text has to match the audio word for word.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
