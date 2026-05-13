"""Episode manifest serialization."""

from __future__ import annotations

import json
import os
from collections import OrderedDict
from dataclasses import asdict
from datetime import datetime
from pathlib import Path

from voice_pipeline.models import (
    EpisodeManifest,
    SegmentEntry,
    SegmentResult,
    SpeakerEntry,
    Turn,
    TurnEntry,
    VoiceConfig,
)

_SCHEMA_VERSION = "1.0.0"
_TARGET_SAMPLE_RATE = 48000


def _segment_relative_path(segment: SegmentResult, output_path: Path) -> str:
    return (
        segment.wav_path.resolve()
        .relative_to(output_path.resolve())
        .as_posix()
    )


def _normalize_source_file(source_file: str) -> str:
    source_path = Path(source_file)
    if not source_path.is_absolute():
        return source_path.as_posix()

    resolved_source = source_path.resolve()
    cwd = Path.cwd().resolve()
    try:
        return resolved_source.relative_to(cwd).as_posix()
    except ValueError:
        return Path(os.path.relpath(resolved_source, cwd)).as_posix()


def _speaker_entries(
    turns: list[Turn], voices: dict[str, VoiceConfig]
) -> list[SpeakerEntry]:
    speakers: OrderedDict[str, tuple[str, int]] = OrderedDict()
    for turn in turns:
        if turn.speaker_id not in speakers:
            speakers[turn.speaker_id] = (turn.display_name, 0)

        display_name, turn_count = speakers[turn.speaker_id]
        speakers[turn.speaker_id] = (display_name, turn_count + 1)

    return [
        SpeakerEntry(
            speaker_id=speaker_id,
            display_name=display_name,
            voice=voices[speaker_id].kokoro_voice,
            turn_count=turn_count,
        )
        for speaker_id, (display_name, turn_count) in speakers.items()
    ]


def _turn_non_speech_gap_ms(turn: Turn, default_gap_ms: int) -> int:
    has_speech = any(
        chunk.kind == "speech" and chunk.text and chunk.text.strip()
        for chunk in turn.markup_chunks
    )
    if has_speech:
        return 0

    silence_ms = sum(
        chunk.duration_ms or 0
        for chunk in turn.markup_chunks
        if chunk.kind == "silence"
    )
    return silence_ms + default_gap_ms


def write_manifest(
    episode_id: str,
    source_file: str,
    model_id: str,
    engine: str,
    turns: list[Turn],
    segments: list[SegmentResult],
    voices: dict[str, VoiceConfig],
    output_path: Path,
    default_gap_ms: int = 0,
) -> Path:
    segments_by_key = {
        (segment.turn_index, segment.chunk_index): segment for segment in segments
    }
    has_audio_segments = bool(segments)
    cursor_ms = 0
    turn_entries: list[TurnEntry] = []

    for turn in turns:
        segment_entries: list[SegmentEntry] = []
        turn_start_ms = 0
        turn_end_ms = 0

        if has_audio_segments:
            turn_segments = [
                segments_by_key[key]
                for key in sorted(segments_by_key)
                if key[0] == turn.turn_index
            ]

            for segment in turn_segments:
                start_ms = cursor_ms
                end_ms = start_ms + segment.duration_ms
                if not segment_entries:
                    turn_start_ms = start_ms
                turn_end_ms = end_ms
                segment_entries.append(
                    SegmentEntry(
                        chunk_index=segment.chunk_index,
                        segment_wav=_segment_relative_path(segment, output_path),
                        duration_ms=segment.duration_ms,
                        start_ms=start_ms,
                        end_ms=end_ms,
                        gap_after_ms=segment.gap_after_ms,
                        checksum=segment.checksum,
                    )
                )
                cursor_ms = end_ms + segment.gap_after_ms
            if not turn_segments:
                turn_start_ms = cursor_ms
                turn_end_ms = turn_start_ms + _turn_non_speech_gap_ms(
                    turn,
                    default_gap_ms,
                )
                cursor_ms = turn_end_ms

        turn_entries.append(
            TurnEntry(
                turn_index=turn.turn_index,
                speaker_id=turn.speaker_id,
                source_timestamp=turn.timestamp_mmss,
                segments=segment_entries,
                start_ms=turn_start_ms,
                end_ms=turn_end_ms,
            )
        )

    manifest = EpisodeManifest(
        schema_version=_SCHEMA_VERSION,
        episode_id=episode_id,
        source_file=_normalize_source_file(source_file),
        model_id=model_id,
        engine=engine,
        sample_rate=_TARGET_SAMPLE_RATE,
        created_at=datetime.utcnow().isoformat() + "Z",
        speakers=_speaker_entries(turns, voices),
        turns=turn_entries,
    )

    output_path.mkdir(parents=True, exist_ok=True)
    manifest_path = output_path / "episode_manifest.json"
    manifest_path.write_text(
        json.dumps(asdict(manifest), indent=2) + "\n",
        encoding="utf-8",
    )
    return manifest_path
