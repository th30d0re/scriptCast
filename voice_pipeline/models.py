"""Shared data models for the voice pipeline."""

from dataclasses import dataclass
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
