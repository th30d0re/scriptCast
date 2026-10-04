"""Read-along timing tests use fake ASR and never load a model."""
import hashlib
import json
from pathlib import Path

import pytest

from scriptcast.parser import parse_transcript
from scriptcast.tools import word_timings as wt


@pytest.fixture
def episode(tmp_path, monkeypatch):
    monkeypatch.delenv("SCRIPTCAST_PROJECT", raising=False)
    script = tmp_path / "script.md"
    script.write_text("Alice (00:00)\nOne missing two three.\n\n"
                      "Alice (00:01)\n[clip:archive]\n\nAlice (00:02)\nLast word.\n")
    turns = parse_transcript(script)
    entries = []
    for turn in turns:
        segments = []
        for index in range(2 if turn.turn_index == 0 else 1):
            path = tmp_path / f"{turn.turn_index}-{index}.wav"
            path.write_bytes(b"fake audio")
            segments.append({"segment_wav": path.name, "start_ms": 10000 + index * 2000,
                             "duration_ms": 2000})
        entries.append({"turn_id": turn.turn_id, "turn_index": turn.turn_index,
                        "start_ms": 10000, "end_ms": 14000, "segments": segments})
    manifest = {"source_file": str(script), "turns": entries}
    (tmp_path / "episode_manifest.json").write_text(json.dumps(manifest))
    calls = []

    def fake(audio, model):
        calls.append((audio.name, model))
        words = {"0-0.wav": [("One", .1, .4), ("two", 1, 1.3)],
                 "0-1.wav": [("extra", 0, .2), ("three", .3, .9)],
                 "2-0.wav": [("Last", .2, .6), ("word", .7, 1)]}
        return [{"word": text, "start": start, "end": end} for text, start, end in words[audio.name]]

    monkeypatch.setattr(wt, "transcribe", fake)
    return tmp_path, script, manifest, turns, calls


def test_segment_offsets_alignment_dropped_extra_and_clips(episode):
    directory, script, manifest, turns, calls = episode
    assert wt.main([str(directory), "--script", str(script)]) == 0
    data = json.loads((directory / "word_timings.json").read_text())
    assert data["manifest_sha256"] == hashlib.sha256((directory / "episode_manifest.json").read_bytes()).hexdigest()
    assert data["model"] == wt.DEFAULT_MODEL
    words = data["turns"][turns[0].turn_id]
    assert [word["text"] for word in words] == ["One", "missing", "two", "three."]
    assert words[0] == {"text": "One", "start_ms": 100, "end_ms": 400}
    assert 400 <= words[1]["start_ms"] < words[1]["end_ms"] <= 1000
    assert words[-1] == {"text": "three.", "start_ms": 2300, "end_ms": 2900}
    assert turns[1].turn_id not in data["turns"]
    assert [name for name, model in calls] == ["0-0.wav", "0-1.wav", "2-0.wav"]


def test_turns_and_custom_output_and_default_script(episode):
    directory, _, _, turns, calls = episode
    output = directory / "custom.json"
    output.write_text("old")
    assert wt.main([str(directory), "--turns", "1-2", "--model", "fake-model", "--out", str(output)]) == 0
    assert set(json.loads(output.read_text())["turns"]) == {turns[2].turn_id}
    assert calls == [("2-0.wav", "fake-model")]
    assert not (directory / "word_timings.json").exists()
    assert wt.turn_selection("5,10-12") == {5, 10, 11, 12}


@pytest.mark.parametrize("value", ["", "1-", "3-2", "-1", "1-2-3", "x"])
def test_invalid_turn_selection(value):
    with pytest.raises(wt.argparse.ArgumentTypeError):
        wt.turn_selection(value)


@pytest.mark.parametrize("stage", ["dump", "replace"])
def test_atomic_failure_preserves_old_output_and_cleans_temp(episode, monkeypatch, stage):
    directory, _, _, _, _ = episode
    output = directory / "word_timings.json"
    output.write_text("previous")
    before = {p.name: p.read_bytes() for p in directory.iterdir()}

    def fail(*args, **kwargs):
        if stage == "replace":
            source, target = args
            assert source.parent == target.parent
            assert json.loads(source.read_text())["turns"]
            assert target.read_text() == "previous"
        raise OSError("injected failure")

    monkeypatch.setattr(wt.json if stage == "dump" else wt.os, stage, fail)
    assert wt.main([str(directory)]) == 1
    assert {p.name: p.read_bytes() for p in directory.iterdir()} == before


def test_atomic_success_and_input_protection(episode, monkeypatch):
    directory, _, _, _, _ = episode
    replace = wt.os.replace
    seen = []

    def publish(source, target):
        assert json.loads(source.read_text())["turns"]
        assert source.parent == target.parent
        seen.append(source)
        replace(source, target)

    monkeypatch.setattr(wt.os, "replace", publish)
    assert wt.main([str(directory)]) == 0
    assert len(seen) == 1 and not seen[0].exists()
    for name in ["episode_manifest.json", "0-0.wav"]:
        original = (directory / name).read_bytes()
        assert wt.main([str(directory), "--out", str(directory / name)]) == 1
        assert (directory / name).read_bytes() == original


def test_unknown_turn_and_changed_script_do_not_publish(episode):
    directory, script, _, _, calls = episode
    assert wt.main([str(directory), "--turns", "99"]) == 1
    assert not calls
    script.write_text(script.read_text().replace("One", "Changed"))
    assert wt.main([str(directory)]) == 1
    assert not (directory / "word_timings.json").exists()


def test_no_matches_fills_every_word_inside_turn(episode, monkeypatch):
    directory, _, _, turns, _ = episode
    monkeypatch.setattr(wt, "transcribe", lambda *_: [])
    assert wt.main([str(directory), "--turns", "0"]) == 0
    words = json.loads((directory / "word_timings.json").read_text())["turns"][turns[0].turn_id]
    assert len(words) == 4
    assert all(0 <= w["start_ms"] < w["end_ms"] <= 4000 for w in words)
