import json

from scriptcast.markup import tokenize_markup
from scriptcast.parser import parse_transcript
from scriptcast.tools import retime_script


def _write_script(tmp_path):
    script = tmp_path / "episode.md"
    script.write_text(
        "Host (00:00)\nAlpha.\n\n"
        "Guest (00:01)\nTwo words.\n\n"
        "Narrator (00:02)\nOmega.\n"
    )
    turns = tokenize_markup(parse_transcript(script))
    return script, turns


def _write_manifest(tmp_path, timings):
    manifest = tmp_path / "manifest.json"
    manifest.write_text(
        json.dumps(
            {
                "turns": [
                    {"turn_id": turn_id, "start_ms": start, "end_ms": end}
                    for turn_id, start, end in timings
                ]
            }
        )
    )
    return manifest


def test_contiguous_measured_turns_use_manifest_starts(tmp_path):
    script, turns = _write_script(tmp_path)
    manifest = _write_manifest(
        tmp_path,
        [
            (turns[0].turn_id, 0, 1_000),
            (turns[1].turn_id, 1_000, 2_000),
            (turns[2].turn_id, 2_000, 3_000),
        ],
    )

    rows, total_ms, estimated, _, _ = retime_script.retime(script, manifest, 350)

    assert [row["new_ms"] for row in rows] == [0, 1_000, 2_000]
    assert total_ms == 3_000
    assert estimated == 0


def test_unrendered_turn_is_estimated_but_next_measured_turn_snaps_back(
    tmp_path, monkeypatch
):
    script, turns = _write_script(tmp_path)
    manifest = _write_manifest(
        tmp_path,
        [
            (turns[0].turn_id, 100, 1_100),
            (turns[2].turn_id, 3_000, 4_000),
        ],
    )
    monkeypatch.setattr(
        retime_script, "_load_known_rates", lambda: {"guest": (100.0, 0.0)}
    )

    rows, total_ms, estimated, _, _ = retime_script.retime(script, manifest, 350)

    assert [row["new_ms"] for row in rows] == [100, 1_100, 3_000]
    assert rows[1]["duration_ms"] == 200
    assert rows[1]["source"] == "estimated"
    assert total_ms == 4_000
    assert estimated == 1


def test_measured_turns_with_real_gaps_use_manifest_starts(tmp_path):
    script, turns = _write_script(tmp_path)
    manifest = _write_manifest(
        tmp_path,
        [
            (turns[0].turn_id, 0, 800),
            (turns[1].turn_id, 1_250, 2_100),
            (turns[2].turn_id, 3_000, 3_900),
        ],
    )

    rows, total_ms, estimated, _, _ = retime_script.retime(script, manifest, 350)

    assert [row["new_ms"] for row in rows] == [0, 1_250, 3_000]
    assert total_ms == 3_900
    assert estimated == 0
