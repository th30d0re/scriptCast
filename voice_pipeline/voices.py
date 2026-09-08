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

    for speaker_id, raw_config in speakers.items():
        if not isinstance(raw_config, dict):
            raise ValueError(f"Speaker '{speaker_id}' must be a mapping")

        config: dict[str, Any] = raw_config
        engine = str(config.get("engine", "kokoro"))

        if engine in ("kokoro", "mlx_kokoro"):
            missing = [
                field for field in ("kokoro_voice", "lang_code")
                if field not in config
            ]
            if missing:
                missing_list = ", ".join(missing)
                raise ValueError(
                    f"Speaker '{speaker_id}' (engine='{engine}') is missing "
                    f"required field(s): {missing_list}"
                )
        elif engine == "elevenlabs":
            if "elevenlabs_voice_id" not in config:
                raise ValueError(
                    f"Speaker '{speaker_id}' (engine='elevenlabs') is missing "
                    f"required field: elevenlabs_voice_id"
                )
        elif engine == "mlx_chatterbox":
            if "reference_audio" not in config:
                raise ValueError(
                    f"Speaker '{speaker_id}' (engine='mlx_chatterbox') is missing "
                    f"required field: reference_audio"
                )
        elif engine == "omnivoice":
            for field in ("reference_audio", "reference_text"):
                if field not in config:
                    raise ValueError(
                        f"Speaker '{speaker_id}' (engine='omnivoice') is missing "
                        f"required field: {field}"
                    )
            # The expressive alternate is optional, and useless half-declared.
            has_audio = "reference_audio_expressive" in config
            has_text = "reference_text_expressive" in config
            if has_audio != has_text:
                raise ValueError(
                    f"Speaker '{speaker_id}' declares only one of "
                    f"reference_audio_expressive / reference_text_expressive. "
                    f"OmniVoice needs the clip and its transcript together."
                )
        else:
            raise ValueError(
                f"Speaker '{speaker_id}' has unknown engine '{engine}'. "
                f"Supported: kokoro, mlx_kokoro, elevenlabs, mlx_chatterbox, omnivoice"
            )

        voices[speaker_id] = VoiceConfig(
            speaker_id=speaker_id,
            kokoro_voice=str(config["kokoro_voice"]) if "kokoro_voice" in config else None,
            lang_code=str(config["lang_code"]) if "lang_code" in config else None,
            speed=float(config.get("speed", 1.0)),
            engine=engine,
            name=str(config["name"]) if "name" in config else None,
            elevenlabs_voice_id=str(config["elevenlabs_voice_id"]) if "elevenlabs_voice_id" in config else None,
            reference_audio=str(config["reference_audio"]) if "reference_audio" in config else None,
            character_profile=str(config.get("character_profile", "")),
            exaggeration=float(config.get("exaggeration", 0.0)),
            temperature=float(config.get("temperature", 0.6)),
            cfg_weight=float(config.get("cfg_weight", 0.7)),
            reference_text=str(config["reference_text"]) if "reference_text" in config else None,
            reference_audio_expressive=(
                str(config["reference_audio_expressive"])
                if "reference_audio_expressive" in config else None
            ),
            reference_text_expressive=(
                str(config["reference_text_expressive"])
                if "reference_text_expressive" in config else None
            ),
        )

    return voices
