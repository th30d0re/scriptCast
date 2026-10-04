from __future__ import annotations

from scriptcast.asr.merge import (
    backfill_from_reference,
    coverage_warnings,
    drop_words_in_spans,
    fill_gaps,
    group_words_into_segments,
    normalize_tokens,
    word_gaps,
)


def _word(text: str, start: float, end: float) -> dict:
    return {"word": text, "start": start, "end": end}


def test_fill_mid_file_gap_from_pass_with_most_words() -> None:
    primary = [_word("before", 0.0, 36.5), _word("after", 48.2, 48.8)]
    sparse = [_word("one", 38.0, 38.4)]
    complete = [
        _word("one", 38.0, 38.4),
        _word("two", 40.0, 40.4),
        _word("three", 45.0, 45.4),
    ]

    merged, filled, unfilled = fill_gaps(primary, [sparse, complete], 49.0)

    assert [word["word"] for word in merged] == ["before", "one", "two", "three", "after"]
    assert filled == 1
    assert unfilled == 0


def test_drop_words_in_spans_uses_half_open_midpoint_boundaries() -> None:
    words = [
        _word("before", 0.0, 1.0),
        _word("at-start", 1.5, 2.5),
        _word("inside", 2.0, 3.0),
        _word("at-end", 3.5, 4.5),
    ]

    remaining = drop_words_in_spans(words, [(2.0, 4.0)])

    assert [word["word"] for word in remaining] == ["before", "at-end"]


def test_word_gaps_includes_opening_trailing_and_empty_ranges() -> None:
    words = [_word("first", 4.8, 5.2), _word("last", 8.0, 8.5)]

    assert word_gaps(words, 12.0) == [(0.0, 4.8), (5.2, 8.0), (8.5, 12.0)]
    assert word_gaps([], 3.0) == [(0.0, 3.0)]
    assert word_gaps([], 2.0) == []


def test_digit_and_spelled_number_normalize_to_the_same_tokens() -> None:
    assert normalize_tokens("78%") == []
    assert normalize_tokens("seventy eight percent") == []
    assert coverage_warnings("The result was 78%.", "The result was seventy eight percent.") == []


def test_coverage_warning_reports_ten_word_span_without_changing_text() -> None:
    merged = "alpha omega"
    check = "alpha missing passage has exactly distinct ordinary words here today coverage omega"

    warnings = coverage_warnings(merged, check)

    assert warnings == [
        {
            "words": 10,
            "excerpt": "missing passage has exactly distinct ordinary words here today coverage",
        }
    ]
    assert merged == "alpha omega"


def test_backfill_inserts_each_reference_segment_once() -> None:
    merged = [{"start": 5.0, "end": 7.0, "text": "later words", "engine": "phonon-2"}]
    reference = [
        {
            "start": 0.0,
            "end": 4.4,
            "text": "it even said like this but this is directed at a certain part of my audience",
        },
        {"start": 5.0, "end": 7.0, "text": "later words"},
    ]

    segments, inserted = backfill_from_reference(merged, reference)

    assert inserted == 1
    assert segments[0]["engine"] == "whisper"
    assert segments[0]["start"] == 0.0


def test_group_words_splits_on_pause_and_maximum_length() -> None:
    words = [
        _word("first", 0.0, 0.3),
        _word("second", 0.5, 0.8),
        _word("pause", 1.7, 2.0),
        _word("late", 32.0, 32.2),
    ]

    segments = group_words_into_segments(words)

    assert [segment["text"] for segment in segments] == ["first second", "pause", "late"]
    assert all(segment["engine"] == "phonon-2" for segment in segments)
