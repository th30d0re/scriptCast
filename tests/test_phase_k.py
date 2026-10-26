import hashlib
import json
from pathlib import Path

import pytest

from scriptcast.parser import parse_transcript
from scriptcast.tools import export_session


@pytest.fixture(params=[b"\n", b"\r\n"], ids=["lf", "crlf"])
def edit_project(tmp_path, monkeypatch, request):
    ending = request.param
    monkeypatch.delenv("SCRIPTCAST_PROJECT", raising=False)
    monkeypatch.chdir(tmp_path)
    (tmp_path / "scriptcast.toml").write_text(
        'voices = "voices.yaml"\nclips = "clips.yaml"\n', encoding="utf-8"
    )
    (tmp_path / "voices.yaml").write_text(
        "speakers:\n"
        "  aisha:\n    name: Aisha\n    engine: kokoro\n"
        "    kokoro_voice: af_heart\n    lang_code: a\n"
        "  bob:\n    engine: omnivoice\n    reference_audio: bob.wav\n"
        "    reference_text: words\n    reference_audio_expressive: expressive.wav\n"
        "    reference_text_expressive: expressive words\n"
        "  archivist:\n    name: Archivist\n    engine: archive\n",
        encoding="utf-8",
    )
    (tmp_path / "clips.yaml").write_text(
        "clips:\n  speech_1:\n    source: source.m4a\n    start: 1.25\n"
        "    end: 2.5\n    citation: Citation\n",
        encoding="utf-8",
    )
    script = tmp_path / "script.md"
    script.write_bytes(
        ending.join([
            b"# preserved preamble",
            b"",
            b"Aisha (00:10)",
            b"First body.",
            b"",
            b"Bob (00:20)",
            b"Second body.",
            b"",
            b"Aisha (00:40)",
            b"Third body.",
            b"",
        ])
    )
    episode = tmp_path / "outputs" / "ep"
    episode.mkdir(parents=True)
    (episode / "render_state.json").write_text(
        json.dumps({"source_file": "script.md"}), encoding="utf-8"
    )
    return episode, script, ending


def _edit(episode: Path, script: Path, tmp_path: Path, operations: list[dict], capsys):
    ops = tmp_path / "ops.json"
    ops.write_text(json.dumps(operations), encoding="utf-8")
    digest = hashlib.sha256(script.read_bytes()).hexdigest()
    code = export_session.main([
        str(episode), "--edit-json", str(ops), "--expect-sha256", digest,
    ])
    captured = capsys.readouterr()
    return code, captured, json.loads(captured.out) if captured.out else None


@pytest.mark.parametrize(
    "operation,expected",
    [
        ({"op": "insert", "position": 1, "speaker_id": "aisha", "timestamp": "00:15",
          "body": "Inserted."}, ["First body.", "Inserted.", "Second body.", "Third body."]),
        ({"op": "delete", "index": 1}, ["First body.", "Third body."]),
        ({"op": "move", "index": 0, "to": 2},
         ["Second body.", "Third body.", "First body."]),
        ({"op": "set_speaker", "index": 1, "speaker_id": "aisha"},
         ["First body.", "Second body.", "Third body."]),
    ],
)
def test_each_edit_preserves_parseable_order(edit_project, tmp_path, capsys, operation, expected):
    episode, script, _ = edit_project
    code, _, result = _edit(episode, script, tmp_path, [operation], capsys)
    assert code == 0
    assert [turn.clean_text for turn in parse_transcript(script)] == expected
    assert [turn["body"] for turn in result["turns"]] == expected
    if operation["op"] == "set_speaker":
        assert parse_transcript(script)[1].display_name == "Aisha"
        assert parse_transcript(script)[1].timestamp_mmss == "00:20"


def test_sequential_ops_use_current_indices_and_preserve_untouched_bytes(
    edit_project, tmp_path, capsys,
):
    episode, script, ending = edit_project
    original = script.read_bytes()
    third_block = ending.join([b"Aisha (00:40)", b"Third body.", b""])
    operations = [
        {"op": "delete", "index": 0},
        {"op": "insert", "position": 1, "speaker_id": "aisha", "timestamp": None,
         "body": "Middle."},
        {"op": "move", "index": 2, "to": 0},
        {"op": "set_speaker", "index": 1, "speaker_id": "aisha"},
    ]
    code, _, _ = _edit(episode, script, tmp_path, operations, capsys)
    assert code == 0
    turns = parse_transcript(script)
    assert [(turn.clean_text, turn.timestamp_mmss) for turn in turns] == [
        ("Third body.", "00:40"), ("Second body.", "00:20"), ("Middle.", "00:30")
    ]
    assert third_block in script.read_bytes()
    assert original.startswith(ending.join([b"# preserved preamble", b""]))
    assert script.read_bytes().startswith(ending.join([b"# preserved preamble", b""]))


@pytest.mark.parametrize(
    "position,expected",
    [(0, "00:00"), (1, "00:15"), (3, "00:45")],
)
def test_null_timestamp_start_midpoint_and_end(edit_project, tmp_path, capsys, position, expected):
    episode, script, _ = edit_project
    operation = {"op": "insert", "position": position, "speaker_id": "aisha",
                 "timestamp": None, "body": "New."}
    assert _edit(episode, script, tmp_path, [operation], capsys)[0] == 0
    assert parse_transcript(script)[position].timestamp_mmss == expected


def test_archive_insert_requires_registered_leading_clip(edit_project, tmp_path, capsys):
    episode, script, _ = edit_project
    before = script.read_bytes()
    invalid = {"op": "insert", "position": 0, "speaker_id": "archivist",
               "timestamp": None, "body": "No clip here."}
    code, output, _ = _edit(episode, script, tmp_path, [invalid], capsys)
    assert code == 1
    assert "op[0]: archive speaker body must start" in output.err
    assert script.read_bytes() == before

    valid = {**invalid, "body": "[clip:speech_1] Quoted words."}
    assert _edit(episode, script, tmp_path, [valid], capsys)[0] == 0
    first = parse_transcript(script)[0]
    assert first.speaker_id == "archivist"
    assert first.clean_text.startswith("[clip:speech_1]")


@pytest.mark.parametrize(
    "operation,message",
    [
        ({"op": "insert", "position": 0, "speaker_id": "missing", "timestamp": None,
          "body": "x"}, "unknown speaker_id"),
        ({"op": "delete", "index": 99}, "out of range"),
        ({"op": "move", "index": 0, "to": 99}, "out of range"),
        ({"op": "set_speaker", "index": -1, "speaker_id": "aisha"}, "out of range"),
    ],
)
def test_invalid_edit_is_atomic(edit_project, tmp_path, capsys, operation, message):
    episode, script, _ = edit_project
    before = script.read_bytes()
    code, output, _ = _edit(episode, script, tmp_path, [operation], capsys)
    assert code == 1
    assert message in output.err
    assert script.read_bytes() == before


def test_stale_hash_exits_three_without_reading_ops(edit_project, tmp_path, capsys):
    episode, script, _ = edit_project
    before = script.read_bytes()
    missing_ops = tmp_path / "absent.json"
    assert export_session.main([
        str(episode), "--edit-json", str(missing_ops), "--expect-sha256", "0" * 64,
    ]) == 3
    output = capsys.readouterr()
    assert output.out == ""
    assert "stale script" in output.err
    assert script.read_bytes() == before


def test_second_op_failure_lists_errors_and_rolls_back(edit_project, tmp_path, capsys):
    episode, script, _ = edit_project
    before = script.read_bytes()
    operations = [
        {"op": "delete", "index": 0},
        {"op": "delete", "index": 99},
        {"op": "set_speaker", "index": 99, "speaker_id": "missing"},
    ]
    code, output, _ = _edit(episode, script, tmp_path, operations, capsys)
    assert code == 1
    assert "op[1]:" in output.err and "op[2]:" in output.err
    assert script.read_bytes() == before


def test_delete_then_insert_canonical_block_restores_bytes(edit_project, tmp_path, capsys):
    episode, script, _ = edit_project
    before = script.read_bytes()
    operations = [
        {"op": "delete", "index": 1},
        {"op": "insert", "position": 1, "speaker_id": "bob", "timestamp": "00:20",
         "body": "Second body."},
    ]
    assert _edit(episode, script, tmp_path, operations, capsys)[0] == 0
    assert script.read_bytes() == before


def test_append_after_final_block_without_newline_remains_parseable(
    edit_project, tmp_path, capsys,
):
    episode, script, _ = edit_project
    script.write_bytes(script.read_bytes().rstrip(b"\r\n"))
    operation = {"op": "insert", "position": 3, "speaker_id": "aisha",
                 "timestamp": None, "body": "Appended."}
    assert _edit(episode, script, tmp_path, [operation], capsys)[0] == 0
    assert [turn.clean_text for turn in parse_transcript(script)][-2:] == [
        "Third body.", "Appended."
    ]


def test_tags_json_all_rules(edit_project, capsys):
    episode, script, ending = edit_project
    script.write_bytes(ending.join([
        b"Aisha (00:00)",
        b"[pause:1] [pause:10000ms] [beat] [emphasis] [tone:warm] [clip:speech_1]",
        b"[pause:0] [pause:10001ms] [tone:] [clip:missing] [clip:speech_1] [other]",
        b"",
        b"Archivist (00:10)",
        b"No archive clip.",
        b"",
    ]))
    assert export_session.main([str(episode), "--tags-json"]) == 0
    result = json.loads(capsys.readouterr().out)
    assert result["script_sha256"] == hashlib.sha256(script.read_bytes()).hexdigest()
    by_tag = [(problem["tag"], problem["message"]) for problem in result["problems"]]
    assert ("[pause:0]", "pause must be between 1 and 10000 ms") in by_tag
    assert ("[pause:10001ms]", "pause must be between 1 and 10000 ms") in by_tag
    assert ("[tone:]", "tone must be non-empty") in by_tag
    assert any(tag == "[clip:missing]" and "not registered" in message for tag, message in by_tag)
    assert ("[clip:speech_1]", "clip appears more than once in this turn") in by_tag
    assert ("[other]", "unrecognised tag; it will be spoken aloud") in by_tag
    assert any(problem["index"] == 1 and problem["tag"] == "[clip:id]"
               for problem in result["problems"])
    assert all(problem["line"] > 0 for problem in result["problems"])


def test_speakers_json_and_voices_override(edit_project, tmp_path, capsys):
    episode, _, _ = edit_project
    assert export_session.main([str(episode), "--speakers-json"]) == 0
    result = json.loads(capsys.readouterr().out)
    assert result["speakers"] == [
        {"id": "aisha", "name": "Aisha", "engine": "kokoro", "has_expressive": False},
        {"id": "bob", "name": "Bob", "engine": "omnivoice", "has_expressive": True},
        {"id": "archivist", "name": "Archivist", "engine": "archive", "has_expressive": False},
    ]
    assert result["clips"] == [{
        "id": "speech_1", "source": "source.m4a", "start": 1.25, "end": 2.5,
        "citation": "Citation",
    }]

    override = tmp_path / "override.yaml"
    override.write_text(
        "speakers:\n  narrator:\n    engine: archive\n", encoding="utf-8"
    )
    assert export_session.main([
        str(episode), "--speakers-json", "--voices", str(override),
    ]) == 0
    assert json.loads(capsys.readouterr().out)["speakers"][0]["name"] == "Narrator"


def test_existing_turns_json_serialization_is_exact(edit_project, capsys):
    episode, script, ending = edit_project
    before = script.read_bytes()
    assert export_session.main([str(episode), "--turns-json"]) == 0
    actual = capsys.readouterr().out
    parsed = json.loads(actual)
    assert actual == json.dumps(parsed) + "\n"
    assert parsed["script_path"] == str(script)
    assert parsed["line_ending"] == ("crlf" if ending == b"\r\n" else "lf")
    assert script.read_bytes() == before
