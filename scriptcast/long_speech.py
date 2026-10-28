"""Split a long speech job, and retry a runaway take, through scriptcast-chunk.

The renderer keeps one wav per speech job. Pieces are crossfaded inside that
job. No engine in this tree publishes a character cap, so `max_chars` stays
unset and a normal take is still one synthesis call. Set `max_chars` on an
engine when that engine has a real limit. A missing binary fails only when a
split or a retry actually needs it.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

import numpy
import soundfile

from scriptcast.models import VoiceConfig
from scriptcast.pronunciation import speech_text_for

FADE_MS = 50.0
MAX_HALVES = 2
# A take this long, and this slow, is a runaway. Ordinary speech is nearer
# a tenth of a second per character, so this does not retry a slow reading.
RUNAWAY_SECONDS = 8.0
RUNAWAY_SECONDS_PER_CHAR = 0.6


def chunk_binary() -> Path:
    """The scriptcast-chunk executable. `SCRIPTCAST_CHUNK` wins over PATH."""
    override = os.environ.get("SCRIPTCAST_CHUNK")
    if override:
        path = Path(override)
        if not path.is_file():
            raise RuntimeError(f"SCRIPTCAST_CHUNK is not a file: {path}")
        return path
    found = shutil.which("scriptcast-chunk")
    if found is None:
        raise RuntimeError(
            "scriptcast-chunk is not on PATH. Build chunk/ and set "
            "SCRIPTCAST_CHUNK, or install the binary."
        )
    return Path(found)


def split_text(text: str, max_chars: int) -> list[str]:
    payload = _invoke(["split", "--text", text, "--max-chars", str(max_chars), "--json"])
    return [str(piece["text"]) for piece in payload["pieces"]]


def halve_text(text: str) -> list[str]:
    payload = _invoke(["halve", "--text", text, "--json"])
    return [str(piece["text"]) for piece in payload["pieces"]]


def is_runaway(audio: numpy.ndarray, sample_rate: int, text: str) -> bool:
    """True when the samples are non-finite or far longer than the text."""
    if audio.size == 0 or not bool(numpy.isfinite(audio).all()):
        return audio.size > 0 and not bool(numpy.isfinite(audio).all())
    seconds = audio.size / sample_rate
    chars = max(1, len(text.strip()))
    return seconds > RUNAWAY_SECONDS and (seconds / chars) > RUNAWAY_SECONDS_PER_CHAR


async def speak(
    engine,
    voice: VoiceConfig,
    text: str,
    *,
    expressive: bool,
) -> numpy.ndarray:
    """Audio for one speech job. One call when the text fits and the take is stable."""
    limit = getattr(engine, "max_chars", None)
    if isinstance(limit, int) and limit > 0 and len(text) > limit:
        pieces = split_text(text, limit)
    else:
        pieces = [text]
    audios = [
        await _piece(engine, voice, piece, expressive=expressive, depth=0)
        for piece in pieces
    ]
    return _join(audios, int(engine.sample_rate))


async def _piece(
    engine,
    voice: VoiceConfig,
    text: str,
    *,
    expressive: bool,
    depth: int,
) -> numpy.ndarray:
    speech = speech_text_for(voice.engine, text)
    if expressive:
        raw = await engine.synthesize_chunk(speech, voice, expressive=True)
    else:
        raw = await engine.synthesize_chunk(speech, voice)
    audio = numpy.asarray(raw, dtype=numpy.float32).reshape(-1)
    runaway = is_runaway(audio, int(engine.sample_rate), text)
    if audio.size == 0 or depth >= MAX_HALVES or not runaway:
        return audio
    halves = halve_text(text)
    if len(halves) < 2:
        return audio
    parts = [
        await _piece(engine, voice, half, expressive=expressive, depth=depth + 1)
        for half in halves
    ]
    return _join(parts, int(engine.sample_rate))


def _join(parts: list[numpy.ndarray], sample_rate: int) -> numpy.ndarray:
    current = parts[0]
    for nxt in parts[1:]:
        current = _crossfade(current, nxt, sample_rate)
    return current


def _crossfade(left: numpy.ndarray, right: numpy.ndarray, sample_rate: int) -> numpy.ndarray:
    if left.size == 0:
        return right
    if right.size == 0:
        return left
    fade_frames = min(
        int(round(sample_rate * FADE_MS / 1000.0)),
        int(left.size),
        int(right.size),
    )
    fade_frames = max(1, fade_frames)
    with tempfile.TemporaryDirectory(prefix="scriptcast-chunk-") as directory:
        root = Path(directory)
        a = root / "a.wav"
        b = root / "b.wav"
        out = root / "out.wav"
        soundfile.write(a, left, sample_rate, subtype="PCM_32")
        soundfile.write(b, right, sample_rate, subtype="PCM_32")
        fade_ms = fade_frames * 1000.0 / sample_rate
        _invoke(
            ["crossfade", "--a", str(a), "--b", str(b), "--out", str(out), "--fade-ms", f"{fade_ms:.6f}"],
            expect_json=False,
        )
        joined, rate = soundfile.read(out, dtype="float32")
    if int(rate) != sample_rate:
        raise RuntimeError(f"scriptcast-chunk crossfade changed the rate to {rate}")
    array = numpy.asarray(joined, dtype=numpy.float32).reshape(-1)
    return array


def _invoke(args: list[str], *, expect_json: bool = True) -> dict:
    proc = subprocess.run(
        [str(chunk_binary()), *args],
        check=False,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or f"exit {proc.returncode}"
        raise RuntimeError(f"scriptcast-chunk {args[0]}: {detail}")
    if not expect_json:
        return {}
    try:
        payload = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"scriptcast-chunk {args[0]} returned {proc.stdout!r}") from exc
    if "pieces" not in payload:
        raise RuntimeError(f"scriptcast-chunk {args[0]} returned no pieces")
    return payload
