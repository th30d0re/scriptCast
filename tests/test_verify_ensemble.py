"""Second-opinion ensemble tests use injected Phonon text; no models load."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

import pytest

from scriptcast.markup import tokenize_markup
from scriptcast.parser import parse_transcript
from scriptcast.tools import verify_ensemble as ve


@pytest.fixture
def episode(tmp_path, monkeypatch):
    monkeypatch.delenv("SCRIPTCAST_PROJECT", raising=False)
    script = tmp_path / "script.md"
    script.write_text(
        "Alice (00:00)\nThe quick brown fox jumps.\n\n"
        "Alice (00:01)\nAlpha beta gamma delta.\n\n"
        "Alice (00:02)\nOne two three four.\n\n"
        "Alice (00:03)\nHello world today.\n"
    )
    turns = tokenize_markup(parse_transcript(script))
    entries = []
    stress = []
    for turn in turns:
        segments = []
        count = 2 if turn.turn_index == 0 else 1
        for index in range(count):
            path = tmp_path / f"{turn.turn_index}-{index}.wav"
            path.write_bytes(b"fake audio")
            segments.append(
                {
                    "segment_wav": path.name,
                    "chunk_index": index,
                    "start_ms": 1000 + index * 1500,
                    "duration_ms": 1500,
                }
            )
        entries.append(
            {
                "turn_id": turn.turn_id,
                "turn_index": turn.turn_index,
                "speaker_id": "alice",
                "start_ms": 1000,
                "end_ms": 1000 + count * 1500,
                "segments": segments,
            }
        )
    # Turn 0: failed, wrong expected word → engines hear real audio (likely_audio)
    stress.append(
        {
            "turn": turns[0].turn_index,
            "manifest_turn": turns[0].turn_index,
            "turn_id": turns[0].turn_id,
            "speaker": "alice",
            "score": 0.7,
            "worst_run": 1,
            "failed": True,
            "expected": "The quick green fox jumps",
            "heard": "The quick brown fox jumps",
            "stress": [],
            "repetition": None,
        }
    )
    # Turn 1: failed, Whisper misheard; Phonon matches expected
    stress.append(
        {
            "turn": turns[1].turn_index,
            "manifest_turn": turns[1].turn_index,
            "turn_id": turns[1].turn_id,
            "speaker": "alice",
            "score": 0.6,
            "worst_run": 2,
            "failed": True,
            "expected": "Alpha beta gamma delta",
            "heard": "Alpha better gamma delta",
            "stress": [],
            "repetition": None,
        }
    )
    # Turn 2: passed — must be skipped by default
    stress.append(
        {
            "turn": turns[2].turn_index,
            "manifest_turn": turns[2].turn_index,
            "turn_id": turns[2].turn_id,
            "speaker": "alice",
            "score": 1.0,
            "worst_run": 0,
            "failed": False,
            "expected": "One two three four",
            "heard": "One two three four",
            "stress": [],
            "repetition": None,
        }
    )
    # Turn 3: failed, for coverage_gap / uncertain cases via custom phonon
    stress.append(
        {
            "turn": turns[3].turn_index,
            "manifest_turn": turns[3].turn_index,
            "turn_id": turns[3].turn_id,
            "speaker": "alice",
            "score": 0.5,
            "worst_run": 2,
            "failed": True,
            "expected": "Hello world today",
            "heard": "Hello word today",
            "stress": [],
            "repetition": None,
        }
    )
    manifest = {"source_file": str(script), "turns": entries}
    (tmp_path / "episode_manifest.json").write_text(json.dumps(manifest))
    (tmp_path / "stress.json").write_text(json.dumps(stress, indent=1))
    return tmp_path, script, manifest, turns, stress


def test_align_verdicts():
    spans = ve.align_spans(
        "The quick green fox",
        "The quick brown fox",
        "The quick brown fox",
    )
    assert any(s["verdict"] == "likely_audio" for s in spans)
    assert ve.summary_verdict(spans) == "likely_audio"

    spans = ve.align_spans(
        "Alpha beta gamma",
        "Alpha better gamma",
        "Alpha beta gamma",
    )
    assert all(s["verdict"] == "likely_recognition" for s in spans)
    assert ve.summary_verdict(spans) == "likely_recognition"

    spans = ve.align_spans(
        "One two three",
        "One too tree",
        "One to free",
    )
    assert any(s["verdict"] == "uncertain" for s in spans)
    assert ve.summary_verdict(spans) == "mixed"

    spans = ve.align_spans(
        "Hello world today",
        "Hello world today",
        "",
    )
    assert any(s["verdict"] == "coverage_gap" for s in spans)


def test_default_failed_turns_offsets_and_scores(episode):
    directory, script, _, turns, _ = episode
    calls: list[str] = []

    def phonon(audio: Path) -> str:
        calls.append(audio.name)
        if audio.name == "0-0.wav":
            return "The quick"
        if audio.name == "0-1.wav":
            return "brown fox jumps"
        if audio.name.startswith("1-"):
            return "Alpha beta gamma delta"
        if audio.name.startswith("3-"):
            return "Hello world today"
        return "One two three four"

    built = ve.build_ensemble(
        directory,
        json.loads((directory / "episode_manifest.json").read_text()),
        json.loads((directory / "stress.json").read_text()),
        turns,
        phonon_fn=phonon,
    )
    assert set(built) == {turns[0].turn_id, turns[1].turn_id, turns[3].turn_id}
    assert turns[2].turn_id not in built
    # Multi-segment turn: both WAVs transcribed in start_ms order
    assert calls[:2] == ["0-0.wav", "0-1.wav"]

    audio_turn = built[turns[0].turn_id]
    assert audio_turn["summary"] == "likely_audio"
    assert any(s["verdict"] == "likely_audio" for s in audio_turn["spans"])
    # Both engines hear the same real audio against a wrong expected word.
    assert audio_turn["score_phonon"] == audio_turn["score_whisper"]

    recog = built[turns[1].turn_id]
    assert recog["summary"] == "likely_recognition"
    assert all(s["verdict"] == "likely_recognition" for s in recog["spans"])
    assert recog["score_phonon"] > recog["score_whisper"]


def test_turns_filter_and_main_writes(episode, monkeypatch):
    directory, script, _, turns, _ = episode

    def phonon(audio: Path) -> str:
        return "Alpha beta gamma delta"

    monkeypatch.setattr(
        ve,
        "phonon_segment_text",
        lambda path, **kwargs: phonon(path),
    )
    out = directory / "stress_ensemble.json"
    assert (
        ve.main(
            [
                str(directory),
                "--script",
                str(script),
                "--turns",
                str(turns[1].turn_index),
                "--out",
                str(out),
            ]
        )
        == 0
    )
    data = json.loads(out.read_text())
    assert data["manifest_sha256"] == hashlib.sha256(
        (directory / "episode_manifest.json").read_bytes()
    ).hexdigest()
    assert data["stress_sha256"] == hashlib.sha256(
        (directory / "stress.json").read_bytes()
    ).hexdigest()
    assert set(data["turns"]) == {turns[1].turn_id}
    # stress.json untouched
    assert json.loads((directory / "stress.json").read_text())[1]["failed"] is True


@pytest.mark.parametrize("stage", ["dump", "replace"])
def test_atomic_failure_preserves_old_output(episode, monkeypatch, stage):
    directory, script, _, _, _ = episode
    output = directory / "stress_ensemble.json"
    output.write_text("previous")
    before = {path.name: path.read_bytes() for path in directory.iterdir()}
    monkeypatch.setattr(
        ve,
        "phonon_segment_text",
        lambda path, **kwargs: "The quick brown fox jumps",
    )

    def fail(*args, **kwargs):
        if stage == "replace":
            source, target = args
            assert source.parent == target.parent
            assert json.loads(source.read_text())["turns"]
            assert target.read_text() == "previous"
        raise OSError("injected failure")

    monkeypatch.setattr(ve.json if stage == "dump" else ve.os, stage, fail)
    assert ve.main([str(directory), "--script", str(script)]) == 1
    assert {path.name: path.read_bytes() for path in directory.iterdir()} == before


def test_atomic_success_uses_os_replace(episode, monkeypatch):
    directory, script, _, _, _ = episode
    monkeypatch.setattr(
        ve,
        "phonon_segment_text",
        lambda path, **kwargs: "The quick brown fox jumps",
    )
    replace = ve.os.replace
    seen = []

    def tracked(source, target):
        seen.append((Path(source).parent, Path(target).name))
        return replace(source, target)

    monkeypatch.setattr(ve.os, "replace", tracked)
    assert ve.main([str(directory), "--script", str(script)]) == 0
    assert seen and seen[0][1] == "stress_ensemble.json"
    assert (directory / "stress_ensemble.json").is_file()


def test_summary_with_no_spans_matches_script():
    from scriptcast.tools.verify_ensemble import summary_verdict

    assert summary_verdict([]) == "matches_script"
