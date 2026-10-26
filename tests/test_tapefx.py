import numpy as np
import pytest

from scriptcast.tools.tapefx import SR, stop_audio, stop_offsets


def test_offsets_slow_to_a_halt_and_consume_a_third_at_power_two():
    offsets = stop_offsets(1.2, 2.0)
    assert offsets[-1] == pytest.approx(0.4, abs=0.01)
    assert np.all(np.diff(offsets) >= 0)
    assert offsets[-1] - offsets[-1000] < 0.001  # nearly stopped by the end


def test_stop_audio_start_and_pitch_falls():
    t = np.arange(4 * SR) / SR
    tone = np.sin(2 * np.pi * 440 * t).astype(np.float32)
    out, start = stop_audio(tone, end_s=3.0, duration=1.2, power=2.0)
    assert start == pytest.approx(2.6, abs=0.01)
    assert len(out) == int(1.2 * SR)

    def dominant(seg):
        spectrum = np.abs(np.fft.rfft(seg * np.hanning(len(seg))))
        return np.argmax(spectrum) * SR / len(seg)

    assert dominant(out[:4800]) > dominant(out[int(0.7 * SR): int(0.7 * SR) + 4800]) + 150


def test_no_room_raises():
    with pytest.raises(ValueError):
        stop_audio(np.zeros(SR, dtype=np.float32), end_s=0.2, duration=1.2, power=2.0)
