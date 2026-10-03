import json
import os
from fractions import Fraction
from pathlib import Path
import subprocess
import sys
import wave
import xml.etree.ElementTree as ET

import pytest

from scriptcast.tools import export_session


@pytest.fixture
def episode(tmp_path):
    episode_dir = tmp_path / "episode"
    episode_dir.mkdir()
    turns = []
    # Includes overlaps, multiple chunks, half-frame ties, and an exact frame.
    for i, (starts, channels) in enumerate([([0, 1250, 1350], 1), ([8620, 5000], 2)]):
        entries = []
        for j, start in enumerate(starts):
            relative = f"Samples/speaker{i}/chunk{j}.wav"
            wav_path = episode_dir / relative
            wav_path.parent.mkdir(parents=True, exist_ok=True)
            with wave.open(str(wav_path), "wb") as wav:
                wav.setnchannels(channels)
                wav.setsampwidth(2)
                wav.setframerate(48000)
                wav.writeframes(b"\0" * (480 * 2 * channels))
            entries.append(dict(chunk_index=j, segment_wav=relative, duration_ms=10,
                                speech_duration_ms=10, start_ms=start, gap_after_ms=100,
                                checksum="fixture-checksum"))
        turns.append(dict(turn_index=i, turn_id=f"turn{i}", speaker_id=f"speaker{i}",
                          segments=entries))
    manifest = dict(episode_id="small_episode", sample_rate=48000, turns=turns)
    (episode_dir / "episode_manifest.json").write_text(json.dumps(manifest))
    return episode_dir, manifest


def inventory(path):
    return {str(p.relative_to(path)): p.read_bytes() if p.is_file() else None
            for p in path.rglob("*")}


def test_default_output_assets_channels_and_rounded_offsets(episode, capsys):
    episode_dir, manifest = episode
    before = inventory(episode_dir)
    assert export_session.main([str(episode_dir), "--fcpxml"]) == 0
    output = episode_dir / "small_episode.fcpxml"
    assert capsys.readouterr().out.strip() == str(output)
    root = ET.parse(output).getroot()
    assets = root.findall(".//asset")
    clips = root.findall(".//asset-clip")
    entries = [entry for turn in manifest["turns"] for entry in turn["segments"]]
    assert len(assets) == len(clips) == len(entries) == 5
    assert [asset.get("audioChannels") for asset in assets] == ["1", "1", "1", "2", "2"]
    for clip, entry in zip(clips, entries):
        actual = Fraction(clip.get("offset").removesuffix("s"))
        start = Fraction(entry["start_ms"], 1000)
        assert actual == Fraction(round(start * 30), 30)
        assert abs(actual - start) <= Fraction(1, 60)
    after = inventory(episode_dir)
    assert after.pop(output.name) == output.read_bytes()
    assert after == before


def test_frame_aligned_start_is_exact(episode):
    episode_dir, _ = episode
    output = export_session.export_fcpxml(episode_dir)
    clip = ET.parse(output).findall(".//asset-clip")[-1]
    assert Fraction(clip.get("offset").removesuffix("s")) == Fraction(5000, 1000)


def test_out_replaces_existing_output_and_writes_nothing_else(episode, tmp_path, capsys):
    episode_dir, _ = episode
    output = tmp_path / "custom.fcpxml"
    output.write_text("old export")
    before = inventory(tmp_path)
    assert export_session.main([str(episode_dir), "--fcpxml", "--out", str(output)]) == 0
    assert capsys.readouterr().out.strip() == str(output)
    assert ET.parse(output).getroot().tag == "fcpxml"
    after = inventory(tmp_path)
    assert after.pop(output.name) != before.pop(output.name)
    assert after == before


def test_missing_wavs_lists_every_path_and_writes_nothing(episode, tmp_path, capsys):
    episode_dir, manifest = episode
    missing = [episode_dir / entry["segment_wav"] for entry in manifest["turns"][0]["segments"]]
    for path in missing:
        path.unlink()
    before = inventory(tmp_path)
    assert export_session.main([str(episode_dir), "--fcpxml"]) == 1
    captured = capsys.readouterr()
    assert not captured.out
    assert "Missing WAV files" in captured.err
    assert all(str(path) in captured.err for path in missing)
    assert inventory(tmp_path) == before


@pytest.mark.parametrize("content", ["{", "[]", "{}", '{"episode_id": 1}',
                                    '{"episode_id": "ep", "sample_rate": true, "turns": []}'])
def test_invalid_manifest(episode, tmp_path, capsys, content):
    episode_dir, _ = episode
    (episode_dir / "episode_manifest.json").write_text(content)
    before = inventory(tmp_path)
    assert export_session.main([str(episode_dir), "--fcpxml"]) == 1
    assert "Invalid manifest" in capsys.readouterr().err
    assert inventory(tmp_path) == before


def test_missing_manifest(episode, tmp_path, capsys):
    episode_dir, _ = episode
    (episode_dir / "episode_manifest.json").unlink()
    before = inventory(tmp_path)
    assert export_session.main([str(episode_dir), "--fcpxml"]) == 1
    assert "Cannot read manifest" in capsys.readouterr().err
    assert inventory(tmp_path) == before


@pytest.mark.parametrize("key,value", [("start_ms", -1), ("start_ms", "5000"),
                                       ("segment_wav", "../outside.wav")])
def test_invalid_segment(episode, tmp_path, capsys, key, value):
    episode_dir, manifest = episode
    manifest["turns"][0]["segments"][0][key] = value
    (episode_dir / "episode_manifest.json").write_text(json.dumps(manifest))
    before = inventory(tmp_path)
    assert export_session.main([str(episode_dir), "--fcpxml"]) == 1
    assert "Invalid manifest" in capsys.readouterr().err
    assert inventory(tmp_path) == before


def test_duplicate_position_keys_are_invalid(episode, tmp_path, capsys):
    episode_dir, manifest = episode
    manifest["turns"][0]["segments"][1]["chunk_index"] = 0
    (episode_dir / "episode_manifest.json").write_text(json.dumps(manifest))
    before = inventory(tmp_path)
    assert export_session.main([str(episode_dir), "--fcpxml"]) == 1
    assert "duplicate segment key" in capsys.readouterr().err
    assert inventory(tmp_path) == before


def test_missing_output_directory_is_not_created(episode, tmp_path, capsys):
    episode_dir, _ = episode
    before = inventory(tmp_path)
    assert export_session.main([str(episode_dir), "--fcpxml", "--out",
                                str(tmp_path / "absent" / "output.fcpxml")]) == 1
    assert "No such file or directory" in capsys.readouterr().err
    assert inventory(tmp_path) == before


def test_invalid_wav_header(episode, tmp_path, capsys):
    episode_dir, manifest = episode
    (episode_dir / manifest["turns"][0]["segments"][0]["segment_wav"]).write_bytes(b"bad WAV")
    before = inventory(tmp_path)
    assert export_session.main([str(episode_dir), "--fcpxml"]) == 1
    assert "Cannot read WAV header" in capsys.readouterr().err
    assert inventory(tmp_path) == before


@pytest.mark.parametrize("stage", ["generate", "replace"])
@pytest.mark.parametrize("existing", [False, True])
def test_atomic_failure_cleans_up_and_preserves_output(episode, tmp_path, monkeypatch, capsys, stage, existing):
    episode_dir, _ = episode
    output = tmp_path / "atomic.fcpxml"
    if existing:
        output.write_text("previous export")
    before = inventory(tmp_path)

    def fail_generate(segments, path, **kwargs):
        assert path.parent == output.parent
        path.write_text("partial XML")
        raise OSError("write failed")

    def fail_replace(source, target):
        assert source.parent == target.parent == output.parent
        assert ET.parse(source).getroot().tag == "fcpxml"
        raise OSError("replace failed")

    if stage == "generate":
        monkeypatch.setattr(export_session, "generate_fcpxml", fail_generate)
    else:
        monkeypatch.setattr(export_session.os, "replace", fail_replace)
    assert export_session.main([str(episode_dir), "--fcpxml", "--out", str(output)]) == 1
    assert "failed" in capsys.readouterr().err
    assert inventory(tmp_path) == before


def test_input_cannot_be_overwritten(episode, tmp_path, capsys):
    episode_dir, _ = episode
    before = inventory(tmp_path)
    assert export_session.main([str(episode_dir), "--fcpxml", "--out",
                                str(episode_dir / "episode_manifest.json")]) == 1
    assert "overwrite an input" in capsys.readouterr().err
    assert inventory(tmp_path) == before


def test_required_format_and_help(episode, capsys):
    episode_dir, _ = episode
    with pytest.raises(SystemExit) as exc:
        export_session.main([str(episode_dir)])
    assert exc.value.code == 2
    with pytest.raises(SystemExit) as exc:
        export_session.main(["--help"])
    assert exc.value.code == 0
    assert export_session._TIMING_HELP in " ".join(capsys.readouterr().out.split())


def test_module_cli_without_engine_or_ml_imports(episode, tmp_path):
    episode_dir, _ = episode
    code = """
import importlib.abc
import runpy
import sys
class NoEngines(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if any(word in fullname.lower() for word in ('mlx', 'whisper', 'engine')):
            raise AssertionError('Forbidden import: ' + fullname)
sys.meta_path.insert(0, NoEngines())
sys.argv = ['scriptcast-export', *sys.argv[1:]]
runpy.run_module('scriptcast.tools.export_session', run_name='__main__')
"""
    output = tmp_path / "module.fcpxml"
    before = inventory(tmp_path)
    result = subprocess.run([sys.executable, "-c", code, str(episode_dir), "--fcpxml",
                             "--out", str(output)], capture_output=True, text=True,
                            env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
    assert result.returncode == 0, result.stderr
    assert result.stdout.strip() == str(output)
    after = inventory(tmp_path)
    after.pop(output.name)
    assert after == before
