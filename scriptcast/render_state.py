"""Render state tracking for selective regeneration and change detection."""

from __future__ import annotations

import hashlib
import json
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path

from scriptcast.models import Turn
from scriptcast.pronunciation import speech_text_hash


@dataclass
class SegmentPosition:
    turn_id: str
    chunk_index: int
    start_ms: int
    duration_ms: int
    speech_duration_ms: int
    gap_after_ms: int


@dataclass
class TurnFingerprint:
    turn_id: str
    speaker_id: str
    text_hash: str
    segment_count: int
    # How this speaker was voiced. Change detection used to compare text alone,
    # so switching a speaker's engine or reference audio left every unchanged
    # turn holding audio from the previous voice. One episode spent a day as a
    # mix of Kokoro, Chatterbox and OmniVoice clips because of it.
    voice_hash: str = ""
    # Identity of the text the engine was actually handed, when a heteronym
    # respelling changed it (pronunciation.py). Empty when nothing changed, so
    # adding a respelling re-renders only the turns that use the word.
    speech_text_hash: str = ""


@dataclass
class RenderState:
    schema_version: str
    source_file: str
    source_hash: str
    rendered_at: str
    turns: list[TurnFingerprint]
    segments: list[SegmentPosition]
    als_path: str | None = None
    fcpxml_path: str | None = None
    ableton_project_als_path: str | None = None

    refit_settings: dict[str, int | float] | None = None

    def to_dict(self) -> dict[str, object]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict[str, object]) -> "RenderState":
        turns = [
            TurnFingerprint(**t) for t in data.get("turns", [])  # type: ignore[arg-type]
        ]
        segments = [
            SegmentPosition(**s) for s in data.get("segments", [])  # type: ignore[arg-type]
        ]
        return cls(
            refit_settings=data.get("refit_settings"),
            schema_version=data.get("schema_version", "1.0"),  # type: ignore[arg-type]
            source_file=data.get("source_file", ""),  # type: ignore[arg-type]
            source_hash=data.get("source_hash", ""),  # type: ignore[arg-type]
            rendered_at=data.get("rendered_at", ""),  # type: ignore[arg-type]
            turns=turns,
            segments=segments,
            als_path=data.get("als_path"),  # type: ignore[arg-type]
            fcpxml_path=data.get("fcpxml_path"),  # type: ignore[arg-type]
            ableton_project_als_path=data.get("ableton_project_als_path"),  # type: ignore[arg-type]
        )


def compute_source_hash(path: Path) -> str:
    hasher = hashlib.sha256()
    hasher.update(path.read_bytes())
    return hasher.hexdigest()


def compute_voice_hash(voice) -> str:
    """Identity of the voice a turn was rendered with.

    Covers everything that changes how the audio sounds: the engine, the model
    checkpoint, the reference clip and its transcript, and the sampling
    settings. A change to any of them invalidates that speaker's turns and
    nobody else's.
    """
    if voice is None:
        return ""
    parts = [
        getattr(voice, "engine", ""),
        getattr(voice, "kokoro_voice", "") or "",
        getattr(voice, "elevenlabs_voice_id", "") or "",
        getattr(voice, "reference_audio", "") or "",
        getattr(voice, "reference_text", "") or "",
        getattr(voice, "reference_audio_expressive", "") or "",
        getattr(voice, "reference_text_expressive", "") or "",
        f"{getattr(voice, 'temperature', '')}",
        f"{getattr(voice, 'cfg_weight', '')}",
        f"{getattr(voice, 'exaggeration', '')}",
        f"{getattr(voice, 'speed', '')}",
    ]
    return hashlib.sha256("|".join(parts).encode("utf-8")).hexdigest()[:16]


def compute_turn_fingerprint(turn: Turn, voice=None) -> TurnFingerprint:
    text_hash = hashlib.sha256(turn.clean_text.encode("utf-8")).hexdigest()
    speech_texts = [
        chunk.text
        for chunk in turn.markup_chunks
        if chunk.kind == "speech" and chunk.text and chunk.text.strip()
    ]
    return TurnFingerprint(
        turn_id=turn.turn_id,
        speaker_id=turn.speaker_id,
        text_hash=text_hash,
        segment_count=len(speech_texts),
        voice_hash=compute_voice_hash(voice),
        speech_text_hash=_audio_source_hash(turn, voice, speech_texts),
    )


def _audio_source_hash(turn: Turn, voice, speech_texts: list[str]) -> str:
    """What the engine renders from, beyond the script text.

    For synthetic voices that is the respelled text, when a respelling changed
    it. For archival clips it is the source file and the cut points, so
    re-trimming a clip re-renders that turn and nothing else.
    """
    if voice is None:
        return ""
    engine = getattr(voice, "engine", "")
    if engine == "archive":
        from scriptcast.archive import clip_fingerprint, clip_id_in

        clip_id = clip_id_in(turn.clean_text)
        return clip_fingerprint(clip_id) if clip_id else ""
    return speech_text_hash(engine, speech_texts)


def _fingerprint_changed(prev: TurnFingerprint, fp: TurnFingerprint) -> bool:
    return bool(
        prev.speaker_id != fp.speaker_id
        or prev.text_hash != fp.text_hash
        or prev.segment_count != fp.segment_count
        # An empty stored hash means the state predates voice tracking;
        # treat that as "unknown" rather than "changed" so an old episode
        # does not re-render wholesale on first contact.
        or (prev.voice_hash and fp.voice_hash and prev.voice_hash != fp.voice_hash)
        # Unlike the voice hash, an empty value here is meaningful: it says the
        # engine got the plain text. A turn rendered before a respelling existed
        # has to re-render to pick it up.
        or (fp.voice_hash and prev.speech_text_hash != fp.speech_text_hash)
    )


def load_render_state(episode_out_dir: Path) -> RenderState | None:
    state_path = episode_out_dir / "render_state.json"
    if not state_path.exists():
        return None
    data = json.loads(state_path.read_text(encoding="utf-8"))
    return RenderState.from_dict(data)


def save_render_state(episode_out_dir: Path, state: RenderState) -> None:
    state_path = episode_out_dir / "render_state.json"
    state_path.write_text(
        json.dumps(state.to_dict(), indent=2) + "\n",
        encoding="utf-8",
    )


def detect_changed_turns(
    current_turns: list[Turn], previous_state: RenderState, voices=None
) -> list[str]:
    """Return turn IDs that differ from the previous render.

    `voices` maps speaker_id to VoiceConfig. Pass it so a change of engine or
    reference audio invalidates that speaker's turns; omit it and only text is
    compared, which is the old behaviour.
    """
    previous_by_id: dict[str, TurnFingerprint] = {
        fp.turn_id: fp for fp in previous_state.turns
    }
    changed: set[str] = set()

    for turn in current_turns:
        fp = compute_turn_fingerprint(
            turn, (voices or {}).get(turn.speaker_id)
        )
        prev = previous_by_id.get(turn.turn_id)
        if prev is None:
            changed.add(turn.turn_id)
        elif _fingerprint_changed(prev, fp):
            changed.add(turn.turn_id)

    # Detect deleted turns (present in previous but not current)
    current_ids = {t.turn_id for t in current_turns}
    for prev_fp in previous_state.turns:
        if prev_fp.turn_id not in current_ids:
            changed.add(prev_fp.turn_id)

    return sorted(changed)


def parse_turn_spec(spec: str) -> list[int]:
    """Parse a turn spec like '5,10-15,20' into [5,10,11,12,13,14,15,20]."""
    indices: list[int] = []
    for part in spec.split(","):
        part = part.strip()
        if "-" in part:
            start_str, end_str = part.split("-", 1)
            start = int(start_str.strip())
            end = int(end_str.strip())
            indices.extend(range(start, end + 1))
        else:
            indices.append(int(part))
    return sorted(set(indices))


def build_position_map(
    state: RenderState,
) -> dict[tuple[str, int], int]:
    """Map (turn_id, chunk_index) → start_ms from stored segment positions."""
    return {
        (seg.turn_id, seg.chunk_index): seg.start_ms
        for seg in state.segments
    }


# ---------------------------------------------------------------------------
# Precision insert planning
# ---------------------------------------------------------------------------

@dataclass
class PrecisionInsertPlan:
    """Result of comparing a new transcript against a previous render state."""

    inserted: list[Turn]
    """New turns that need synthesis."""

    modified: list[Turn]
    """Existing turns whose content changed and need re-synthesis."""

    deleted_ids: list[str]
    """IDs of turns present in old state but not in new transcript."""

    unchanged: list[Turn]
    """Existing turns that can be loaded from disk without re-synthesis."""


def plan_precision_insert(
    current_turns: list[Turn], previous_state: RenderState, voices=None
) -> PrecisionInsertPlan:
    """Compare current transcript to previous render state and classify turns.

    - **Inserted**: turn_id not in previous state → synthesize
    - **Modified**: turn_id in previous state but fingerprint differs → resynthesize
    - **Deleted**: turn_id in previous state but not in current → delete files
    - **Unchanged**: turn_id in previous state and fingerprint matches → load existing

    Pass `voices` (speaker_id to VoiceConfig). Without it a change of reference
    audio or engine goes unnoticed and the episode keeps the old voice's clips.
    """
    previous_by_id: dict[str, TurnFingerprint] = {
        fp.turn_id: fp for fp in previous_state.turns
    }

    inserted: list[Turn] = []
    modified: list[Turn] = []
    unchanged: list[Turn] = []
    current_ids: set[str] = set()

    for turn in current_turns:
        current_ids.add(turn.turn_id)
        fp = compute_turn_fingerprint(turn, (voices or {}).get(turn.speaker_id))
        prev = previous_by_id.get(turn.turn_id)
        if prev is None:
            inserted.append(turn)
        elif _fingerprint_changed(prev, fp):
            modified.append(turn)
        else:
            unchanged.append(turn)

    deleted_ids = [
        prev_fp.turn_id
        for prev_fp in previous_state.turns
        if prev_fp.turn_id not in current_ids
    ]

    return PrecisionInsertPlan(
        inserted=inserted,
        modified=modified,
        deleted_ids=deleted_ids,
        unchanged=unchanged,
    )


# ---------------------------------------------------------------------------
# Migration helpers (v1.0 → v2.0)
# ---------------------------------------------------------------------------

def _is_v1_state(data: dict[str, object]) -> bool:
    """Detect legacy v1.0 state that used turn_index instead of turn_id."""
    turns = data.get("turns", [])
    if not turns:
        return False
    first_turn = turns[0]
    if isinstance(first_turn, dict):
        return "turn_index" in first_turn and "turn_id" not in first_turn
    return False


def _migrate_v1_to_v2(
    data: dict[str, object], current_turns: list[Turn]
) -> RenderState:
    """Upgrade a v1.0 state to v2.0 by matching old fingerprints to current turns."""
    # Build lookup from old (turn_index, speaker_id, text_hash) -> current turn
    current_by_hash: dict[tuple[str, str], Turn] = {}
    for turn in current_turns:
        text_hash = hashlib.sha256(turn.clean_text.encode("utf-8")).hexdigest()
        key = (turn.speaker_id, text_hash)
        # If duplicates exist, the last one wins (rare edge case)
        current_by_hash[key] = turn

    old_turns = data.get("turns", [])
    old_segments = data.get("segments", [])

    migrated_turns: list[TurnFingerprint] = []
    id_mapping: dict[int, str] = {}  # old turn_index -> new turn_id

    for old_turn in old_turns:
        if not isinstance(old_turn, dict):
            continue
        old_index = old_turn.get("turn_index")
        old_speaker = old_turn.get("speaker_id", "")
        old_hash = old_turn.get("text_hash", "")
        old_count = old_turn.get("segment_count", 0)

        matched = current_by_hash.get((old_speaker, old_hash))
        if matched is not None:
            id_mapping[old_index] = matched.turn_id
            migrated_turns.append(
                TurnFingerprint(
                    turn_id=matched.turn_id,
                    speaker_id=old_speaker,
                    text_hash=old_hash,
                    segment_count=old_count,
                )
            )

    migrated_segments: list[SegmentPosition] = []
    for old_seg in old_segments:
        if not isinstance(old_seg, dict):
            continue
        old_index = old_seg.get("turn_index")
        new_id = id_mapping.get(old_index)
        if new_id is not None:
            migrated_segments.append(
                SegmentPosition(
                    turn_id=new_id,
                    chunk_index=old_seg.get("chunk_index", 0),
                    start_ms=old_seg.get("start_ms", 0),
                    duration_ms=old_seg.get("duration_ms", 0),
                    speech_duration_ms=old_seg.get("speech_duration_ms", 0),
                    gap_after_ms=old_seg.get("gap_after_ms", 0),
                )
            )

    return RenderState(
        schema_version="2.0",
        source_file=data.get("source_file", ""),  # type: ignore[arg-type]
        source_hash=data.get("source_hash", ""),  # type: ignore[arg-type]
        rendered_at=data.get("rendered_at", ""),  # type: ignore[arg-type]
        turns=migrated_turns,
        segments=migrated_segments,
        als_path=data.get("als_path"),  # type: ignore[arg-type]
        fcpxml_path=data.get("fcpxml_path"),  # type: ignore[arg-type]
        ableton_project_als_path=data.get("ableton_project_als_path"),  # type: ignore[arg-type]
    )


def load_render_state_with_migration(
    episode_out_dir: Path, current_turns: list[Turn] | None = None
) -> RenderState | None:
    """Load render state, automatically migrating v1.0 → v2.0 if needed."""
    state_path = episode_out_dir / "render_state.json"
    if not state_path.exists():
        return None

    data = json.loads(state_path.read_text(encoding="utf-8"))

    if _is_v1_state(data):
        if current_turns is None:
            raise RuntimeError(
                "Legacy v1.0 render state detected but no transcript turns provided. "
                "Run with --migrate to upgrade, or use --overwrite to regenerate."
            )
        return _migrate_v1_to_v2(data, current_turns)

    return RenderState.from_dict(data)
