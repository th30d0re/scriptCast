"""Pure helpers for merging transcripts from multiple speech recognizers."""

from __future__ import annotations

import difflib
import re

Word = dict

_NUMBER_WORDS = {
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
    "thirty",
    "forty",
    "fifty",
    "sixty",
    "seventy",
    "eighty",
    "ninety",
    "hundred",
    "thousand",
    "million",
    "percent",
    "point",
    "dollars",
}
_TOKEN_RE = re.compile(r"[^\W\d_]+(?:['’][^\W\d_]+)*|\d+", re.UNICODE)


def normalize_tokens(text: str) -> list[str]:
    """Return comparable word tokens while ignoring numeric expressions."""
    tokens = _TOKEN_RE.findall(text.casefold().replace("%", " "))
    return [token for token in tokens if not token.isdigit() and token not in _NUMBER_WORDS]


def word_gaps(
    words: list[Word], duration: float, threshold: float = 2.0
) -> list[tuple[float, float]]:
    """Find leading, internal, and trailing gaps longer than ``threshold``."""
    ordered = sorted(words, key=lambda word: float(word["start"]))
    if not ordered:
        return [(0.0, duration)] if duration > threshold else []

    gaps: list[tuple[float, float]] = []
    first_start = float(ordered[0]["start"])
    if first_start > threshold:
        gaps.append((0.0, first_start))
    for previous, following in zip(ordered, ordered[1:]):
        gap_start = float(previous["end"])
        gap_end = float(following["start"])
        if gap_end - gap_start > threshold:
            gaps.append((gap_start, gap_end))
    last_end = float(ordered[-1]["end"])
    if duration - last_end > threshold:
        gaps.append((last_end, duration))
    return gaps


def _midpoint(word: Word) -> float:
    return (float(word["start"]) + float(word["end"])) / 2.0


def _is_duplicate(word: Word, existing: list[Word]) -> bool:
    text = str(word.get("word", "")).casefold()
    start = float(word["start"])
    return any(
        abs(float(other["start"]) - start) <= 0.05
        and str(other.get("word", "")).casefold() == text
        for other in existing
    )


def fill_gaps(
    primary: list[Word],
    passes: list[list[Word]],
    duration: float,
    threshold: float = 2.0,
) -> tuple[list[Word], int, int]:
    """Fill each primary-pass gap from the alternate pass with most words."""
    merged = [dict(word) for word in primary]
    filled = 0
    unfilled = 0
    for gap_start, gap_end in word_gaps(primary, duration, threshold):
        candidates = [
            [word for word in words if gap_start <= _midpoint(word) < gap_end]
            for words in passes
        ]
        best = max(candidates, key=len, default=[])
        added = 0
        for word in best:
            if not _is_duplicate(word, merged):
                merged.append(dict(word))
                added += 1
        if added:
            filled += 1
        else:
            unfilled += 1
    merged.sort(key=lambda word: float(word["start"]))
    return merged, filled, unfilled


def drop_words_in_spans(
    words: list[Word], spans: list[tuple[float, float]]
) -> list[Word]:
    """Drop words whose midpoints fall inside any half-open time span."""
    return [
        word
        for word in words
        if not any(start <= _midpoint(word) < end for start, end in spans)
    ]


def group_words_into_segments(
    words: list[Word], pause: float = 0.8, max_len: float = 30.0
) -> list[dict]:
    """Group timestamped words at pauses and maximum segment duration."""
    ordered = sorted(words, key=lambda word: float(word["start"]))
    groups: list[list[Word]] = []
    for word in ordered:
        if not groups:
            groups.append([word])
            continue
        current = groups[-1]
        gap = float(word["start"]) - float(current[-1]["end"])
        length = float(word["end"]) - float(current[0]["start"])
        if gap > pause or length > max_len:
            groups.append([word])
        else:
            current.append(word)

    return [
        {
            "start": float(group[0]["start"]),
            "end": float(group[-1]["end"]),
            "text": " ".join(str(word.get("word", "")).strip() for word in group).strip(),
            "engine": "phonon-2",
        }
        for group in groups
    ]


def _segment_token_ranges(segments: list[dict]) -> tuple[list[str], list[tuple[int, int]]]:
    tokens: list[str] = []
    ranges: list[tuple[int, int]] = []
    for segment in segments:
        start = len(tokens)
        tokens.extend(normalize_tokens(str(segment.get("text", ""))))
        ranges.append((start, len(tokens)))
    return tokens, ranges


def _missing_spans(
    reference_tokens: list[str], merged_tokens: list[str], min_words: int
) -> list[tuple[int, int, int]]:
    matcher = difflib.SequenceMatcher(None, reference_tokens, merged_tokens, autojunk=False)
    spans: list[tuple[int, int, int]] = []
    for tag, ref_start, ref_end, merged_start, merged_end in matcher.get_opcodes():
        missing = (ref_end - ref_start) - (merged_end - merged_start)
        if tag in {"delete", "replace"} and missing >= min_words:
            spans.append((ref_start, ref_end, missing))
    return spans


def backfill_from_reference(
    segments: list[dict], reference: list[dict], min_words: int = 8
) -> tuple[list[dict], int]:
    """Restore reference segments that substantially overlap missing token spans."""
    merged_tokens = normalize_tokens(" ".join(str(item.get("text", "")) for item in segments))
    reference_tokens, ranges = _segment_token_ranges(reference)
    selected: set[int] = set()
    for missing_start, missing_end, _missing in _missing_spans(
        reference_tokens, merged_tokens, min_words
    ):
        for index, (segment_start, segment_end) in enumerate(ranges):
            token_count = segment_end - segment_start
            overlap = max(0, min(segment_end, missing_end) - max(segment_start, missing_start))
            if token_count and overlap * 2 >= token_count:
                selected.add(index)

    output = [dict(segment) for segment in segments]
    for index in sorted(selected):
        output.append({**reference[index], "engine": "whisper"})
    output.sort(key=lambda segment: float(segment.get("start", 0.0)))
    return output, len(selected)


def coverage_warnings(
    merged_text: str, check_text: str, min_words: int = 8
) -> list[dict]:
    """Report long spans present in the coverage check and absent from merged text."""
    merged_tokens = normalize_tokens(merged_text)
    check_tokens = normalize_tokens(check_text)
    return [
        {
            "words": missing,
            "excerpt": " ".join(check_tokens[start:end])[:120],
        }
        for start, end, missing in _missing_spans(check_tokens, merged_tokens, min_words)
    ]
