"""Clip ensemble transcript tests use injected engines and never load a model."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

import pytest

from scriptcast.parser import parse_transcript
from scriptcast.tools import clip_transcribe as ct


@pytest.fixture
def episode(tmp_path, monkeypatch):
    monkeypatch.delenv("SCRIPTCAST_PROJECT", raising=False)
    script = tmp_path / "script.md"
    script.write_text(
        "Alice (00:00)\nOne spoken turn.\n\n"
        "Archive (00:01)\n[clip:atwater_1981] historical audio here.\n\n"
        "Alice (00:02)\nLast spoken turn.\n\n"
        "Archive (00:03)\n[clip:other_clip] short archival line.\n"
    )
    turns = parse_transcript(script)
    entries = []
    for turn in turns:
        segments = []
        count = 2 if turn.turn_index == 1 else 1
        for index in range(count):
            path = tmp_path / f"{turn.turn_index}-{index}.wav"
            path.write_bytes(b"fake audio")
            segments.append(
                {
                    "segment_wav": path.name,
                    "start_ms": 10000 + index * 2000,
                    "duration_ms": 2000,
                }
            )
        entries.append(
            {
                "turn_id": turn.turn_id,
                "turn_index": turn.turn_index,
                "start_ms": 10000,
                "end_ms": 14000,
                "segments": segments,
            }
        )
    manifest = {"source_file": str(script), "turns": entries}
    (tmp_path / "episode_manifest.json").write_text(json.dumps(manifest))
    calls: list[str] = []

    def fake_ensemble(audio: Path):
        calls.append(audio.name)
        if audio.name == "1-0.wav":
            return {
                "words": [
                    {"text": "First", "start": 0.1, "end": 0.4, "engine": "phonon-2"},
                    {"text": "half", "start": 0.5, "end": 0.9, "engine": "phonon-2"},
                ],
                "segments": [],
                "coverage_warnings": [{"words": 8, "excerpt": "missing span here"}],
                "whisper_disagreement_pct": 4.0,
                "engines_used": ["phonon-2", "mlx-whisper", "phonon-1-big"],
                "text": "First half",
            }
        if audio.name == "1-1.wav":
            return {
                "words": [
                    {"text": "second", "start": 0.0, "end": 0.3, "engine": "whisper"},
                    {"text": "half", "start": 0.4, "end": 0.8, "engine": "whisper"},
                ],
                "segments": [],
                "coverage_warnings": [],
                "whisper_disagreement_pct": 20.0,
                "engines_used": ["mlx-whisper"],
                "text": "second half",
            }
        if audio.name == "3-0.wav":
            return {
                "words": [{"text": "short", "start": 0.1, "end": 0.5, "engine": "phonon-2"}],
                "segments": [],
                "coverage_warnings": [],
                "whisper_disagreement_pct": 1.0,
                "engines_used": ["phonon-2", "mlx-whisper"],
                "text": "short",
            }
        return {
            "words": [],
            "segments": [],
            "coverage_warnings": [],
            "whisper_disagreement_pct": 0.0,
            "engines_used": [],
            "text": "",
        }

    monkeypatch.setattr(ct, "ensemble", fake_ensemble)
    return tmp_path, script, manifest, turns, calls


def test_clip_only_selection_offsets_and_listen_manually(episode):
    directory, script, _, turns, calls = episode
    assert ct.main([str(directory), "--script", str(script)]) == 0
    data = json.loads((directory / "clip_transcripts.json").read_text())
    assert data["manifest_sha256"] == hashlib.sha256(
        (directory / "episode_manifest.json").read_bytes()
    ).hexdigest()
    clip_turn = turns[1]
    other = turns[3]
    assert set(data["turns"]) == {clip_turn.turn_id, other.turn_id}
    assert turns[0].turn_id not in data["turns"]
    assert turns[2].turn_id not in data["turns"]

    entry = data["turns"][clip_turn.turn_id]
    assert entry["clip_id"] == "atwater_1981"
    assert [word["text"] for word in entry["words"]] == ["First", "half", "second", "half"]
    assert entry["words"][0] == {
        "text": "First",
        "start_ms": 100,
        "end_ms": 400,
        "engine": "phonon-2",
    }
    assert entry["words"][2] == {
        "text": "second",
        "start_ms": 2000,
        "end_ms": 2300,
        "engine": "whisper",
    }
    assert entry["whisper_disagreement_pct"] == 20.0
    assert entry["listen_manually"] is True
    assert entry["coverage_warnings"] == [{"words": 8, "excerpt": "missing span here"}]
    assert calls == ["1-0.wav", "1-1.wav", "3-0.wav"]

    quiet = data["turns"][other.turn_id]
    assert quiet["listen_manually"] is False
    assert quiet["whisper_disagreement_pct"] == 1.0


def test_turns_filter_and_custom_output(episode):
    directory, script, _, turns, calls = episode
    output = directory / "custom_clips.json"
    output.write_text("old")
    assert (
        ct.main(
            [
                str(directory),
                "--script",
                str(script),
                "--turns",
                "2-3",
                "--out",
                str(output),
            ]
        )
        == 0
    )
    data = json.loads(output.read_text())
    assert set(data["turns"]) == {turns[3].turn_id}
    assert calls == ["3-0.wav"]
    assert not (directory / "clip_transcripts.json").exists()


@pytest.mark.parametrize("stage", ["dump", "replace"])
def test_atomic_failure_preserves_old_output(episode, monkeypatch, stage):
    directory, script, _, _, _ = episode
    output = directory / "clip_transcripts.json"
    output.write_text("previous")
    before = {path.name: path.read_bytes() for path in directory.iterdir()}

    def fail(*args, **kwargs):
        if stage == "replace":
            source, target = args
            assert source.parent == target.parent
            assert json.loads(source.read_text())["turns"]
            assert target.read_text() == "previous"
        raise OSError("injected failure")

    monkeypatch.setattr(ct.json if stage == "dump" else ct.os, stage, fail)
    assert ct.main([str(directory), "--script", str(script)]) == 1
    assert {path.name: path.read_bytes() for path in directory.iterdir()} == before


def test_atomic_success_uses_os_replace(episode, monkeypatch):
    directory, script, _, _, _ = episode
    replace = ct.os.replace
    seen = []

    def tracked(source, target):
        seen.append((Path(source).parent, Path(target).name))
        return replace(source, target)

    monkeypatch.setattr(ct.os, "replace", tracked)
    assert ct.main([str(directory), "--script", str(script)]) == 0
    assert seen and seen[0][1] == "clip_transcripts.json"
    assert (directory / "clip_transcripts.json").is_file()


@pytest.mark.parametrize(
    ("token", "expected"),
    [
        ("1954", ["nineteen", "fifty", "four"]),
        ("1968", ["nineteen", "sixty", "eight"]),
        ("2012", ["twenty", "twelve"]),
        ("21", ["twenty", "one"]),
        ("0", ["zero"]),
        ("100", ["one", "hundred"]),
        ("4000", ["four", "thousand"]),
    ],
)
def test_spoken_number_tokens(token, expected):
    assert ct.spoken_number_tokens(token) == expected
    assert ct._year_two_thousand_tokens("2012") == [
        "two",
        "thousand",
        "twelve",
    ]


def test_align_script_words_year_filler_and_gaps():
    from types import SimpleNamespace

    turn = SimpleNamespace(
        turn_index=1,
        clean_text="[clip:demo] In 1954 he said hello world.",
    )
    ensemble = [
        {"text": "In", "start_ms": 0, "end_ms": 100, "engine": "phonon-2"},
        {"text": "nineteen", "start_ms": 100, "end_ms": 200, "engine": "phonon-2"},
        {"text": "fifty", "start_ms": 200, "end_ms": 300, "engine": "phonon-2"},
        {"text": "four", "start_ms": 300, "end_ms": 400, "engine": "phonon-2"},
        {"text": "he", "start_ms": 400, "end_ms": 500, "engine": "phonon-2"},
        {"text": "said", "start_ms": 500, "end_ms": 600, "engine": "phonon-2"},
        {"text": "uh", "start_ms": 600, "end_ms": 700, "engine": "phonon-2"},
        {"text": "hallo", "start_ms": 700, "end_ms": 800, "engine": "phonon-2"},
        {"text": "world", "start_ms": 900, "end_ms": 1000, "engine": "phonon-2"},
    ]
    aligned = ct.align_script_words(turn, ensemble, duration=1.2)
    by_text = {word["text"].strip(".,!?"): word for word in aligned}
    assert [w["text"] for w in aligned] == [
        "In",
        "1954",
        "he",
        "said",
        "hello",
        "world.",
    ]
    assert by_text["1954"]["matched"] is True
    assert by_text["1954"]["start_ms"] == 100
    assert by_text["1954"]["end_ms"] == 400
    assert by_text["In"]["matched"] is True
    assert by_text["he"]["matched"] is True
    assert by_text["said"]["matched"] is True
    assert by_text["world"]["matched"] is True
    # Filler "uh" is ASR-only; misheard "hello"/"hallo" is unmatched and gap-filled.
    assert by_text["hello"]["matched"] is False
    assert by_text["1954"]["end_ms"] <= by_text["he"]["start_ms"]
    assert by_text["said"]["end_ms"] <= by_text["hello"]["start_ms"]
    assert by_text["hello"]["end_ms"] <= by_text["world"]["start_ms"]


def test_version_2_includes_script_words_keeps_old_fields(episode):
    directory, script, _, turns, _ = episode
    assert ct.main([str(directory), "--script", str(script)]) == 0
    data = json.loads((directory / "clip_transcripts.json").read_text())
    assert data["version"] == 2
    entry = data["turns"][turns[1].turn_id]
    assert [word["text"] for word in entry["words"]] == [
        "First",
        "half",
        "second",
        "half",
    ]
    assert entry["clip_id"] == "atwater_1981"
    assert entry["whisper_disagreement_pct"] == 20.0
    assert entry["listen_manually"] is True
    assert "script_words" in entry
    assert len(entry["script_words"]) == 3  # historical audio here
    assert {key for key in entry["script_words"][0]} >= {
        "text",
        "start_ms",
        "end_ms",
        "matched",
    }
