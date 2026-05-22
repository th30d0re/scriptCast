import tempfile
from pathlib import Path
from unittest import TestCase

from voice_pipeline.logic_exporter import (
    _escape_applescript_string,
    _generate_applescript,
    _ms_to_smpte,
)
from voice_pipeline.models import SegmentResult


class LogicExporterTests(TestCase):
    def test_escape_applescript_string_escapes_quotes_and_backslashes(self) -> None:
        self.assertEqual(_escape_applescript_string('hello "world"'), 'hello \\"world\\"')
        self.assertEqual(_escape_applescript_string("path\\to\\file"), "path\\\\to\\\\file")

    def test_ms_to_smpte_converts_to_timecode(self) -> None:
        self.assertEqual(_ms_to_smpte(0), "00:00:00:00")
        self.assertEqual(_ms_to_smpte(1000), "00:00:01:00")
        self.assertEqual(_ms_to_smpte(1500), "00:00:01:15")
        self.assertEqual(_ms_to_smpte(3661000), "01:01:01:00")

    def test_generate_applescript_creates_track_per_speaker(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            segments = [
                self._segment(project_root, 0, "emmanuel_theodore", 1000, 950, 250),
                self._segment(project_root, 1, "toussaint", 500, 450, 100),
                self._segment(project_root, 2, "aisha", 750, 700, 0),
            ]
            script = _generate_applescript(segments, None, "TestEpisode", update_existing=False)

        self.assertIn('tell application "Logic Pro"', script)
        self.assertIn("set proj to make new project", script)
        self.assertIn('make new audio track at end of tracks with properties {name:"Emmanuel Theodore"}', script)
        self.assertIn('make new audio track at end of tracks with properties {name:"Toussaint"}', script)
        self.assertIn('make new audio track at end of tracks with properties {name:"Aisha"}', script)

    def test_generate_applescript_uses_existing_project_when_update_existing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            segments = [
                self._segment(project_root, 0, "toussaint", 500, 450, 100),
            ]
            script = _generate_applescript(segments, None, "TestEpisode", update_existing=True)

        self.assertIn("set proj to front project", script)
        self.assertNotIn("make new project", script)

    def test_generate_applescript_positions_segments_correctly(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            segments = [
                self._segment(project_root, 0, "toussaint", 1000, 950, 250),
            ]
            position_map = {("id0", 0): 5000}
            script = _generate_applescript(segments, position_map, "TestEpisode")

        self.assertIn("00:00:05:00", script)
        self.assertIn("set position of newRegion to", script)

    def test_generate_applescript_imports_to_correct_track(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            project_root = Path(tmp)
            segments = [
                self._segment(project_root, 0, "emmanuel_theodore", 1000, 950, 250),
                self._segment(project_root, 1, "toussaint", 500, 450, 100),
            ]
            script = _generate_applescript(segments, None, "TestEpisode")

        # Emmanuel Theodore is first speaker -> track 1
        # Toussaint is second speaker -> track 2
        self.assertIn("tell track 1 of proj", script)
        self.assertIn("tell track 2 of proj", script)

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
        wav_path.write_bytes(b"RIFF" + b"\x00" * 100)
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
