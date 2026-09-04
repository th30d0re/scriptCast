"""Transcript parsing for the voice pipeline."""

from __future__ import annotations

import hashlib
import logging
import re
from pathlib import Path

from voice_pipeline.models import Turn

_LOGGER = logging.getLogger(__name__)
_SPEAKER_RE = re.compile(
    r"^\s*(?P<display_name>.+?)\s*\((?P<timestamp>\d{1,2}:\d{2})\)\s*$"
)
_MARKDOWN_FORMATTING_RE = re.compile(r"\*\*?|\_\_?")


def _speaker_id(display_name: str) -> str:
    return "_".join(display_name.lower().split())


def _timestamp_to_ms(timestamp_mmss: str) -> int:
    minutes_text, seconds_text = timestamp_mmss.split(":", maxsplit=1)
    return int(minutes_text) * 60_000 + int(seconds_text) * 1_000


def _clean_text(raw_text: str) -> str:
    return _MARKDOWN_FORMATTING_RE.sub("", raw_text).strip()


def _compute_turn_id(
    speaker_id: str, clean_text: str, occurrence: int = 0
) -> str:
    """Stable identity for a turn: who says it and what they say.

    The timestamp is deliberately excluded. It is a source reference, not part
    of the content, and inter-turn spacing comes from --gap-ms rather than from
    these values. Including it meant that inserting one turn near the top of a
    script renumbered every timestamp below it, changed every turn_id, and
    forced a full re-synthesis of an episode where only one turn was new.

    `occurrence` disambiguates a speaker repeating a line verbatim, so ids stay
    unique without reintroducing positional coupling.
    """
    payload = f"{speaker_id}:{clean_text}"
    if occurrence:
        payload = f"{payload}:{occurrence}"
    return hashlib.sha256(payload.encode()).hexdigest()[:16]


def parse_transcript(path: Path) -> list[Turn]:
    turns: list[Turn] = []
    seen_turns: dict[tuple[str, str], int] = {}
    current_header: re.Match[str] | None = None
    current_header_line = 0
    current_body_lines: list[str] = []
    current_body_last_line = 0

    def flush_current() -> None:
        nonlocal current_header, current_header_line, current_body_lines
        nonlocal current_body_last_line

        if current_header is None:
            return

        display_name = current_header.group("display_name").strip()
        timestamp_mmss = current_header.group("timestamp")
        raw_text = "\n".join(current_body_lines).strip()
        clean_text = _clean_text(raw_text)
        speaker_id = _speaker_id(display_name)
        line_end = current_body_last_line or current_header_line
        key = (speaker_id, clean_text)
        occurrence = seen_turns.get(key, 0)
        seen_turns[key] = occurrence + 1

        turns.append(
            Turn(
                turn_index=len(turns),
                turn_id=_compute_turn_id(speaker_id, clean_text, occurrence),
                speaker_id=speaker_id,
                display_name=display_name,
                timestamp_mmss=timestamp_mmss,
                timestamp_ms=_timestamp_to_ms(timestamp_mmss),
                raw_text=raw_text,
                clean_text=clean_text,
                line_span=(current_header_line, line_end),
            )
        )

        current_header = None
        current_header_line = 0
        current_body_lines = []
        current_body_last_line = 0

    with path.open("r", encoding="utf-8") as transcript:
        for line_number, line in enumerate(transcript, start=1):
            stripped_line = line.rstrip("\n")
            speaker_match = _SPEAKER_RE.match(stripped_line)
            if speaker_match:
                flush_current()
                current_header = speaker_match
                current_header_line = line_number
                current_body_lines = []
                current_body_last_line = 0
                continue

            if current_header is None:
                if stripped_line.strip():
                    _LOGGER.warning(
                        "Skipping non-speaker preamble line %s in %s",
                        line_number,
                        path,
                    )
                continue

            current_body_lines.append(stripped_line)
            current_body_last_line = line_number

    flush_current()

    if not turns:
        raise ValueError(f"No speaker turns found in transcript: {path}")

    return turns
