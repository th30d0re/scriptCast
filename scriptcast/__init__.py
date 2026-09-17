"""MLX Local TTS Pipeline - voice_pipeline package."""

from importlib import import_module
from typing import Any

__all__ = [
    "ENGINE_REGISTRY",
    "EpisodeManifest",
    "MarkupChunk",
    "MLXKokoroEngine",
    "SegmentEntry",
    "SegmentResult",
    "SpeakerEntry",
    "TTSEngine",
    "Turn",
    "TurnEntry",
    "VoiceConfig",
    "generate_als",
    "load_voices",
    "parse_transcript",
    "process_segment",
    "tokenize_markup",
    "write_manifest",
]

_LAZY_EXPORTS = {
    "ENGINE_REGISTRY": ("voice_pipeline.engine", "ENGINE_REGISTRY"),
    "EpisodeManifest": ("voice_pipeline.models", "EpisodeManifest"),
    "MarkupChunk": ("voice_pipeline.models", "MarkupChunk"),
    "MLXKokoroEngine": ("voice_pipeline.engine", "MLXKokoroEngine"),
    "SegmentEntry": ("voice_pipeline.models", "SegmentEntry"),
    "SegmentResult": ("voice_pipeline.models", "SegmentResult"),
    "SpeakerEntry": ("voice_pipeline.models", "SpeakerEntry"),
    "TTSEngine": ("voice_pipeline.engine", "TTSEngine"),
    "Turn": ("voice_pipeline.models", "Turn"),
    "TurnEntry": ("voice_pipeline.models", "TurnEntry"),
    "VoiceConfig": ("voice_pipeline.models", "VoiceConfig"),
    "generate_als": ("voice_pipeline.als_generator", "generate_als"),
    "load_voices": ("voice_pipeline.voices", "load_voices"),
    "parse_transcript": ("voice_pipeline.parser", "parse_transcript"),
    "process_segment": ("voice_pipeline.post_processor", "process_segment"),
    "tokenize_markup": ("voice_pipeline.markup", "tokenize_markup"),
    "write_manifest": ("voice_pipeline.manifest", "write_manifest"),
}


def __getattr__(name: str) -> Any:
    if name not in _LAZY_EXPORTS:
        raise AttributeError(f"module {__name__!r} has no attribute {name!r}")

    module_name, attr_name = _LAZY_EXPORTS[name]
    value = getattr(import_module(module_name), attr_name)
    globals()[name] = value
    return value


def __dir__() -> list[str]:
    return sorted(set(globals()) | set(__all__))
