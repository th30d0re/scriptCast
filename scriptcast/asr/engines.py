"""Speech-recognition engine adapters for the scriptCast ASR ensemble.

Ported from videolab's ``asr.py`` (Phonon-2 windowed passes, Phonon-1-Big
coverage check, mlx-whisper / OpenAI Whisper fallback). The merge order matches
videolab; the return value is a dict suited to clip transcripts rather than a
job-directory writer.
"""

from __future__ import annotations

import difflib
import json
import os
import re
import shutil
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Callable

from scriptcast.asr.merge import (
    backfill_from_reference,
    coverage_warnings,
    drop_words_in_spans,
    fill_gaps,
    group_words_into_segments,
    normalize_tokens,
)

DEFAULT_MODEL = "mlx-community/whisper-large-v3-turbo"
PHONON_MODEL = "FermionResearch/Phonon-2"
PHONON_BIG_MODEL = "FermionResearch/Phonon-1-Big"

# Known fermion install used by TheOriginalPower live runs (read-only tree).
_THEORY_FERMION = (
    Path("/Users/emmanuel/Documents/Theory/TheOriginalPower")
    / ".venv-fermion"
    / "bin"
    / "fermion"
)


def _free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def _fermion_executable() -> str:
    for key in ("SCRIPTCAST_FERMION", "VIDEOLAB_FERMION"):
        executable = os.environ.get(key)
        if executable:
            return executable

    if _THEORY_FERMION.is_file():
        return str(_THEORY_FERMION)

    executable = shutil.which("fermion")
    if not executable:
        raise RuntimeError(
            "fermion is unavailable; set SCRIPTCAST_FERMION or VIDEOLAB_FERMION or add it to PATH"
        )
    return executable


def _wait_for_health(port: int, timeout: float = 120.0) -> None:
    deadline = time.monotonic() + timeout
    url = f"http://127.0.0.1:{port}/health"
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            with urllib.request.urlopen(url, timeout=1.0) as response:
                if response.status == 200:
                    return
        except (OSError, urllib.error.URLError) as exc:
            last_error = exc
        time.sleep(0.25)
    detail = f": {last_error}" if last_error else ""
    raise TimeoutError(f"fermion health check timed out after {timeout:g} seconds{detail}")


def _run_fermion_server(
    model: str,
    action: Callable[[int], Any],
    *,
    executable: str | None = None,
    process_factory: Callable[..., Any] = subprocess.Popen,
) -> Any:
    port = _free_port()
    process = process_factory(
        [executable or _fermion_executable(), "serve", model, "--port", str(port)],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    try:
        _wait_for_health(port)
        return action(port)
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def _post_audio(
    port: int, audio_path: Path, *, word_timestamps: bool
) -> dict[str, Any]:
    boundary = f"scriptcast-{os.getpid()}-{time.monotonic_ns()}"
    fields = [("response_format", "verbose_json")]
    if word_timestamps:
        fields.append(("timestamp_granularities[]", "word"))
    pieces: list[bytes] = []
    for name, value in fields:
        pieces.extend(
            [
                f"--{boundary}\r\n".encode(),
                f'Content-Disposition: form-data; name="{name}"\r\n\r\n'.encode(),
                value.encode(),
                b"\r\n",
            ]
        )
    pieces.extend(
        [
            f"--{boundary}\r\n".encode(),
            (
                'Content-Disposition: form-data; name="file"; '
                f'filename="{audio_path.name}"\r\n'
            ).encode(),
            b"Content-Type: audio/wav\r\n\r\n",
            audio_path.read_bytes(),
            b"\r\n",
            f"--{boundary}--\r\n".encode(),
        ]
    )
    request = urllib.request.Request(
        f"http://127.0.0.1:{port}/v1/audio/transcriptions",
        data=b"".join(pieces),
        headers={"Content-Type": f"multipart/form-data; boundary={boundary}"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=120.0) as response:
        return json.loads(response.read().decode("utf-8"))


def _response_words(response: dict[str, Any], shift: float = 0.0) -> list[dict]:
    return [
        {
            "word": str(word.get("word", "")),
            "start": float(word.get("start", 0.0)) + shift,
            "end": float(word.get("end", 0.0)) + shift,
        }
        for word in response.get("words", [])
        if str(word.get("word", "")).strip()
    ]


def _slice_audio(audio_path: Path, output: Path, start: float, duration: float) -> None:
    executable = shutil.which("ffmpeg")
    if executable is None:
        raise RuntimeError("ffmpeg is unavailable on PATH")
    completed = subprocess.run(
        [
            executable,
            "-nostdin",
            "-y",
            "-ss",
            f"{start:.3f}",
            "-i",
            str(audio_path),
            "-t",
            f"{duration:.3f}",
            "-ac",
            "1",
            "-ar",
            "16000",
            str(output),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or "ffmpeg audio slice failed")


def phonon_words(audio_path: Path) -> tuple[list[dict], list[list[dict]], float]:
    """Phonon-2 full-file primary plus 15 s windows at offsets 0, 5 and 10 s."""

    def transcribe_at_port(port: int) -> tuple[list[dict], list[list[dict]], float]:
        primary_response = _post_audio(port, audio_path, word_timestamps=True)
        primary = _response_words(primary_response)
        duration = float(primary_response.get("duration", 0.0))
        if duration <= 0.0:
            duration = max((float(word["end"]) for word in primary), default=0.0)
        passes: list[list[dict]] = []
        with tempfile.TemporaryDirectory(prefix=".phonon-windows-", dir=audio_path.parent) as temporary:
            temporary_path = Path(temporary)
            for pass_index, offset in enumerate((0.0, 5.0, 10.0)):
                pass_words: list[dict] = []
                window_index = 0
                start = offset
                while start < duration:
                    window_duration = min(15.0, duration - start)
                    window_path = temporary_path / f"pass-{pass_index}-{window_index}.wav"
                    _slice_audio(audio_path, window_path, start, window_duration)
                    response = _post_audio(port, window_path, word_timestamps=True)
                    for word in _response_words(response, shift=start):
                        midpoint = (float(word["start"]) + float(word["end"])) / 2.0
                        owned_start = 0.0 if window_index == 0 else start + 1.0
                        owned_end = duration if start + 15.0 >= duration else start + 14.0
                        if owned_start <= midpoint < owned_end:
                            pass_words.append(word)
                    start += 13.0
                    window_index += 1
                passes.append(pass_words)
        return primary, passes, duration

    return _run_fermion_server("phonon-2", transcribe_at_port)


def big_text(audio_path: Path) -> str:
    """Whole-file Phonon-1-Big text (coverage check only; no word timestamps)."""

    def transcribe_at_port(port: int) -> str:
        response = _post_audio(port, audio_path, word_timestamps=False)
        return str(response.get("text", ""))

    return _run_fermion_server("phonon-1-big", transcribe_at_port)


def _words(text: str) -> list[str]:
    return re.findall(r"[^\W_]+(?:['’][^\W_]+)*", text.casefold(), flags=re.UNICODE)


def _is_degenerate_segment(segment: dict[str, Any]) -> bool:
    words = _words(str(segment.get("text", "")))
    if not words:
        return False

    for width in range(1, min(4, len(words)) + 1):
        repetitions, remainder = divmod(len(words), width)
        if remainder == 0 and repetitions > 5:
            phrase = words[:width]
            if all(words[offset : offset + width] == phrase for offset in range(0, len(words), width)):
                return True

    duration = float(segment.get("end", 0.0)) - float(segment.get("start", 0.0))
    return duration <= 0.0 or len(words) / duration > 12.0


def _filter_degenerate_segments(result: dict[str, Any]) -> tuple[dict[str, Any], int]:
    segments = [segment for segment in result["segments"] if not _is_degenerate_segment(segment)]
    dropped_segments = len(result["segments"]) - len(segments)
    filtered = {
        **result,
        "text": " ".join(segment["text"] for segment in segments).strip(),
        "segments": segments,
    }
    return filtered, dropped_segments


def _normalized_result(raw: dict[str, Any]) -> dict[str, Any]:
    segments = [
        {
            "start": float(segment.get("start", 0.0)),
            "end": float(segment.get("end", 0.0)),
            "text": str(segment.get("text", "")).strip(),
        }
        for segment in raw.get("segments", [])
    ]
    text = str(raw.get("text", "")).strip()
    if not text:
        text = " ".join(segment["text"] for segment in segments).strip()
    language = raw.get("language")
    words = [
        {
            "word": str(word.get("word", word.get("text", ""))),
            "start": float(word.get("start", 0.0)),
            "end": float(word.get("end", 0.0)),
        }
        for word in raw.get("words", [])
        if str(word.get("word", word.get("text", ""))).strip()
    ]
    return {"text": text, "segments": segments, "language": language, "words": words}


def _mlx_transcribe(audio_path: Path, model: str) -> dict[str, Any]:
    import mlx_whisper

    return mlx_whisper.transcribe(
        str(audio_path),
        path_or_hf_repo=model,
        verbose=None,
        condition_on_previous_text=False,
        temperature=(0.0, 0.2, 0.4, 0.6, 0.8, 1.0),
        compression_ratio_threshold=2.4,
        no_speech_threshold=0.6,
    )


def _fallback_model(model: str) -> str:
    if model.endswith("whisper-large-v3-turbo"):
        return "turbo"
    return model.rsplit("/", 1)[-1].removeprefix("whisper-")


def _openai_transcribe(audio_path: Path, model: str) -> dict[str, Any]:
    executable = shutil.which("whisper")
    if executable is None:
        raise RuntimeError("OpenAI Whisper fallback is unavailable on PATH")
    with tempfile.TemporaryDirectory(prefix=".asr-work-") as temporary:
        command = [
            executable,
            str(audio_path),
            "--model",
            _fallback_model(model),
            "--device",
            "cpu",
            "--fp16",
            "False",
            "--output_dir",
            temporary,
            "--output_format",
            "json",
            "--verbose",
            "False",
        ]
        completed = subprocess.run(command, check=False, capture_output=True, text=True)
        if completed.returncode != 0:
            raise RuntimeError(completed.stderr.strip() or "OpenAI Whisper failed")
        result_path = Path(temporary) / f"{audio_path.stem}.json"
        if not result_path.is_file():
            raise RuntimeError("OpenAI Whisper produced no JSON transcript")
        return json.loads(result_path.read_text(encoding="utf-8"))


def _whisper_pass(
    audio_path: Path,
    model: str,
    transcriber: Callable[[Path, str], dict[str, Any]] | None = None,
) -> tuple[dict[str, Any], str, str, int]:
    engine = "mlx-whisper"
    try:
        raw = (transcriber or _mlx_transcribe)(audio_path, model)
    except Exception:
        engine = "openai-whisper"
        raw = _openai_transcribe(audio_path, model)
    result, dropped = _filter_degenerate_segments(_normalized_result(raw))
    actual_model = model if engine == "mlx-whisper" else _fallback_model(model)
    return result, engine, actual_model, dropped


def whisper_disagreement_pct(merged_text: str, whisper_text: str) -> float:
    """Percent of words that disagree after number-aware normalization.

    Matches the measurement framing in videolab's asr_phonon2_vs_whisper note:
    disagreement locates error without naming which engine is wrong. Values over
    15 flag a clip for manual listening.
    """
    merged_tokens = normalize_tokens(merged_text)
    whisper_tokens = normalize_tokens(whisper_text)
    if not merged_tokens and not whisper_tokens:
        return 0.0
    total = max(len(merged_tokens), len(whisper_tokens), 1)
    matcher = difflib.SequenceMatcher(None, merged_tokens, whisper_tokens, autojunk=False)
    mismatched = 0
    for tag, i1, i2, j1, j2 in matcher.get_opcodes():
        if tag == "equal":
            continue
        mismatched += max(i2 - i1, j2 - j1)
    return round(100.0 * mismatched / total, 1)


def _words_from_segment(segment: dict[str, Any], engine: str) -> list[dict]:
    text = str(segment.get("text", "")).strip()
    tokens = text.split()
    if not tokens:
        return []
    start = float(segment.get("start", 0.0))
    end = float(segment.get("end", start))
    span = max(end - start, 0.0)
    step = span / len(tokens) if tokens else 0.0
    return [
        {
            "text": token,
            "start": start + index * step,
            "end": start + (index + 1) * step,
            "engine": engine,
        }
        for index, token in enumerate(tokens)
    ]


def _tag_phonon_words(words: list[dict]) -> list[dict]:
    return [
        {
            "text": str(word.get("word", word.get("text", ""))).strip(),
            "start": float(word["start"]),
            "end": float(word["end"]),
            "engine": "phonon-2",
        }
        for word in words
        if str(word.get("word", word.get("text", ""))).strip()
    ]


def _whisper_only_words(whisper_result: dict[str, Any]) -> list[dict]:
    if whisper_result.get("words"):
        return [
            {
                "text": str(word.get("word", "")).strip(),
                "start": float(word["start"]),
                "end": float(word["end"]),
                "engine": "whisper",
            }
            for word in whisper_result["words"]
            if str(word.get("word", "")).strip()
        ]
    words: list[dict] = []
    for segment in whisper_result.get("segments", []):
        words.extend(_words_from_segment(segment, "whisper"))
    return words


def ensemble(
    audio_path: Path,
    *,
    model: str = DEFAULT_MODEL,
    whisper: Callable[[Path, str], dict[str, Any]] | None = None,
    phonon_words_fn: Callable[[Path], tuple[list[dict], list[list[dict]], float]] | None = None,
    big_text_fn: Callable[[Path], str] | None = None,
) -> dict[str, Any]:
    """Run the videolab ensemble and return words, segments, and coverage stats.

    Order: Whisper → Phonon primary + gap fill → Whisper backfill (≥ 8 words) →
    drop overlapping Phonon words → Phonon-1-Big coverage check (warn only).
    Phonon failure falls back to Whisper-only output.
    """
    audio_path = Path(audio_path).resolve()
    if not audio_path.is_file():
        raise FileNotFoundError(f"audio file does not exist: {audio_path}")

    whisper_result, whisper_engine, _actual_model, _dropped = _whisper_pass(
        audio_path, model, whisper
    )
    whisper_text = str(whisper_result.get("text", ""))

    try:
        primary, passes, duration = (phonon_words_fn or phonon_words)(audio_path)
        merged_words, _filled, _unfilled = fill_gaps(primary, passes, duration)
        phonon_segments = group_words_into_segments(merged_words)
        candidate_segments, _backfilled = backfill_from_reference(
            phonon_segments, whisper_result["segments"]
        )
        inserted_segments = [
            segment
            for segment in candidate_segments
            if segment.get("engine") == "whisper"
        ]
        merged_words = drop_words_in_spans(
            merged_words,
            [
                (float(segment["start"]), float(segment["end"]))
                for segment in inserted_segments
            ],
        )
        segments = group_words_into_segments(merged_words) + inserted_segments
        segments.sort(key=lambda segment: float(segment.get("start", 0.0)))
    except Exception:
        words = _whisper_only_words(whisper_result)
        segments = [
            {**segment, "engine": "whisper"}
            for segment in whisper_result.get("segments", [])
        ]
        return {
            "words": words,
            "segments": segments,
            "coverage_warnings": [],
            "whisper_disagreement_pct": 0.0,
            "engines_used": [whisper_engine],
            "text": whisper_text,
        }

    words = _tag_phonon_words(merged_words)
    for segment in inserted_segments:
        words.extend(_words_from_segment(segment, "whisper"))
    words.sort(key=lambda word: float(word["start"]))

    merged_text = " ".join(str(segment.get("text", "")) for segment in segments).strip()
    warnings: list[dict] = []
    engines_used = ["phonon-2", whisper_engine]
    coverage_error: str | None = None
    try:
        warnings = coverage_warnings(merged_text, (big_text_fn or big_text)(audio_path))
        engines_used.append("phonon-1-big")
    except Exception as exc:
        # Videolab keeps the merge and records coverage_check_error; same here.
        coverage_error = str(exc)

    result = {
        "words": words,
        "segments": segments,
        "coverage_warnings": warnings,
        "whisper_disagreement_pct": whisper_disagreement_pct(merged_text, whisper_text),
        "engines_used": engines_used,
        "text": merged_text,
    }
    if coverage_error is not None:
        result["coverage_check_error"] = coverage_error
    return result
