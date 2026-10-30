"""Persistent local Qwen3 VoiceDesign worker for scriptcast-tone."""

from __future__ import annotations

import contextlib
import json
import os
import sys
from pathlib import Path
from typing import Any

import numpy
import soundfile

DEFAULT_MODEL = Path(
    "/Users/emmanuel/.cache/huggingface/hub/"
    "models--mlx-community--Qwen3-TTS-12Hz-1.7B-VoiceDesign-bf16/"
    "snapshots/7d3824abff87e49756bb0f83fb5411de75d160c4"
)
SAMPLE_RATE = 24000


def _model_path() -> Path:
    value = os.environ.get("SCRIPTCAST_TONE_MODEL")
    path = Path(value).expanduser() if value else DEFAULT_MODEL
    if not path.is_dir():
        raise FileNotFoundError(f"Tone model directory not found: {path}")
    return path


def _load_model() -> Any:
    from mlx_audio.tts.utils import load

    with contextlib.redirect_stdout(sys.stderr):
        return load(_model_path())


def _allowed_languages(model: Any) -> set[str]:
    talker_config = getattr(getattr(model, "config", None), "talker_config", None)
    language_ids = getattr(talker_config, "codec_language_id", {}) or {}
    return {str(value).lower() for value in language_ids} | {"auto"}


def _request(model: Any, payload: dict[str, Any]) -> dict[str, Any]:
    text = payload.get("text")
    instruct = payload.get("instruct")
    out_value = payload.get("out")
    lang = str(payload.get("lang", "english")).lower()
    if not isinstance(text, str) or not text:
        raise ValueError("text must be a non-empty string")
    if not isinstance(instruct, str) or not instruct.strip():
        raise ValueError("instruct must be a non-empty string")
    if not isinstance(out_value, str) or not out_value:
        raise ValueError("out must be a non-empty path")
    allowed = _allowed_languages(model)
    if lang not in allowed:
        raise ValueError(
            f"unsupported language {lang!r}; allowed: {', '.join(sorted(allowed))}"
        )

    out = Path(out_value)
    out.unlink(missing_ok=True)
    tmp = out.with_name(f".{out.name}.{os.getpid()}.part")
    tmp.unlink(missing_ok=True)
    try:
        with contextlib.redirect_stdout(sys.stderr):
            results = list(
                model.generate(text=text, instruct=instruct.strip(), lang_code=lang)
            )
        if not results:
            raise RuntimeError("Tone generation returned no audio")
        rates = {int(result.sample_rate) for result in results}
        if rates != {SAMPLE_RATE}:
            found = ", ".join(str(rate) for rate in sorted(rates)) or "none"
            raise RuntimeError(
                f"Tone generation returned sample rate(s) {found}; expected {SAMPLE_RATE}"
            )
        parts = [numpy.asarray(result.audio, dtype=numpy.float32).reshape(-1) for result in results]
        audio = numpy.concatenate(parts) if len(parts) > 1 else parts[0]
        if audio.size == 0:
            raise RuntimeError("Tone generation returned empty audio")
        out.parent.mkdir(parents=True, exist_ok=True)
        soundfile.write(tmp, audio, SAMPLE_RATE, format="WAV")
        os.replace(tmp, out)
        return {
            "ok": True,
            "out": out_value,
            "samples": int(audio.size),
            "sample_rate": SAMPLE_RATE,
        }
    except Exception:
        tmp.unlink(missing_ok=True)
        out.unlink(missing_ok=True)
        raise


def main() -> int:
    try:
        model = _load_model()
    except Exception as exc:
        print(f"scriptcast-tone worker: {exc}", file=sys.stderr, flush=True)
        return 1

    print(json.dumps({"ready": True}), flush=True)
    for line in sys.stdin:
        payload: Any = None
        try:
            payload = json.loads(line)
            if not isinstance(payload, dict):
                raise ValueError("request must be a JSON object")
            reply = _request(model, payload)
        except Exception as exc:
            out_value = payload.get("out") if isinstance(payload, dict) else None
            if isinstance(out_value, str) and out_value:
                Path(out_value).unlink(missing_ok=True)
            reply = {"ok": False, "error": str(exc)}
        print(json.dumps(reply), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
