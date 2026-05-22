from pathlib import Path
from unittest import TestCase

from voice_pipeline.models import Turn
from voice_pipeline.render_state import (
    RenderState,
    SegmentPosition,
    TurnFingerprint,
    build_position_map,
    compute_source_hash,
    compute_turn_fingerprint,
    detect_changed_turns,
    load_render_state,
    parse_turn_spec,
    plan_precision_insert,
    save_render_state,
)


class ParseTurnSpecTests(TestCase):
    def test_single_index(self) -> None:
        self.assertEqual(parse_turn_spec("5"), [5])

    def test_multiple_indices(self) -> None:
        self.assertEqual(parse_turn_spec("5,10,15"), [5, 10, 15])

    def test_range(self) -> None:
        self.assertEqual(parse_turn_spec("5-8"), [5, 6, 7, 8])

    def test_mixed(self) -> None:
        self.assertEqual(parse_turn_spec("5,10-12,20"), [5, 10, 11, 12, 20])

    def test_deduplicates(self) -> None:
        self.assertEqual(parse_turn_spec("5,5,5-7"), [5, 6, 7])


class ComputeTurnFingerprintTests(TestCase):
    def test_same_turn_same_fingerprint(self) -> None:
        turn = Turn(
            turn_index=0,
            turn_id="abc123",
            speaker_id="toussaint",
            display_name="Toussaint",
            timestamp_mmss="00:00",
            timestamp_ms=0,
            raw_text="Hello world.",
            clean_text="Hello world.",
            line_span=(1, 2),
        )
        fp1 = compute_turn_fingerprint(turn)
        fp2 = compute_turn_fingerprint(turn)
        self.assertEqual(fp1.text_hash, fp2.text_hash)
        self.assertEqual(fp1.speaker_id, "toussaint")
        self.assertEqual(fp1.turn_id, "abc123")

    def test_different_text_different_hash(self) -> None:
        turn1 = Turn(
            turn_index=0,
            turn_id="abc123",
            speaker_id="toussaint",
            display_name="Toussaint",
            timestamp_mmss="00:00",
            timestamp_ms=0,
            raw_text="Hello.",
            clean_text="Hello.",
            line_span=(1, 2),
        )
        turn2 = Turn(
            turn_index=0,
            turn_id="abc123",
            speaker_id="toussaint",
            display_name="Toussaint",
            timestamp_mmss="00:00",
            timestamp_ms=0,
            raw_text="Goodbye.",
            clean_text="Goodbye.",
            line_span=(1, 2),
        )
        fp1 = compute_turn_fingerprint(turn1)
        fp2 = compute_turn_fingerprint(turn2)
        self.assertNotEqual(fp1.text_hash, fp2.text_hash)


class DetectChangedTurnsTests(TestCase):
    def test_no_changes(self) -> None:
        turns = [
            Turn(
                turn_index=0,
                turn_id="abc123",
                speaker_id="toussaint",
                display_name="Toussaint",
                timestamp_mmss="00:00",
                timestamp_ms=0,
                raw_text="Hello.",
                clean_text="Hello.",
                line_span=(1, 2),
            )
        ]
        previous = RenderState(
            schema_version="2.0",
            source_file="test.md",
            source_hash="abc",
            rendered_at="2026-01-01",
            turns=[compute_turn_fingerprint(turns[0])],
            segments=[],
        )
        self.assertEqual(detect_changed_turns(turns, previous), [])

    def test_text_changed(self) -> None:
        turns = [
            Turn(
                turn_index=0,
                turn_id="abc123",
                speaker_id="toussaint",
                display_name="Toussaint",
                timestamp_mmss="00:00",
                timestamp_ms=0,
                raw_text="Goodbye.",
                clean_text="Goodbye.",
                line_span=(1, 2),
            )
        ]
        previous = RenderState(
            schema_version="2.0",
            source_file="test.md",
            source_hash="abc",
            rendered_at="2026-01-01",
            turns=[
                TurnFingerprint(
                    turn_id="abc123",
                    speaker_id="toussaint",
                    text_hash="oldhash",
                    segment_count=1,
                )
            ],
            segments=[],
        )
        self.assertEqual(detect_changed_turns(turns, previous), [0])

    def test_new_turn(self) -> None:
        turns = [
            Turn(
                turn_index=0,
                turn_id="abc123",
                speaker_id="toussaint",
                display_name="Toussaint",
                timestamp_mmss="00:00",
                timestamp_ms=0,
                raw_text="Hello.",
                clean_text="Hello.",
                line_span=(1, 2),
            ),
            Turn(
                turn_index=1,
                turn_id="def456",
                speaker_id="aisha",
                display_name="Aisha",
                timestamp_mmss="00:01",
                timestamp_ms=1000,
                raw_text="Hi.",
                clean_text="Hi.",
                line_span=(3, 4),
            ),
        ]
        previous = RenderState(
            schema_version="2.0",
            source_file="test.md",
            source_hash="abc",
            rendered_at="2026-01-01",
            turns=[compute_turn_fingerprint(turns[0])],
            segments=[],
        )
        self.assertEqual(detect_changed_turns(turns, previous), [1])

    def test_deleted_turn(self) -> None:
        turns = [
            Turn(
                turn_index=0,
                turn_id="abc123",
                speaker_id="toussaint",
                display_name="Toussaint",
                timestamp_mmss="00:00",
                timestamp_ms=0,
                raw_text="Hello.",
                clean_text="Hello.",
                line_span=(1, 2),
            )
        ]
        previous = RenderState(
            schema_version="2.0",
            source_file="test.md",
            source_hash="abc",
            rendered_at="2026-01-01",
            turns=[
                compute_turn_fingerprint(turns[0]),
                TurnFingerprint(
                    turn_id="def456",
                    speaker_id="aisha",
                    text_hash="oldhash",
                    segment_count=1,
                ),
            ],
            segments=[],
        )
        self.assertEqual(detect_changed_turns(turns, previous), ["def456"])


class SaveLoadRenderStateTests(TestCase):
    def test_round_trip(self) -> None:
        state = RenderState(
            schema_version="2.0",
            source_file="test.md",
            source_hash="abc123",
            rendered_at="2026-01-01T00:00:00Z",
            turns=[
                TurnFingerprint(
                    turn_id="abc123",
                    speaker_id="toussaint",
                    text_hash="hash1",
                    segment_count=1,
                )
            ],
            segments=[
                SegmentPosition(
                    turn_id="abc123",
                    chunk_index=0,
                    start_ms=0,
                    duration_ms=1000,
                    speech_duration_ms=850,
                    gap_after_ms=250,
                )
            ],
            als_path="/path/to/file.als",
            ableton_project_als_path="/path/to/ableton.als",
        )
        import tempfile
        with tempfile.TemporaryDirectory() as tmp:
            out_dir = Path(tmp) / "episode"
            out_dir.mkdir()
            save_render_state(out_dir, state)
            loaded = load_render_state(out_dir)
            assert loaded is not None
            self.assertEqual(loaded.source_hash, "abc123")
            self.assertEqual(loaded.turns[0].speaker_id, "toussaint")
            self.assertEqual(loaded.segments[0].start_ms, 0)
            self.assertEqual(loaded.als_path, "/path/to/file.als")


class BuildPositionMapTests(TestCase):
    def test_maps_segments(self) -> None:
        state = RenderState(
            schema_version="2.0",
            source_file="test.md",
            source_hash="abc",
            rendered_at="2026-01-01",
            turns=[],
            segments=[
                SegmentPosition(
                    turn_id="abc123",
                    chunk_index=0,
                    start_ms=0,
                    duration_ms=1000,
                    speech_duration_ms=850,
                    gap_after_ms=250,
                ),
                SegmentPosition(
                    turn_id="def456",
                    chunk_index=0,
                    start_ms=1250,
                    duration_ms=1000,
                    speech_duration_ms=850,
                    gap_after_ms=250,
                ),
            ],
        )
        position_map = build_position_map(state)
        self.assertEqual(position_map[("abc123", 0)], 0)
        self.assertEqual(position_map[("def456", 0)], 1250)
