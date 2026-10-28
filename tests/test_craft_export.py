import json
from pathlib import Path
import wave
import xml.etree.ElementTree as ET

from scriptcast.tools import craft_export


def _manifest(turns, duration_ms=None):
    manifest = {"episode_id": "craft_ep", "sample_rate": 48000, "turns": turns}
    if duration_ms is not None:
        manifest["duration_ms"] = duration_ms
    return manifest


def test_cue_sheet_uses_manifest_duration_and_card_frame(tmp_path):
    episode_dir = tmp_path / "episode"
    episode_dir.mkdir()
    manifest = _manifest(
        [
            {"turn_index": 0, "start_ms": 0, "end_ms": 4000},
            {"turn_index": 1, "start_ms": 4000, "end_ms": 8000},
        ],
        duration_ms=9000,
    )
    (episode_dir / "episode_manifest.json").write_text(json.dumps(manifest))
    out = tmp_path / "cue.json"
    assert craft_export.main(["cue", str(episode_dir), "--out", str(out)]) == 0
    sheet = json.loads(out.read_text())
    assert sheet["duration_ms"] == 9000
    assert sheet["fps"] == 30
    assert sheet["width"] == 1080
    assert sheet["height"] == 1920


def test_overlapping_cues_shorten_the_earlier_one(tmp_path):
    episode_dir = tmp_path / "episode"
    episode_dir.mkdir()
    manifest = _manifest(
        [
            {"turn_index": 1, "start_ms": 3000, "end_ms": 8000},
            {"turn_index": 0, "start_ms": 0, "end_ms": 5000},
        ],
        duration_ms=8000,
    )
    (episode_dir / "episode_manifest.json").write_text(json.dumps(manifest))
    out = tmp_path / "cue.json"
    assert craft_export.main(["cue", str(episode_dir), "--out", str(out)]) == 0
    cues = json.loads(out.read_text())["cues"]
    assert cues[0]["start_ms"] == 0
    assert cues[0]["end_ms"] == 3000
    assert cues[1]["start_ms"] == 3000
    assert cues[1]["end_ms"] == 8000


def _wav(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(48000)
        handle.writeframes(b"\0" * (480 * 2))


def _timeline(tmp_path, cues):
    episode_dir = tmp_path / "episode"
    relative = "Samples/host/chunk0.wav"
    _wav(episode_dir / relative)
    manifest = {
        "episode_id": "craft_ep",
        "sample_rate": 48000,
        "turns": [
            {
                "turn_index": 0,
                "turn_id": "turn0",
                "speaker_id": "host",
                "start_ms": 0,
                "end_ms": 1000,
                "segments": [
                    {
                        "chunk_index": 0,
                        "segment_wav": relative,
                        "duration_ms": 1000,
                        "speech_duration_ms": 1000,
                        "start_ms": 0,
                        "gap_after_ms": 0,
                        "checksum": "fixture",
                    }
                ],
            }
        ],
    }
    (episode_dir / "episode_manifest.json").write_text(json.dumps(manifest))
    cue_path = tmp_path / "cue.json"
    cue_path.write_text(json.dumps({
        "fps": 30,
        "width": 1080,
        "height": 1920,
        "duration_ms": 2500,
        "cues": cues,
    }))
    captions_path = tmp_path / "captions.json"
    captions = [{"turn_index": 0, "start_ms": 1000, "end_ms": 2500, "text": "Hello there"}]
    captions_path.write_text(json.dumps(captions))
    out = tmp_path / "episode.fcpxml"
    assert craft_export.main([
        "timeline", str(episode_dir),
        "--cue", str(cue_path),
        "--captions", str(captions_path),
        "--out", str(out),
    ]) == 0
    return out, ET.parse(out).getroot(), captions


def test_timeline_places_png_on_the_cue_clock(tmp_path):
    plate = tmp_path / "plate.png"
    plate.write_bytes(b"\x89PNG\r\n\x1a\n")
    out, root, _ = _timeline(tmp_path, [
        {"start_ms": 1000, "end_ms": 2500, "src": str(plate.resolve()), "kind": "still"},
        {"start_ms": 0, "end_ms": 500, "src": "", "kind": "page"},
    ])
    assert root.tag == "fcpxml"
    assert root.get("version") == "1.10"
    fmt = root.find("./resources/format")
    assert fmt.get("width") == "1080"
    assert fmt.get("height") == "1920"
    assert fmt.get("frameDuration") == "1/30s"
    assets = root.findall(".//asset")
    wav = next(asset for asset in assets if asset.get("hasVideo") == "0")
    picture = next(asset for asset in assets if asset.get("hasVideo") == "1")
    assert wav.get("hasAudio") == "1"
    assert picture.get("hasAudio") == "0"
    clip = next(item for item in root.findall(".//asset-clip") if item.get("ref") == picture.get("id"))
    assert clip.get("offset") == "30/30s"
    assert clip.get("duration") == "45/30s"
    assert out.with_suffix(".srt").is_file()
    assert out.with_suffix(".vtt").is_file()


def test_timeline_writes_srt_and_webvtt_from_captions_json(tmp_path):
    plate = tmp_path / "plate.png"
    plate.write_bytes(b"\x89PNG\r\n\x1a\n")
    out, _, captions = _timeline(tmp_path, [
        {"start_ms": 1000, "end_ms": 2500, "src": str(plate.resolve()), "kind": "still"},
    ])
    srt = out.with_suffix(".srt").read_text()
    vtt = out.with_suffix(".vtt").read_text()
    assert srt.startswith("1\n00:00:01,000 --> 00:00:02,500\nHello there\n")
    assert vtt.startswith("WEBVTT\n\n00:00:01.000 --> 00:00:02.500\nHello there\n")
    assert "Hello there" in srt and "Hello there" in vtt
    assert captions[0]["text"] == "Hello there"
    assert captions[0]["start_ms"] == 1000
    assert captions[0]["end_ms"] == 2500


def test_empty_src_adds_no_video_asset(tmp_path):
    _, root, _ = _timeline(tmp_path, [
        {"start_ms": 0, "end_ms": 1000, "src": "", "kind": "still"},
    ])
    assets = root.findall(".//asset")
    assert len(assets) == 1
    assert assets[0].get("hasVideo") == "0"
    assert assets[0].get("hasAudio") == "1"
