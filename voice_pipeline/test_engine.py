import asyncio

import numpy
import pytest

from voice_pipeline.engine import MLXKokoroEngine
from voice_pipeline.models import VoiceConfig


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
