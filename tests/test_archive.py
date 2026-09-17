from pathlib import Path

import numpy
import soundfile
import yaml

from scriptcast import archive
from scriptcast.markup import tokenize_markup
from scriptcast.parser import _clean_text
from scriptcast.models import Turn


def _registry(tmp_path: Path, start: float, end: float) -> Path:
    rate = 48000
    t = numpy.arange(int(3 * rate)) / rate
    tone = (0.05 * numpy.sin(2 * numpy.pi * 220 * t)).astype(numpy.float32)
    source = tmp_path / "source.wav"
    soundfile.write(source, tone, rate)
    registry = tmp_path / "clips.yaml"
    registry.write_text(yaml.safe_dump({"clips": {"quote_one": {
        "source": str(source), "start": start, "end": end}}}))
    return registry


def test_clip_tag_becomes_annotation_and_transcript_stays_speech():
    turn = Turn(0, "id", "lee_atwater", "Lee Atwater", "00:00", 0,
                "[clip:quote_one] What he said.", "[clip:quote_one] What he said.", (1, 2))
    tokenize_markup([turn])
    assert [c.kind for c in turn.markup_chunks] == ["annotation", "speech"]
    assert turn.markup_chunks[0].tag == "clip:quote_one"
    assert turn.markup_chunks[1].text == "What he said."


def test_markdown_cleaning_keeps_underscores_inside_tags():
    assert _clean_text("__bold__ [clip:atwater_1981] text") == "bold [clip:atwater_1981] text"


def test_render_clip_cuts_and_matches_voice_loudness(tmp_path):
    registry = _registry(tmp_path, 0.5, 2.0)
    audio = archive.render_clip("quote_one", registry)
    assert abs(audio.size / archive.SAMPLE_RATE - 1.5) < 0.01
    level = 20 * numpy.log10(archive._active_rms(audio))
    assert abs(level - archive.TARGET_ACTIVE_RMS_DBFS) < 0.5


def test_fingerprint_moves_with_the_cut(tmp_path):
    first = archive.clip_fingerprint("quote_one", _registry(tmp_path, 0.5, 2.0))
    second = archive.clip_fingerprint("quote_one", _registry(tmp_path, 0.5, 2.5))
    assert first != second
