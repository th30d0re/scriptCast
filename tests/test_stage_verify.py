"""Tests for scriptcast-stage-verify. Each check has a synthetic still."""
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

from scriptcast.tools.stage_verify import (
    CROWDED_INK,
    EMPTY_INK,
    Element,
    OcrWord,
    TesseractBackend,
    _crushed_subtitles,
    check_border,
    check_containers,
    check_density,
    check_overlap,
    check_strings,
    classify_filler,
    evaluate_fixtures,
    fit_baseline,
)

FIXTURES = Path(__file__).resolve().parents[1] / "video" / "fixtures" / "verify"
FONT = "/System/Library/Fonts/Supplemental/Arial.ttf"


def canvas(colour=(12, 16, 28)) -> Image.Image:
    return Image.new("RGB", (1920, 1080), colour)


def font(size: int) -> ImageFont.ImageFont:
    return ImageFont.truetype(FONT, size)


def test_ocr_readback_present_absent_and_internal():
    image = canvas()
    draw = ImageDraw.Draw(image)
    face = font(64)
    draw.text((180, 180), "PRIVATE HOSTILITY", fill=(232, 220, 190), font=face)
    draw.text((180, 320), "TURN 63", fill=(232, 220, 190), font=face)
    draw.text((180, 460), "ZEBRAWORD", fill=(232, 220, 190), font=face)
    words = TesseractBackend().read(image)
    heard = " ".join(w.text for w in words)
    assert "HOSTILITY" in heard
    assert "TURN" in heard

    reasons = check_strings(
        words,
        due=["PRIVATE HOSTILITY"],
        later=["a string from the next cue"],
        declared=["PRIVATE HOSTILITY"],
        source_lines=[],
    )
    checks = {r.check for r in reasons}
    assert "missing" not in checks
    assert "premature" not in checks
    assert "internal" in checks
    assert "undeclared" in checks
    assert any("zebraword" in r.detail for r in reasons if r.check == "undeclared")

    # "Turner" is a name in the approved stills, not an internal turn label.
    turner = check_strings(
        [OcrWord("Turner", 95, 100, 100, 80, 24)],
        due=[], later=[], declared=["Turner"], source_lines=[],
    )
    assert not any(r.check == "internal" for r in turner)


def test_filler_needs_both_word_count_and_prose():
    prose = [OcrWord(word, 90, 10, 20 + i * 18, 80, 16) for i, word in enumerate(
        ["alpha", "bravo", "charlie", "delta"] * 20
    )]
    assert classify_filler(prose[:60]) is None
    flagged = classify_filler(prose)
    assert flagged is not None and flagged.check == "filler"
    labels = [OcrWord(word, 90, 10, 20, 80, 16) for word in ["BIAS", "PREJUDICE"] * 40]
    assert classify_filler(labels) is None


def test_empty_and_crowded_containers():
    image = canvas()
    empty = Element("card", "frame", 200, 200, 500, 360)
    reasons = check_containers(image, [empty])
    assert [r.check for r in reasons] == ["empty"]
    assert reasons[0].elements == ["card"]
    assert "1.2%" in reasons[0].detail or f"{EMPTY_INK:.1%}" in reasons[0].detail

    packed = canvas()
    ImageDraw.Draw(packed).rectangle([200, 200, 700, 560], fill=(236, 228, 210))
    crowded = check_containers(packed, [empty])
    assert [r.check for r in crowded] == ["crowded"]
    assert crowded[0].elements == ["card"]

    half = canvas()
    ImageDraw.Draw(half).rectangle([200, 200, 400, 380], fill=(236, 228, 210))
    assert check_containers(half, [empty]) == []
    assert CROWDED_INK > EMPTY_INK


def test_border_band_ignores_a_hairline_and_flags_a_blob():
    hairline = canvas()
    draw = ImageDraw.Draw(hairline)
    draw.rectangle([8, 8, 1911, 1071], outline=(190, 176, 140))
    assert check_border(hairline) == []

    blob = canvas()
    ImageDraw.Draw(blob).rectangle([120, 8, 220, 58], fill=(232, 220, 190))
    reasons = check_border(blob)
    assert [r.check for r in reasons] == ["border"]
    assert "70 px" in reasons[0].detail

    notch = canvas()
    ImageDraw.Draw(notch).rectangle([900, 8, 1020, 62], fill=(232, 220, 190))
    assert any(r.check == "border" for r in check_border(notch))


def test_density_outlier_against_a_tight_baseline():
    def speck(width: int) -> Image.Image:
        image = canvas()
        ImageDraw.Draw(image).rectangle([120, 140, 120 + width, 200], fill=(220, 214, 196))
        return image

    baseline = fit_baseline([speck(n) for n in (24, 30, 27, 33, 28)])
    assert check_density(speck(29), baseline) == []
    outlier = check_density(speck(900), baseline)
    assert len(outlier) == 1 and outlier[0].check == "density"
    assert "z=" in outlier[0].detail


def test_declared_overlap_text_line_and_clearance():
    title = Element("title", "text", 400, 300, 280, 48, text="Discriminatory Actions")
    subtitle = Element("sub", "text", 420, 330, 240, 36, text="causal node 2")
    clear = Element("sub", "text", 420, 400, 240, 36, text="causal node 2")
    assert any(r.check == "overlap" for r in check_overlap([title, subtitle]))
    assert check_overlap([title, clear]) == []

    arrow = Element("arrow", "line", 0, 0, 0, 0, points=[(100, 320), (800, 320)])
    crossed = check_overlap([title, arrow])
    assert any("crosses" in r.detail for r in crossed)

    mark = Element("person", "mark", 430, 310, 40, 40)
    assert any("mark" in r.detail for r in check_overlap([title, mark]))


def test_crushed_subtitle_is_overlap_and_a_name_is_not():
    crushed = _crushed_subtitles([
        OcrWord("Discriminatory", 96, 995, 407, 202, 28),
        OcrWord("Astiangode", 36, 993, 442, 150, 29),
    ])
    assert len(crushed) == 1 and crushed[0].check == "overlap"
    assert _crushed_subtitles([OcrWord("Turner", 92, 200, 200, 90, 24)]) == []


def test_fixtures_flag_every_labelled_defect_and_pass_the_good_frames():
    out = Path("/tmp/l4-verify-pytest")
    report = evaluate_fixtures(FIXTURES, out)
    assert report["good_failures"] == []
    assert report["misses"] == []
    for kind, row in report["precision_recall"].items():
        assert row["recall"] == 1.0, kind
    assert report["seconds_per_still"] > 0
    assert (out / "verify.json").is_file()
    assert (out / "verify_sheet.png").is_file()
    saved = json.loads((out / "verify.json").read_text())
    assert saved["thresholds"]["E"] == EMPTY_INK
    assert saved["thresholds"]["z"] == 8.0
