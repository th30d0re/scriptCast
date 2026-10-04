"""Review log, accepted hearings, and vocabulary hints."""
from __future__ import annotations

import json
from pathlib import Path

import yaml

from scriptcast import review


def test_append_review_and_round_trip(tmp_path: Path):
    record = {
        "turn_id": "abc123",
        "episode": "ATO_EP03",
        "span": {"start_token": 2, "end_token": 3},
        "script_text": "the potestas clause",
        "heard": {"whisper": "the potty clause", "phonon": "the potestas clause"},
        "outcome": "recognizer_misheard",
        "timestamp": "2026-10-04T12:00:00Z",
    }
    path = review.append_review(tmp_path, record)
    assert path == tmp_path / "review_log.jsonl"
    lines = path.read_text(encoding="utf-8").strip().splitlines()
    assert len(lines) == 1
    loaded = json.loads(lines[0])
    assert loaded["turn_id"] == "abc123"
    assert loaded["heard"]["phonon"] == "the potestas clause"
    assert loaded["outcome"] == "recognizer_misheard"
    review.append_review(
        tmp_path,
        {
            **record,
            "outcome": "audio_wrong",
            "timestamp": "2026-10-04T12:01:00Z",
            "reviewer_correction": "keep audio",
        },
    )
    assert len(review.load_reviews(tmp_path)) == 2


def test_accepted_hearing_round_trip(tmp_path: Path):
    review.add_accepted_hearing(tmp_path, "Pati", "Potty!", "2026-10-04T12:00:00Z")
    path = tmp_path / "accepted_hearings.yaml"
    text = path.read_text(encoding="utf-8")
    assert "review 2026-10-04T12:00:00Z" in text
    loaded = review.load_accepted_hearings(tmp_path)
    assert loaded == {"pati": {"potty"}}
    review.add_accepted_hearing(tmp_path, "pati", "poti", "2026-10-04T13:00:00Z")
    loaded = review.load_accepted_hearings(tmp_path)
    assert loaded["pati"] == {"potty", "poti"}
    # Prior review comments survive later appends.
    text = path.read_text(encoding="utf-8")
    assert "review 2026-10-04T12:00:00Z" in text
    assert "review 2026-10-04T13:00:00Z" in text
    # Idempotent on the same pair.
    review.add_accepted_hearing(tmp_path, "pati", "potty", "2026-10-04T14:00:00Z")
    assert review.load_accepted_hearings(tmp_path)["pati"] == {"potty", "poti"}
    raw = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert raw["pati"] == ["potty", "poti"]


def test_vocabulary_hints_order_and_limit(tmp_path: Path):
    assert review.vocabulary_hints(tmp_path) == ""
    for index, word in enumerate(["alpha", "beta", "gamma", "alpha", "delta"]):
        review.append_review(
            tmp_path,
            {
                "turn_id": f"t{index}",
                "episode": "EP",
                "span": {"start_token": 0, "end_token": 1},
                "script_text": f"say {word} now",
                "heard": {"whisper": f"say {word}x now"},
                "outcome": "recognizer_misheard",
                "timestamp": f"2026-10-04T12:0{index}:00Z",
            },
        )
    review.append_review(
        tmp_path,
        {
            "turn_id": "noise",
            "episode": "EP",
            "span": {"start_token": 0, "end_token": 1},
            "script_text": "ignore me",
            "heard": {"whisper": "ignore me"},
            "outcome": "audio_wrong",
            "timestamp": "2026-10-04T12:09:00Z",
        },
    )
    hints = review.vocabulary_hints(tmp_path, limit=5)
    # Newest recognizer_misheard first; distinct; audio_wrong excluded.
    assert hints == "say delta now alpha gamma"
    limited = review.vocabulary_hints(tmp_path, limit=2)
    assert limited == "say delta"


def test_cli_add_accepts_hearing(tmp_path: Path):
    code = review.main(
        [
            "add",
            str(tmp_path),
            "--turn-id",
            "t1",
            "--episode",
            "ATO_EP03",
            "--start-token",
            "1",
            "--end-token",
            "2",
            "--script-text",
            "potestas",
            "--heard-whisper",
            "potty",
            "--outcome",
            "recognizer_misheard",
            "--timestamp",
            "2026-10-04T15:00:00Z",
            "--accept",
            "potestas=potty",
        ]
    )
    assert code == 0
    assert review.load_accepted_hearings(tmp_path) == {"potestas": {"potty"}}
    assert len(review.load_reviews(tmp_path)) == 1
