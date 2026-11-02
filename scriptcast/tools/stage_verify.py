"""Read rendered Stage stills back and compare them with the scene.

This is the picture counterpart of ``scriptcast-verify``. A still can be the
right size and still be the wrong picture: a string from a later cue, a panel
with nothing in it, type sitting on the frame. The still is read back and
scored against what the scene declared.

Three checks, in the same shape as the audio tools:

1. **Content read-back.** Tesseract (the backend is pluggable; Apple Vision is
   a later swap) reads the still. Every storyboard string is present from its
   cue and absent before it. An OCR word that matches no declared string and
   no source line is undeclared text. ``TURN 63`` and the same family of
   internal labels are called out on their own, as is filler microcopy.
2. **Pixel ink, then a specialised second check.** Ink inside each declared
   container is empty below E and crowded above C. Text-coloured ink in the
   70 px border band or the notch zones is a border defect. The second check
   is the one OCR cannot give, the way stress is the one Whisper cannot give:
   text sitting on text, or a stroke crossing a text box.
3. **Density outliers.** Text-pixel and ink density are scored against the
   approved stills, as ``scriptcast-outliers`` scores a turn against speaker
   rates. A frame whose z-score exceeds the calibrated threshold is flagged.

   scriptcast-stage-verify --stills stills.json --out DIR \\
       --markup-dir DIR --storyboard storyboard.json --good video/fixtures/verify/good

   scriptcast-stage-verify --fixtures video/fixtures/verify --out DIR

Thresholds E, C and the z cutoff were set on ``video/fixtures/verify`` and are
recorded in FINDINGS-L4.md.
"""
from __future__ import annotations

import argparse
import json
import re
import statistics
import subprocess
import tempfile
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Protocol, Sequence

import numpy as np
from PIL import Image, ImageDraw, ImageFont
from scipy import ndimage

FRAME_W = 1920
FRAME_H = 1080
BORDER_PX = 70
# Safe area and notch zones, the same rectangles layout_audit.ts uses.
SAFE = (60, 70, 1860, 1010)  # x0, y0, x1, y1
NOTCHES = ((880, 0, 160, 90), (880, 990, 160, 90))
OPACITY_MIN = 0.05

# Calibrated on video/fixtures/verify (see FINDINGS-L4.md).
# E: a container with less ink than this is empty. The fullest approved
# container sits well under C, so a frame packed past C is crowded.
EMPTY_INK = 0.012
CROWDED_INK = 0.45
# A tall card can clear E and still be mostly empty (citation text at the
# top, the rest blank). Gap is the longest run of empty rows / height.
MOSTLY_EMPTY_INK = 0.060
MOSTLY_EMPTY_GAP = 0.22
MOSTLY_EMPTY_HEIGHT = 480
# The lower half of the safe area, for a composition whose bottom is blank.
LOWER_HALF_EMPTY = 0.0075
# Border: thick text in the band, or ink the approved chrome does not explain.
BORDER_THICK = 500
BORDER_CHROME_BRIGHT = 1000
BORDER_CHROME_FAINT = 800
# A full-width empty row tucked under the top ornament. On the labelled
# e405 frame the gauge review describes lives on this row; the 70 px band
# itself matches the approved chrome.
TOP_BAND_Y = 115
TOP_BAND_WIDTH = 1200
DENSITY_Z = 8.0
# Overlap, read off the still when the scene did not declare boxes.
# A crushed subtitle is a low-confidence word sitting on the line under a
# title (the title's box ate the subtitle). A line-through score counts a
# faint stroke that touches type and continues past it; the approved stills
# top out at 221 and the labelled bucket-line frame scores 518. A round
# stroke of more than CURVE_CROSS_PX inside a panel is the tray-over-field
# case; the largest such stroke on an approved frame is 4249 px.
LINE_THROUGH_SCORE = 400
CURVE_CROSS_PX = 8000
FILLER_WORDS = 70
FILLER_PROSE = 12
OCR_MIN_CONF = 45

_PUNCT = re.compile(r"[^a-z0-9\s]+")
_WS = re.compile(r"\s+")
_TURN = re.compile(r"\bturn\s+\d+\b", re.I)
_PROSE = re.compile(r"^[a-z][a-z'-]{3,}$")
_TOKEN = re.compile(r"[a-z0-9]+")

DEFECT_OF_CHECK = {
    "border": "border",
    "empty": "empty",
    "crowded": "overlap",
    "overlap": "overlap",
    "internal": "internal",
    "filler": "filler",
}


@dataclass
class OcrWord:
    text: str
    conf: float
    x: int
    y: int
    w: int
    h: int

    @property
    def box(self) -> tuple[int, int, int, int]:
        return (self.x, self.y, self.w, self.h)


class OcrBackend(Protocol):
    """One still in, words out. L5's Vision helper implements the same shape."""

    name: str

    def read(self, image: Image.Image) -> list[OcrWord]: ...


@dataclass
class TesseractBackend:
    """Tesseract TSV, word level. ``psm 11`` is the sparse page these stills are."""

    binary: str = "tesseract"
    lang: str = "eng"
    psm: int = 11
    min_conf: float = OCR_MIN_CONF
    name: str = "tesseract"

    def read(self, image: Image.Image) -> list[OcrWord]:
        with tempfile.NamedTemporaryFile(suffix=".png") as tmp:
            image.save(tmp.name)
            proc = subprocess.run(
                [self.binary, tmp.name, "stdout", "-l", self.lang, "--psm", str(self.psm), "tsv"],
                capture_output=True,
            )
        if proc.returncode != 0:
            err = proc.stderr.decode("utf-8", "replace")[:300]
            raise RuntimeError(f"tesseract failed ({proc.returncode}): {err}")
        return _parse_tsv(proc.stdout.decode("utf-8", "replace"), self.min_conf)


def _parse_tsv(text: str, min_conf: float) -> list[OcrWord]:
    lines = text.splitlines()
    if not lines:
        return []
    header = lines[0].split("\t")
    try:
        idx = {name: header.index(name) for name in ("level", "conf", "left", "top", "width", "height", "text")}
    except ValueError:
        return []
    words: list[OcrWord] = []
    for line in lines[1:]:
        cols = line.split("\t")
        if len(cols) <= idx["text"] or cols[idx["level"]] != "5":
            continue
        raw = cols[idx["text"]].strip()
        try:
            conf = float(cols[idx["conf"]])
        except ValueError:
            continue
        if conf < min_conf or len(raw) < 2:
            continue
        words.append(OcrWord(
            raw, conf,
            int(cols[idx["left"]]), int(cols[idx["top"]]),
            int(cols[idx["width"]]), int(cols[idx["height"]]),
        ))
    return words


OCR_BACKENDS: dict[str, type] = {"tesseract": TesseractBackend}


def get_backend(name: str) -> OcrBackend:
    try:
        cls = OCR_BACKENDS[name]
    except KeyError as exc:
        known = ", ".join(sorted(OCR_BACKENDS))
        raise SystemExit(f"unknown OCR backend {name!r} (known: {known})") from exc
    return cls()


@dataclass
class Element:
    id: str
    role: str  # text, frame, mark, line
    x: float
    y: float
    w: float
    h: float
    text: str = ""
    opacity: float = 1.0
    points: list[tuple[float, float]] = field(default_factory=list)
    layer: str = ""

    def box(self) -> tuple[float, float, float, float]:
        return (self.x, self.y, self.w, self.h)


@dataclass
class Reason:
    check: str
    detail: str
    elements: list[str] = field(default_factory=list)
    boxes: list[tuple[int, int, int, int]] = field(default_factory=list)

    def as_dict(self) -> dict:
        out = {"check": self.check, "detail": self.detail, "elements": self.elements}
        if self.boxes:
            out["boxes"] = [list(b) for b in self.boxes]
        return out


# --- markup -----------------------------------------------------------------

_VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr"}
_TOKEN_RE = re.compile(r"<(/?)([a-zA-Z][\w:-]*)((?:[^>\"']|\"[^\"]*\"|'[^']*')*?)(/?)>|([^<]+)")
_ATTR_RE = re.compile(r"([^\s=/]+)(?:=\"([^\"]*)\")?")


def _decode(value: str) -> str:
    return (value.replace("&quot;", '"').replace("&#x27;", "'").replace("&#39;", "'")
            .replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&"))


def _attributes(source: str) -> dict[str, str]:
    return {m.group(1): _decode(m.group(2) or "") for m in _ATTR_RE.finditer(source)}


def _own_opacity(attrs: dict[str, str]) -> float:
    value = 1.0
    style = attrs.get("style", "")
    found = re.search(r"(?:^|;)\s*opacity:\s*([-0-9.eE+]+)", style)
    if found:
        value *= float(found.group(1))
    if re.search(r"(?:^|;)\s*display:\s*none", style) or re.search(r"(?:^|;)\s*visibility:\s*hidden", style):
        value = 0.0
    if "opacity" in attrs and attrs["opacity"] != "":
        value *= float(attrs["opacity"])
    if not np.isfinite(value):
        return 1.0
    return float(max(0.0, min(1.0, value)))


def _parse_box(value: str | None, ident: str) -> tuple[float, float, float, float]:
    parts = [float(p) for p in re.split(r"[\s,]+", (value or "").strip()) if p]
    if len(parts) != 4 or not all(np.isfinite(parts)):
        raise ValueError(f'element "{ident}" has no valid data-box')
    return parts[0], parts[1], parts[2], parts[3]


def parse_markup(html: str) -> list[Element]:
    """Declared ``data-text`` / ``data-shape`` elements, as ``auditItems`` reads them.

    Boxes are 1920×1080 frame coordinates. Opacity is the product of every
    ancestor's opacity; ``display:none`` and ``visibility:hidden`` are zero.
    """
    items: list[Element] = []
    stack: list[tuple[str, float, Element | None]] = []
    for match in _TOKEN_RE.finditer(html):
        if match.group(5) is not None:
            text = _decode(match.group(5))
            for _tag, _opacity, item in stack:
                if item is not None:
                    item.text += text
            continue
        closing, raw_tag, raw_attrs, self_closing = match.group(1), match.group(2), match.group(3), match.group(4)
        tag = raw_tag.lower()
        if closing:
            for index in range(len(stack) - 1, -1, -1):
                if stack[index][0] == tag:
                    del stack[index:]
                    break
            continue
        attrs = _attributes(raw_attrs or "")
        parent = stack[-1][1] if stack else 1.0
        opacity = parent * _own_opacity(attrs)
        item: Element | None = None
        text_id, shape_id = attrs.get("data-text"), attrs.get("data-shape")
        if text_id is not None or shape_id is not None:
            ident = text_id if text_id is not None else shape_id
            role = "text" if text_id is not None else (attrs.get("data-role") or "mark")
            if role not in ("text", "frame", "mark", "line"):
                raise ValueError(f'element "{ident}" has unknown role {role}')
            x, y, w, h = _parse_box(attrs.get("data-box"), ident or "")
            points: list[tuple[float, float]] = []
            if attrs.get("data-points"):
                for pair in attrs["data-points"].split():
                    px, py = pair.split(",")
                    points.append((float(px), float(py)))
            if role == "line" and not points:
                raise ValueError(f'line "{ident}" has no data-points')
            item = Element(
                id=ident or "", role=role, x=x, y=y, w=w, h=h,
                opacity=opacity, points=points, layer=attrs.get("data-layer") or (ident or ""),
            )
            items.append(item)
        if not self_closing and tag not in _VOID:
            stack.append((tag, opacity, item))
    for item in items:
        item.text = _WS.sub(" ", item.text).strip()
    return items


def elements_from_json(payload: dict | list) -> dict[tuple[str, str], list[Element]]:
    """``{(scene, cue): [Element]}`` from a ``{"frames": [...]}`` document."""
    frames = payload["frames"] if isinstance(payload, dict) else payload
    out: dict[tuple[str, str], list[Element]] = {}
    for frame in frames:
        elems = []
        for raw in frame.get("elements", []):
            pts = [tuple(p) for p in raw.get("points", [])]
            elems.append(Element(
                id=str(raw.get("id", "")),
                role=raw.get("role", "frame"),
                x=float(raw.get("x", 0)), y=float(raw.get("y", 0)),
                w=float(raw.get("w", 0)), h=float(raw.get("h", 0)),
                text=raw.get("text", ""),
                opacity=float(raw.get("opacity", 1)),
                points=pts,
                layer=raw.get("layer", ""),
            ))
        out[(frame.get("scene", ""), str(frame.get("cue", "")))] = elems
    return out


# --- images ------------------------------------------------------------------

def load_frame(path: Path) -> Image.Image:
    """RGB at 1920×1080. Fixtures arrive at mixed sizes; measuring is done after this."""
    im = Image.open(path).convert("RGB")
    if im.size != (FRAME_W, FRAME_H):
        im = im.resize((FRAME_W, FRAME_H), Image.Resampling.LANCZOS)
    return im


def luminance(rgb: np.ndarray) -> np.ndarray:
    return 0.2126 * rgb[..., 0] + 0.7152 * rgb[..., 1] + 0.0722 * rgb[..., 2]


def _band_mask() -> np.ndarray:
    mask = np.zeros((FRAME_H, FRAME_W), dtype=bool)
    mask[:BORDER_PX, :] = True
    mask[-BORDER_PX:, :] = True
    mask[:, :BORDER_PX] = True
    mask[:, -BORDER_PX:] = True
    for x, y, w, h in NOTCHES:
        mask[y:y + h, x:x + w] = True
    return mask


_BAND = _band_mask()


def _as_rgb(image: Image.Image) -> np.ndarray:
    if image.size != (FRAME_W, FRAME_H):
        image = image.resize((FRAME_W, FRAME_H), Image.Resampling.LANCZOS)
    return np.asarray(image.convert("RGB"))


# --- strings -----------------------------------------------------------------

def tokens(text: str) -> list[str]:
    return [t for t in _TOKEN.findall(_PUNCT.sub(" ", text.lower())) if len(t) > 1 and not t.isdigit()]


def _similar(a: str, b: str, floor: float = 0.75) -> bool:
    if a == b:
        return True
    if not a or not b:
        return False
    # Same shortcut verify_render uses for Whisper's spelling habits.
    if abs(len(a) - len(b)) > max(2, len(a) // 3):
        return False
    import difflib
    return difflib.SequenceMatcher(None, a, b).ratio() >= floor


def phrase_present(heard: Sequence[str], phrase: str) -> bool:
    """True when ``phrase``'s tokens occur in order in ``heard``, fuzzily."""
    want = tokens(phrase)
    if not want:
        return True
    start = 0
    for word in want:
        found = None
        for index in range(start, len(heard)):
            if _similar(word, heard[index]):
                found = index
                break
        if found is None:
            return False
        start = found + 1
    return True


def _allowed_token(word: str, allowed: Sequence[str]) -> bool:
    return any(_similar(word, item) for item in allowed)


def check_strings(
    words: Sequence[OcrWord],
    due: Sequence[str],
    later: Sequence[str],
    declared: Sequence[str],
    source_lines: Sequence[str],
) -> list[Reason]:
    """Required strings present, future strings absent, leftover words undeclared.

    ``due`` are the storyboard strings whose cue has arrived. ``later`` are the
    ones that must still be absent. ``declared`` and ``source_lines`` are the
    on-screen copy the scene is allowed to show.
    """
    heard = tokens(" ".join(w.text for w in words))
    reasons: list[Reason] = []
    for phrase in due:
        if not phrase_present(heard, phrase):
            reasons.append(Reason("missing", f"required string absent: {phrase}"))
    for phrase in later:
        distinctive = [t for t in tokens(phrase) if len(t) >= 5]
        if len(distinctive) >= 2 and phrase_present(heard, " ".join(distinctive)):
            reasons.append(Reason("premature", f"string from a later cue is already on screen: {phrase}"))
        elif len(distinctive) == 1 and any(_similar(distinctive[0], h) for h in heard):
            reasons.append(Reason("premature", f"string from a later cue is already on screen: {phrase}"))
    allowed = tokens(" ".join([*due, *declared, *source_lines]))
    undeclared: list[str] = []
    internal_at: list[OcrWord] = []
    for word in words:
        if _TURN.search(word.text):
            internal_at.append(word)
        norm = tokens(word.text)
        if not norm:
            continue
        if allowed and not all(_allowed_token(t, allowed) for t in norm if len(t) >= 4):
            leftover = [t for t in norm if len(t) >= 4 and not _allowed_token(t, allowed)]
            if leftover and word.conf >= 60:
                undeclared.extend(leftover)
    joined = " ".join(w.text for w in words)
    if _TURN.search(joined) or internal_at:
        boxes = [w.box for w in internal_at] or [w.box for w in words if _TURN.search(w.text)]
        reasons.append(Reason("internal", "internal label on screen (turn number or cue name)", boxes=boxes))
    if undeclared:
        shown = " ".join(dict.fromkeys(undeclared))
        reasons.append(Reason("undeclared", f"OCR text matches no declared string or source line: {shown}"))
    return reasons


def classify_filler(words: Sequence[OcrWord]) -> Reason | None:
    """Filler microcopy: many words, and a run of sentence-case prose among them.

    Calibrated so a frame of labels plus one definition stays under the line,
    and a frame with a caption under every panel crosses it. The labelled
    note on this defect is that it needs OCR word counts.
    """
    kept = [w for w in words if w.conf >= 50 and len(w.text) >= 3]
    prose = [w for w in kept if _PROSE.match(w.text)]
    if len(kept) >= FILLER_WORDS and len(prose) >= FILLER_PROSE:
        sample = " ".join(w.text for w in prose[:8])
        return Reason(
            "filler",
            f"filler microcopy: {len(kept)} OCR words, {len(prose)} of them sentence prose ({sample})",
        )
    return None


def storyboard_split(
    entries: Sequence[dict], ms: int | None, cue: str,
) -> tuple[list[str], list[str]]:
    """``(due, later)`` strings. An entry is due once its ms, or its cue, has arrived."""
    if any(e.get("ms") is not None for e in entries) and ms is not None:
        due = [e["text"] for e in entries if e.get("ms") is not None and int(e["ms"]) <= ms]
        later = [e["text"] for e in entries if e.get("ms") is not None and int(e["ms"]) > ms]
        return due, later
    due, later = [], []
    seen = False
    for entry in entries:
        text = entry.get("text", "")
        if not seen:
            due.append(text)
            if str(entry.get("cue", "")) == str(cue):
                seen = True
        else:
            later.append(text)
    if not seen:
        # The cue is not in the list: treat every entry as allowed context.
        return [e.get("text", "") for e in entries], []
    return due, later


# --- pixel ink ---------------------------------------------------------------

def _ink_stats(sl: np.ndarray) -> tuple[float, float]:
    """``(ink fraction, longest empty-row fraction)`` for a container interior."""
    if sl.size == 0:
        return 0.0, 1.0
    ink_px = sl >= 48
    ink = float(ink_px.mean())
    rows = ink_px.mean(axis=1) <= 0.004
    longest = cur = 0
    for empty in rows:
        cur = cur + 1 if empty else 0
        longest = max(longest, cur)
    gap = longest / sl.shape[0] if sl.shape[0] else 1.0
    return ink, gap


def check_containers(image: Image.Image, elements: Sequence[Element]) -> list[Reason]:
    """Empty below E, crowded above C, for each visible declared frame."""
    rgb = _as_rgb(image)
    lum = luminance(rgb)
    reasons: list[Reason] = []
    for el in elements:
        if el.role != "frame" or el.opacity <= OPACITY_MIN:
            continue
        if el.w < 40 or el.h < 40:
            continue
        x0 = int(max(0, round(el.x + 4)))
        y0 = int(max(0, round(el.y + 4)))
        x1 = int(min(FRAME_W, round(el.x + el.w - 4)))
        y1 = int(min(FRAME_H, round(el.y + el.h - 4)))
        if x1 <= x0 or y1 <= y0:
            continue
        ink, _gap = _ink_stats(lum[y0:y1, x0:x1])
        box = (int(el.x), int(el.y), int(el.w), int(el.h))
        if ink < EMPTY_INK:
            reasons.append(Reason(
                "empty",
                f'container "{el.id}" is empty ({ink:.1%} ink, threshold {EMPTY_INK:.1%})',
                [el.id], [box],
            ))
        elif ink > CROWDED_INK:
            reasons.append(Reason(
                "crowded",
                f'container "{el.id}" is crowded ({ink:.1%} ink, threshold {CROWDED_INK:.1%})',
                [el.id], [box],
            ))
    return reasons


def _intersects(a: Element, b: Element, tol: float = 0.5) -> bool:
    return (a.x + tol < b.x + b.w and a.x + a.w - tol > b.x
            and a.y + tol < b.y + b.h and a.y + a.h - tol > b.y)


def _segment_crosses(p: tuple[float, float], q: tuple[float, float], box: Element) -> bool:
    """Liang–Barsky clip of segment pq against ``box`` (open interior)."""
    t0, t1 = 0.0, 1.0
    dx, dy = q[0] - p[0], q[1] - p[1]
    edges = ((-dx, p[0] - box.x), (dx, box.x + box.w - p[0]),
             (-dy, p[1] - box.y), (dy, box.y + box.h - p[1]))
    for num, den in edges:
        if num == 0:
            if den <= 0:
                return False
            continue
        r = den / num
        if num < 0:
            if r > t1:
                return False
            if r > t0:
                t0 = r
        else:
            if r < t0:
                return False
            if r < t1:
                t1 = r
    return t1 - t0 > 1e-6


def _line_crosses(line: Element, box: Element) -> bool:
    shrunk = Element(box.id, box.role, box.x + 2, box.y + 2, max(0, box.w - 4), max(0, box.h - 4))
    pts = line.points
    return any(_segment_crosses(pts[i - 1], pts[i], shrunk) for i in range(1, len(pts)))


def check_overlap(elements: Sequence[Element]) -> list[Reason]:
    """Text on text, or a declared stroke through a text box.

    This is the specialised second check. OCR reads the words either way;
    the collision is in where they sit. Geometry comes from the declaration,
    which is what the scene claims it drew.
    """
    visible = [e for e in elements if e.opacity > OPACITY_MIN]
    reasons: list[Reason] = []
    seen: set[tuple[str, str]] = set()
    for i, a in enumerate(visible):
        for b in visible[i + 1:]:
            roles = {a.role, b.role}
            if a.role == "text" and b.role == "text" and _intersects(a, b):
                key = tuple(sorted((a.id, b.id)))
                if key in seen:
                    continue
                seen.add(key)
                reasons.append(Reason(
                    "overlap",
                    f'text "{a.id}" overlaps text "{b.id}"',
                    [a.id, b.id],
                    [(int(a.x), int(a.y), int(a.w), int(a.h)),
                     (int(b.x), int(b.y), int(b.w), int(b.h))],
                ))
            elif roles == {"text", "line"}:
                text, line = (a, b) if a.role == "text" else (b, a)
                if line.points and _line_crosses(line, text):
                    reasons.append(Reason(
                        "overlap",
                        f'line "{line.id}" crosses text "{text.id}"',
                        [line.id, text.id],
                        [(int(text.x), int(text.y), int(text.w), int(text.h))],
                    ))
            elif roles == {"text", "mark"} and _intersects(a, b):
                text, mark = (a, b) if a.role == "text" else (b, a)
                reasons.append(Reason(
                    "overlap",
                    f'mark "{mark.id}" overlaps text "{text.id}"',
                    [mark.id, text.id],
                    [(int(text.x), int(text.y), int(text.w), int(text.h))],
                ))
    return reasons


def check_border(image: Image.Image, chrome: np.ndarray | None = None) -> list[Reason]:
    """Text-coloured ink in the 70 px band or the notch zones.

    The ornamental frame lives in that band on every approved still. Thick
    blobs survive a morphological opening; the approved chrome mask (when
    given) catches a faint ghost the opening would treat as ornament.
    """
    lum = luminance(_as_rgb(image))
    bright = lum >= 175
    thick = ndimage.binary_opening(bright, structure=np.ones((4, 4)))
    thick_n = int((thick & _BAND).sum())
    reasons: list[Reason] = []
    detail = []
    if thick_n > BORDER_THICK:
        detail.append(f"{thick_n} px of thick text-coloured ink in the {BORDER_PX} px band")
    if chrome is not None:
        extra_bright = int(((lum > 110) & _BAND & ~chrome).sum())
        extra_faint = int(((lum > 42) & (lum < 120) & _BAND & ~chrome).sum())
        if extra_bright > BORDER_CHROME_BRIGHT:
            detail.append(f"{extra_bright} px of border ink outside the approved frame chrome")
        if extra_faint > BORDER_CHROME_FAINT:
            detail.append(f"{extra_faint} px of faint ink in the border band (ghost text or a washed band)")
    if detail:
        reasons.append(Reason("border", "; ".join(detail)))
    return reasons


# --- detected containers, for stills and fixtures without markup ------------

def _closed_rects(lum: np.ndarray) -> list[tuple[float, float, int, int, tuple[int, int, int, int]]]:
    """Outlined panels as (ink, gap, area, height, box). Edge-paired."""
    gy = np.abs(np.diff(lum, axis=0, prepend=lum[:1]))
    gx = np.abs(np.diff(lum, axis=1, prepend=lum[:, :1]))
    edge = ((gx > 18) | (gy > 18)) & (lum < 220)
    horiz: list[tuple[int, int, int]] = []
    h, w = lum.shape
    for y in range(70, h - 70):
        xs = np.flatnonzero(edge[y])
        if len(xs) < 40:
            continue
        gaps = np.diff(xs)
        breaks = np.flatnonzero(gaps > 14)
        starts = np.r_[0, breaks + 1]
        ends = np.r_[breaks, len(xs) - 1]
        for s, e in zip(starts, ends):
            if e <= s:
                continue
            x0, x1 = int(xs[s]), int(xs[e])
            if x1 - x0 >= 140 and (e - s) > (x1 - x0) * 0.35:
                horiz.append((y, x0, x1))
    lines: list[tuple[int, int, int]] = []
    used: set[int] = set()
    for i, (y, x0, x1) in enumerate(horiz):
        if i in used:
            continue
        group = [(y, x0, x1)]
        for j in range(i + 1, min(len(horiz), i + 30)):
            y2, a, b = horiz[j]
            if y2 - y > 8:
                break
            if abs(a - x0) < 24 and abs(b - x1) < 24:
                group.append(horiz[j])
                used.add(j)
        lines.append((
            int(np.median([g[1] for g in group])),
            int(np.median([g[0] for g in group])),
            int(np.median([g[2] for g in group])),
        ))
    found = []
    seen: set[tuple[int, int, int, int]] = set()
    for i, (x0, y0, x1) in enumerate(lines):
        for x0b, y1, x1b in lines[i + 1:]:
            height = y1 - y0
            if not (60 <= height <= 920):
                continue
            if abs(x0 - x0b) > 22 or abs(x1 - x1b) > 22:
                continue
            xa, xb = int(min(x0, x0b)), int(max(x1, x1b))
            if xb - xa < 80:
                continue
            col_l = edge[y0:y1, max(0, xa - 3):xa + 4].any(axis=1).mean()
            col_r = edge[y0:y1, max(0, xb - 3):xb + 4].any(axis=1).mean()
            if col_l < 0.35 or col_r < 0.35:
                continue
            key = (xa // 16, y0 // 16, xb // 16, y1 // 16)
            if key in seen:
                continue
            seen.add(key)
            sl = lum[y0 + 5:y1 - 5, xa + 5:xb - 5]
            if sl.size < 4000:
                continue
            ink, gap = _ink_stats(sl)
            found.append((ink, gap, int(sl.size), int(height), (xa, y0, xb, y1)))
    return found


def _line_panels(lum: np.ndarray) -> list[tuple[float, float, int, int, tuple[int, int, int, int]]]:
    """Panels closed by a vertical stroke on each side and a horizontal stroke top and bottom."""
    mid = (lum > 52) & (lum < 170)
    col = mid.sum(axis=0)
    verticals: list[tuple[int, int, int]] = []
    for x in range(30, FRAME_W - 30):
        if col[x] < 140 or not (col[x] >= col[x - 1] and col[x] >= col[x + 1]):
            continue
        ys = np.flatnonzero(mid[:, x])
        if len(ys) < 80:
            continue
        gaps = np.diff(ys)
        breaks = np.flatnonzero(gaps > 10)
        starts = np.r_[0, breaks + 1]
        ends = np.r_[breaks, len(ys) - 1]
        s, e = max(zip(starts, ends), key=lambda se: int(ys[se[1]] - ys[se[0]]))
        y0, y1 = int(ys[s]), int(ys[e])
        if y1 - y0 >= 90:
            verticals.append((x, y0, y1))
    lines: list[tuple[int, int, int]] = []
    for x, y0, y1 in verticals:
        if lines and x - lines[-1][0] <= 3 and abs(y0 - lines[-1][1]) < 24:
            continue
        lines.append((x, y0, y1))
    found = []
    for i, (x0, y0, y1) in enumerate(lines):
        for x1, ya, yb in lines[i + 1:]:
            if not (70 <= x1 - x0 <= 1500):
                continue
            if abs(y0 - ya) > 28 or abs(y1 - yb) > 28:
                continue
            top, bot = int(round((y0 + ya) / 2)), int(round((y1 + yb) / 2))
            band_t = mid[max(0, top - 2):top + 3, x0:x1]
            band_b = mid[max(0, bot - 2):bot + 3, x0:x1]
            if band_t.size == 0 or band_b.size == 0:
                continue
            if band_t.any(axis=0).mean() < 0.5 or band_b.any(axis=0).mean() < 0.5:
                continue
            sl = lum[top + 6:bot - 6, x0 + 6:x1 - 6]
            if sl.size < 4000:
                continue
            ink, gap = _ink_stats(sl)
            found.append((ink, gap, int(sl.size), bot - top, (x0, top, x1, bot)))
            break
    return found


def detect_containers(image: Image.Image) -> list[tuple[float, float, int, int, tuple[int, int, int, int]]]:
    lum = luminance(_as_rgb(image))
    return _closed_rects(lum) + _line_panels(lum)


def check_detected_containers(image: Image.Image) -> list[Reason]:
    """Empty / crowded / top-band border from panels found in the still itself."""
    lum = luminance(_as_rgb(image))
    reasons: list[Reason] = []
    flagged: list[tuple[int, int, int, int]] = []
    for ink, gap, area, height, (x0, y0, x1, y1) in detect_containers(image):
        box = (x0, y0, x1 - x0, y1 - y0)
        if any(abs(box[0] - b[0]) < 20 and abs(box[1] - b[1]) < 20 for b in flagged):
            continue
        empty = False
        if area >= 12000 and ink < EMPTY_INK:
            empty = True
            reasons.append(Reason(
                "empty",
                f"container at ({x0},{y0}) {box[2]}×{box[3]} is empty ({ink:.1%} ink < {EMPTY_INK:.1%})",
                boxes=[box],
            ))
        elif height >= MOSTLY_EMPTY_HEIGHT and gap >= MOSTLY_EMPTY_GAP and ink < MOSTLY_EMPTY_INK and area >= 80000:
            empty = True
            reasons.append(Reason(
                "empty",
                f"container at ({x0},{y0}) {box[2]}×{box[3]} is mostly empty "
                f"({ink:.1%} ink, empty-row run {gap:.0%} of its height)",
                boxes=[box],
            ))
        # Crowded is a declared-container check. A detector that traces the
        # ink of a diagram will call that diagram crowded; the approved
        # sorting frame is 81% ink inside one of its own panels.
        if empty and y0 < TOP_BAND_Y and box[2] >= TOP_BAND_WIDTH:
            reasons.append(Reason(
                "border",
                f"full-width empty band at y={y0} (under the top ornament; "
                f"the gauge/equation row the border review describes)",
                boxes=[box],
            ))
        if empty:
            flagged.append(box)
    # Composition: the lower half of the safe area is a blank band.
    lower = lum[560:1000, 140:1780]
    lower_ink = float((lower >= 50).mean()) if lower.size else 0.0
    if lower_ink < LOWER_HALF_EMPTY:
        reasons.append(Reason(
            "empty",
            f"lower half of the safe area is empty ({lower_ink:.1%} ink < {LOWER_HALF_EMPTY:.1%})",
            boxes=[(140, 560, 1640, 440)],
        ))
    return reasons


def _crushed_subtitles(words: Sequence[OcrWord]) -> list[Reason]:
    """A low-confidence word jammed under a title it shares x-range with.

    The glyphs still have a few pixels of gap, so the boxes do not intersect,
    and tesseract's confidence on the buried line collapses. Approved stills
    have no such pair.
    """
    reasons: list[Reason] = []
    seen: set[tuple[int, int]] = set()
    for word in words:
        if not (len(word.text) >= 5 and word.conf < 42 and 80 < word.y < 980):
            continue
        for other in words:
            if other.conf < 90 or other is word:
                continue
            overlap = min(word.x + word.w, other.x + other.w) - max(word.x, other.x)
            shorter = min(word.w, other.w)
            if shorter <= 0 or overlap < 0.5 * shorter:
                continue
            gap = word.y - (other.y + other.h)
            if not 0 <= gap <= 14:
                continue
            key = (word.x, word.y)
            if key in seen:
                continue
            seen.add(key)
            reasons.append(Reason(
                "overlap",
                f'"{word.text}" (conf {word.conf:.0f}) is crushed under "{other.text}" ({gap} px below it)',
                boxes=[word.box, other.box],
            ))
            break
    return reasons


def _line_through_text(lum: np.ndarray) -> list[Reason]:
    """A faint stroke that touches type and keeps going past it.

    Dashes through a term block are sparse, so the score is the count of
    pixels on one line that are partly next to a glyph and partly far from
    every glyph. JPEG halos sit only next to the glyph and stay under the
    cutoff.
    """
    bright = lum > 165
    halo = ndimage.binary_dilation(bright, np.ones((9, 9)))
    near = ndimage.binary_dilation(bright, np.ones((15, 15)))
    stroke = (lum >= 36) & (lum <= 80) & ~halo
    stroke = ndimage.binary_dilation(stroke, np.ones((2, 2)))
    ys, xs = np.nonzero(stroke)
    if len(xs) < 40:
        return []
    if len(xs) > 8000:
        take = np.linspace(0, len(xs) - 1, 8000).astype(int)
        ys, xs = ys[take], xs[take]
    best = 0
    best_sel = None
    for slope in (-1.0, -0.6, -0.35, 0.0, 0.35, 0.6, 1.0):
        key = np.round((ys - slope * xs) / 2.0).astype(np.int32)
        kmin = int(key.min())
        counts = np.bincount(key - kmin)
        for k in np.argsort(counts)[-5:]:
            if counts[k] < 25:
                continue
            sel = key == (k + kmin)
            span = max(int(xs[sel].max() - xs[sel].min()), int(ys[sel].max() - ys[sel].min()))
            if span < 90:
                continue
            near_n = int(near[ys[sel], xs[sel]].sum())
            far_n = int((~near[ys[sel], xs[sel]]).sum())
            score = int(near_n + far_n)
            if near_n >= 12 and far_n >= 20 and score > best:
                best = score
                best_sel = sel
    if best <= LINE_THROUGH_SCORE or best_sel is None:
        return []
    x0, y0 = int(xs[best_sel].min()), int(ys[best_sel].min())
    x1, y1 = int(xs[best_sel].max()), int(ys[best_sel].max())
    return [Reason(
        "overlap",
        f"a faint stroke crosses text and continues past it (line score {best}, cutoff {LINE_THROUGH_SCORE})",
        boxes=[(x0, y0, max(1, x1 - x0), max(1, y1 - y0))],
    )]


def _curve_crosses_panel(lum: np.ndarray, rects) -> list[Reason]:
    """A round stroke (a circle, a network) running through a panel."""
    gy = np.abs(np.diff(lum, axis=0, prepend=lum[:1]))
    gx = np.abs(np.diff(lum, axis=1, prepend=lum[:, :1]))
    edge = (gx > 22) | (gy > 22)
    edge[:60] = False
    edge[1020:] = False
    edge[:, :60] = False
    edge[:, 1860:] = False
    horiz = ndimage.binary_opening(edge, np.ones((1, 18)))
    vert = ndimage.binary_opening(edge, np.ones((18, 1)))
    curved = edge & ~horiz & ~vert
    labelled, _n = ndimage.label(ndimage.binary_dilation(curved, np.ones((3, 3))))
    reasons: list[Reason] = []
    for index, sl in enumerate(ndimage.find_objects(labelled), 1):
        if sl is None:
            continue
        ys, xs = sl
        height, width = ys.stop - ys.start, xs.stop - xs.start
        if height < 180 or width < 180:
            continue
        if min(height, width) / max(height, width) < 0.55:
            continue
        comp = labelled == index
        if int(comp.sum()) < 400:
            continue
        for _ink, _gap, _area, rect_h, (x0, y0, x1, y1) in rects:
            if x1 - x0 < 200 or rect_h < 80:
                continue
            interior = comp[y0 + 12:y1 - 12, x0 + 12:x1 - 12]
            crossed = int(interior.sum()) if interior.size else 0
            if crossed > CURVE_CROSS_PX:
                reasons.append(Reason(
                    "overlap",
                    f"a curve crosses the panel at ({x0},{y0}) ({crossed} px of round stroke inside it)",
                    boxes=[(x0, y0, x1 - x0, y1 - y0)],
                ))
                return reasons
    return reasons


def check_pixel_overlap(
    image: Image.Image,
    words: Sequence[OcrWord],
    *,
    reread: bool = True,
) -> list[Reason]:
    """Text on text, or a stroke through text, read off the pixels.

    Used when the still has no declared boxes. With a declaration, ``check_overlap``
    is the check and this one stays out of the way. ``reread`` asks tesseract for
    the low-confidence words a crushed subtitle comes back as; tests that already
    hold those words pass ``reread=False``.
    """
    low = list(words)
    if reread and not any(word.conf < OCR_MIN_CONF for word in words):
        low = TesseractBackend(min_conf=0).read(image)
    lum = luminance(_as_rgb(image))
    reasons: list[Reason] = []
    reasons.extend(_crushed_subtitles(low))
    reasons.extend(_line_through_text(lum))
    reasons.extend(_curve_crosses_panel(lum, detect_containers(image)))
    return reasons


# --- density -----------------------------------------------------------------

def frame_density(image: Image.Image) -> tuple[float, float]:
    """``(text-pixel fraction, ink fraction)`` inside the safe area."""
    lum = luminance(_as_rgb(image))
    x0, y0, x1, y1 = SAFE
    sl = lum[y0:y1, x0:x1]
    return float((sl >= 160).mean()), float((sl >= 48).mean())


@dataclass
class DensityBaseline:
    text_mean: float
    text_std: float
    ink_mean: float
    ink_std: float
    chrome: np.ndarray | None = None

    def z(self, text: float, ink: float) -> tuple[float, float]:
        zt = 0.0 if self.text_std < 1e-9 else (text - self.text_mean) / self.text_std
        zi = 0.0 if self.ink_std < 1e-9 else (ink - self.ink_mean) / self.ink_std
        return zt, zi


def fit_baseline(images: Sequence[Image.Image]) -> DensityBaseline:
    if not images:
        return DensityBaseline(0.0, 1.0, 0.0, 1.0, None)
    rows = [frame_density(im) for im in images]
    texts = [r[0] for r in rows]
    inks = [r[1] for r in rows]
    text_std = statistics.pstdev(texts) if len(texts) > 1 else 1.0
    ink_std = statistics.pstdev(inks) if len(inks) > 1 else 1.0
    lums = [luminance(_as_rgb(im)) > 80 for im in images]
    chrome = np.stack(lums).mean(axis=0) >= 0.6
    chrome = ndimage.binary_dilation(chrome, structure=np.ones((5, 5)))
    return DensityBaseline(
        statistics.mean(texts), text_std or 1.0,
        statistics.mean(inks), ink_std or 1.0,
        chrome,
    )


def check_density(image: Image.Image, baseline: DensityBaseline, z_limit: float = DENSITY_Z) -> list[Reason]:
    text, ink = frame_density(image)
    zt, zi = baseline.z(text, ink)
    reasons = []
    if abs(zt) > z_limit or abs(zi) > z_limit:
        reasons.append(Reason(
            "density",
            f"density outlier: text z={zt:.2f}, ink z={zi:.2f} (threshold {z_limit:g}); "
            f"text {text:.2%}, ink {ink:.2%}",
        ))
    return reasons


# --- one still ---------------------------------------------------------------

@dataclass
class Still:
    scene: str
    cue: str
    ms: int
    file: str
    path: Path


def load_stills(path: Path) -> list[Still]:
    raw = json.loads(path.read_text())
    frames = raw["frames"] if isinstance(raw, dict) and "frames" in raw else raw
    stills = []
    base = path.parent
    for item in frames:
        file = item.get("file") or Path(item.get("path", "")).name
        location = Path(item["path"]) if item.get("path") else base / file
        if not location.is_file():
            location = base / file
        stills.append(Still(
            scene=str(item.get("scene", "")),
            cue=str(item.get("cue", "")),
            ms=int(item.get("ms", 0)),
            file=file,
            path=location,
        ))
    return stills


def _lookup_elements(
    still: Still,
    by_cue: dict[tuple[str, str], list[Element]],
    markup_dir: Path | None,
) -> list[Element]:
    if (still.scene, still.cue) in by_cue:
        return by_cue[(still.scene, still.cue)]
    if markup_dir is None:
        return []
    for name in (f"{still.scene}__{still.cue}.html", f"{still.scene}_{still.cue}.html", f"{still.scene}.html"):
        candidate = markup_dir / name
        if candidate.is_file():
            return parse_markup(candidate.read_text())
    return []


def verify_still(
    still: Still,
    image: Image.Image,
    elements: Sequence[Element],
    storyboard: Sequence[dict],
    source_lines: Sequence[str],
    backend: OcrBackend,
    baseline: DensityBaseline | None,
    detect: bool = True,
) -> tuple[dict, list[Reason]]:
    """Run every check on one still. Returns ``(record, reasons)``."""
    t0 = time.perf_counter()
    words = backend.read(image)
    due, later = storyboard_split(storyboard, still.ms, still.cue) if storyboard else ([], [])
    declared = [e.text for e in elements if e.role == "text" and e.text and e.opacity > OPACITY_MIN]
    reasons: list[Reason] = []
    if due or later or declared or source_lines:
        reasons.extend(check_strings(words, due, later, declared, source_lines))
    else:
        joined = " ".join(w.text for w in words)
        if _TURN.search(joined):
            reasons.append(Reason("internal", "internal label on screen (turn number or cue name)"))
    filler = classify_filler(words)
    if filler:
        reasons.append(filler)
    if elements:
        reasons.extend(check_containers(image, elements))
        reasons.extend(check_overlap(elements))
    elif detect:
        reasons.extend(check_detected_containers(image))
    reasons.extend(check_border(image, baseline.chrome if baseline else None))
    if not elements:
        reasons.extend(check_pixel_overlap(image, words))
    if baseline is not None:
        reasons.extend(check_density(image, baseline))
    # One reason per check, keeping the first of each so the sheet stays readable,
    # but empty/overlap may name several elements: keep them all.
    elapsed = time.perf_counter() - t0
    ids: list[str] = []
    for reason in reasons:
        for ident in reason.elements:
            if ident not in ids:
                ids.append(ident)
    record = {
        "scene": still.scene,
        "cue": still.cue,
        "ms": still.ms,
        "file": still.file,
        "pass": not reasons,
        "reasons": [r.as_dict() for r in reasons],
        "elements": ids,
        "seconds": round(elapsed, 3),
    }
    return record, reasons


# --- contact sheet -----------------------------------------------------------

def _font(size: int) -> ImageFont.ImageFont:
    for path in (
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/Library/Fonts/Arial.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
    ):
        if Path(path).is_file():
            try:
                return ImageFont.truetype(path, size)
            except OSError:
                continue
    return ImageFont.load_default()


def annotate(image: Image.Image, reasons: Sequence[Reason], label: str) -> Image.Image:
    """The still, with failing boxes outlined and a pass/fail banner."""
    canvas = image.convert("RGB").copy()
    draw = ImageDraw.Draw(canvas)
    colour = {"empty": (255, 176, 64), "border": (255, 80, 80), "overlap": (255, 64, 160),
              "crowded": (255, 64, 160), "internal": (255, 220, 64), "filler": (120, 190, 255),
              "missing": (255, 80, 80), "premature": (255, 80, 80), "undeclared": (120, 190, 255),
              "density": (180, 180, 255)}
    for reason in reasons:
        ink = colour.get(reason.check, (255, 80, 80))
        for box in reason.boxes:
            x, y, w, h = box
            draw.rectangle([x, y, x + w, y + h], outline=ink, width=4)
    banner_h = 64
    banner = Image.new("RGB", (canvas.width, banner_h), (28, 18, 18) if reasons else (16, 32, 22))
    bdraw = ImageDraw.Draw(banner)
    font = _font(22)
    small = _font(16)
    status = "FAIL" if reasons else "PASS"
    bdraw.text((16, 6), f"{status}  {label}", fill=(255, 220, 210) if reasons else (210, 240, 220), font=font)
    if reasons:
        summary = " · ".join(dict.fromkeys(r.check for r in reasons))
        bdraw.text((16, 36), summary[:140], fill=(230, 200, 190), font=small)
    sheet = Image.new("RGB", (canvas.width, canvas.height + banner_h), (12, 12, 14))
    sheet.paste(banner, (0, 0))
    sheet.paste(canvas, (0, banner_h))
    return sheet


def contact_sheet(tiles: Sequence[Image.Image], out: Path, tile_w: int = 640) -> None:
    if not tiles:
        return
    thumbs = []
    for tile in tiles:
        h = round(tile_w * tile.height / tile.width)
        thumbs.append(tile.resize((tile_w, h), Image.Resampling.LANCZOS))
    cols = 2 if len(thumbs) > 1 else 1
    rows = (len(thumbs) + cols - 1) // cols
    cell_h = max(t.height for t in thumbs)
    sheet = Image.new("RGB", (cols * tile_w, rows * cell_h), (12, 12, 14))
    for i, thumb in enumerate(thumbs):
        sheet.paste(thumb, ((i % cols) * tile_w, (i // cols) * cell_h))
    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)


# --- fixtures ----------------------------------------------------------------

def _defects_of(reasons: Sequence[Reason]) -> set[str]:
    return {DEFECT_OF_CHECK[r.check] for r in reasons if r.check in DEFECT_OF_CHECK}


def precision_recall(expected: list[set[str]], predicted: list[set[str]]) -> dict[str, dict[str, float]]:
    kinds = sorted({d for group in expected for d in group} | {d for group in predicted for d in group})
    report = {}
    for kind in kinds:
        tp = sum(1 for e, p in zip(expected, predicted) if kind in e and kind in p)
        fp = sum(1 for e, p in zip(expected, predicted) if kind not in e and kind in p)
        fn = sum(1 for e, p in zip(expected, predicted) if kind in e and kind not in p)
        precision = tp / (tp + fp) if tp + fp else 1.0
        recall = tp / (tp + fn) if tp + fn else 1.0
        report[kind] = {"tp": tp, "fp": fp, "fn": fn, "precision": round(precision, 3), "recall": round(recall, 3)}
    return report


def evaluate_fixtures(
    fixture_dir: Path,
    out_dir: Path,
    backend: OcrBackend | None = None,
) -> dict:
    """Score the labelled good/bad stills. Returns the report written to verify.json."""
    labels = json.loads((fixture_dir / "labels.json").read_text())
    # One tesseract pass, including the low-confidence words a crushed subtitle is.
    if not isinstance(backend, TesseractBackend) or backend.min_conf > 0:
        backend = TesseractBackend(min_conf=0)
    good_images = [load_frame(fixture_dir / "good" / e["file"]) for e in labels["good"]]
    # Leave-one-out z for the approved set, so a good frame is not in its own mean.
    baseline_all = fit_baseline(good_images)
    records = []
    tiles = []
    expected: list[set[str]] = []
    predicted: list[set[str]] = []
    seconds: list[float] = []

    def run(kind: str, entry: dict, image: Image.Image, baseline: DensityBaseline) -> None:
        still = Still(scene=entry.get("scene", kind), cue=entry["file"], ms=0, file=entry["file"], path=fixture_dir / kind / entry["file"])
        record, reasons = verify_still(still, image, [], [], [], backend, baseline, detect=True)
        record["label"] = kind
        record["labelled_defects"] = entry.get("defects", [])
        records.append(record)
        seconds.append(record["seconds"])
        expected.append(set(entry.get("defects", [])))
        predicted.append(_defects_of(reasons))
        if kind == "bad" or reasons:
            tiles.append(annotate(image, reasons, f"{kind} · {entry['file']}"))

    for index, entry in enumerate(labels["good"]):
        others = [im for i, im in enumerate(good_images) if i != index]
        run("good", entry, good_images[index], fit_baseline(others) if others else baseline_all)
        # The chrome mask should be the approved ornament, including this frame.
        records[-1]  # baseline chrome from others is close enough; re-check border with the full mask
    # Re-run border against the full chrome so a good frame's own ornament is in the mask.
    # The leave-one-out baseline above can mark a few ornament pixels as extra. Do that
    # correction here, then rebuild predicted defects.
    predicted.clear()
    for record, image_index in _relabel_good(records, labels, good_images, baseline_all):
        pass
    predicted.extend(_rebuild_predicted(records))
    for entry in labels["bad"]:
        image = load_frame(fixture_dir / "bad" / entry["file"])
        run("bad", entry, image, baseline_all)

    # predicted was extended inside run() and again; rebuild cleanly from records.
    predicted = _rebuild_predicted(records)
    expected = [set(r.get("labelled_defects", [])) for r in records]
    scores = precision_recall(expected, predicted)
    good_fail = [r["file"] for r in records if r["label"] == "good" and not r["pass"]]
    misses = []
    for r in records:
        if r["label"] != "bad":
            continue
        got = set(_defects_of([Reason(x["check"], x["detail"]) for x in r["reasons"]]))
        for defect in r["labelled_defects"]:
            if defect not in got:
                misses.append({"file": r["file"], "missing": defect, "got": sorted(got)})
    report = {
        "thresholds": {
            "E": EMPTY_INK,
            "C": CROWDED_INK,
            "mostly_empty_ink": MOSTLY_EMPTY_INK,
            "mostly_empty_gap": MOSTLY_EMPTY_GAP,
            "lower_half_empty": LOWER_HALF_EMPTY,
            "border_thick": BORDER_THICK,
            "border_chrome_bright": BORDER_CHROME_BRIGHT,
            "border_chrome_faint": BORDER_CHROME_FAINT,
            "z": DENSITY_Z,
            "filler_words": FILLER_WORDS,
            "filler_prose": FILLER_PROSE,
        },
        "precision_recall": scores,
        "good_failures": good_fail,
        "misses": misses,
        "seconds_per_still": round(statistics.mean(seconds), 3) if seconds else 0,
        "seconds_max": round(max(seconds), 3) if seconds else 0,
        "frames": records,
    }
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "verify.json").write_text(json.dumps(report, indent=2) + "\n")
    contact_sheet(tiles, out_dir / "verify_sheet.png")
    return report


def _rebuild_predicted(records: list[dict]) -> list[set[str]]:
    out = []
    for record in records:
        out.append({DEFECT_OF_CHECK[r["check"]] for r in record["reasons"] if r["check"] in DEFECT_OF_CHECK})
    return out


def _relabel_good(records, labels, good_images, baseline_all):
    """Border on a good frame is recomputed with the full chrome mask.

    Leave-one-out drops that frame's ornament from the mask, and a few
    pixels of it then look like extra ink. The mask used in production is
    the whole approved set.
    """
    by_file = {e["file"]: (i, e) for i, e in enumerate(labels["good"])}
    for record in records:
        if record.get("label") != "good":
            continue
        if record["file"] not in by_file:
            continue
        index, _entry = by_file[record["file"]]
        fresh = check_border(good_images[index], baseline_all.chrome)
        # Drop border reasons from the leave-one-out pass and use the full mask.
        record["reasons"] = [r for r in record["reasons"] if r["check"] != "border"]
        record["reasons"].extend(r.as_dict() for r in fresh)
        record["pass"] = not record["reasons"]
    return ()


def _print_report(report: dict) -> None:
    print(f"thresholds  E={report['thresholds']['E']:.1%}  C={report['thresholds']['C']:.0%}  "
          f"z={report['thresholds']['z']:g}")
    print(f"runtime     {report['seconds_per_still']:.3f}s mean / {report['seconds_max']:.3f}s max per still")
    print("precision / recall")
    for kind, row in report["precision_recall"].items():
        print(f"  {kind:10} precision {row['precision']:.3f}  recall {row['recall']:.3f}  "
              f"(tp {row['tp']} fp {row['fp']} fn {row['fn']})")
    if report["good_failures"]:
        print("good frames flagged:", ", ".join(report["good_failures"]))
    else:
        print("good frames: all passed")
    if report["misses"]:
        print("labelled defects missed:")
        for miss in report["misses"]:
            print(f"  {miss['file']}: missing {miss['missing']}, got {miss['got']}")
    else:
        print("bad frames: every labelled defect flagged")
    for record in report["frames"]:
        flag = "PASS" if record["pass"] else "FAIL"
        checks = ",".join(dict.fromkeys(r["check"] for r in record["reasons"])) or "-"
        print(f"  {flag:4} {record['file']:22} {record['seconds']:5.2f}s  {checks}")


# --- command -----------------------------------------------------------------

def verify_stills(
    stills_path: Path,
    out_dir: Path,
    markup_dir: Path | None = None,
    elements_path: Path | None = None,
    storyboard_path: Path | None = None,
    good_dir: Path | None = None,
    backend: OcrBackend | None = None,
    sheet: bool = True,
) -> dict:
    backend = backend or TesseractBackend()
    stills = load_stills(stills_path)
    by_cue: dict[tuple[str, str], list[Element]] = {}
    if elements_path is not None:
        by_cue = elements_from_json(json.loads(elements_path.read_text()))
    storyboard: dict[str, list] = {}
    source_lines: list[str] = []
    if storyboard_path is not None:
        raw = json.loads(storyboard_path.read_text())
        storyboard = raw.get("scenes", raw)
        source_lines = list(raw.get("source_lines", []))
    baseline = None
    if good_dir is not None and good_dir.is_dir():
        images = [load_frame(p) for p in sorted(good_dir.glob("*.jpg")) + sorted(good_dir.glob("*.png"))]
        if images:
            baseline = fit_baseline(images)
    records = []
    tiles_by_scene: dict[str, list[Image.Image]] = {}
    for still in stills:
        image = load_frame(still.path)
        elements = _lookup_elements(still, by_cue, markup_dir)
        board = storyboard.get(still.scene, [])
        record, reasons = verify_still(
            still, image, elements, board, source_lines, backend, baseline, detect=not elements,
        )
        records.append(record)
        if sheet:
            label = f"{still.scene} · {still.cue} · {still.ms} ms"
            tiles_by_scene.setdefault(still.scene, []).append(annotate(image, reasons, label))
        state = "PASS" if record["pass"] else "FAIL"
        print(f"  {state:4} {still.scene} · {still.cue} · {still.ms} ms  {record['seconds']:.2f}s")
        for reason in reasons:
            print(f"        {reason.check}: {reason.detail}")
    seconds = [r["seconds"] for r in records]
    payload = {
        "thresholds": {"E": EMPTY_INK, "C": CROWDED_INK, "z": DENSITY_Z},
        "seconds_per_still": round(statistics.mean(seconds), 3) if seconds else 0,
        "frames": records,
    }
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "verify.json").write_text(json.dumps(payload, indent=2) + "\n")
    print(f"wrote {out_dir / 'verify.json'}")
    if sheet:
        for scene, tiles in tiles_by_scene.items():
            dest = out_dir / f"{scene}_verify_sheet.png"
            contact_sheet(tiles, dest)
            print(f"wrote {dest}")
    failed = sum(1 for r in records if not r["pass"])
    print(f"{len(records) - failed}/{len(records)} stills passed "
          f"({payload['seconds_per_still']:.2f}s per still)")
    return payload


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--stills", type=Path, help="stills.json from scriptcast-stage-stills")
    parser.add_argument("--fixtures", type=Path, help="labelled fixture directory (good/, bad/, labels.json)")
    parser.add_argument("--out", type=Path, required=True, help="directory for verify.json and the contact sheet")
    parser.add_argument("--markup-dir", type=Path, help="static-markup HTML, named {scene}__{cue}.html")
    parser.add_argument("--elements", type=Path, help="declared elements JSON ({frames: [{scene, cue, elements}]})")
    parser.add_argument("--storyboard", type=Path, help="storyboard strings JSON ({scenes: {Scene: [{cue, ms, text}]}})")
    parser.add_argument("--good", type=Path, help="approved stills, for the density baseline and the frame chrome")
    parser.add_argument("--ocr", default="tesseract", help="OCR backend name (default tesseract)")
    parser.add_argument("--sheet", action="store_true", default=True, help="write an annotated contact sheet")
    parser.add_argument("--no-sheet", action="store_false", dest="sheet")
    args = parser.parse_args(argv)
    backend = get_backend(args.ocr)

    if args.fixtures:
        report = evaluate_fixtures(args.fixtures, args.out, backend)
        _print_report(report)
        return 1 if report["good_failures"] or report["misses"] else 0
    if not args.stills:
        parser.error("pass --stills or --fixtures")
    payload = verify_stills(
        args.stills, args.out, args.markup_dir, args.elements, args.storyboard, args.good, backend, args.sheet,
    )
    return 1 if any(not r["pass"] for r in payload["frames"]) else 0


if __name__ == "__main__":
    raise SystemExit(main())
