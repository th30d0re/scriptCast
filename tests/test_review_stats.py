"""review_stats report numbers on a fixture log."""
from __future__ import annotations

from pathlib import Path

from scriptcast import review
from scriptcast.tools import review_stats


def test_review_stats_rates_offenders_and_before_after(tmp_path: Path):
    review.append_review(
        tmp_path,
        {
            "turn_id": "a",
            "episode": "EP01",
            "span": {"start_token": 0, "end_token": 1},
            "script_text": "potestas matters",
            "heard": {"whisper": "potty matters"},
            "outcome": "recognizer_misheard",
            "timestamp": "2026-10-01T10:00:00Z",
        },
    )
    review.append_review(
        tmp_path,
        {
            "turn_id": "b",
            "episode": "EP01",
            "span": {"start_token": 0, "end_token": 1},
            "script_text": "other failure",
            "heard": {"whisper": "other failure"},
            "outcome": "audio_wrong",
            "timestamp": "2026-10-01T11:00:00Z",
        },
    )
    review.append_review(
        tmp_path,
        {
            "turn_id": "c",
            "episode": "EP02",
            "span": {"start_token": 0, "end_token": 1},
            "script_text": "potestas again",
            "heard": {"whisper": "potty again"},
            "outcome": "recognizer_misheard",
            "timestamp": "2026-10-02T10:00:00Z",
        },
    )
    review.add_accepted_hearing(
        tmp_path, "potestas", "potty", "2026-10-02T12:00:00Z"
    )

    ep = tmp_path / "outputs" / "EP01"
    ep.mkdir(parents=True)
    (ep / "stress.json").write_text(
        """[
  {"turn_id": "a", "failed": true, "expected": "potestas matters",
   "heard": "potty matters", "timestamp": "2026-10-01T09:00:00Z"},
  {"turn_id": "z", "failed": true, "expected": "potestas matters",
   "heard": "potty matters", "timestamp": "2026-10-03T09:00:00Z"},
  {"turn_id": "ok", "failed": false, "expected": "potestas matters",
   "heard": "potestas matters", "timestamp": "2026-10-03T10:00:00Z"}
]
""",
        encoding="utf-8",
    )

    report = review_stats.render_report(tmp_path)
    assert "EP01: 1/2 = 50.0%" in report
    assert "EP02: 1/1 = 100.0%" in report
    assert "potestas:" in report
    assert "across 2 episode(s)" in report
    assert "potestas ← [potty] added" in report
    assert "before 1 · after 1" in report

    filtered = review_stats.render_report(tmp_path, episode="EP02")
    assert "episode filter: EP02" in filtered
    assert "EP01:" not in filtered
    assert "EP02: 1/1 = 100.0%" in filtered
