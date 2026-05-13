"""Post-processing helpers for synthesized audio segments."""

import hashlib
from pathlib import Path

import numpy
import soundfile
from scipy.signal import resample_poly

from voice_pipeline.models import SegmentResult

_FRAME_MS = 10
_PAD_MS = 20
_PEAK_FRACTION = 0.15
_MIN_THRESHOLD = 0.005
_MAX_TRAILING_MS = 100


def _trim_edge_silence(audio: numpy.ndarray, sample_rate: int) -> numpy.ndarray:
    if audio.size == 0:
        return audio

    frame_size = max(1, int(sample_rate * _FRAME_MS / 1000))
    frame_rms_values = []
    for start in range(0, len(audio), frame_size):
        frame = audio[start : start + frame_size]
        if frame.size == 0:
            continue
        frame_rms_values.append(float(numpy.sqrt(numpy.mean(frame**2))))

    peak_rms = max(frame_rms_values) if frame_rms_values else 0.0
    if peak_rms == 0.0:
        return numpy.array([], dtype=audio.dtype)

    threshold = max(_MIN_THRESHOLD, peak_rms * _PEAK_FRACTION)
    first_voiced = next(
        (
            frame_index
            for frame_index, frame_rms in enumerate(frame_rms_values)
            if frame_rms >= threshold
        ),
        None,
    )
    last_voiced = next(
        (
            frame_index
            for frame_index, frame_rms in reversed(list(enumerate(frame_rms_values)))
            if frame_rms >= threshold
        ),
        None,
    )
    if first_voiced is None or last_voiced is None:
        return numpy.array([], dtype=audio.dtype)

    pad_samples = int(sample_rate * _PAD_MS / 1000)
    start_sample = max(0, first_voiced * frame_size - pad_samples)
    last_voiced_sample = (last_voiced + 1) * frame_size
    end_sample = min(len(audio), last_voiced_sample + pad_samples)
    max_trailing_samples = int(sample_rate * _MAX_TRAILING_MS / 1000)
    end_sample = min(end_sample, last_voiced_sample + max_trailing_samples)
    return audio[start_sample:end_sample].astype(audio.dtype, copy=False)


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
    if audio.size == 0:
        raise ValueError("Cannot process empty audio segment")

    resampled_audio = resample_poly(audio, target_rate, source_rate)
    processed_audio = _trim_edge_silence(resampled_audio, target_rate)
    if processed_audio.size == 0:
        raise ValueError(
            "Audio segment became empty after edge trimming; refusing zero-duration output"
        )

    speaker_dir = output_path / "Samples" / "Processed" / speaker_id
    speaker_dir.mkdir(parents=True, exist_ok=True)

    wav_path = speaker_dir / f"turn_{turn_index:04d}_chunk_{chunk_index:04d}.wav"
    soundfile.write(wav_path, processed_audio, target_rate, subtype="PCM_16")

    duration_ms = int(len(processed_audio) / target_rate * 1000)
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
