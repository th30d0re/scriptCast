from dataclasses import replace

import pytest

from scriptcast.models import MarkupChunk, Turn, VoiceConfig
from scriptcast.pronunciation import (
    find_heteronyms,
    respell,
    speech_text_for,
    speech_text_hash,
)
from scriptcast.render_state import (
    RenderState,
    compute_turn_fingerprint,
    detect_changed_turns,
    plan_precision_insert,
)

LEXICON = {"record": {"default": "reckerd", "verb": "ree-CORD"}}


@pytest.mark.parametrize(
    "sentence, word, key",
    [
        ("It includes one specific historical record.", "record", "default"),
        ("We record every line before we render it.", "record", "verb"),
        ("I want to present the mechanism plainly.", "present", "verb"),
        ("Their conduct was deputized by law.", "conduct", "default"),
        ("Patrols conduct searches without warrants.", "conduct", "verb"),
    ],
)
def test_reading_follows_part_of_speech(sentence, word, key):
    readings = [r for r in find_heteronyms(sentence) if r.word.lower() == word]
    assert len(readings) == 1
    assert readings[0].key == key
    assert sentence[readings[0].start : readings[0].end].lower() == word


def test_stress_only_function_words_are_ignored():
    assert find_heteronyms("That is what this has been.") == []


def test_respell_swaps_only_listed_words_and_keeps_the_rest():
    text = "The record shows it. We record it again, and present the record."
    assert respell(text, LEXICON) == (
        "The reckerd shows it. We ree-CORD it again, and present the reckerd."
    )


def test_respell_keeps_sentence_initial_capital():
    assert respell("Record it now.", LEXICON) == "Ree-CORD it now."


def test_respell_is_a_no_op_without_listed_words():
    text = "Nothing here needs a different spelling."
    assert respell(text, LEXICON) == text


def test_kokoro_gets_plain_text():
    text = "The historical record."
    assert speech_text_for("mlx_kokoro", text) == text
    assert speech_text_hash("mlx_kokoro", [text]) == ""


def _turn(text: str, turn_id: str = "t1") -> Turn:
    return Turn(
        turn_index=0,
        turn_id=turn_id,
        speaker_id="emmanuel_theodore",
        display_name="Emmanuel Theodore",
        timestamp_mmss="00:00",
        timestamp_ms=0,
        raw_text=text,
        clean_text=text,
        line_span=(1, 2),
        markup_chunks=[MarkupChunk(kind="speech", text=text)],
    )


VOICE = VoiceConfig(speaker_id="emmanuel_theodore", engine="omnivoice",
                    reference_audio="ref.wav", reference_text="ref")


def _state(fingerprints):
    return RenderState(schema_version="2.0", source_file="s.md", source_hash="x",
                       rendered_at="now", turns=fingerprints, segments=[])


def test_turn_rendered_before_its_respelling_existed_re_renders():
    turn = _turn("The historical record.")
    before = replace(compute_turn_fingerprint(turn, VOICE), speech_text_hash="")
    voices = {"emmanuel_theodore": VOICE}
    assert detect_changed_turns([turn], _state([before]), voices) == ["t1"]
    assert plan_precision_insert([turn], _state([before]), voices).modified == [turn]


def test_turn_without_heteronyms_stays_put():
    turn = _turn("Nothing here changes.")
    fp = compute_turn_fingerprint(turn, VOICE)
    voices = {"emmanuel_theodore": VOICE}
    assert detect_changed_turns([turn], _state([fp]), voices) == []
    assert plan_precision_insert([turn], _state([fp]), voices).unchanged == [turn]


def test_precision_insert_notices_a_new_reference_clip():
    turn = _turn("Nothing here changes.")
    old = compute_turn_fingerprint(turn, VOICE)
    new_voice = replace(VOICE, reference_audio="clean.wav")
    plan = plan_precision_insert([turn], _state([old]), {"emmanuel_theodore": new_voice})
    assert plan.modified == [turn]
