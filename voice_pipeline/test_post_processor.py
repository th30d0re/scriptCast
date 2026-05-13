import numpy
import pytest
import asyncio

from voice_pipeline.post_processor import _trim_edge_silence


def test_trim_edge_silence_preserves_internal_pauses() -> None:
    sample_rate = 1000
    tone = numpy.full(200, 0.05, dtype=numpy.float32)
    silence = numpy.zeros(500, dtype=numpy.float32)
    audio = numpy.concatenate([silence, tone, silence, tone, silence])

    trimmed = _trim_edge_silence(audio, sample_rate)

    assert len(trimmed) == 20 + 200 + 500 + 200 + 20


def test_trim_edge_silence_dynamic_threshold_high_noise_floor() -> None:
    sample_rate = 1000
    edge_noise = numpy.full(200, 0.03, dtype=numpy.float32)
    voiced = numpy.full(500, 0.3, dtype=numpy.float32)
    audio = numpy.concatenate([edge_noise, voiced, edge_noise])

    trimmed = _trim_edge_silence(audio, sample_rate)

    # With peak-relative threshold at 0.15, 0.03 tail noise is below the
    # 0.045 threshold and must be trimmed.  Total padding is 2 * 20 ms.
    assert len(trimmed) <= 500 + 2 * 20 + 10
    trailing_silence_ms = (len(trimmed) - 500 - 20) * 1000 / sample_rate
    assert trailing_silence_ms <= 120


def test_trim_edge_silence_dynamic_threshold_true_silence() -> None:
    sample_rate = 1000
    silence = numpy.zeros(200, dtype=numpy.float32)
    voiced = numpy.full(500, 0.3, dtype=numpy.float32)
    audio = numpy.concatenate([silence, voiced, silence])

    trimmed = _trim_edge_silence(audio, sample_rate)

    assert len(trimmed) == 20 + 500 + 20


def test_trim_edge_silence_trailing_noise_bounded() -> None:
    sample_rate = 1000
    leading_noise = numpy.full(100, 0.03, dtype=numpy.float32)
    voiced = numpy.full(500, 0.3, dtype=numpy.float32)
    trailing_noise = numpy.full(800, 0.03, dtype=numpy.float32)
    audio = numpy.concatenate([leading_noise, voiced, trailing_noise])

    trimmed = _trim_edge_silence(audio, sample_rate)

    trailing_near_silence = len(trimmed) - (20 + 500)
    assert trailing_near_silence <= 120


def test_trim_edge_silence_quiet_clip_preserve_falloff() -> None:
    sample_rate = 1000
    leading_speech = numpy.full(80, 0.08, dtype=numpy.float32)
    body_speech = numpy.full(300, 0.06, dtype=numpy.float32)
    low_energy_falloff = numpy.full(80, 0.02, dtype=numpy.float32)
    edge_noise = numpy.full(30, 0.01, dtype=numpy.float32)
    audio = numpy.concatenate([leading_speech, body_speech, low_energy_falloff, edge_noise])

    trimmed = _trim_edge_silence(audio, sample_rate)

    # For a quiet clip (peak ~0.08) the 0.02 falloff is above the adaptive
    # threshold and should remain voiced; only the terminal edge noise is cut.
    assert len(trimmed) >= 80 + 300 + 80


def test_trim_edge_silence_all_zero_returns_empty() -> None:
    sample_rate = 1000
    audio = numpy.zeros(500, dtype=numpy.float32)
    trimmed = _trim_edge_silence(audio, sample_rate)
    assert trimmed.size == 0


def test_process_segment_rejects_empty_audio(tmp_path) -> None:
    from voice_pipeline.post_processor import process_segment

    with pytest.raises(ValueError, match="empty audio segment"):
        asyncio.run(
            process_segment(
                numpy.array([], dtype=numpy.float32),
                source_rate=24000,
                target_rate=48000,
                output_path=tmp_path,
                turn_index=0,
                chunk_index=0,
                speaker_id="ai_1",
                gap_after_ms=0,
            )
        )
