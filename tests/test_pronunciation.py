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
        speaker_id="host",
        display_name="Host",
        timestamp_mmss="00:00",
        timestamp_ms=0,
        raw_text=text,
        clean_text=text,
        line_span=(1, 2),
        markup_chunks=[MarkupChunk(kind="speech", text=text)],
    )


VOICE = VoiceConfig(speaker_id="host", engine="omnivoice",
                    reference_audio="ref.wav", reference_text="ref")


def _state(fingerprints):
    return RenderState(schema_version="2.0", source_file="s.md", source_hash="x",
                       rendered_at="now", turns=fingerprints, segments=[])


def test_turn_rendered_before_its_respelling_existed_re_renders(tmp_path, monkeypatch):
    # A project whose pronunciations.yaml respells "record", so the turn's
    # speech-text hash differs from the "" stored before the respelling existed.
    (tmp_path / "pronunciations.yaml").write_text(
        "record:\n  default: reckerd\n", encoding="utf-8"
    )
    monkeypatch.setenv("SCRIPTCAST_PROJECT", str(tmp_path))
    turn = _turn("The historical record.")
    before = replace(compute_turn_fingerprint(turn, VOICE), speech_text_hash="")
    voices = {"host": VOICE}
    assert detect_changed_turns([turn], _state([before]), voices) == ["t1"]
    assert plan_precision_insert([turn], _state([before]), voices).modified == [turn]


def test_turn_without_heteronyms_stays_put():
    turn = _turn("Nothing here changes.")
    fp = compute_turn_fingerprint(turn, VOICE)
    voices = {"host": VOICE}
    assert detect_changed_turns([turn], _state([fp]), voices) == []
    assert plan_precision_insert([turn], _state([fp]), voices).unchanged == [turn]


def test_precision_insert_notices_a_new_reference_clip():
    turn = _turn("Nothing here changes.")
    old = compute_turn_fingerprint(turn, VOICE)
    new_voice = replace(VOICE, reference_audio="clean.wav")
    plan = plan_precision_insert([turn], _state([old]), {"host": new_voice})
    assert plan.modified == [turn]


def test_always_reading_applies_to_any_context() -> None:
    respellings = {"lead": {"always": "led"}}
    assert respell("Childhood lead exposure.", respellings) == "Childhood led exposure."
    assert respell("Lead poisoning, leaded gas.", respellings) == "Led poisoning, leaded gas."


def test_always_reading_covers_numbers() -> None:
    respellings = {"4473": {"always": "forty-four seventy-three"}}
    assert respell("Fill out a Form 4473 today.", respellings) == "Fill out a Form forty-four seventy-three today."
    assert respell("Room 44730 stays.", respellings) == "Room 44730 stays."
