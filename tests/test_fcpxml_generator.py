import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path
from unittest import TestCase

from voice_pipeline.fcpxml_generator import generate_fcpxml
from voice_pipeline.models import SegmentResult


class GenerateFcpxmlTests(TestCase):
    def test_generates_fcpxml_with_resources_and_clips(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            output_path = project_root / "test.fcpxml"
            segments = [
                self._segment(project_root, 0, "emmanuel_theodore", 1000, 950, 250),
                self._segment(project_root, 1, "toussaint", 500, 450, 100),
                self._segment(project_root, 2, "aisha", 750, 700, 0),
            ]

            result_path = generate_fcpxml(segments, output_path, episode_id="test_ep")
            root = ET.fromstring(result_path.read_bytes())

        self.assertEqual(result_path, output_path)
        self.assertEqual(root.tag, "fcpxml")
        self.assertEqual(root.attrib.get("version"), "1.10")

        # Resources
        resources = root.find("resources")
        self.assertIsNotNone(resources)
        assets = resources.findall("asset")
        self.assertEqual(len(assets), len(segments))

        # Library > Event > Project > Sequence > Spine
        spine = root.find(".//spine")
        self.assertIsNotNone(spine)

        # Gap on primary storyline
        gap = spine.find("gap")
        self.assertIsNotNone(gap)

        # Clips are children of the gap
        clips = gap.findall("asset-clip")
        self.assertEqual(len(clips), len(segments))

        # Verify lanes are assigned per speaker
        lanes = {clip.attrib["lane"] for clip in clips}
        self.assertEqual(len(lanes), 3)  # 3 speakers, 3 lanes

        # Verify offsets match sequential placement (rounded to 1/30s frames)
        self.assertEqual(clips[0].attrib["offset"], "0s")
        # 1250ms = 1.25s -> 37.5 frames -> rounds to 38 frames = 38/30s
        self.assertEqual(clips[1].attrib["offset"], "38/30s")

    def test_position_map_preserves_segment_positions(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            output_path = project_root / "test.fcpxml"
            segments = [
                self._segment(project_root, 0, "emmanuel_theodore", 1000, 950, 250),
                self._segment(project_root, 1, "toussaint", 500, 450, 100),
            ]
            position_map = {("id0", 0): 0, ("id1", 0): 5000}

            result_path = generate_fcpxml(segments, output_path, position_map=position_map)
            root = ET.fromstring(result_path.read_bytes())

        clips = root.findall(".//asset-clip")
        self.assertEqual(clips[0].attrib["offset"], "0s")
        # 5000ms = 5s -> 150 frames exactly
        self.assertEqual(clips[1].attrib["offset"], "150/30s")

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
        wav_path = project_root / f"{turn_id}_chunk_0000.wav"
        wav_path.write_bytes(b"RIFF" + b"\x00" * 100)  # dummy wav header
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
            checksum="abcd",
        )
