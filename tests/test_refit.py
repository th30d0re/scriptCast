"""Synthetic audio regression checks for retained speech tails."""
import asyncio

import numpy as np
import pytest
import soundfile

from scriptcast.__main__ import _segment_from_wav, render_loop
from scriptcast.models import MarkupChunk, Turn, VoiceConfig


@pytest.mark.parametrize("tail_ms", [400, 700])
def test_unchanged_turn_reload_keeps_tail(tmp_path, tail_ms):
    sr = 10000
    audio = np.concatenate((np.full(5000, 0.2), np.linspace(0.029, 0.001, 8000)))
    wav = tmp_path / "Samples" / "Processed" / "host" / "turn_a_chunk_0000.wav"
    wav.parent.mkdir(parents=True)
    soundfile.write(wav, audio, sr)
    turn = Turn(0, "turn_a", "host", "Host", "00:00", 0, "Hi", "Hi", (1, 1),
                [MarkupChunk(kind="speech", text="Hi")])

    class ReloadOnlyEngine:
        async def load(self):
            pass

        async def synthesize_chunk(self, *args, **kwargs):
            pytest.fail("An unchanged turn must not synthesize audio")

    result = asyncio.run(render_loop(
        [turn], {"kokoro": ReloadOnlyEngine()}, {"host": VoiceConfig("host")},
        tmp_path, "episode", 100, resume=True,
        speech_threshold=0.03, tail_ms=tail_ms,
    ))
    assert len(result) == 1
    assert result[0].speech_duration_ms == 500 + tail_ms
    assert result[0].speech_duration_ms == _segment_from_wav(
        wav, 0, "turn_a", 0, "host", 100, 0.03, tail_ms,
    ).speech_duration_ms


@pytest.fixture
def episode(tmp_path):
    import json
    wav = tmp_path / "tail.wav"
    audio = np.concatenate((np.full(5000, 0.2), np.linspace(0.029, 0.001, 8000)))
    soundfile.write(wav, audio, 10000)
    turns = []
    positions = []
    for index in range(2):
        seg = dict(chunk_index=0, segment_wav="tail.wav", duration_ms=1300,
                   speech_duration_ms=500, start_ms=index * 2000,
                   end_ms=index * 2000 + 500, gap_after_ms=100, checksum="old")
        turns.append(dict(turn_index=index, turn_id=f"turn_{index}", speaker_id="host",
                          segments=[seg], start_ms=seg["start_ms"], end_ms=seg["end_ms"]))
        positions.append({"turn_id": f"turn_{index}", **{k: seg[k] for k in (
            "chunk_index", "start_ms", "duration_ms", "speech_duration_ms", "gap_after_ms")}})
    (tmp_path / "episode_manifest.json").write_text(json.dumps(
        dict(episode_id="test", sample_rate=10000, turns=turns)))
    (tmp_path / "render_state.json").write_text(json.dumps(
        dict(schema_version="2.0", segments=positions, turns=[], source_hash="keep")))
    return tmp_path


@pytest.mark.parametrize("tail_ms,expected", [(400, 900), (5000, 1300)])
def test_refit_grows_but_never_exceeds_duration_and_is_idempotent(episode, tail_ms, expected):
    import json
    from scriptcast.tools.refit_render import refit_episode
    from scriptcast.render_state import RenderState
    manifest_path = episode / "episode_manifest.json"
    original = manifest_path.read_bytes()
    wav = (episode / "tail.wav").read_bytes()
    refit_episode(episode, tail_ms, 0.03)
    assert manifest_path.with_suffix(".json.bak").read_bytes() == original
    manifest = json.loads(manifest_path.read_text())
    for index, turn in enumerate(manifest["turns"]):
        seg = turn["segments"][0]
        assert seg["speech_duration_ms"] == expected <= seg["duration_ms"]
        assert seg["start_ms"] == index * 2000
        assert turn["end_ms"] == seg["end_ms"] == index * 2000 + expected
    state_path = episode / "render_state.json"
    state = RenderState.from_dict(json.loads(state_path.read_text()))
    assert state.source_hash == "keep"
    assert state.refit_settings == {"tail_ms": tail_ms, "speech_threshold": 0.03}
    for seg in state.segments:
        assert seg.speech_duration_ms == expected
        reloaded = _segment_from_wav(episode / "tail.wav", 0, seg.turn_id, 0, "host", 100,
                                     **state.refit_settings)
        assert reloaded.speech_duration_ms == seg.speech_duration_ms
    first = (manifest_path.read_bytes(), state_path.read_bytes())
    refit_episode(episode, tail_ms, 0.03)
    assert first == (manifest_path.read_bytes(), state_path.read_bytes())
    assert (episode / "tail.wav").read_bytes() == wav


def test_refit_relayout_updates_state_and_als(episode, monkeypatch):
    import json
    from scriptcast.tools import refit_render
    calls = []
    def generate(results, path):
        calls.append(results)
        return path
    monkeypatch.setattr(refit_render, "generate_als", generate)
    refit_render.refit_episode(episode, 400, 0.03, 250)
    manifest = json.loads((episode / "episode_manifest.json").read_text())
    state = json.loads((episode / "render_state.json").read_text())
    assert [t["start_ms"] for t in manifest["turns"]] == [0, 1150]
    assert [s["start_ms"] for s in state["segments"]] == [0, 1150]
    assert all(s["gap_after_ms"] == 250 for s in state["segments"])
    assert len(calls) == 1
    assert all(s.speech_duration_ms == 900 and s.gap_after_ms == 250 for s in calls[0])


def test_missing_wav_does_not_rewrite_json(episode):
    from scriptcast.tools.refit_render import refit_episode
    paths = [episode / "episode_manifest.json", episode / "render_state.json"]
    before = [p.read_bytes() for p in paths]
    (episode / "tail.wav").unlink()
    with pytest.raises(SystemExit, match="missing sample"):
        refit_episode(episode, 400, 0.03)
    assert before == [p.read_bytes() for p in paths]
    assert not (episode / "episode_manifest.json.bak").exists()


def test_numeric_project_settings(tmp_path):
    from scriptcast.project import Project
    project = Project(tmp_path)
    assert project.tail_ms == 400
    assert project.speech_threshold == 0.03
    (tmp_path / "scriptcast.toml").write_text("tail_ms = 650\nspeech_threshold = 0.02\n")
    project = Project(tmp_path)
    assert project.tail_ms == 650
    assert project.speech_threshold == 0.02


def test_refit_cli_uses_project_settings_and_explicit_overrides(tmp_path, monkeypatch):
    from scriptcast.tools import refit_render
    monkeypatch.setenv("SCRIPTCAST_PROJECT", str(tmp_path))
    (tmp_path / "scriptcast.toml").write_text("tail_ms = 650\nspeech_threshold = 0.02\n")
    calls = []
    monkeypatch.setattr(refit_render, "refit_episode", lambda *args: calls.append(args))
    monkeypatch.setattr("sys.argv", ["scriptcast-refit", str(tmp_path)])
    assert refit_render.main() == 0
    assert calls[-1] == (tmp_path, 650, 0.02, None)
    monkeypatch.setattr("sys.argv", ["scriptcast-refit", str(tmp_path), "--tail-ms", "0",
                                    "--speech-threshold", "0.04", "--gap-ms", "200"])
    assert refit_render.main() == 0
    assert calls[-1] == (tmp_path, 0, 0.04, 200)
