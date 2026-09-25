import numpy as np
import pytest

from scriptcast.tools.tapestop import apply_tapestop, stop_positions

SR = 8000


def _tone(seconds, hz=440.0):
    t = np.arange(int(seconds * SR)) / SR
    return np.sin(2 * np.pi * hz * t).astype(np.float32)


def test_positions_stop_and_consume_less_than_duration():
    offsets = stop_positions(SR, 0.6, 1.6)
    assert offsets[0] > 0 and np.all(np.diff(offsets) >= 0)
    assert offsets[-1] < 0.6
    assert abs(offsets[-1] - offsets[-2]) < 1e-3  # speed has fallen to about zero


def test_output_length_and_untouched_head_are_unchanged():
    audio = _tone(2.0)
    out, start = apply_tapestop(audio, SR, end_s=1.5, duration=0.6)
    assert len(out) == len(audio)
    assert np.array_equal(out[: int(start * SR) - 1], audio[: int(start * SR) - 1])
    assert np.abs(out[-1]) < 1e-6 or len(out) > 0


def test_pitch_falls_across_the_stop():
    audio = _tone(2.0, 440.0)
    out, start = apply_tapestop(audio, SR, end_s=1.5, duration=0.6)

    def dominant(seg):
        spectrum = np.abs(np.fft.rfft(seg * np.hanning(len(seg))))
        return np.argmax(spectrum) * SR / len(seg)

    first = out[int(start * SR): int(start * SR) + 800]
    last = out[int((start + 0.5) * SR): int((start + 0.5) * SR) + 800]
    assert dominant(first) > dominant(last) + 100


def test_rejects_when_there_is_no_room():
    with pytest.raises(ValueError):
        apply_tapestop(_tone(1.0), SR, end_s=0.2, duration=0.6)
