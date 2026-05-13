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
                self._segment(project_root, 0, "emmanuel_theodore", 1000, 250),
                self._segment(project_root, 1, "ai_1", 500, 100),
                self._segment(project_root, 2, "ai_2", 250, 0),
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

        self.assertEqual(
            {
                "turn_0000_chunk_0000": "0",
                "turn_0001_chunk_0000": "2.5",
                "turn_0002_chunk_0000": "3.7",
            },
            {
                clip.find("./Name").attrib["Value"]: clip.attrib["Time"]
                for clip in clips
            },
        )
        first_clip = next(
            clip
            for clip in clips
            if clip.find("./Name").attrib["Value"] == "turn_0000_chunk_0000"
        )
        self.assertEqual(first_clip.find("./CurrentEnd").attrib["Value"], "2")
        self.assertEqual(first_clip.find("./Loop/LoopEnd").attrib["Value"], "2")
        self.assertEqual(first_clip.find("./Loop/OutMarker").attrib["Value"], "2")

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

    def test_unknown_speaker_id_fails_before_writing_als(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            output_path = project_root / "ATO_EP0.als"
            segment = self._segment(project_root, 0, "guest", 1000, 0)

            with self.assertRaisesRegex(ValueError, "Unknown speaker_id 'guest'"):
                generate_als([segment], output_path)

            self.assertFalse(output_path.exists())

    def _segment(
        self,
        project_root: Path,
        turn_index: int,
        speaker_id: str,
        duration_ms: int,
        gap_after_ms: int,
    ) -> SegmentResult:
        wav_path = (
            project_root
            / "Samples"
            / "Processed"
            / speaker_id
            / f"turn_{turn_index:04d}_chunk_0000.wav"
        )
        wav_path.parent.mkdir(parents=True, exist_ok=True)
        wav_path.write_bytes(b"test wav content")

        return SegmentResult(
            turn_index=turn_index,
            chunk_index=0,
            speaker_id=speaker_id,
            wav_path=wav_path,
            duration_ms=duration_ms,
            sample_rate=48000,
            gap_after_ms=gap_after_ms,
            checksum=f"checksum-{turn_index}",
        )
