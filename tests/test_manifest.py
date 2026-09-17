import json
from pathlib import Path

from voice_pipeline.manifest import write_manifest
from voice_pipeline.models import SegmentResult, Turn, VoiceConfig


def test_manifest_paths_are_relative_to_manifest_directory(tmp_path) -> None:
    episode_dir = tmp_path / "ATO_EP0"
    wav_path = (
        episode_dir
        / "Samples"
        / "Processed"
        / "ai_1"
        / "id0_chunk_0000.wav"
    )
    wav_path.parent.mkdir(parents=True, exist_ok=True)
    wav_path.write_bytes(b"fake wav")
    source_path = Path("Architecting_the_operation/podcasts/ATO_EP0.md").resolve()
    turns = [
        Turn(
            turn_index=0,
            turn_id="id0",
            speaker_id="ai_1",
            display_name="AI 1",
            timestamp_mmss="00:00",
            timestamp_ms=0,
            raw_text="Hello.",
            clean_text="Hello.",
            line_span=(1, 2),
        )
    ]
    voices = {
        "ai_1": VoiceConfig(
            speaker_id="ai_1",
            kokoro_voice="af_heart",
            lang_code="a",
            speed=1.0,
        )
    }
    segments = [
        SegmentResult(
            turn_index=0,
            turn_id="id0",
            chunk_index=0,
            speaker_id="ai_1",
            wav_path=wav_path,
            duration_ms=1000,
            speech_duration_ms=850,
            sample_rate=48000,
            gap_after_ms=250,
            checksum="checksum",
        )
    ]

    manifest_path = write_manifest(
        episode_id="ATO_EP0",
        source_file=str(source_path),
        model_id="model",
        engine="mlx_kokoro",
        turns=turns,
        segments=segments,
        voices=voices,
        output_path=episode_dir,
    )

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    segment_wav = manifest["turns"][0]["segments"][0]["segment_wav"]

    assert segment_wav.startswith("Samples/Processed/")
    assert (manifest_path.parent / segment_wav).resolve() == wav_path.resolve()
    assert manifest["source_file"] == "Architecting_the_operation/podcasts/ATO_EP0.md"
    assert not Path(manifest["source_file"]).is_absolute()
    # speech_duration_ms should be present in the segment entry
    assert manifest["turns"][0]["segments"][0]["speech_duration_ms"] == 850


def test_manifest_includes_engine_and_character_profile_for_elevenlabs(tmp_path) -> None:
    episode_dir = tmp_path / "ATO_EP0"
    wav_path = (
        episode_dir
        / "Samples"
        / "Processed"
        / "kareem"
        / "id0_chunk_0000.wav"
    )
    wav_path.parent.mkdir(parents=True, exist_ok=True)
    wav_path.write_bytes(b"fake wav")
    turns = [
        Turn(
            turn_index=0,
            turn_id="id0",
            speaker_id="kareem",
            display_name="Kareem",
            timestamp_mmss="00:00",
            timestamp_ms=0,
            raw_text="Hello.",
            clean_text="Hello.",
            line_span=(1, 2),
        )
    ]
    voices = {
        "kareem": VoiceConfig(
            speaker_id="kareem",
            engine="elevenlabs",
            elevenlabs_voice_id="VlUmeC1Uzj3NnwiVR9K9",
            character_profile="Black male perspective",
        )
    }
    segments = [
        SegmentResult(
            turn_index=0,
            turn_id="id0",
            chunk_index=0,
            speaker_id="kareem",
            wav_path=wav_path,
            duration_ms=1000,
            speech_duration_ms=850,
            sample_rate=48000,
            gap_after_ms=250,
            checksum="checksum",
        )
    ]

    manifest_path = write_manifest(
        episode_id="ATO_EP0",
        source_file="test.md",
        model_id="eleven_multilingual_v2",
        engine="elevenlabs",
        turns=turns,
        segments=segments,
        voices=voices,
        output_path=episode_dir,
    )

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    speaker = manifest["speakers"][0]
    assert speaker["speaker_id"] == "kareem"
    assert speaker["engine"] == "elevenlabs"
    assert speaker["voice"] == "VlUmeC1Uzj3NnwiVR9K9"
    assert speaker["character_profile"] == "Black male perspective"
