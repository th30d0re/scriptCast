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


def test_retime_round_trips_three_digit_minutes(tmp_path, monkeypatch):
    monkeypatch.setattr(retime_script, "_load_known_rates", lambda: {})
    script = tmp_path / "episode.md"
    original = "Emmanuel Theodore (118:40)\nHello.\n"
    script.write_text(original)
    turn = parse_transcript(script)[0]
    manifest = _write_manifest(
        tmp_path, [(turn.turn_id, 7_120_000, 7_121_000)]
    )

    rows, _, estimated, _, _ = retime_script.retime(script, manifest, 350)

    assert estimated == 0
    assert rows[0]["old"] == rows[0]["new"] == "118:40"
    assert retime_script.rewrite_script(script, rows) == 0
    assert script.read_text() == original
    assert parse_transcript(script)[0].timestamp_ms == 7_120_000

    # Exercise writing a changed header and remapping three-digit references.
    rows[0].update(new="119:40", new_ms=7_180_000)
    assert retime_script.rewrite_script(script, rows) == 1
    assert parse_transcript(script)[0].timestamp_mmss == "119:40"
    shotlist = tmp_path / "shotlist.md"
    shotlist.write_text(
        "`Emmanuel Theodore (118:40)`\n**Hold:** through 118:45\n"
    )
    assert retime_script.rewrite_shotlist(shotlist, rows) == (2, [], [])
    assert shotlist.read_text() == (
        "`Emmanuel Theodore (119:40)`\n**Hold:** through 119:45\n"
    )


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
