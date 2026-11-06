"""Reject VLM claims that cannot be tied back to the Stage plan or still OCR."""
from __future__ import annotations

import re
from pathlib import Path
from typing import Iterable

from PIL import Image, ImageChops, ImageStat


def overlap_fraction(region: list[float], box: list[float]) -> float:
    """Intersection as a fraction of the claimed region (all boxes are xywh)."""
    x, y, w, h = region
    bx, by, bw, bh = box
    area = max(0.0, w) * max(0.0, h)
    if not area:
        return 0.0
    iw = max(0.0, min(x + w, bx + bw) - max(x, bx))
    ih = max(0.0, min(y + h, by + bh) - max(y, by))
    return iw * ih / area


def quoted_text(evidence: str) -> list[str]:
    return [x.strip() for x in re.findall(r'["“”\']([^"“”\']+)["“”\']', evidence) if x.strip()]


def pixel_diff(approved_png: str | Path, new_png: str | Path, box: list[float]) -> float:
    """Return mean absolute RGB difference in a normalised xywh element box.

    The value is in the inclusive 0..1 range.  Boxes that fall outside the
    image, or have no area after clipping, have no changed pixels and return 0.
    """
    with Image.open(approved_png) as approved, Image.open(new_png) as new:
        approved = approved.convert("RGB")
        new = new.convert("RGB")
        if approved.size != new.size:
            raise ValueError("approved and candidate stills must have the same dimensions")
        width, height = approved.size
        x, y, w, h = box
        left = max(0, min(width, int(x * width)))
        top = max(0, min(height, int(y * height)))
        right = max(left, min(width, int((x + w) * width + 0.999999)))
        bottom = max(top, min(height, int((y + h) * height + 0.999999)))
        if left == right or top == bottom:
            return 0.0
        difference = ImageChops.difference(
            approved.crop((left, top, right, bottom)),
            new.crop((left, top, right, bottom)),
        )
        return sum(ImageStat.Stat(difference).mean) / (255.0 * 3)


def ground_defects(defects: Iterable[dict], elements: dict[str, list[float]], ocr_text: str,
                   approved_png: str | Path | None = None,
                   new_png: str | Path | None = None) -> tuple[list[dict], int]:
    """Return supported defects and the number rejected as ungrounded.

    A named defect needs only a declared element.  Model-supplied regions are
    unreliable, so their overlap is recorded as a tie-breaker instead of being
    used to reject a claim.  When approved and candidate stills are available,
    a named element must have visibly changed.  An anonymous Content claim needs
    quoted text that OCR actually sees.
    """
    kept, dropped = [], 0
    haystack = " ".join(ocr_text.lower().split())
    for defect in defects:
        ident = defect.get("element_id")
        if ident is not None:
            box = elements.get(ident)
            region_agrees = box is not None and overlap_fraction(defect["region"], box) >= 0.30
            valid = box is not None
            if valid and approved_png is not None and new_png is not None:
                valid = pixel_diff(approved_png, new_png, box) >= 0.01
        elif defect.get("category") == "Content":
            quotes = quoted_text(defect.get("evidence", ""))
            valid = bool(quotes) and all(" ".join(q.lower().split()) in haystack for q in quotes)
            region_agrees = False
        else:
            valid = False
            region_agrees = False
        if valid:
            kept.append(dict(defect, region_agrees=region_agrees))
        else:
            dropped += 1
    return kept, dropped
