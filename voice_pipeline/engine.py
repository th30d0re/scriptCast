"""TTS engine interfaces and implementations."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
import uuid
from abc import ABC, abstractmethod
from pathlib import Path
from typing import Any

import numpy

from voice_pipeline.models import VoiceConfig
from voice_pipeline.post_processor import _trim_edge_silence


class TTSEngine(ABC):

    #: Whether this engine can render a turn from an alternate, more expressive
    #: reference when the script marks it [emphasis].
    supports_expressive: bool = False
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


def _elevenlabs_api_key() -> str:
    key = os.environ.get("ELEVENLABS_API_KEY")
    if not key:
        raise SystemExit(
            "ELEVENLABS_API_KEY not set. "
            "Add it to .env or export it in your shell."
        )
    return key


class MLXKokoroEngine(TTSEngine):
    def __init__(self, model_id: str, trim_edges: bool = True) -> None:
        self.model_id = model_id
        self._pipeline: Any | None = None
        self._pipeline_lang_code = "a"
        self.trim_edges = trim_edges

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
            if self.trim_edges:
                trimmed_array = _trim_edge_silence(array, self.sample_rate)
                if trimmed_array.size == 0:
                    continue
                arrays.append(trimmed_array)
            else:
                arrays.append(array)

        if not arrays:
            return numpy.array([], dtype=numpy.float32)

        return numpy.concatenate(arrays)

    @property
    def sample_rate(self) -> int:
        return 24000


class ElevenLabsEngine(TTSEngine):
    def __init__(self, model_id: str, trim_edges: bool = True) -> None:
        self.model_id = model_id
        self.trim_edges = trim_edges
        self._client: Any | None = None

    async def load(self) -> None:
        if self._client is not None:
            return
        from elevenlabs import ElevenLabs

        self._client = ElevenLabs(api_key=_elevenlabs_api_key())

    async def synthesize_chunk(
        self, text: str, voice_config: VoiceConfig
    ) -> numpy.ndarray:
        await self.load()
        if self._client is None:
            raise RuntimeError("ElevenLabs client failed to initialize")

        voice_id = voice_config.elevenlabs_voice_id
        if not voice_id:
            raise RuntimeError(
                f"Speaker {voice_config.speaker_id!r} uses engine 'elevenlabs' "
                "but has no elevenlabs_voice_id configured."
            )

        from elevenlabs import VoiceSettings

        voice_settings = None
        if voice_config.speed != 1.0:
            voice_settings = VoiceSettings(speed=voice_config.speed)

        audio_iterator = self._client.text_to_speech.convert(
            text=text,
            voice_id=voice_id,
            model_id=self.model_id,
            output_format="pcm_48000",
            voice_settings=voice_settings,
        )

        audio_bytes = b"".join(audio_iterator)
        audio_array = numpy.frombuffer(audio_bytes, dtype=numpy.int16)
        audio_array = audio_array.astype(numpy.float32) / 32768.0

        if self.trim_edges and audio_array.size > 0:
            trimmed = _trim_edge_silence(audio_array, self.sample_rate)
            return trimmed if trimmed.size > 0 else numpy.array([], dtype=numpy.float32)

        return audio_array

    @property
    def sample_rate(self) -> int:
        return 48000


class MLXDiaEngine(TTSEngine):
    def __init__(self, model_id: str, **kwargs: Any) -> None:
        del model_id, kwargs

    async def load(self) -> None:
        raise NotImplementedError("MLXDiaEngine is not yet implemented. Use mlx_kokoro.")

    async def synthesize_chunk(self, text: str, voice_config: VoiceConfig) -> numpy.ndarray:
        raise NotImplementedError("MLXDiaEngine is not yet implemented. Use mlx_kokoro.")

    @property
    def sample_rate(self) -> int:
        return 24000


class MLXChatterboxEngine(TTSEngine):
    def __init__(self, model_id: str, trim_edges: bool = True) -> None:
        self.model_id = model_id
        self.trim_edges = trim_edges
        self._model: Any | None = None
        self._reference_cache: dict[str, Any] = {}

    async def load(self) -> None:
        if self._model is not None:
            return
        from mlx_audio.tts.models.chatterbox import Model

        self._model = Model.from_pretrained(self.model_id)

    async def synthesize_chunk(self, text: str, voice_config: VoiceConfig) -> numpy.ndarray:
        await self.load()
        if self._model is None:
            raise RuntimeError("MLX Chatterbox model failed to load")

        ref_path = voice_config.reference_audio
        if not ref_path:
            raise RuntimeError(
                f"Speaker {voice_config.speaker_id!r} uses engine 'mlx_chatterbox' "
                "but has no reference_audio configured."
            )
        ref_path = Path(ref_path).expanduser()
        if not ref_path.exists():
            raise RuntimeError(f"Reference audio not found: {ref_path}")

        speaker_id = voice_config.speaker_id
        if speaker_id not in self._reference_cache:
            from mlx_audio.audio_io import read
            import mlx.core as mx

            ref_np, sr = read(str(ref_path), always_2d=False, dtype="float32")
            if sr != self.sample_rate:
                raise RuntimeError(
                    f"Reference audio sample rate {sr} does not match "
                    f"engine sample rate {self.sample_rate}"
                )
            self._reference_cache[speaker_id] = mx.array(ref_np)

        result = next(
            self._model.generate(
                text=text,
                audio_prompt=self._reference_cache[speaker_id],
                audio_prompt_sr=self.sample_rate,
                exaggeration=voice_config.exaggeration,
                cfg_weight=voice_config.cfg_weight,
                temperature=voice_config.temperature,
                stream=False,
            )
        )
        audio = getattr(result, "audio", result)
        if audio is None:
            return numpy.array([], dtype=numpy.float32)

        array = numpy.asarray(audio)
        if array.ndim > 1:
            array = numpy.squeeze(array)
        if self.trim_edges:
            trimmed_array = _trim_edge_silence(array, self.sample_rate)
            if trimmed_array.size == 0:
                return numpy.array([], dtype=numpy.float32)
            return trimmed_array
        return array

    @property
    def sample_rate(self) -> int:
        return 24000



class OmniVoiceEngine(TTSEngine):
    """OmniVoice, driven through a persistent worker in its own virtualenv.

    OmniVoice pins torch 2.8 and transformers 5.16 against this package's torch
    2.11 and MLX stack, so it runs out of process. The worker holds the model
    across the whole render; loading it costs about a minute and doing that per
    turn would dominate the run.

    Unlike Chatterbox, this engine needs the reference transcript as well as the
    clip, and a turn marked [emphasis] is synthesised from the speaker's
    expressive reference when one is configured.
    """

    _READY_TIMEOUT_S = 600
    supports_expressive = True

    def __init__(
        self,
        model_id: str = "k2-fsa/OmniVoice",
        python: str | None = None,
        device: str = "mps",
        trim_edges: bool = True,
    ) -> None:
        self.model_id = model_id
        self.device = device
        self.trim_edges = trim_edges
        self.python = python or str(
            Path(__file__).resolve().parent.parent / ".venv-omnivoice" / "bin" / "python"
        )
        self._process: subprocess.Popen[str] | None = None
        self._scratch: tempfile.TemporaryDirectory[str] | None = None

    async def load(self) -> None:
        if self._process is not None:
            return
        if not Path(self.python).exists():
            raise RuntimeError(
                f"OmniVoice interpreter not found at {self.python}. Create it with:\n"
                "  python3 -m venv .venv-omnivoice\n"
                "  .venv-omnivoice/bin/pip install torch==2.8.0 torchaudio==2.8.0 omnivoice"
            )
        worker = Path(__file__).resolve().parent / "omnivoice_worker.py"
        self._scratch = tempfile.TemporaryDirectory(prefix="omnivoice_")
        self._process = subprocess.Popen(
            [self.python, str(worker), self.model_id, self.device],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, bufsize=1,
            cwd=str(Path(__file__).resolve().parent.parent),
        )
        assert self._process.stdout is not None
        line = self._process.stdout.readline()
        if not line:
            stderr = self._process.stderr.read() if self._process.stderr else ""
            raise RuntimeError(f"OmniVoice worker failed to start:\n{stderr[-2000:]}")
        if not json.loads(line).get("ready"):
            raise RuntimeError(f"OmniVoice worker sent an unexpected greeting: {line!r}")

    def _reference(self, voice_config: VoiceConfig, expressive: bool) -> tuple[str, str]:
        audio = voice_config.reference_audio
        text = voice_config.reference_text
        if expressive and voice_config.reference_audio_expressive:
            audio = voice_config.reference_audio_expressive
            text = voice_config.reference_text_expressive or text
        if not audio:
            raise RuntimeError(
                f"Speaker {voice_config.speaker_id!r} uses engine 'omnivoice' but has "
                "no reference_audio configured."
            )
        if not text:
            raise RuntimeError(
                f"Speaker {voice_config.speaker_id!r} has no reference_text. OmniVoice "
                "conditions on the clip and its transcript together, so the text is "
                "required and must match the clip word for word."
            )
        path = Path(audio).expanduser()
        if not path.exists():
            raise RuntimeError(f"Reference audio not found: {path}")
        return str(path), text

    async def synthesize_chunk(
        self, text: str, voice_config: VoiceConfig, expressive: bool = False
    ) -> numpy.ndarray:
        await self.load()
        if self._process is None or self._scratch is None:
            raise RuntimeError("OmniVoice worker is not running")

        ref_audio, ref_text = self._reference(voice_config, expressive)
        out_path = Path(self._scratch.name) / f"{uuid.uuid4().hex}.wav"
        request = {
            "text": text, "ref_audio": ref_audio,
            "ref_text": ref_text, "out": str(out_path),
        }
        assert self._process.stdin is not None and self._process.stdout is not None
        self._process.stdin.write(json.dumps(request) + "\n")
        self._process.stdin.flush()
        line = self._process.stdout.readline()
        if not line:
            stderr = self._process.stderr.read() if self._process.stderr else ""
            raise RuntimeError(f"OmniVoice worker died:\n{stderr[-2000:]}")
        reply = json.loads(line)
        if not reply.get("ok"):
            raise RuntimeError(f"OmniVoice synthesis failed: {reply.get('error')}")

        import soundfile

        array, _ = soundfile.read(str(out_path), dtype="float32", always_2d=False)
        out_path.unlink(missing_ok=True)
        array = numpy.asarray(array)
        if array.ndim > 1:
            array = numpy.squeeze(array)
        if self.trim_edges:
            trimmed = _trim_edge_silence(array, self.sample_rate)
            if trimmed.size == 0:
                return numpy.array([], dtype=numpy.float32)
            return trimmed
        return array

    def close(self) -> None:
        if self._process is not None:
            if self._process.stdin is not None:
                self._process.stdin.close()
            self._process.wait(timeout=30)
            self._process = None
        if self._scratch is not None:
            self._scratch.cleanup()
            self._scratch = None

    @property
    def sample_rate(self) -> int:
        return 24000


ENGINE_REGISTRY: dict[str, type[TTSEngine]] = {
    "mlx_kokoro": MLXKokoroEngine,
    "elevenlabs": ElevenLabsEngine,
    "mlx_dia": MLXDiaEngine,
    "mlx_chatterbox": MLXChatterboxEngine,
    "omnivoice": OmniVoiceEngine,
}
