"""Shared data models for the voice pipeline."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from pathlib import Path


@dataclass
class VoiceConfig:
    speaker_id: str
    kokoro_voice: str
    lang_code: str
    speed: float


@dataclass
class SegmentResult:
    turn_index: int
    chunk_index: int
    speaker_id: str
    wav_path: Path
    duration_ms: int
    sample_rate: int
    gap_after_ms: int
    checksum: str


@dataclass
class MarkupChunk:
    kind: str
    text: str | None = None
    duration_ms: int | None = None
    tag: str | None = None


@dataclass
class Turn:
    turn_index: int
    speaker_id: str
    display_name: str
    timestamp_mmss: str
    timestamp_ms: int
    raw_text: str
    clean_text: str
    line_span: tuple[int, int]
    markup_chunks: list[MarkupChunk] = field(default_factory=list)


@dataclass
class SegmentEntry:
    chunk_index: int
    segment_wav: str
    duration_ms: int
    start_ms: int
    end_ms: int
    gap_after_ms: int
    checksum: str


@dataclass
class TurnEntry:
    turn_index: int
    speaker_id: str
    source_timestamp: str
    segments: list[SegmentEntry] = field(default_factory=list)
    start_ms: int = 0
    end_ms: int = 0


@dataclass
class SpeakerEntry:
    speaker_id: str
    display_name: str
    voice: str
    turn_count: int


@dataclass
class EpisodeManifest:
    schema_version: str
    episode_id: str
    source_file: str
    model_id: str
    engine: str
    sample_rate: int
    created_at: str
    speakers: list[SpeakerEntry] = field(default_factory=list)
    turns: list[TurnEntry] = field(default_factory=list)

    def to_dict(self) -> dict[str, object]:
        return asdict(self)
