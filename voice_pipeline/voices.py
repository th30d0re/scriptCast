"""Voice configuration loading."""

from pathlib import Path
from typing import Any

import yaml

from voice_pipeline.models import VoiceConfig


def load_voices(path: Path) -> dict[str, VoiceConfig]:
    with path.open("r", encoding="utf-8") as file:
        data = yaml.safe_load(file) or {}

    speakers = data.get("speakers")
    if not isinstance(speakers, dict):
        raise ValueError(f"{path} must contain a 'speakers' mapping")

    voices: dict[str, VoiceConfig] = {}
    required_fields = ("kokoro_voice", "lang_code")

    for speaker_id, raw_config in speakers.items():
        if not isinstance(raw_config, dict):
            raise ValueError(f"Speaker '{speaker_id}' must be a mapping")

        missing = [field for field in required_fields if field not in raw_config]
        if missing:
            missing_list = ", ".join(missing)
            raise ValueError(f"Speaker '{speaker_id}' is missing required field(s): {missing_list}")

        config: dict[str, Any] = raw_config
        voices[speaker_id] = VoiceConfig(
            speaker_id=speaker_id,
            kokoro_voice=str(config["kokoro_voice"]),
            lang_code=str(config["lang_code"]),
            speed=float(config.get("speed", 1.0)),
        )

    return voices
