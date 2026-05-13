"""TTS engine interfaces and implementations."""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any

import numpy

from voice_pipeline.models import VoiceConfig
from voice_pipeline.post_processor import _trim_edge_silence


class TTSEngine(ABC):
    @abstractmethod
    async def load(self) -> None:
        """Load the engine resources."""

    @abstractmethod
    async def synthesize_chunk(self, text: str, voice_config: VoiceConfig) -> numpy.ndarray:
        """Synthesize a single text chunk into waveform samples."""

    @property
    @abstractmethod
    def sample_rate(self) -> int:
        """Native sample rate emitted by this engine."""


class MLXKokoroEngine(TTSEngine):
    def __init__(self, model_id: str) -> None:
        self.model_id = model_id
        self._pipeline: Any | None = None
        self._pipeline_lang_code = "a"

    async def load(self) -> None:
        if self._pipeline is not None:
            return

        from mlx_audio.tts import load_model
        from mlx_audio.tts.models.kokoro import KokoroPipeline

        model = load_model(self.model_id)
        self._pipeline = KokoroPipeline(
            lang_code=self._pipeline_lang_code,
            model=model,
            repo_id=self.model_id,
        )

    async def synthesize_chunk(self, text: str, voice_config: VoiceConfig) -> numpy.ndarray:
        await self.load()
        if self._pipeline is None:
            raise RuntimeError("MLX Kokoro pipeline failed to load")

        if voice_config.lang_code != self._pipeline_lang_code:
            from mlx_audio.tts.models.kokoro import KokoroPipeline

            self._pipeline = KokoroPipeline(
                lang_code=voice_config.lang_code,
                model=self._pipeline.model,
                repo_id=self.model_id,
            )
            self._pipeline_lang_code = voice_config.lang_code

        chunks = self._pipeline(
            text,
            voice=voice_config.kokoro_voice,
            speed=voice_config.speed,
        )
        arrays = []
        for chunk in chunks:
            audio = getattr(chunk, "audio", chunk)
            if isinstance(audio, tuple) and len(audio) >= 3:
                audio = audio[2]
            if audio is None:
                continue

            array = numpy.asarray(audio)
            if array.ndim > 1:
                array = numpy.squeeze(array)
            trimmed_array = _trim_edge_silence(array, self.sample_rate)
            if trimmed_array.size == 0:
                continue
            arrays.append(trimmed_array)

        if not arrays:
            return numpy.array([], dtype=numpy.float32)

        return numpy.concatenate(arrays)

    @property
    def sample_rate(self) -> int:
        return 24000


class MLXDiaEngine(TTSEngine):
    async def load(self) -> None:
        raise NotImplementedError("MLXDiaEngine is not yet implemented. Use mlx_kokoro.")

    async def synthesize_chunk(self, text: str, voice_config: VoiceConfig) -> numpy.ndarray:
        raise NotImplementedError("MLXDiaEngine is not yet implemented. Use mlx_kokoro.")

    @property
    def sample_rate(self) -> int:
        return 24000


class MLXChatterboxEngine(TTSEngine):
    async def load(self) -> None:
        raise NotImplementedError("MLXChatterboxEngine is not yet implemented. Use mlx_kokoro.")

    async def synthesize_chunk(self, text: str, voice_config: VoiceConfig) -> numpy.ndarray:
        raise NotImplementedError("MLXChatterboxEngine is not yet implemented. Use mlx_kokoro.")

    @property
    def sample_rate(self) -> int:
        return 24000


ENGINE_REGISTRY: dict[str, type[TTSEngine]] = {
    "mlx_kokoro": MLXKokoroEngine,
    "mlx_dia": MLXDiaEngine,
    "mlx_chatterbox": MLXChatterboxEngine,
}
