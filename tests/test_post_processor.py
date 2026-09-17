import numpy
import pytest
import asyncio

from voice_pipeline.post_processor import _trim_edge_silence, _measure_speech_duration


def test_trim_edge_silence_preserves_internal_pauses() -> None:
    sample_rate = 1000
    tone = numpy.full(200, 0.05, dtype=numpy.float32)
    silence = numpy.zeros(500, dtype=numpy.float32)
    audio = numpy.concatenate([silence, tone, silence, tone, silence])

    trimmed = _trim_edge_silence(audio, sample_rate)

    assert len(trimmed) == 20 + 200 + 500 + 200 + 20


def test_trim_edge_silence_fixed_threshold_leaves_noise_floor() -> None:
    sample_rate = 1000
    edge_noise = numpy.full(200, 0.03, dtype=numpy.float32)
    voiced = numpy.full(500, 0.3, dtype=numpy.float32)
    audio = numpy.concatenate([edge_noise, voiced, edge_noise])

    trimmed = _trim_edge_silence(audio, sample_rate)

    # With fixed threshold 0.005, 0.03 tail noise is well above threshold
    # and must be preserved (only true digital silence < 0.005 is trimmed).
    assert len(trimmed) == len(audio)


def test_trim_edge_silence_fixed_threshold_true_silence() -> None:
    sample_rate = 1000
    silence = numpy.zeros(200, dtype=numpy.float32)
    voiced = numpy.full(500, 0.3, dtype=numpy.float32)
    audio = numpy.concatenate([silence, voiced, silence])

    trimmed = _trim_edge_silence(audio, sample_rate)

    assert len(trimmed) == 20 + 500 + 20


def test_trim_edge_silence_low_energy_below_threshold() -> None:
    sample_rate = 1000
    leading_noise = numpy.full(100, 0.003, dtype=numpy.float32)
    voiced = numpy.full(500, 0.3, dtype=numpy.float32)
    trailing_noise = numpy.full(800, 0.003, dtype=numpy.float32)
    audio = numpy.concatenate([leading_noise, voiced, trailing_noise])

    trimmed = _trim_edge_silence(audio, sample_rate)

    # 0.003 is below the 0.005 threshold, so it gets trimmed.
    assert len(trimmed) == 20 + 500 + 20


def test_trim_edge_silence_quiet_clip_preserve_falloff() -> None:
    sample_rate = 1000
    leading_speech = numpy.full(80, 0.08, dtype=numpy.float32)
    body_speech = numpy.full(300, 0.06, dtype=numpy.float32)
    low_energy_falloff = numpy.full(80, 0.02, dtype=numpy.float32)
    edge_noise = numpy.full(30, 0.001, dtype=numpy.float32)
    audio = numpy.concatenate([leading_speech, body_speech, low_energy_falloff, edge_noise])

    trimmed = _trim_edge_silence(audio, sample_rate)

    # 0.02 falloff is above the 0.005 fixed threshold and should remain;
    # only the terminal 0.001 edge noise is cut.
    assert len(trimmed) >= 80 + 300 + 80


def test_trim_edge_silence_all_zero_returns_empty() -> None:
    sample_rate = 1000
    audio = numpy.zeros(500, dtype=numpy.float32)
    trimmed = _trim_edge_silence(audio, sample_rate)
    assert trimmed.size == 0


def test_measure_speech_duration_finds_last_voiced_frame() -> None:
    sample_rate = 1000
    silence = numpy.zeros(100, dtype=numpy.float32)
    voiced = numpy.full(300, 0.3, dtype=numpy.float32)
    tail_noise = numpy.full(200, 0.03, dtype=numpy.float32)
    audio = numpy.concatenate([silence, voiced, tail_noise])

    speech_ms = _measure_speech_duration(audio, sample_rate)

    # With threshold 0.04, tail noise at 0.03 is below the threshold and is excluded.
    # Only the voiced section (300 ms = frames 10-39) qualifies.
    # Last qualifying frame is 39 -> (39 + 1) * 10 = 400 ms.
    assert speech_ms == 400


def test_measure_speech_duration_no_qualifying_frames_fallback() -> None:
    sample_rate = 1000
    audio = numpy.full(500, 0.01, dtype=numpy.float32)

    speech_ms = _measure_speech_duration(audio, sample_rate)

    # 0.01 is below 0.04 threshold, so fallback to full duration.
    assert speech_ms == 500


def test_measure_speech_duration_threshold_parameter() -> None:
    sample_rate = 1000
    silence = numpy.zeros(100, dtype=numpy.float32)
    voiced = numpy.full(300, 0.3, dtype=numpy.float32)
    tail_noise = numpy.full(200, 0.03, dtype=numpy.float32)
    audio = numpy.concatenate([silence, voiced, tail_noise])

    # Default threshold 0.04 excludes 0.03 tail noise.
    assert _measure_speech_duration(audio, sample_rate, 0.04) == 400

    # Lower threshold 0.02 includes the 0.03 tail noise.
    assert _measure_speech_duration(audio, sample_rate, 0.02) == 600

    # Higher threshold 0.06 still excludes tail noise, same as 0.04.
    assert _measure_speech_duration(audio, sample_rate, 0.06) == 400


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
                turn_id="id0",
                chunk_index=0,
                speaker_id="ai_1",
                gap_after_ms=0,
            )
        )
