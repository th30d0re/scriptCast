"""MLX Local TTS Pipeline - scriptcast package."""

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
    "ENGINE_REGISTRY": ("scriptcast.engine", "ENGINE_REGISTRY"),
    "EpisodeManifest": ("scriptcast.models", "EpisodeManifest"),
    "MarkupChunk": ("scriptcast.models", "MarkupChunk"),
    "MLXKokoroEngine": ("scriptcast.engine", "MLXKokoroEngine"),
    "SegmentEntry": ("scriptcast.models", "SegmentEntry"),
    "SegmentResult": ("scriptcast.models", "SegmentResult"),
    "SpeakerEntry": ("scriptcast.models", "SpeakerEntry"),
    "TTSEngine": ("scriptcast.engine", "TTSEngine"),
    "Turn": ("scriptcast.models", "Turn"),
    "TurnEntry": ("scriptcast.models", "TurnEntry"),
    "VoiceConfig": ("scriptcast.models", "VoiceConfig"),
    "generate_als": ("scriptcast.als_generator", "generate_als"),
    "load_voices": ("scriptcast.voices", "load_voices"),
    "parse_transcript": ("scriptcast.parser", "parse_transcript"),
    "process_segment": ("scriptcast.post_processor", "process_segment"),
    "tokenize_markup": ("scriptcast.markup", "tokenize_markup"),
    "write_manifest": ("scriptcast.manifest", "write_manifest"),
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
