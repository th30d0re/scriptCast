"""Tests for the Vision OCR backend (L5). The Swift helper runs only when built."""
import json
import os
from pathlib import Path

import pytest
from PIL import Image

from scriptcast.tools.stage_verify import (
    OCR_BACKENDS,
    TesseractBackend,
    VisionBackend,
    get_backend,
    load_frame,
)

FIXTURES = Path(__file__).resolve().parents[1] / "video" / "fixtures" / "verify"


def _stage_ocr_bin() -> Path:
    env = os.environ.get("STAGE_OCR_BIN")
    if env:
        return Path(env)
    return Path(__file__).resolve().parents[1] / "swift" / "StageOCR" / ".build" / "release" / "stage-ocr"


needs_binary = pytest.mark.skipif(
    not _stage_ocr_bin().is_file(),
    reason="stage-ocr not built; run `cd swift/StageOCR && swift build -c release`",
)

CANNED = json.dumps({
    "path": "/tmp/still.png",
    "width": 1920,
    "height": 1080,
    "words": [
        {"text": "Discriminatory", "conf": 100.0, "x": 995, "y": 407, "w": 202, "h": 28},
        {"text": "Astiangode", "conf": 36.0, "x": 993, "y": 442, "w": 150, "h": 29},
        {"text": "x", "conf": 95.0, "x": 10, "y": 10, "w": 8, "h": 8},
        {"text": "faint", "conf": 44.9, "x": 20, "y": 20, "w": 40, "h": 12},
    ],
})


def test_parse_canned_payload_coordinates_confidence_and_filter():
    words = VisionBackend.parse_words(CANNED, 45)
    assert [w.text for w in words] == ["Discriminatory"]
    word = words[0]
    assert word.conf == 100.0
    assert word.box == (995, 407, 202, 28)

    # min_conf 0 keeps the low-confidence words a crushed subtitle comes back
    # as; the two-character floor still applies, as in _parse_tsv.
    words = VisionBackend.parse_words(CANNED, 0)
    assert [(w.text, w.conf) for w in words] == [
        ("Discriminatory", 100.0),
        ("Astiangode", 36.0),
        ("faint", 44.9),
    ]
    assert words[1].box == (993, 442, 150, 29)


def test_parse_handles_one_json_object_per_line():
    other = json.loads(CANNED)
    other["path"] = "/tmp/second.png"
    words = VisionBackend.parse_words(CANNED + "\n" + json.dumps(other) + "\n", 45)
    assert [w.text for w in words] == ["Discriminatory", "Discriminatory"]


def test_registered_as_vision():
    assert OCR_BACKENDS["vision"] is VisionBackend
    backend = get_backend("vision")
    assert backend.name == "vision"
    # evaluate_fixtures spares only a TesseractBackend at min_conf 0.
    assert isinstance(backend, TesseractBackend)
    assert backend.min_conf == 0


def test_missing_binary_raises_build_hint(tmp_path, monkeypatch):
    missing = tmp_path / "no-such-binary"
    monkeypatch.setenv("STAGE_OCR_BIN", str(missing))
    backend = VisionBackend()
    with pytest.raises(RuntimeError, match="swift build -c release"):
        backend.read(Image.new("RGB", (64, 64)))


@needs_binary
def test_real_binary_reads_a_fixture_still():
    image = load_frame(FIXTURES / "bad" / "h_v1_e751.jpg")
    vision = VisionBackend().read(image)
    assert vision, "vision read no words"
    for word in vision:
        assert 0 <= word.x <= 1920 and 0 <= word.y <= 1080
        assert word.w > 0 and word.h > 0
        assert 0 <= word.conf <= 100
    heard = {w.text for w in vision}
    tesseract = {w.text for w in TesseractBackend(min_conf=0).read(image)}
    shared = heard & tesseract
    assert shared, f"no word both backends find: vision={sorted(heard)}, tesseract={sorted(tesseract)}"
    # This fixture's internal label; both backends must see it.
    assert "TURN" in heard and "TURN" in tesseract
