"""Feedback hooks in verify_render: byte-identical when absent; allowlist; hints."""
from __future__ import annotations

import json
from pathlib import Path

import pytest

from scriptcast.tools import verify_render as vr


EXPECTED = "The potestas of the magistrate was clear."
HEARD_FAIL = "The potty of the magistrate was clear."
HEARD_OK = EXPECTED


def test_score_byte_identical_without_accepted():
    """No accepted map → same score tuple as the two-arg call path."""
    a = vr._score(EXPECTED, HEARD_FAIL)
    b = vr._score(EXPECTED, HEARD_FAIL, accepted=None)
    c = vr._score(EXPECTED, HEARD_OK)
    d = vr._score(EXPECTED, HEARD_OK, accepted=None)
    assert a == b
    assert c == d
    assert a[0] < 1.0
    assert a[1] >= 1
    assert c == (1.0, 0)


def test_diff_byte_identical_without_accepted():
    assert vr._diff(EXPECTED, HEARD_FAIL) == vr._diff(
        EXPECTED, HEARD_FAIL, accepted=None
    )


def test_accepted_hearing_turns_failure_into_pass():
    bare_ratio, bare_run = vr._score(EXPECTED, HEARD_FAIL)
    assert bare_ratio < 0.90 or bare_run > 0
    accepted = {"potestas": {"potty"}}
    ratio, worst, hits = vr._score_with_feedback(EXPECTED, HEARD_FAIL, accepted)
    assert hits == 1
    assert worst == 0
    assert ratio >= 0.90
    # Empty accepted keeps the failing score.
    ratio0, worst0, hits0 = vr._score_with_feedback(EXPECTED, HEARD_FAIL, {})
    assert hits0 == 0
    assert (ratio0, worst0) == (bare_ratio, bare_run)


def test_json_out_omits_feedback_key_when_unused(tmp_path: Path, monkeypatch):
    """Full verify path with no project feedback → no feedback key; scoring intact."""
    script = tmp_path / "script.md"
    script.write_text("Alice (00:00)\nHello world today.\n", encoding="utf-8")
    wav = tmp_path / "0-0.wav"
    wav.write_bytes(b"RIFF....")
    from scriptcast.parser import parse_transcript
    from scriptcast.markup import tokenize_markup

    turns = tokenize_markup(parse_transcript(script))
    turn = turns[0]
    manifest = {
        "turns": [
            {
                "turn_id": turn.turn_id,
                "turn_index": 0,
                "speaker_id": "alice",
                "segments": [
                    {
                        "segment_wav": wav.name,
                        "chunk_index": 0,
                        "start_ms": 0,
                        "duration_ms": 1000,
                    }
                ],
            }
        ]
    }
    (tmp_path / "episode_manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    (tmp_path / "scriptcast.toml").write_text('voices = "voices.yaml"\n', encoding="utf-8")
    (tmp_path / "voices.yaml").write_text("alice: {}\n", encoding="utf-8")

    calls = []

    def fake_transcribe(path, **kwargs):
        calls.append(kwargs)
        return {"text": "Hello world today.", "segments": []}

    monkeypatch.setenv("SCRIPTCAST_PROJECT", str(tmp_path))
    monkeypatch.setitem(__import__("sys").modules, "mlx_whisper", type("M", (), {
        "transcribe": staticmethod(fake_transcribe),
    })())

    out = tmp_path / "stress.json"
    monkeypatch.setattr(
        "sys.argv",
        [
            "scriptcast-verify",
            str(tmp_path),
            "--transcript",
            str(script),
            "--json-out",
            str(out),
            "--no-stress",
        ],
    )
    assert vr.main() == 0
    data = json.loads(out.read_text(encoding="utf-8"))
    assert len(data) == 1
    assert "feedback" not in data[0]
    assert data[0]["failed"] is False
    assert "initial_prompt" not in calls[0]


def test_hints_passed_to_transcribe(tmp_path: Path, monkeypatch):
    script = tmp_path / "script.md"
    script.write_text("Alice (00:00)\nHello potestas today.\n", encoding="utf-8")
    wav = tmp_path / "0-0.wav"
    wav.write_bytes(b"RIFF....")
    from scriptcast.parser import parse_transcript
    from scriptcast.markup import tokenize_markup

    turns = tokenize_markup(parse_transcript(script))
    turn = turns[0]
    manifest = {
        "turns": [
            {
                "turn_id": turn.turn_id,
                "turn_index": 0,
                "speaker_id": "alice",
                "segments": [
                    {
                        "segment_wav": wav.name,
                        "chunk_index": 0,
                        "start_ms": 0,
                        "duration_ms": 1000,
                    }
                ],
            }
        ]
    }
    (tmp_path / "episode_manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    (tmp_path / "scriptcast.toml").write_text('voices = "voices.yaml"\n', encoding="utf-8")
    (tmp_path / "voices.yaml").write_text("alice: {}\n", encoding="utf-8")
    from scriptcast import review

    review.append_review(
        tmp_path,
        {
            "turn_id": "x",
            "episode": "EP",
            "span": {"start_token": 0, "end_token": 1},
            "script_text": "rare potestas lemma",
            "heard": {"whisper": "rare potty lemma"},
            "outcome": "recognizer_misheard",
            "timestamp": "2026-10-04T12:00:00Z",
        },
    )
    calls = []

    def fake_transcribe(path, **kwargs):
        calls.append(kwargs)
        return {"text": "Hello potestas today.", "segments": []}

    monkeypatch.setenv("SCRIPTCAST_PROJECT", str(tmp_path))
    monkeypatch.setitem(__import__("sys").modules, "mlx_whisper", type("M", (), {
        "transcribe": staticmethod(fake_transcribe),
    })())
    out = tmp_path / "stress.json"
    monkeypatch.setattr(
        "sys.argv",
        [
            "scriptcast-verify",
            str(tmp_path),
            "--transcript",
            str(script),
            "--json-out",
            str(out),
            "--no-stress",
        ],
    )
    assert vr.main() == 0
    assert calls and calls[0].get("initial_prompt") == "rare potestas lemma"
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data[0]["feedback"] == {"accepted": 0, "hints": True}


def test_no_feedback_flag_skips_hints_and_accepted(tmp_path: Path, monkeypatch):
    script = tmp_path / "script.md"
    script.write_text("Alice (00:00)\nHello potestas today.\n", encoding="utf-8")
    wav = tmp_path / "0-0.wav"
    wav.write_bytes(b"RIFF....")
    from scriptcast.parser import parse_transcript
    from scriptcast.markup import tokenize_markup

    turns = tokenize_markup(parse_transcript(script))
    turn = turns[0]
    manifest = {
        "turns": [
            {
                "turn_id": turn.turn_id,
                "turn_index": 0,
                "speaker_id": "alice",
                "segments": [
                    {
                        "segment_wav": wav.name,
                        "chunk_index": 0,
                        "start_ms": 0,
                        "duration_ms": 1000,
                    }
                ],
            }
        ]
    }
    (tmp_path / "episode_manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    (tmp_path / "scriptcast.toml").write_text('voices = "voices.yaml"\n', encoding="utf-8")
    (tmp_path / "voices.yaml").write_text("alice: {}\n", encoding="utf-8")
    from scriptcast import review

    review.add_accepted_hearing(tmp_path, "potestas", "potty", "2026-10-04T12:00:00Z")
    review.append_review(
        tmp_path,
        {
            "turn_id": "x",
            "episode": "EP",
            "span": {"start_token": 0, "end_token": 1},
            "script_text": "rare potestas lemma",
            "heard": {"whisper": "rare potty lemma"},
            "outcome": "recognizer_misheard",
            "timestamp": "2026-10-04T12:00:00Z",
        },
    )
    calls = []

    def fake_transcribe(path, **kwargs):
        calls.append(kwargs)
        return {"text": "Hello potty today.", "segments": []}

    monkeypatch.setenv("SCRIPTCAST_PROJECT", str(tmp_path))
    monkeypatch.setitem(__import__("sys").modules, "mlx_whisper", type("M", (), {
        "transcribe": staticmethod(fake_transcribe),
    })())
    out = tmp_path / "stress.json"
    monkeypatch.setattr(
        "sys.argv",
        [
            "scriptcast-verify",
            str(tmp_path),
            "--transcript",
            str(script),
            "--json-out",
            str(out),
            "--no-stress",
            "--no-feedback",
        ],
    )
    code = vr.main()
    assert "initial_prompt" not in calls[0]
    data = json.loads(out.read_text(encoding="utf-8"))
    assert "feedback" not in data[0]
    assert data[0]["failed"] is True
    assert code == 1


def test_word_timings_passes_hints(tmp_path: Path, monkeypatch):
    from scriptcast.tools import word_timings as wt
    from scriptcast import review

    monkeypatch.delenv("SCRIPTCAST_PROJECT", raising=False)
    script = tmp_path / "script.md"
    script.write_text("Alice (00:00)\nOne two three.\n", encoding="utf-8")
    from scriptcast.parser import parse_transcript

    turns = parse_transcript(script)
    wav = tmp_path / "0-0.wav"
    wav.write_bytes(b"fake")
    manifest = {
        "source_file": str(script),
        "turns": [
            {
                "turn_id": turns[0].turn_id,
                "turn_index": 0,
                "start_ms": 0,
                "end_ms": 2000,
                "segments": [
                    {"segment_wav": wav.name, "start_ms": 0, "duration_ms": 2000}
                ],
            }
        ],
    }
    (tmp_path / "episode_manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    (tmp_path / "scriptcast.toml").write_text('voices = "voices.yaml"\n', encoding="utf-8")
    (tmp_path / "voices.yaml").write_text("alice: {}\n", encoding="utf-8")
    review.append_review(
        tmp_path,
        {
            "turn_id": "x",
            "episode": "EP",
            "span": {"start_token": 0, "end_token": 1},
            "script_text": "hintword",
            "heard": {"whisper": "hinturd"},
            "outcome": "recognizer_misheard",
            "timestamp": "2026-10-04T12:00:00Z",
        },
    )
    seen = []

    def fake(audio, model, *, initial_prompt=None):
        seen.append(initial_prompt)
        return [{"word": "One", "start": 0.1, "end": 0.3},
                {"word": "two", "start": 0.4, "end": 0.6},
                {"word": "three", "start": 0.7, "end": 0.9}]

    monkeypatch.setattr(wt, "transcribe_with_prompt", fake)
    monkeypatch.setenv("SCRIPTCAST_PROJECT", str(tmp_path))
    assert wt.main([str(tmp_path), "--script", str(script)]) == 0
    assert seen == ["hintword"]
