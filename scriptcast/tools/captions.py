"""Burned-in caption timing from real audio.

Transcribes the stitched episode once with word timestamps (mlx-whisper), aligns the
recognised words to the script's own words, and writes phrase-sized caption entries for
every spoken turn that is not an archive clip. Archive clips keep whatever captions their
source video carries. The alignment is a pure function so it can be tested without ASR.
"""
from __future__ import annotations

import argparse
import difflib
import json
import re
import sys
from pathlib import Path

from scriptcast.parser import parse_transcript

_CLIP_TAG = re.compile(r"\[clip:[^\]]*\]")
_NORMALIZE = re.compile(r"[^a-z0-9]")
DEFAULT_MODEL = "mlx-community/whisper-small-mlx"


def normalize(word: str) -> str:
    return _NORMALIZE.sub("", word.lower())


def script_words(script_turns) -> list[dict]:
    """Every word of the script, in order, with its turn and whether the turn is a clip."""
    words = []
    for turn in script_turns:
        is_clip = bool(_CLIP_TAG.search(turn.clean_text))
        text = _CLIP_TAG.sub(" ", turn.clean_text)
        for position, raw in enumerate(text.split()):
            words.append({"turn": turn.turn_index, "position": position, "text": raw,
                          "norm": normalize(raw), "clip": is_clip})
    return words


def _assign_times(words: list[dict], asr_words: list[dict]) -> None:
    script_norm = [w["norm"] for w in words]
    asr_norm = [normalize(a["word"]) for a in asr_words]
    matcher = difflib.SequenceMatcher(None, script_norm, asr_norm, autojunk=False)
    for block in matcher.get_matching_blocks():
        for offset in range(block.size):
            token, heard = words[block.a + offset], asr_words[block.b + offset]
            if token["norm"]:
                token["start"], token["end"] = float(heard["start"]), float(heard["end"])


def _fill_gaps(words: list[dict], windows: dict[int, tuple[float, float]]) -> None:
    """Give unmatched words a time between their matched neighbours, inside the turn window."""
    by_turn: dict[int, list[dict]] = {}
    for word in words:
        by_turn.setdefault(word["turn"], []).append(word)
    for turn_index, group in by_turn.items():
        lo, hi = windows[turn_index]
        anchors = [(i, w) for i, w in enumerate(group) if "start" in w]
        count = len(group)
        for i, word in enumerate(group):
            if "start" in word:
                continue
            before = max((a for a in anchors if a[0] < i), default=None, key=lambda a: a[0])
            after = min((a for a in anchors if a[0] > i), default=None, key=lambda a: a[0])
            left_t = before[1]["end"] if before else lo
            left_i = before[0] if before else -1
            right_t = after[1]["start"] if after else hi
            right_i = after[0] if after else count
            span = max(right_i - left_i, 1)
            slot = (right_t - left_t) / span
            word["start"] = left_t + slot * (i - left_i)
            word["end"] = word["start"] + slot
        # keep monotone and inside the window
        previous = lo
        for word in group:
            word["start"] = max(previous, min(word["start"], hi))
            word["end"] = max(word["start"], min(word["end"], hi))
            previous = word["start"]


def chunk_words(group: list[dict], max_chars: int = 34) -> list[list[dict]]:
    chunks, current = [], []
    for word in group:
        candidate = " ".join(w["text"] for w in current + [word])
        if current and len(candidate) > max_chars:
            chunks.append(current)
            current = []
        current.append(word)
        text = word["text"]
        length = len(" ".join(w["text"] for w in current))
        if text.endswith((".", "?", "!")) or (text.endswith((",", ";", ":", "—")) and length >= 18):
            chunks.append(current)
            current = []
    if current:
        chunks.append(current)
    return chunks


def build_captions(asr_words: list[dict], script_turns, manifest: dict, *, max_chars: int = 34,
                   tail_ms: int = 150, merge_gap_ms: int = 300) -> list[dict]:
    words = script_words(script_turns)
    windows = {t["turn_index"]: (t["start_ms"] / 1000, t["end_ms"] / 1000) for t in manifest["turns"]}
    missing = {w["turn"] for w in words} - set(windows)
    if missing:
        raise ValueError(f"Script turns absent from the manifest: {sorted(missing)[:5]}")
    _assign_times(words, asr_words)
    _fill_gaps(words, windows)
    captions = []
    for turn_index in sorted({w["turn"] for w in words if not w["clip"]}):
        group = [w for w in words if w["turn"] == turn_index]
        lo, hi = windows[turn_index]
        for chunk in chunk_words(group, max_chars):
            start = chunk[0]["start"]
            end = min(chunk[-1]["end"] + tail_ms / 1000, hi + 0.2)
            captions.append({"turn_index": turn_index, "start_ms": round(start * 1000),
                             "end_ms": round(max(end, start + 0.2) * 1000),
                             "text": " ".join(w["text"] for w in chunk)})
    for earlier, later in zip(captions, captions[1:]):
        if 0 <= later["start_ms"] - earlier["end_ms"] < merge_gap_ms:
            earlier["end_ms"] = later["start_ms"]
        elif earlier["end_ms"] > later["start_ms"]:
            earlier["end_ms"] = later["start_ms"]
    return [c for c in captions if c["end_ms"] > c["start_ms"]]


def transcribe(audio: Path, model: str) -> list[dict]:
    try:
        import mlx_whisper
    except ImportError as exc:  # pragma: no cover - depends on the environment
        raise SystemExit("scriptcast-captions needs mlx-whisper (pip install mlx-whisper)") from exc
    result = mlx_whisper.transcribe(str(audio), path_or_hf_repo=model, language="en",
                                    word_timestamps=True, condition_on_previous_text=False)
    return [{"word": w["word"].strip(), "start": w["start"], "end": w["end"]}
            for segment in result["segments"] for w in segment.get("words", [])]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Write caption timing for an episode from its audio and script")
    parser.add_argument("episode_dir", type=Path)
    parser.add_argument("--script", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--audio", type=Path, help="defaults to the single .mp3 in episode_dir")
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--asr", type=Path, help="reuse recognised words from this JSON instead of transcribing")
    parser.add_argument("--max-chars", type=int, default=34)
    args = parser.parse_args(argv)
    manifest = json.loads((args.episode_dir / "episode_manifest.json").read_text())
    if args.asr:
        asr_words = json.loads(args.asr.read_text())
    else:
        audio = args.audio
        if audio is None:
            found = sorted(args.episode_dir.glob("*.mp3"))
            if len(found) != 1:
                parser.error("Expected exactly one .mp3 in the episode directory, or pass --audio")
            audio = found[0]
        asr_words = transcribe(audio, args.model)
        cache = Path(str(args.out) + ".asr.json")
        cache.write_text(json.dumps(asr_words))
    captions = build_captions(asr_words, parse_transcript(args.script), manifest, max_chars=args.max_chars)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(captions, indent=1))
    print(f"wrote {len(captions)} captions to {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
