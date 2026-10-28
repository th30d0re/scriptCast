"""The renderer shells out to scriptcast-chunk only when a job needs it."""

from __future__ import annotations

import asyncio
import subprocess
from pathlib import Path

import numpy
import pytest

from scriptcast.long_speech import is_runaway, speak
from scriptcast.models import VoiceConfig

BINARY = Path(__file__).resolve().parents[1] / "chunk" / "target" / "debug" / "scriptcast-chunk"


@pytest.fixture(scope="module")
def chunk_bin() -> Path:
    subprocess.run(
        ["cargo", "build", "--manifest-path", str(BINARY.parents[2] / "Cargo.toml")],
        check=True,
        cwd=BINARY.parents[2].parent,
    )
    # cargo --manifest-path chunk/Cargo.toml writes chunk/target/debug/scriptcast-chunk
    built = BINARY
    assert built.is_file(), built
    return built


def test_a_stable_short_take_does_not_call_the_binary(monkeypatch) -> None:
    def explode(*_args, **_kwargs):
        raise AssertionError("scriptcast-chunk was called")

    monkeypatch.setattr("scriptcast.long_speech.chunk_binary", explode)

    class Engine:
        sample_rate = 8000
        max_chars = None

        async def synthesize_chunk(self, text, voice, expressive=False):
            del voice, expressive
            assert text == "Hello."
            return numpy.ones(800, dtype=numpy.float32) * 0.1

    voice = VoiceConfig(speaker_id="host", engine="kokoro")
    audio = asyncio.run(speak(Engine(), voice, "Hello.", expressive=False))
    assert audio.shape == (800,)


def test_a_long_job_is_split_and_crossfaded(chunk_bin, monkeypatch) -> None:
    monkeypatch.setenv("SCRIPTCAST_CHUNK", str(chunk_bin))
    seen: list[str] = []

    class Engine:
        sample_rate = 8000
        max_chars = 20

        async def synthesize_chunk(self, text, voice, expressive=False):
            del voice, expressive
            seen.append(text)
            return numpy.full(800, 0.2 if text.startswith("Mr") else 0.4, dtype=numpy.float32)

    voice = VoiceConfig(speaker_id="host", engine="kokoro")
    audio = asyncio.run(
        speak(Engine(), voice, "Mr. Smith went home. He sat.", expressive=False)
    )
    assert seen == ["Mr. Smith went home. ", "He sat."]
    # 800 + 800 - fade. 50 ms at 8 kHz is 400 frames.
    assert audio.shape == (1200,)
    assert float(audio[0]) == pytest.approx(0.2)
    assert float(audio[-1]) == pytest.approx(0.4)


def test_a_runaway_take_is_halved_at_most_twice(chunk_bin, monkeypatch) -> None:
    monkeypatch.setenv("SCRIPTCAST_CHUNK", str(chunk_bin))
    calls: list[str] = []

    class Engine:
        sample_rate = 8000
        max_chars = None

        async def synthesize_chunk(self, text, voice, expressive=False):
            del voice, expressive
            calls.append(text)
            # The first take runs away. Anything shorter is a tenth of a second.
            if text == "alpha beta gamma delta":
                return numpy.ones(160_000, dtype=numpy.float32)
            return numpy.ones(800, dtype=numpy.float32) * 0.1

    voice = VoiceConfig(speaker_id="host", engine="kokoro")
    audio = asyncio.run(
        speak(Engine(), voice, "alpha beta gamma delta", expressive=False)
    )
    assert calls == ["alpha beta gamma delta", "alpha beta ", "gamma delta"]
    assert audio.shape == (1200,)
    assert numpy.isfinite(audio).all()


def test_empty_audio_is_returned_for_the_existing_abort(monkeypatch) -> None:
    def explode(*_args, **_kwargs):
        raise AssertionError("scriptcast-chunk was called")

    monkeypatch.setattr("scriptcast.long_speech.chunk_binary", explode)

    class Engine:
        sample_rate = 8000

        async def synthesize_chunk(self, text, voice, expressive=False):
            del text, voice, expressive
            return numpy.zeros(0, dtype=numpy.float32)

    voice = VoiceConfig(speaker_id="host", engine="kokoro")
    audio = asyncio.run(speak(Engine(), voice, "hello", expressive=False))
    assert audio.size == 0


def test_a_missing_binary_fails_the_split(monkeypatch) -> None:
    monkeypatch.setenv("SCRIPTCAST_CHUNK", "/tmp/does-not-exist-scriptcast-chunk")

    class Engine:
        sample_rate = 8000
        max_chars = 5

        async def synthesize_chunk(self, text, voice, expressive=False):
            del text, voice, expressive
            return numpy.ones(80, dtype=numpy.float32)

    voice = VoiceConfig(speaker_id="host", engine="kokoro")
    with pytest.raises(RuntimeError, match="SCRIPTCAST_CHUNK is not a file"):
        asyncio.run(speak(Engine(), voice, "hello there friend", expressive=False))


def test_runaway_bounds() -> None:
    rate = 8000
    assert not is_runaway(numpy.ones(rate, dtype=numpy.float32), rate, "hello")
    assert is_runaway(numpy.ones(rate * 20, dtype=numpy.float32), rate, "hi")
    assert is_runaway(numpy.array([numpy.nan], dtype=numpy.float32), rate, "hi")
    assert not is_runaway(numpy.zeros(0, dtype=numpy.float32), rate, "hi")
