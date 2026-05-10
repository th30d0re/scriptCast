"""Post-processing helpers for synthesized audio segments."""

import hashlib
from pathlib import Path

import numpy
import soundfile
from scipy.signal import resample_poly

from voice_pipeline.models import SegmentResult


async def process_segment(
    audio: numpy.ndarray,
    source_rate: int,
    target_rate: int,
    output_path: Path,
    turn_index: int,
    chunk_index: int,
    speaker_id: str,
    gap_after_ms: int,
) -> SegmentResult:
    resampled_audio = resample_poly(audio, target_rate, source_rate)

    speaker_dir = output_path / "Samples" / "Processed" / speaker_id
    speaker_dir.mkdir(parents=True, exist_ok=True)

    wav_path = speaker_dir / f"turn_{turn_index:04d}_chunk_{chunk_index:04d}.wav"
    soundfile.write(wav_path, resampled_audio, target_rate, subtype="PCM_16")

    duration_ms = int(len(resampled_audio) / target_rate * 1000)
    checksum = hashlib.sha256(wav_path.read_bytes()).hexdigest()

    return SegmentResult(
        turn_index=turn_index,
        chunk_index=chunk_index,
        speaker_id=speaker_id,
        wav_path=wav_path,
        duration_ms=duration_ms,
        sample_rate=target_rate,
        gap_after_ms=gap_after_ms,
        checksum=checksum,
    )
