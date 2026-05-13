from voice_pipeline.markup import tokenize_markup
from voice_pipeline.models import Turn


def _make_turn(clean_text: str, turn_index: int = 0) -> Turn:
    return Turn(
        turn_index=turn_index,
        speaker_id="ai_1",
        display_name="AI 1",
        timestamp_mmss="00:00",
        timestamp_ms=0,
        raw_text=clean_text,
        clean_text=clean_text,
        line_span=(1, 1),
    )


def test_no_markup_produces_single_speech_chunk() -> None:
    turn = _make_turn("hello world")
    tokenize_markup([turn])

    assert len(turn.markup_chunks) == 1
    assert turn.markup_chunks[0].kind == "speech"
    assert turn.markup_chunks[0].text == "hello world"


def test_pause_tag_produces_silence_chunk() -> None:
    turn = _make_turn("hello [pause:500ms] world")
    tokenize_markup([turn])

    assert [chunk.kind for chunk in turn.markup_chunks] == ["speech", "silence", "speech"]
    assert turn.markup_chunks[0].text == "hello"
    assert turn.markup_chunks[1].duration_ms == 500
    assert turn.markup_chunks[2].text == "world"


def test_beat_tag_produces_silence_chunk_400ms() -> None:
    turn = _make_turn("hello [beat] world")
    tokenize_markup([turn])

    assert turn.markup_chunks[1].kind == "silence"
    assert turn.markup_chunks[1].duration_ms == 400


def test_emphasis_tag_produces_annotation_chunk() -> None:
    turn = _make_turn("[emphasis:word]")
    tokenize_markup([turn])

    assert len(turn.markup_chunks) == 1
    assert turn.markup_chunks[0].kind == "annotation"
    assert turn.markup_chunks[0].tag == "emphasis"


def test_tone_tag_produces_annotation_chunk() -> None:
    turn = _make_turn("[tone:soft] hello")
    tokenize_markup([turn])

    assert len(turn.markup_chunks) == 2
    assert turn.markup_chunks[0].kind == "annotation"
    assert turn.markup_chunks[0].tag == "tone"
    assert turn.markup_chunks[1].kind == "speech"


def test_mixed_tags_correct_order() -> None:
    turn = _make_turn("intro [pause:200ms] middle [beat] end")
    tokenize_markup([turn])

    assert [chunk.kind for chunk in turn.markup_chunks] == [
        "speech",
        "silence",
        "speech",
        "silence",
        "speech",
    ]
    assert turn.markup_chunks[1].duration_ms == 200
    assert turn.markup_chunks[3].duration_ms == 400


def test_invalid_pause_tag_skipped() -> None:
    turn = _make_turn("[pause:bad]")
    tokenize_markup([turn])

    assert not any(chunk.kind == "silence" for chunk in turn.markup_chunks)


def test_tokenize_markup_returns_same_turns_list() -> None:
    turns = [_make_turn("hello")]
    returned = tokenize_markup(turns)

    assert returned is turns


def test_multiple_turns_all_get_chunks() -> None:
    turns = [
        _make_turn("hello", turn_index=0),
        _make_turn("[beat] hi", turn_index=1),
        _make_turn("[tone:soft] bye", turn_index=2),
    ]

    tokenize_markup(turns)

    assert all(len(turn.markup_chunks) >= 1 for turn in turns)


def test_empty_clean_text_produces_no_chunks() -> None:
    turn = _make_turn("")
    tokenize_markup([turn])

    assert turn.markup_chunks == []
