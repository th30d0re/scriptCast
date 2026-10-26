"""Ensemble ASR for archive clip turns; writes clip_transcripts.json."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import sys
import tempfile

from scriptcast.archive import clip_id_in
from scriptcast.asr.engines import ensemble
from scriptcast.parser import parse_transcript
from scriptcast.project import current
from scriptcast.tools.captions import _CLIP_TAG, _assign_times, _fill_gaps, normalize
from scriptcast.tools.word_timings import turn_selection


LISTEN_MANUALLY_THRESHOLD = 15.0
CLIP_TRANSCRIPTS_VERSION = 2

_ONES = "zero one two three four five six seven eight nine".split()
_TEENS = (
    "ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen"
).split()
_TENS = "twenty thirty forty fifty sixty seventy eighty ninety".split()


def atomic_write(path: Path, data: dict) -> None:
    """Publish only a complete JSON file; keep the previous output on failure."""
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            dir=path.parent,
            prefix=f".{path.name}.",
            suffix=".tmp",
            delete=False,
        ) as handle:
            temporary = Path(handle.name)
            json.dump(data, handle, ensure_ascii=False, indent=2, allow_nan=False)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def _under_100(n: int) -> list[str]:
    if n < 10:
        return [_ONES[n]]
    if n < 20:
        return [_TEENS[n - 10]]
    tens, ones = divmod(n, 10)
    out = [_TENS[tens - 2]]
    if ones:
        out.append(_ONES[ones])
    return out


def _under_1000(n: int) -> list[str]:
    if n < 100:
        return _under_100(n)
    hundreds, rest = divmod(n, 100)
    out = [_ONES[hundreds], "hundred"]
    if rest:
        out.extend(_under_100(rest))
    return out


def spoken_number_tokens(token: str) -> list[str]:
    """Spoken English tokens for a plain integer; empty if not expandable."""
    digits = normalize(token)
    if not digits.isdigit():
        return []
    n = int(digits)
    if n > 9999:
        return []
    if 1100 <= n <= 2099:
        hi, lo = divmod(n, 100)
        if lo == 0:
            return _under_100(hi) + ["hundred"]
        return _under_100(hi) + _under_100(lo)
    if n < 1000:
        return _under_1000(n)
    thousands, rest = divmod(n, 1000)
    out = [_ONES[thousands], "thousand"]
    if rest:
        out.extend(_under_1000(rest))
    return out


def _year_two_thousand_tokens(token: str) -> list[str] | None:
    """Alternate 2000–2099 reading: 'two thousand …'."""
    digits = normalize(token)
    if not digits.isdigit():
        return None
    n = int(digits)
    if not 2000 <= n <= 2099:
        return None
    out = ["two", "thousand"]
    rest = n - 2000
    if rest:
        out.extend(_under_100(rest))
    return out


def _contains_sequence(haystack: list[str], needle: list[str]) -> bool:
    if not needle:
        return False
    size = len(needle)
    for start in range(len(haystack) - size + 1):
        if haystack[start : start + size] == needle:
            return True
    return False


def _match_norms(token: str, asr_norm: list[str]) -> list[str]:
    """Norms to align against ASR; expand integers when their spoken form appears."""
    spoken = spoken_number_tokens(token)
    if not spoken:
        return [normalize(token)]
    candidates = [spoken]
    alternate = _year_two_thousand_tokens(token)
    if alternate and alternate != spoken:
        candidates.append(alternate)
    for expansion in candidates:
        if _contains_sequence(asr_norm, expansion):
            return expansion
    return spoken


def clip_turn_script_words(script_turn) -> list[dict]:
    """Script tokens after the clip tag, same tokenisation as captions.script_words."""
    text = _CLIP_TAG.sub(" ", script_turn.clean_text)
    words = []
    for position, raw in enumerate(text.split()):
        words.append(
            {
                "turn": script_turn.turn_index,
                "position": position,
                "text": raw,
                "norm": normalize(raw),
                "clip": False,
            }
        )
    return words


def align_script_words(
    script_turn, ensemble_words: list[dict], duration: float
) -> list[dict]:
    """Align ensemble words to script words; expand spoken numbers for matching."""
    words = clip_turn_script_words(script_turn)
    if not words:
        return []
    heard = [
        {
            "word": word["text"],
            "start": word["start_ms"] / 1000,
            "end": word["end_ms"] / 1000,
        }
        for word in ensemble_words
    ]
    asr_norm = [normalize(item["word"]) for item in heard]
    expanded: list[dict] = []
    groups: list[tuple[int, int]] = []
    for index, word in enumerate(words):
        norms = _match_norms(word["text"], asr_norm)
        for norm in norms:
            expanded.append(
                {
                    "turn": word["turn"],
                    "position": word["position"],
                    "text": word["text"],
                    "norm": norm,
                    "clip": False,
                }
            )
        groups.append((index, len(norms)))
    # Expanded number tokens share one script word; times collapse to that span.
    _assign_times(expanded, heard)
    offset = 0
    for index, count in groups:
        parts = expanded[offset : offset + count]
        offset += count
        if count and all("start" in part for part in parts):
            starts = [part["start"] for part in parts]
            ends = [part["end"] for part in parts]
            # Each expanded token carries its ASR time; the number keeps the span.
            words[index]["start"] = min(starts)
            words[index]["end"] = max(ends)
            words[index]["matched"] = True
        else:
            words[index]["matched"] = False
    _fill_gaps(words, {script_turn.turn_index: (0.0, duration)})
    return [
        {
            "text": word["text"],
            "start_ms": round(word["start"] * 1000),
            "end_ms": round(word["end"] * 1000),
            "matched": bool(word.get("matched")),
        }
        for word in words
    ]


def _offset_words(words: list[dict], offset: float, limit: float) -> list[dict]:
    """Shift engine words into turn-relative seconds and clamp to the segment."""
    shifted: list[dict] = []
    for word in words:
        lo, hi = float(word["start"]), float(word["end"])
        if not math.isfinite(lo) or not math.isfinite(hi) or hi < lo:
            raise ValueError("Invalid transcribed word time")
        shifted.append(
            {
                "text": str(word["text"]),
                "start_ms": round((offset + max(0.0, min(lo, limit))) * 1000),
                "end_ms": round((offset + max(0.0, min(hi, limit))) * 1000),
                "engine": str(word.get("engine", "unknown")),
            }
        )
    return shifted


def build_clip_transcripts(
    episode_dir: Path,
    manifest: dict,
    script_turns,
    turns: set[int] | None = None,
    *,
    ensemble_fn=ensemble,
) -> dict:
    """Run the ensemble on each clip turn's processed WAVs; ignore non-clips."""
    by_id = {turn.turn_id: turn for turn in script_turns}
    entries = manifest["turns"]
    missing = (turns or set()) - {turn["turn_index"] for turn in entries}
    if missing:
        raise ValueError(f"Turns absent from manifest: {sorted(missing)}")

    result: dict[str, dict] = {}
    for turn in entries:
        if turns is not None and turn["turn_index"] not in turns:
            continue
        script_turn = by_id.get(turn["turn_id"])
        if script_turn is None:
            raise ValueError(
                f"Turn {turn['turn_index']} no longer matches the script; re-render first"
            )
        clip_id = clip_id_in(script_turn.clean_text)
        if clip_id is None:
            continue

        start = turn["start_ms"]
        duration = (turn["end_ms"] - start) / 1000
        if duration < 0 or not turn["segments"]:
            raise ValueError(f"Invalid audio window for turn {turn['turn_index']}")

        words_ms: list[dict] = []
        warnings: list[dict] = []
        disagreement_values: list[float] = []
        engines: list[str] = []
        for segment in sorted(turn["segments"], key=lambda item: item["start_ms"]):
            audio = (episode_dir / segment["segment_wav"]).resolve()
            if not audio.is_relative_to(episode_dir.resolve()):
                raise ValueError(f"Segment outside episode: {audio}")
            if not audio.is_file():
                raise ValueError(f"Missing WAV: {audio}")
            offset = (segment["start_ms"] - start) / 1000
            if offset < 0:
                raise ValueError("Segment starts before its turn")
            limit = segment["duration_ms"] / 1000
            payload = ensemble_fn(audio)
            words_ms.extend(_offset_words(payload.get("words", []), offset, limit))
            warnings.extend(payload.get("coverage_warnings") or [])
            disagreement_values.append(float(payload.get("whisper_disagreement_pct") or 0.0))
            for name in payload.get("engines_used") or []:
                if name not in engines:
                    engines.append(name)

        disagreement = max(disagreement_values) if disagreement_values else 0.0
        script_words = align_script_words(script_turn, words_ms, duration)
        result[turn["turn_id"]] = {
            "clip_id": clip_id,
            "words": words_ms,
            "script_words": script_words,
            "coverage_warnings": warnings,
            "whisper_disagreement_pct": disagreement,
            "listen_manually": disagreement > LISTEN_MANUALLY_THRESHOLD,
            "engines_used": engines,
        }
        print(
            f"Turn {turn['turn_index']} ({clip_id}): {len(words_ms)} words, "
            f"disagree {disagreement}%",
            flush=True,
        )
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("episode_dir", type=Path)
    parser.add_argument("--script", type=Path)
    parser.add_argument("--turns", type=turn_selection)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args(argv)
    try:
        episode_dir = args.episode_dir.resolve()
        manifest_path = episode_dir / "episode_manifest.json"
        original = manifest_path.read_bytes()
        manifest = json.loads(original)
        script = args.script
        if script is None:
            source = manifest.get("source_file")
            if not source:
                source = json.loads((episode_dir / "render_state.json").read_text())[
                    "source_file"
                ]
            script = current(episode_dir).resolve(source)
        output = args.out or episode_dir / "clip_transcripts.json"
        inputs = {manifest_path, script.resolve(), episode_dir / "render_state.json"}
        inputs.update(
            (episode_dir / segment["segment_wav"]).resolve()
            for turn in manifest["turns"]
            for segment in turn["segments"]
        )
        if output.resolve() in inputs or output.suffix.lower() != ".json":
            raise ValueError("Output must be a .json file and cannot overwrite an input")
        data = {
            "version": CLIP_TRANSCRIPTS_VERSION,
            "manifest_sha256": hashlib.sha256(original).hexdigest(),
            "turns": build_clip_transcripts(
                episode_dir,
                manifest,
                parse_transcript(script),
                args.turns,
                ensemble_fn=ensemble,
            ),
        }
        if manifest_path.read_bytes() != original:
            raise ValueError("Manifest changed during transcription; rerun")
        atomic_write(output, data)
        print(f"Wrote {len(data['turns'])} clip turns to {output}")
        return 0
    except (OSError, ValueError, KeyError, TypeError, FileNotFoundError) as exc:
        print(f"scriptcast-clip-transcribe: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
