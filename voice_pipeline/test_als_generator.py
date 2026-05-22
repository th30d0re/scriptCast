import gzip
import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path
from unittest import TestCase

from voice_pipeline.als_generator import generate_als
from voice_pipeline.models import SegmentResult


class GenerateAlsTests(TestCase):
    def test_generates_gzip_xml_with_tracks_clips_timeline_and_file_refs(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            output_path = project_root / "ATO_EP0.als"
            segments = [
                self._segment(project_root, 0, "emmanuel_theodore", 1000, 800, 250),
                self._segment(project_root, 1, "ai_1", 500, 400, 100),
                self._segment(project_root, 2, "ai_2", 250, 200, 0),
            ]

            result_path = generate_als(segments, output_path)
            xml_bytes = gzip.open(result_path, "rb").read()
            root = ET.fromstring(xml_bytes)

        self.assertEqual(result_path, output_path)
        self.assertEqual(root.tag, "Ableton")
        self.assertIsNotNone(root.find("./LiveSet/MainTrack"))

        tracks = root.findall("./LiveSet/Tracks/AudioTrack")
        track_names = [
            track.find("./Name/EffectiveName").attrib["Value"] for track in tracks
        ]
        self.assertIn("emmanuel_theodore", track_names)
        self.assertIn("ai_1", track_names)
        self.assertIn("ai_2", track_names)

        for track in tracks:
            self.assertIsNotNone(track.find("./AutomationEnvelopes"))
            self.assertIsNotNone(
                track.find(
                    "./DeviceChain/MainSequencer/Sample/ArrangerAutomation/Events"
                )
            )

        clips = root.findall(".//AudioClip")
        self.assertEqual(len(clips), len(segments))

        # Cursor advances by speech_duration_ms + gap_after_ms:
        # seg0: 0 + 800 + 250 = 1050ms -> 1050/500 = 2.1 beats
        # seg1: 2.1 + 400 + 100 = 2600ms -> 2600/500 = 5.2 beats (but wait, cursor is in ms)
        # Actually cursor_ms: 0 -> 0 + 800 + 250 = 1050 -> 1050 + 400 + 100 = 1550
        # In beats: 0, 1050/500=2.1, 1550/500=3.1
        self.assertEqual(
            {
                "id0_chunk_0000": "0",
                "id1_chunk_0000": "2.1",
                "id2_chunk_0000": "3.1",
            },
            {
                clip.find("./Name").attrib["Value"]: clip.attrib["Time"]
                for clip in clips
            },
        )
        first_clip = next(
            clip
            for clip in clips
            if clip.find("./Name").attrib["Value"] == "id0_chunk_0000"
        )
        # length_beats from speech_duration_ms = 800/500 = 1.6
        self.assertEqual(first_clip.find("./CurrentEnd").attrib["Value"], "1.6")
        self.assertEqual(first_clip.find("./Loop/LoopEnd").attrib["Value"], "1.6")
        self.assertEqual(first_clip.find("./Loop/OutMarker").attrib["Value"], "1.6")

        # Warp marker BeatTime must describe the full WAV file, not the clipped region.
        # duration_ms = 1000 -> 1000/500 = 2.0 beats
        warp_markers = first_clip.findall("./WarpMarkers/WarpMarker")
        self.assertEqual(len(warp_markers), 2)
        self.assertEqual(warp_markers[0].attrib["BeatTime"], "0")
        self.assertEqual(warp_markers[1].attrib["BeatTime"], "2")

        event_clips = root.findall(
            ".//MainSequencer/Sample/ArrangerAutomation/Events/AudioClip"
        )
        self.assertEqual(len(event_clips), len(segments))
        self.assertEqual(
            [clip.attrib["Time"] for clip in clips],
            [clip.attrib["Time"] for clip in event_clips],
        )

        file_refs = root.findall(".//AudioClip/SampleRef/FileRef")
        self.assertEqual(len(file_refs), len(segments))
        for file_ref in file_refs:
            relative_path = file_ref.find("./RelativePath").attrib["Value"]
            self.assertEqual(file_ref.find("./RelativePathType").attrib["Value"], "1")
            self.assertTrue(relative_path.startswith("Samples/Processed/"))
            self.assertFalse(Path(relative_path).is_absolute())

    def test_dynamic_speaker_discovery_orders_by_first_appearance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            output_path = project_root / "ATO_EP0.als"
            # ai_1 appears first, then emmanuel_theodore, then guest
            segments = [
                self._segment(project_root, 0, "ai_1", 500, 400, 100),
                self._segment(project_root, 1, "emmanuel_theodore", 1000, 800, 250),
                self._segment(project_root, 2, "guest", 250, 200, 0),
            ]

            result_path = generate_als(segments, output_path)
            xml_bytes = gzip.open(result_path, "rb").read()
            root = ET.fromstring(xml_bytes)

        tracks = root.findall("./LiveSet/Tracks/AudioTrack")
        track_names = [
            track.find("./Name/EffectiveName").attrib["Value"]
            for track in tracks
            if track.find("./Name/EffectiveName") is not None
        ]
        # Only the used speaker tracks are renamed; filter for our speakers
        self.assertEqual(
            [name for name in track_names if name in ("ai_1", "emmanuel_theodore", "guest")],
            ["ai_1", "emmanuel_theodore", "guest"],
        )

    def test_empty_segments_raises(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            output_path = project_root / "ATO_EP0.als"

            with self.assertRaisesRegex(ValueError, "No segments"):
                generate_als([], output_path)

    def test_position_map_preserves_exact_start_ms(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            output_path = project_root / "ATO_EP0.als"
            segments = [
                self._segment(project_root, 0, "emmanuel_theodore", 1000, 800, 250),
                self._segment(project_root, 1, "aisha", 500, 400, 100),
                self._segment(project_root, 2, "toussaint", 250, 200, 0),
            ]
            # Force custom positions: seg0 at 0ms, seg1 at 5000ms, seg2 at 10000ms
            position_map = {
                ("id0", 0): 0,
                ("id1", 0): 5000,
                ("id2", 0): 10000,
            }

            result_path = generate_als(segments, output_path, position_map=position_map)
            xml_bytes = gzip.open(result_path, "rb").read()
            root = ET.fromstring(xml_bytes)
            clips = root.findall(".//AudioClip")

            self.assertEqual(
                {
                    "id0_chunk_0000": "0",
                    "id1_chunk_0000": "10",       # 5000ms / 500ms_per_beat = 10 beats
                    "id2_chunk_0000": "20",       # 10000ms / 500 = 20 beats
                },
                {
                    clip.find("./Name").attrib["Value"]: clip.attrib["Time"]
                    for clip in clips
                },
            )

    def test_backup_created_on_overwrite(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            output_path = project_root / "ATO_EP0.als"
            segments = [
                self._segment(project_root, 0, "emmanuel_theodore", 1000, 800, 250),
            ]
            generate_als(segments, output_path)
            self.assertTrue(output_path.exists())

            # Generate again — should create a backup
            generate_als(segments, output_path)
            backups = list(project_root.glob("*.backup_*.als"))
            self.assertEqual(len(backups), 1)
            self.assertTrue(backups[0].name.startswith("ATO_EP0.backup_"))

    def _segment(
        self,
        project_root: Path,
        turn_index: int,
        speaker_id: str,
        duration_ms: int,
        speech_duration_ms: int,
        gap_after_ms: int,
    ) -> SegmentResult:
        turn_id = f"id{turn_index}"
        wav_path = (
            project_root
            / "Samples"
            / "Processed"
            / speaker_id
            / f"{turn_id}_chunk_0000.wav"
        )
        wav_path.parent.mkdir(parents=True, exist_ok=True)
        wav_path.write_bytes(b"test wav content")

        return SegmentResult(
            turn_index=turn_index,
            turn_id=turn_id,
            chunk_index=0,
            speaker_id=speaker_id,
            wav_path=wav_path,
            duration_ms=duration_ms,
            speech_duration_ms=speech_duration_ms,
            sample_rate=48000,
            gap_after_ms=gap_after_ms,
            checksum=f"checksum-{turn_index}",
        )
