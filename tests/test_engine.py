import asyncio
from unittest.mock import MagicMock, patch

import numpy
import pytest

from scriptcast.engine import (
    ElevenLabsEngine,
    MLXChatterboxEngine,
    MLXKokoroEngine,
    PhononEngine,
    _elevenlabs_api_key,
)
from scriptcast.models import VoiceConfig


def test_synthesize_chunk_skips_silent_sub_chunks() -> None:
    """Silent sub-chunks emitted by the pipeline must be discarded, not concatenated."""
    engine = MLXKokoroEngine("dummy_model")

    class FakePipeline:
        def __call__(self, *args, **kwargs):
            yield type("Chunk", (), {"audio": numpy.zeros(1000, dtype=numpy.float32)})()
            yield type("Chunk", (), {"audio": numpy.zeros(500, dtype=numpy.float32)})()

    engine._pipeline = FakePipeline()
    engine._pipeline_lang_code = "a"

    config = VoiceConfig(
        speaker_id="ai_1",
        kokoro_voice="af_bella",
        lang_code="a",
        speed=1.0,
    )
    result = asyncio.run(engine.synthesize_chunk("hello", config))

    assert result.size == 0


def test_elevenlabs_api_key_missing_raises() -> None:
    with patch.dict("os.environ", {}, clear=True):
        with pytest.raises(SystemExit, match="ELEVENLABS_API_KEY"):
            _elevenlabs_api_key()


def test_elevenlabs_api_key_present_returns_value() -> None:
    with patch.dict("os.environ", {"ELEVENLABS_API_KEY": "sk_test_key"}):
        assert _elevenlabs_api_key() == "sk_test_key"


def test_elevenlabs_engine_synthesize_chunk() -> None:
    """ElevenLabsEngine buffers iterator output and converts to float32."""
    # Inject a fake elevenlabs module so the lazy import inside synthesize_chunk works
    import sys
    fake_elevenlabs = type(sys)("elevenlabs")
    fake_elevenlabs.VoiceSettings = lambda **kwargs: kwargs
    sys.modules["elevenlabs"] = fake_elevenlabs

    try:
        engine = ElevenLabsEngine("eleven_multilingual_v2", trim_edges=False)

        fake_client = MagicMock()
        # Simulate 48kHz PCM 16-bit: 960 samples = 20ms of audio
        fake_audio = numpy.ones(960, dtype=numpy.int16).tobytes()
        fake_client.text_to_speech.convert.return_value = iter([fake_audio])
        engine._client = fake_client

        config = VoiceConfig(
            speaker_id="host",
            engine="elevenlabs",
            elevenlabs_voice_id="VlUmeC1Uzj3NnwiVR9K9",
        )
        result = asyncio.run(engine.synthesize_chunk("hello", config))

        assert result.dtype == numpy.float32
        assert result.size == 960
        numpy.testing.assert_allclose(result, 1.0 / 32768.0, rtol=1e-6)
        fake_client.text_to_speech.convert.assert_called_once_with(
            text="hello",
            voice_id="VlUmeC1Uzj3NnwiVR9K9",
            model_id="eleven_multilingual_v2",
            output_format="pcm_48000",
            voice_settings=None,
        )
    finally:
        del sys.modules["elevenlabs"]


def test_elevenlabs_engine_missing_voice_id_raises() -> None:
    engine = ElevenLabsEngine("eleven_multilingual_v2")
    engine._client = MagicMock()

    config = VoiceConfig(
        speaker_id="host",
        engine="elevenlabs",
        elevenlabs_voice_id=None,
    )
    with pytest.raises(RuntimeError, match="elevenlabs_voice_id"):
        asyncio.run(engine.synthesize_chunk("hello", config))


def test_elevenlabs_engine_sample_rate_is_48000() -> None:
    engine = ElevenLabsEngine("eleven_multilingual_v2")
    assert engine.sample_rate == 48000


def test_mlx_chatterbox_engine_synthesize_chunk(tmp_path) -> None:
    """MLXChatterboxEngine synthesizes via reference audio and model.generate."""
    engine = MLXChatterboxEngine("dummy_model", trim_edges=False)

    # Create a dummy reference audio file
    ref_path = tmp_path / "host_reference.wav"
    ref_path.write_bytes(b"fake wav content")

    # Bypass load() and reference-cache loading by pre-seeding internals
    fake_audio = numpy.ones(960, dtype=numpy.float32)
    fake_result = type("FakeResult", (), {"audio": fake_audio})()

    class FakeModel:
        def generate(self, **kwargs):
            return iter([fake_result])

    engine._model = FakeModel()
    engine._reference_cache["host"] = object()  # any truthy value; generate() mocked anyway

    config = VoiceConfig(
        speaker_id="host",
        engine="mlx_chatterbox",
        reference_audio=str(ref_path),
        exaggeration=0.3,
    )
    result = asyncio.run(engine.synthesize_chunk("hello", config))

    assert result.dtype == numpy.float32
    assert numpy.array_equal(result, fake_audio)


def test_mlx_chatterbox_engine_missing_reference_audio_raises() -> None:
    engine = MLXChatterboxEngine("dummy_model")
    engine._model = object()  # bypass load()

    config = VoiceConfig(
        speaker_id="host",
        engine="mlx_chatterbox",
        reference_audio=None,
    )
    with pytest.raises(RuntimeError, match="reference_audio"):
        asyncio.run(engine.synthesize_chunk("hello", config))


def test_mlx_chatterbox_engine_missing_reference_file_raises(tmp_path) -> None:
    engine = MLXChatterboxEngine("dummy_model")
    engine._model = object()  # bypass load()

    config = VoiceConfig(
        speaker_id="host",
        engine="mlx_chatterbox",
        reference_audio=str(tmp_path / "nonexistent.wav"),
    )
    with pytest.raises(RuntimeError, match="Reference audio not found"):
        asyncio.run(engine.synthesize_chunk("hello", config))


def test_mlx_chatterbox_engine_sample_rate_is_24000() -> None:
    engine = MLXChatterboxEngine("dummy_model")
    assert engine.sample_rate == 24000


def test_phonon_engine_registers_voice_file_once_and_converts_speed(tmp_path) -> None:
    voice_file = tmp_path / "host.safetensors"
    voice_file.write_bytes(b"")
    engine = PhononEngine(trim_edges=False)
    engine._tts = MagicMock()  # bypass load()
    engine._tts.synth.return_value = numpy.ones(240, dtype=numpy.float32)

    config = VoiceConfig(
        speaker_id="host", engine="phonon",
        phonon_voice=str(voice_file), speed=0.8,
    )
    audio = asyncio.run(engine.synthesize_chunk("hello", config))
    asyncio.run(engine.synthesize_chunk("again", config))

    assert audio.shape == (240,)
    engine._tts.add_voice.assert_called_once_with("scriptcast_host", str(voice_file))
    kwargs = engine._tts.synth.call_args.kwargs
    assert kwargs["voice"] == "scriptcast_host"
    assert kwargs["conditions"]["duration_delta"] == pytest.approx(0.25)


def test_phonon_engine_catalog_voice_passes_name_through() -> None:
    engine = PhononEngine()
    engine._tts = MagicMock()
    engine._tts.synth.return_value = numpy.zeros(0, dtype=numpy.float32)

    config = VoiceConfig(speaker_id="host", engine="phonon", phonon_voice="Zoey")
    asyncio.run(engine.synthesize_chunk("hello", config))

    engine._tts.add_voice.assert_not_called()
    assert engine._tts.synth.call_args.kwargs == {"voice": "Zoey"}


def test_phonon_engine_missing_voice_raises(tmp_path) -> None:
    engine = PhononEngine()
    engine._tts = MagicMock()
    with pytest.raises(RuntimeError, match="phonon_voice"):
        asyncio.run(engine.synthesize_chunk(
            "hello", VoiceConfig(speaker_id="host", engine="phonon")))
    missing = VoiceConfig(
        speaker_id="host", engine="phonon",
        phonon_voice=str(tmp_path / "gone.safetensors"),
    )
    with pytest.raises(RuntimeError, match="voice file not found"):
        asyncio.run(engine.synthesize_chunk("hello", missing))


def test_phonon_engine_seed_comes_from_environment(monkeypatch) -> None:
    engine = PhononEngine()
    engine._tts = MagicMock()
    engine._tts.synth.return_value = numpy.zeros(0, dtype=numpy.float32)
    config = VoiceConfig(speaker_id="host", engine="phonon", phonon_voice="Zoey")

    monkeypatch.setenv("SCRIPTCAST_PHONON_SEED", "3")
    asyncio.run(engine.synthesize_chunk("hello", config))
    assert engine._tts.synth.call_args.kwargs == {"voice": "Zoey", "seed": 3}
