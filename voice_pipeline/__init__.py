"""MLX Local TTS Pipeline - voice_pipeline package."""

from importlib import import_module
from typing import Any

__all__ = [
    "ENGINE_REGISTRY",
    "MLXKokoroEngine",
    "SegmentResult",
    "TTSEngine",
    "VoiceConfig",
    "generate_als",
    "load_voices",
    "process_segment",
]

_LAZY_EXPORTS = {
    "ENGINE_REGISTRY": ("voice_pipeline.engine", "ENGINE_REGISTRY"),
    "MLXKokoroEngine": ("voice_pipeline.engine", "MLXKokoroEngine"),
    "SegmentResult": ("voice_pipeline.models", "SegmentResult"),
    "TTSEngine": ("voice_pipeline.engine", "TTSEngine"),
    "VoiceConfig": ("voice_pipeline.models", "VoiceConfig"),
    "generate_als": ("voice_pipeline.als_generator", "generate_als"),
    "load_voices": ("voice_pipeline.voices", "load_voices"),
    "process_segment": ("voice_pipeline.post_processor", "process_segment"),
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
