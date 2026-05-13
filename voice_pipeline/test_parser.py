from pathlib import Path

import pytest

from voice_pipeline.parser import parse_transcript


def test_parse_transcript_turn_count() -> None:
    turns = parse_transcript(Path("Architecting_the_operation/podcasts/ATO_EP0.md"))
    assert len(turns) >= 130


def test_parse_transcript_turn_order() -> None:
    turns = parse_transcript(Path("Architecting_the_operation/podcasts/ATO_EP0.md"))
    assert [turn.turn_index for turn in turns] == list(range(len(turns)))


def test_parse_transcript_speaker_ids() -> None:
    turns = parse_transcript(Path("Architecting_the_operation/podcasts/ATO_EP0.md"))
    assert {turn.speaker_id for turn in turns} == {
        "emmanuel_theodore",
        "ai_1",
        "ai_2",
    }


def test_parse_transcript_first_turn() -> None:
    turns = parse_transcript(Path("Architecting_the_operation/podcasts/ATO_EP0.md"))
    assert turns[0].speaker_id == "emmanuel_theodore"
    assert turns[0].timestamp_mmss == "00:01"
    assert turns[0].display_name == "Emmanuel Theodore"


def test_parse_transcript_clean_text_no_asterisks() -> None:
    turns = parse_transcript(Path("Architecting_the_operation/podcasts/ATO_EP0.md"))
    assert all("*" not in turn.clean_text and "_" not in turn.clean_text for turn in turns)


def test_parse_transcript_preamble_skipped(tmp_path) -> None:
    transcript_path = tmp_path / "episode.md"
    transcript_path.write_text(
        "\n".join(
            [
                "This is preamble text.",
                "",
                "Emmanuel Theodore (00:00)",
                "Hello.",
                "",
                "AI 1 (00:03)",
                "Reply.",
            ]
        ),
        encoding="utf-8",
    )

    turns = parse_transcript(transcript_path)

    assert len(turns) == 2
    assert all("This is preamble text." not in turn.raw_text for turn in turns)


def test_parse_transcript_no_speaker_lines_raises(tmp_path) -> None:
    transcript_path = tmp_path / "episode.md"
    transcript_path.write_text(
        "\n".join(
            [
                "Just body text.",
                "No speaker headers here.",
            ]
        ),
        encoding="utf-8",
    )

    with pytest.raises(ValueError, match="No speaker turns found"):
        parse_transcript(transcript_path)


def test_parse_transcript_timestamp_ms() -> None:
    turns = parse_transcript(Path("Architecting_the_operation/podcasts/ATO_EP0.md"))

    assert turns[0].timestamp_ms == 1000

    later_turn = turns[10]
    minutes_text, seconds_text = later_turn.timestamp_mmss.split(":", maxsplit=1)
    expected_ms = int(minutes_text) * 60_000 + int(seconds_text) * 1_000
    assert later_turn.timestamp_ms == expected_ms


def test_parse_transcript_line_span() -> None:
    turns = parse_transcript(Path("Architecting_the_operation/podcasts/ATO_EP0.md"))

    assert all(
        isinstance(turn.line_span, tuple)
        and len(turn.line_span) == 2
        and all(isinstance(value, int) for value in turn.line_span)
        and turn.line_span[0] <= turn.line_span[1]
        for turn in turns
    )
