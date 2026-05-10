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
        self.assertIsNotNone(root.find("./LiveSet/MainSequence"))

        tracks = root.findall("./LiveSet/Tracks/AudioTrack")
        self.assertEqual(len(tracks), 3)
        self.assertEqual(
            [
                "emmanuel_theodore",
                "ai_1",
                "ai_2",
            ],
            [
                track.find("./Name/EffectiveName").attrib["Value"]
                for track in tracks
            ],
        )

        for track in tracks:
            self.assertIsNotNone(track.find("./AutomationEnvelopes"))
            self.assertIsNotNone(
                track.find("./DeviceChain/MainSequence/ClipTimeable/ArrangedClips")
            )

        clips = root.findall(".//AudioClip")
        self.assertEqual(len(clips), len(segments))

        self.assertEqual(
            {
                "turn_0000_chunk_0000.wav": "0.0",
                "turn_0001_chunk_0000.wav": "2.5",
                "turn_0002_chunk_0000.wav": "3.7",
            },
            {
                clip.find("./Name").attrib["Value"]: clip.attrib["Time"]
                for clip in clips
            },
        )

        arranged_clips = root.findall(".//ArrangedClip")
        self.assertEqual(len(arranged_clips), len(segments))
        self.assertEqual(
            [clip.attrib["Time"] for clip in clips],
            [arranged_clip.attrib["Time"] for arranged_clip in arranged_clips],
        )

        file_refs = root.findall(".//FileRef")
        self.assertEqual(len(file_refs), len(segments))
        for file_ref in file_refs:
            relative_path = file_ref.attrib["RelativePath"]
            self.assertEqual(file_ref.attrib["HasRelativePath"], "true")
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
        return SegmentResult(
            turn_index=turn_index,
            chunk_index=0,
            speaker_id=speaker_id,
            wav_path=(
                project_root
                / "Samples"
                / "Processed"
                / speaker_id
                / f"turn_{turn_index:04d}_chunk_0000.wav"
            ),
            duration_ms=duration_ms,
            sample_rate=48000,
            gap_after_ms=gap_after_ms,
            checksum=f"checksum-{turn_index}",
        )
