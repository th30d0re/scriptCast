from __future__ import annotations

import sys
from pathlib import Path
from types import SimpleNamespace

from scriptcast.asr import engines


def test_mlx_transcribe_passes_decoding_guards(tmp_path: Path, monkeypatch) -> None:
    captured: dict[str, object] = {}

    def fake_transcribe(audio: str, **kwargs: object) -> dict[str, object]:
        captured.update({"audio": audio, **kwargs})
        return {"text": "", "segments": []}

    monkeypatch.setitem(sys.modules, "mlx_whisper", SimpleNamespace(transcribe=fake_transcribe))
    audio_path = tmp_path / "audio.wav"
    engines._mlx_transcribe(audio_path, "model/test")

    assert captured == {
        "audio": str(audio_path),
        "path_or_hf_repo": "model/test",
        "verbose": None,
        "condition_on_previous_text": False,
        "temperature": (0.0, 0.2, 0.4, 0.6, 0.8, 1.0),
        "compression_ratio_threshold": 2.4,
        "no_speech_threshold": 0.6,
    }


def test_filter_drops_repeated_phrase_and_implausible_speech_rate() -> None:
    result = {
        "text": "raw decoder text",
        "language": "en",
        "segments": [
            {"start": 0.0, "end": 8.0, "text": "to process " * 6},
            {
                "start": 8.0,
                "end": 8.1,
                "text": "thirteen words fit inside this tiny segment and make its speech rate clearly impossible now",
            },
            {"start": 9.0, "end": 12.0, "text": "within fracturing tawakkul"},
        ],
    }

    filtered, dropped_segments = engines._filter_degenerate_segments(result)

    assert dropped_segments == 2
    assert filtered["segments"] == [
        {"start": 9.0, "end": 12.0, "text": "within fracturing tawakkul"}
    ]
    assert filtered["text"] == "within fracturing tawakkul"


def test_ensemble_backfills_unfilled_opening_gap(tmp_path: Path) -> None:
    audio_path = tmp_path / "audio.wav"
    audio_path.write_bytes(b"fixture")
    opening = "it even said like this but this is directed at a certain part of my audience"

    def fake_whisper(_audio: Path, _model: str) -> dict[str, object]:
        return {
            "text": f"{opening} later words",
            "language": "en",
            "segments": [
                {"start": 0.0, "end": 4.4, "text": opening},
                {"start": 4.8, "end": 6.0, "text": "later words"},
            ],
        }

    def fake_phonon(_audio: Path) -> tuple[list[dict], list[list[dict]], float]:
        return (
            [
                {"word": "later", "start": 4.8, "end": 5.2},
                {"word": "words", "start": 5.4, "end": 6.0},
            ],
            [[]],
            6.0,
        )

    result = engines.ensemble(
        audio_path,
        whisper=fake_whisper,
        phonon_words_fn=fake_phonon,
        big_text_fn=lambda _audio: f"{opening} later words",
    )

    assert result["segments"][0]["engine"] == "whisper"
    assert "phonon-2" in result["engines_used"]
    assert any(word["engine"] == "whisper" for word in result["words"])
    assert any(word["text"] == "later" and word["engine"] == "phonon-2" for word in result["words"])


def test_ensemble_phonon_failure_writes_whisper_only_outputs(tmp_path: Path) -> None:
    audio_path = tmp_path / "audio.wav"
    audio_path.write_bytes(b"fixture")

    def fake_whisper(_audio: Path, _model: str) -> dict[str, object]:
        return {
            "text": "stable whisper transcript",
            "language": "en",
            "segments": [{"start": 0.0, "end": 2.0, "text": "stable whisper transcript"}],
        }

    def failed_phonon(_audio: Path):
        raise RuntimeError("phonon unavailable")

    result = engines.ensemble(
        audio_path, whisper=fake_whisper, phonon_words_fn=failed_phonon
    )

    assert result["engines_used"] == ["mlx-whisper"]
    assert result["text"] == "stable whisper transcript"
    assert result["coverage_warnings"] == []
    assert result["whisper_disagreement_pct"] == 0.0
    assert all(word["engine"] == "whisper" for word in result["words"])


def test_ensemble_keeps_merge_when_big_coverage_check_fails(tmp_path: Path) -> None:
    audio_path = tmp_path / "audio.wav"
    audio_path.write_bytes(b"fixture")

    def whisper(_audio: Path, _model: str) -> dict[str, object]:
        return {
            "text": "merged transcript",
            "language": "en",
            "segments": [{"start": 0.0, "end": 1.0, "text": "merged transcript"}],
        }

    def phonon(_audio: Path) -> tuple[list[dict], list[list[dict]], float]:
        return (
            [
                {"word": "merged", "start": 0.0, "end": 0.4},
                {"word": "transcript", "start": 0.5, "end": 1.0},
            ],
            [],
            1.0,
        )

    result = engines.ensemble(
        audio_path,
        whisper=whisper,
        phonon_words_fn=phonon,
        big_text_fn=lambda _audio: (_ for _ in ()).throw(RuntimeError("big failed")),
    )

    assert "phonon-2" in result["engines_used"]
    assert "phonon-1-big" not in result["engines_used"]
    assert [word["text"] for word in result["words"]] == ["merged", "transcript"]


def test_ensemble_backfill_replaces_overlapping_phonon_words(tmp_path: Path) -> None:
    audio_path = tmp_path / "audio.wav"
    audio_path.write_bytes(b"fixture")
    opening_words = (
        "male and female sexuality too and are shamed for having a lot of sex"
    ).split()
    overlap_words = "sex is treated as something that a man".split()
    primary = [
        {
            "word": word,
            "start": 29.8 + index * 0.6,
            "end": 30.2 + index * 0.6,
        }
        for index, word in enumerate(opening_words)
    ]
    primary.extend(
        {
            "word": word,
            "start": 38.3 + index * 0.2,
            "end": 38.7 + index * 0.2,
        }
        for index, word in enumerate(overlap_words)
    )
    primary.extend(
        [
            {"word": "following", "start": 43.0, "end": 43.4},
            {"word": "words", "start": 43.5, "end": 43.9},
        ]
    )
    sentence = (
        "Sex is treated as something that a man takes from a woman, "
        "as if it decreases her value."
    )

    def fake_whisper(_audio: Path, _model: str) -> dict[str, object]:
        return {
            "text": sentence,
            "language": "en",
            "segments": [{"start": 38.5, "end": 42.6, "text": sentence}],
        }

    def fake_phonon(_audio: Path) -> tuple[list[dict], list[list[dict]], float]:
        return primary, [], 44.0

    result = engines.ensemble(
        audio_path,
        whisper=fake_whisper,
        phonon_words_fn=fake_phonon,
        big_text_fn=lambda _audio: sentence,
    )

    assert result["text"].count("Sex is treated as something that a man takes from a woman") == 1
    assert all(
        not (38.5 <= (float(word["start"]) + float(word["end"])) / 2.0 < 42.6)
        or word["engine"] == "whisper"
        for word in result["words"]
    )


def test_fermion_server_is_terminated_when_request_raises(monkeypatch) -> None:
    class FakeProcess:
        def __init__(self) -> None:
            self.terminated = False

        def terminate(self) -> None:
            self.terminated = True

        def wait(self, timeout: float) -> None:
            assert timeout == 5

    process = FakeProcess()
    monkeypatch.setattr(engines, "_free_port", lambda: 5278)
    monkeypatch.setattr(engines, "_wait_for_health", lambda _port: None)

    try:
        engines._run_fermion_server(
            "phonon-2",
            lambda _port: (_ for _ in ()).throw(RuntimeError("request failed")),
            executable="fermion",
            process_factory=lambda *_args, **_kwargs: process,
        )
    except RuntimeError as exc:
        assert str(exc) == "request failed"
    else:
        raise AssertionError("request failure did not propagate")

    assert process.terminated is True


def test_fermion_executable_prefers_environment(monkeypatch) -> None:
    monkeypatch.setenv("SCRIPTCAST_FERMION", "/custom/bin/fermion")
    monkeypatch.setattr(engines.shutil, "which", lambda _name: "/path/bin/fermion")

    assert engines._fermion_executable() == "/custom/bin/fermion"


def test_whisper_disagreement_pct_flags_listen_manually_threshold() -> None:
    same = engines.whisper_disagreement_pct("alpha beta gamma", "alpha beta gamma")
    assert same == 0.0
    high = engines.whisper_disagreement_pct(
        "alpha beta gamma delta epsilon zeta eta theta",
        "iota kappa lambda mu nu xi omicron pi",
    )
    assert high > 15.0
