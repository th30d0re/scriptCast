"""Inline markup tokenization for transcript turns."""

from __future__ import annotations

import logging
import re

from voice_pipeline.models import MarkupChunk, Turn

_LOGGER = logging.getLogger(__name__)
_TAG_RE = re.compile(r"\[(\w+)(?::([^\]]+))?\]")
_PAUSE_RE = re.compile(r"^\s*(\d+)\s*ms\s*$", re.IGNORECASE)
_BEAT_MS = 400


def _append_speech(chunks: list[MarkupChunk], text: str) -> None:
    speech = text.strip()
    if speech:
        chunks.append(MarkupChunk(kind="speech", text=speech))


def tokenize_markup(turns: list[Turn]) -> list[Turn]:
    for turn in turns:
        chunks: list[MarkupChunk] = []
        position = 0
        found_tag = False

        for match in _TAG_RE.finditer(turn.clean_text):
            found_tag = True
            _append_speech(chunks, turn.clean_text[position : match.start()])

            tag_name = match.group(1).lower()
            tag_value = match.group(2)

            if tag_name == "pause":
                pause_match = _PAUSE_RE.match(tag_value or "")
                if pause_match:
                    chunks.append(
                        MarkupChunk(
                            kind="silence",
                            duration_ms=int(pause_match.group(1)),
                        )
                    )
                else:
                    _LOGGER.warning(
                        "Ignoring invalid pause tag in turn %s: %s",
                        turn.turn_index,
                        match.group(0),
                    )
            elif tag_name == "beat":
                chunks.append(MarkupChunk(kind="silence", duration_ms=_BEAT_MS))
            elif tag_name in {"emphasis", "tone"}:
                chunks.append(MarkupChunk(kind="annotation", tag=tag_name))
                _LOGGER.info(
                    "Ignoring no-op markup tag %s in turn %s for v1",
                    tag_name,
                    turn.turn_index,
                )
            else:
                _append_speech(chunks, match.group(0))

            position = match.end()

        if found_tag:
            _append_speech(chunks, turn.clean_text[position:])
        else:
            _append_speech(chunks, turn.clean_text)

        turn.markup_chunks = chunks

    return turns
