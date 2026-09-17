"""Archival audio: play the real recording where the script quotes it.

A turn whose speaker uses the "archive" engine plays a registered excerpt
instead of synthesizing speech:

    Archivist (14:02)
    [clip:speech_1963] <verbatim transcript of the excerpt>

The transcript after the tag is what captions show and what `scriptcast-verify`
checks the audio against, so a mis-cut clip fails verification the same way a
garbled synthetic line does. The registry, `archive/clips.yaml` under the
project root, holds where each excerpt comes from:

    clips:
      speech_1963:
        source: archive/sources/speech_1963.m4a
        start: 12.40          # seconds into the source
        end: 48.15
        citation: "Author, A. (Year). Title. Publisher."
        origin_url: https://...
        content_note: Contains strong language.

`scriptcast-clip` finds the start and end by searching the source's word
timings for a phrase, so nobody trims by hand. Sources stay out of git.
"""
from __future__ import annotations

import hashlib
import re
import subprocess
import tempfile
from functools import lru_cache
from pathlib import Path

import numpy
import soundfile
import yaml

from scriptcast.project import current

SAMPLE_RATE = 24000
# Median loudness of active speech in the rendered voices, measured over a
# full rendered episode. Clips are matched to it so a quote neither jumps out
# nor disappears beside the synthetic hosts.
TARGET_ACTIVE_RMS_DBFS = -18.5
_FADE_S = 0.015
_CLIP_TAG = re.compile(r"\[clip:([A-Za-z0-9_\-]+)\]")


def clip_id_in(text: str) -> str | None:
    match = _CLIP_TAG.search(text)
    return match.group(1) if match else None


def load_registry(path: Path | None = None) -> dict[str, dict]:
    path = path or current().clips
    if not path.exists():
        return {}
    data = yaml.safe_load(path.read_text()) or {}
    return dict(data.get("clips") or {})


def clip_entry(clip_id: str, path: Path | None = None) -> dict:
    entry = load_registry(path).get(clip_id)
    if entry is None:
        raise KeyError(f"clip {clip_id!r} is not registered in {path or current().clips}")
    for field in ("source", "start", "end"):
        if field not in entry:
            raise ValueError(f"clip {clip_id!r} is missing {field!r}")
    if float(entry["end"]) <= float(entry["start"]):
        raise ValueError(f"clip {clip_id!r} ends before it starts")
    return entry


def _source_path(entry: dict) -> Path:
    return current().resolve(str(entry["source"]))


def clip_fingerprint(clip_id: str, path: Path | None = None) -> str:
    """Changes when the excerpt would sound different: new source, new cut."""
    entry = clip_entry(clip_id, path)
    source = _source_path(entry)
    size = source.stat().st_size if source.exists() else -1
    identity = f"{source}|{size}|{float(entry['start']):.3f}|{float(entry['end']):.3f}"
    return hashlib.sha256(identity.encode("utf-8")).hexdigest()[:16]


@lru_cache(maxsize=8)
def _decode(source: str) -> numpy.ndarray:
    """Any format ffmpeg reads, as mono float32 at the pipeline's rate."""
    with tempfile.TemporaryDirectory() as scratch:
        wav = Path(scratch) / "decoded.wav"
        subprocess.run(
            ["ffmpeg", "-v", "error", "-y", "-i", source, "-ac", "1",
             "-ar", str(SAMPLE_RATE), "-c:a", "pcm_f32le", str(wav)],
            check=True,
        )
        audio, _ = soundfile.read(str(wav), dtype="float32")
    return audio


def _active_rms(audio: numpy.ndarray) -> float:
    frame = int(0.03 * SAMPLE_RATE)
    if audio.size < frame:
        return float(numpy.sqrt(numpy.mean(audio**2)))
    frames = audio[: audio.size // frame * frame].reshape(-1, frame)
    rms = numpy.sqrt(numpy.mean(frames**2, axis=1))
    active = rms[rms > rms.max() * 0.1]
    return float(numpy.sqrt(numpy.mean(active**2))) if active.size else 0.0


def render_clip(clip_id: str, path: Path | None = None) -> numpy.ndarray:
    entry = clip_entry(clip_id, path)
    source = _source_path(entry)
    if not source.exists():
        raise FileNotFoundError(f"clip {clip_id!r}: source not found at {source}")
    audio = _decode(str(source))
    start = int(float(entry["start"]) * SAMPLE_RATE)
    end = min(audio.size, int(float(entry["end"]) * SAMPLE_RATE))
    excerpt = audio[start:end].astype(numpy.float32)
    if excerpt.size == 0:
        raise ValueError(f"clip {clip_id!r} is empty; check start and end against the source")

    rms = _active_rms(excerpt)
    if rms > 0:
        excerpt = excerpt * (10 ** (TARGET_ACTIVE_RMS_DBFS / 20) / rms)
    peak = float(numpy.max(numpy.abs(excerpt)))
    if peak > 0.98:
        excerpt = excerpt * (0.98 / peak)

    fade = min(int(_FADE_S * SAMPLE_RATE), excerpt.size // 2)
    if fade:
        ramp = numpy.linspace(0.0, 1.0, fade, dtype=numpy.float32)
        excerpt[:fade] *= ramp
        excerpt[-fade:] *= ramp[::-1]
    return excerpt
