"""Entry point for the local voice pipeline."""

from __future__ import annotations

import argparse
import asyncio
import hashlib
import json
import logging
import shutil
import time
from datetime import datetime
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

import numpy
import soundfile

from voice_pipeline.als_generator import generate_als
from voice_pipeline.engine import ENGINE_REGISTRY, TTSEngine
from voice_pipeline.fcpxml_generator import generate_fcpxml
from voice_pipeline.logic_exporter import export_to_logic
from voice_pipeline.manifest import write_manifest
from voice_pipeline.markup import tokenize_markup
from voice_pipeline.models import SegmentResult, Turn, VoiceConfig
from voice_pipeline.parser import parse_transcript
from voice_pipeline.platform_check import require_apple_silicon
from voice_pipeline.post_processor import _measure_speech_duration, process_segment
from voice_pipeline.render_state import (
    RenderState,
    SegmentPosition,
    build_position_map,
    compute_source_hash,
    compute_turn_fingerprint,
    detect_changed_turns,
    load_render_state_with_migration,
    parse_turn_spec,
    plan_precision_insert,
    save_render_state,
)
from voice_pipeline.voices import load_voices

_DEFAULT_MODEL = "prince-canuma/Kokoro-82M"
_DEFAULT_ENGINE = "mlx_kokoro"
_TARGET_SAMPLE_RATE = 48000
_DEFAULT_ELEVENLABS_MODEL = "eleven_multilingual_v2"


@dataclass(frozen=True)
class _SpeechJob:
    chunk_index: int
    markup_index: int
    text: str
    gap_after_ms: int


def _build_arg_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Render a transcript into voice segments."
    )
    parser.add_argument(
        "--transcript",
        type=Path,
        required=True,
        help="Path to markdown transcript",
    )
    parser.add_argument(
        "--episode-id",
        type=str,
        default=None,
        help="Episode identifier",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("./outputs"),
        help="Root output directory",
    )
    parser.add_argument(
        "--model",
        type=str,
        default=_DEFAULT_MODEL,
        help="HuggingFace model repo ID or ElevenLabs model ID",
    )
    parser.add_argument(
        "--engine",
        type=str,
        default=_DEFAULT_ENGINE,
        help="Engine registry key (legacy single-engine mode; ignored when voices.yaml specifies per-speaker engines)",
    )
    parser.add_argument(
        "--gap-ms",
        type=int,
        default=0,
        help="Default inter-turn silence in ms",
    )
    parser.add_argument(
        "--sample-seconds",
        type=float,
        default=None,
        help="Limit synthesis to first N seconds",
    )
    parser.add_argument(
        "--max-turns",
        type=int,
        default=None,
        help="Limit synthesis to first N turns",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Parse + manifest only, no audio",
    )
    parser.add_argument(
        "--overwrite",
        "-verwrite",
        action="store_true",
        help="Bypass resume/overwrite prompt",
    )
    parser.add_argument(
        "--speech-threshold",
        type=float,
        default=0.04,
        help="RMS threshold for speech-end detection (default: 0.04). "
             "Lower = looser trim, higher = tighter trim.",
    )
    parser.add_argument(
        "--compare-thresholds",
        nargs="?",
        const="default",
        default=None,
        help="Generate comparison ALS files at multiple thresholds without "
             "re-synthesizing audio. Use default set (0.02,0.03,0.04,0.05,0.06,0.08) "
             "or provide comma-separated values (e.g., 0.03,0.05,0.07).",
    )
    parser.add_argument(
        "--no-trim",
        action="store_true",
        help="Disable destructive edge-silence trimming on output WAVs. "
             "Preserves full audio including any leading/trailing silence.",
    )
    parser.add_argument(
        "--regenerate-turns",
        type=str,
        default=None,
        help="Comma-separated turn indices or ranges to re-synthesize (e.g. 5,10-15,20). "
             "Deletes existing WAVs for those turns and re-renders them. "
             "Other turns keep their exact start positions.",
    )
    parser.add_argument(
        "--detect-changes",
        action="store_true",
        help="Compare current transcript to previous render state and print which turns changed. "
             "No audio is synthesized.",
    )
    parser.add_argument(
        "--precision-insert",
        action="store_true",
        help="Only synthesize newly inserted or modified turns. Existing unchanged audio "
             "is loaded from disk and repositioned. Requires a previous render state.",
    )
    parser.add_argument(
        "--migrate",
        action="store_true",
        help="One-time migration: rename existing WAV files from old turn-index naming "
             "to new turn-id naming, and upgrade render_state.json from v1.0 to v2.0. "
             "No audio is synthesized.",
    )
    parser.add_argument(
        "--ableton-project-als",
        type=Path,
        default=None,
        help="Path to the Ableton Live project .als file. When rendering, the generated ALS "
             "is also copied here so Ableton sees the update.",
    )
    parser.add_argument(
        "--skip-als",
        action="store_true",
        help="Skip ALS generation. Only synthesize WAV files. Useful when you want to "
             "manually swap clips in an existing Ableton project.",
    )
    parser.add_argument(
        "--fcpxml",
        action="store_true",
        help="Generate a Final Cut Pro XML (.fcpxml) file alongside the ALS.",
    )
    parser.add_argument(
        "--logic",
        action="store_true",
        help="Export to Logic Pro X via AppleScript. Requires Logic Pro X to be "
             "installed and running on macOS.",
    )
    parser.add_argument(
        "--logic-dry-run",
        action="store_true",
        help="Generate the AppleScript for Logic Pro X export but do not execute it. "
             "Prints the script path instead.",
    )
    return parser


def _filter_turns(
    turns: list[Turn],
    max_turns: int | None,
) -> list[Turn]:
    filtered_turns = turns

    if max_turns is not None:
        if max_turns <= 0:
            raise SystemExit("--max-turns must be greater than 0")
        filtered_turns = filtered_turns[:max_turns]

    if not filtered_turns:
        raise SystemExit("No turns remain after applying filters")

    return filtered_turns


def _sample_budget_ms(sample_seconds: float | None) -> int | None:
    if sample_seconds is None:
        return None
    if sample_seconds <= 0:
        raise SystemExit("--sample-seconds must be greater than 0")
    return int(sample_seconds * 1000)


def _validate_voices(turns: list[Turn], voices: dict[str, VoiceConfig]) -> None:
    missing_speakers = sorted(
        {turn.speaker_id for turn in turns if turn.speaker_id not in voices}
    )
    if missing_speakers:
        missing_list = ", ".join(missing_speakers)
        raise SystemExit(f"No voice config found for speaker(s): {missing_list}")


def _has_existing_output(episode_out_dir: Path) -> bool:
    return (episode_out_dir / "episode_manifest.json").exists() or any(
        episode_out_dir.rglob("*.wav")
    )


def _prepare_episode_output(episode_out_dir: Path, overwrite: bool) -> bool:
    if not _has_existing_output(episode_out_dir):
        episode_out_dir.mkdir(parents=True, exist_ok=True)
        return False

    if overwrite:
        shutil.rmtree(episode_out_dir)
        episode_out_dir.mkdir(parents=True, exist_ok=True)
        return False

    while True:
        try:
            choice = input(
                "Existing output found. Type 'resume', 'overwrite', or 'abort': "
            ).strip()
        except EOFError as exc:
            raise SystemExit(
                "Existing output found; rerun with --overwrite or use an "
                "interactive terminal to resume."
            ) from exc

        choice = choice.lower()
        if choice in {"resume", "r"}:
            episode_out_dir.mkdir(parents=True, exist_ok=True)
            return True
        if choice in {"overwrite", "o"}:
            shutil.rmtree(episode_out_dir)
            episode_out_dir.mkdir(parents=True, exist_ok=True)
            return False
        if choice in {"abort", "a", "quit", "q"}:
            raise SystemExit("Aborted")

        print("Please type 'resume', 'overwrite', or 'abort'.")


def _speech_jobs(turn: Turn, default_gap_ms: int) -> list[_SpeechJob]:
    speech_indices = [
        index
        for index, chunk in enumerate(turn.markup_chunks)
        if chunk.kind == "speech" and chunk.text and chunk.text.strip()
    ]
    jobs: list[_SpeechJob] = []

    for speech_position, markup_index in enumerate(speech_indices):
        chunk = turn.markup_chunks[markup_index]
        next_speech_index = (
            speech_indices[speech_position + 1]
            if speech_position + 1 < len(speech_indices)
            else len(turn.markup_chunks)
        )
        gap_after_ms = sum(
            between.duration_ms or 0
            for between in turn.markup_chunks[markup_index + 1 : next_speech_index]
            if between.kind == "silence"
        )
        if speech_position + 1 == len(speech_indices):
            gap_after_ms += default_gap_ms

        jobs.append(
            _SpeechJob(
                chunk_index=speech_position,
                markup_index=markup_index,
                text=chunk.text.strip(),
                gap_after_ms=gap_after_ms,
            )
        )

    return jobs


def _non_speech_turn_gap_ms(turn: Turn, default_gap_ms: int) -> int:
    has_speech = any(
        chunk.kind == "speech" and chunk.text and chunk.text.strip()
        for chunk in turn.markup_chunks
    )
    if has_speech:
        return 0

    silence_ms = sum(
        chunk.duration_ms or 0
        for chunk in turn.markup_chunks
        if chunk.kind == "silence"
    )
    return silence_ms + default_gap_ms


def _chunk_wav_path(
    output_path: Path,
    speaker_id: str,
    turn_id: str,
    chunk_index: int,
) -> Path:
    return (
        output_path
        / "Samples"
        / "Processed"
        / speaker_id
        / f"{turn_id}_chunk_{chunk_index:04d}.wav"
    )


def _segment_from_wav(
    wav_path: Path,
    turn_index: int,
    turn_id: str,
    chunk_index: int,
    speaker_id: str,
    gap_after_ms: int,
    speech_threshold: float = 0.04,
) -> SegmentResult:
    info = soundfile.info(wav_path)
    duration_ms = int(info.frames / info.samplerate * 1000)
    audio, sr = soundfile.read(wav_path, dtype="float32")
    if audio.ndim > 1:
        audio = audio.mean(axis=1)
    speech_duration_ms = _measure_speech_duration(audio, sr, speech_threshold)
    checksum = hashlib.sha256(wav_path.read_bytes()).hexdigest()
    return SegmentResult(
        turn_index=turn_index,
        turn_id=turn_id,
        chunk_index=chunk_index,
        speaker_id=speaker_id,
        wav_path=wav_path,
        duration_ms=duration_ms,
        speech_duration_ms=speech_duration_ms,
        sample_rate=int(info.samplerate),
        gap_after_ms=gap_after_ms,
        checksum=checksum,
    )


def _load_completed_turn_segments(
    turn: Turn,
    jobs: list[_SpeechJob],
    output_path: Path,
    speech_threshold: float = 0.04,
) -> list[SegmentResult] | None:
    if not jobs:
        return []

    expected_paths = [
        (
            job.chunk_index,
            job.gap_after_ms,
            _chunk_wav_path(
                output_path,
                turn.speaker_id,
                turn.turn_id,
                job.chunk_index,
            ),
        )
        for job in jobs
    ]
    if not all(
        wav_path.exists()
        for _chunk_index, _gap_after_ms, wav_path in expected_paths
    ):
        return None

    return [
        _segment_from_wav(
            wav_path,
            turn.turn_index,
            turn.turn_id,
            chunk_index,
            turn.speaker_id,
            gap_after_ms,
            speech_threshold,
        )
        for chunk_index, gap_after_ms, wav_path in expected_paths
    ]


def _delete_turn_chunks_by_id(turn_id: str, speaker_id: str, output_path: Path) -> None:
    speaker_dir = output_path / "Samples" / "Processed" / speaker_id
    if not speaker_dir.exists():
        return

    for wav_path in speaker_dir.glob(f"{turn_id}_chunk_*.wav"):
        wav_path.unlink()


def _delete_partial_turn_chunks(turn: Turn, output_path: Path) -> None:
    _delete_turn_chunks_by_id(turn.turn_id, turn.speaker_id, output_path)


def _print_turn_progress(
    turn: Turn,
    total_turns: int,
    turns_done: int,
    started_at: float,
) -> None:
    elapsed = time.monotonic() - started_at
    turns_remaining = total_turns - turns_done
    eta = (elapsed / turns_done) * turns_remaining if turns_done else 0.0
    print(
        f"[{turn.turn_index + 1}/{total_turns}] {turn.display_name} "
        f"\u2014 {elapsed:.1f}s elapsed, ~{eta:.0f}s remaining",
        flush=True,
    )


async def render_loop(
    turns: list[Turn],
    engines: dict[str, TTSEngine],
    voices: dict[str, VoiceConfig],
    out_dir: Path,
    episode_id: str,
    gap_ms: int,
    resume: bool = False,
    sample_budget_ms: int | None = None,
    speech_threshold: float = 0.04,
    trim_edges: bool = True,
    regenerate_turn_indices: set[int] | None = None,
    regenerate_turn_ids: set[str] | None = None,
) -> list[SegmentResult]:
    del episode_id
    if regenerate_turn_indices is None:
        regenerate_turn_indices = set()
    if regenerate_turn_ids is None:
        regenerate_turn_ids = set()
    for engine in engines.values():
        await engine.load()

    segment_results: list[SegmentResult] = []
    started_at = time.monotonic()
    total_turns = len(turns)
    emitted_ms = 0

    for turns_done, turn in enumerate(turns, start=1):
        if sample_budget_ms is not None and emitted_ms >= sample_budget_ms:
            break

        jobs = _speech_jobs(turn, gap_ms)
        non_speech_gap_ms = _non_speech_turn_gap_ms(turn, gap_ms)
        stop_rendering = False

        voice_config = voices[turn.speaker_id]
        engine = engines[voice_config.engine]

        force_regenerate = (
            turn.turn_index in regenerate_turn_indices
            or turn.turn_id in regenerate_turn_ids
        )

        if resume and not force_regenerate:
            existing_segments = _load_completed_turn_segments(
                turn, jobs, out_dir, speech_threshold
            )
            if existing_segments is not None:
                for segment in existing_segments:
                    if (
                        sample_budget_ms is not None
                        and emitted_ms >= sample_budget_ms
                    ):
                        stop_rendering = True
                        break
                    segment_results.append(segment)
                    emitted_ms += segment.duration_ms + segment.gap_after_ms
                if not jobs:
                    emitted_ms += non_speech_gap_ms
                _print_turn_progress(turn, total_turns, turns_done, started_at)
                if stop_rendering:
                    break
                continue
            _delete_partial_turn_chunks(turn, out_dir)

        for job in jobs:
            if sample_budget_ms is not None and emitted_ms >= sample_budget_ms:
                stop_rendering = True
                break

            raw_audio = await engine.synthesize_chunk(
                job.text,
                voice_config,
            )
            raw_audio_array = numpy.asarray(raw_audio)
            if raw_audio_array.size == 0:
                raise RuntimeError(
                    "Synthesis produced empty audio after trimming for "
                    f"turn {turn.turn_index} ({turn.turn_id}), chunk {job.chunk_index}; "
                    "aborting to avoid zero-duration segment emission."
                )
            segment_result = await process_segment(
                raw_audio_array,
                engine.sample_rate,
                _TARGET_SAMPLE_RATE,
                out_dir,
                turn.turn_index,
                turn.turn_id,
                job.chunk_index,
                turn.speaker_id,
                job.gap_after_ms,
                speech_threshold,
                trim_edges,
            )
            segment_results.append(segment_result)
            emitted_ms += segment_result.duration_ms + segment_result.gap_after_ms

            if sample_budget_ms is not None and emitted_ms >= sample_budget_ms:
                stop_rendering = True
                break

        if not jobs:
            emitted_ms += non_speech_gap_ms

        _print_turn_progress(turn, total_turns, turns_done, started_at)
        if stop_rendering:
            break

    return segment_results


def _engine_for_key(
    engine_key: str, model_id: str, trim_edges: bool = True
) -> TTSEngine:
    engine_class = ENGINE_REGISTRY.get(engine_key)
    if engine_class is None:
        available = ", ".join(sorted(ENGINE_REGISTRY))
        raise SystemExit(
            f"Unknown engine {engine_key!r}. Available engine keys: {available}"
        )

    # If the model_id looks like a HuggingFace repo (contains "/") and the engine
    # is ElevenLabs, substitute the default ElevenLabs model.
    if engine_key == "elevenlabs" and "/" in model_id:
        model_id = _DEFAULT_ELEVENLABS_MODEL

    try:
        return engine_class(model_id, trim_edges=trim_edges)
    except TypeError as exc:
        raise SystemExit(
            f"Engine {engine_key!r} cannot be instantiated with model {model_id!r}"
        ) from exc


def _format_duration(total_ms: int) -> str:
    total_seconds = total_ms / 1000
    minutes = int(total_seconds // 60)
    seconds = total_seconds - minutes * 60
    if minutes:
        return f"{minutes}m {seconds:.1f}s"
    return f"{seconds:.1f}s"


def _print_completion_summary(
    turns: list[Turn],
    segment_results: list[SegmentResult],
    manifest_path: Path,
    als_path: Path | None,
    fcpxml_path: Path | None,
    logic_script: str | None,
    voices: dict[str, VoiceConfig],
) -> None:
    segment_counts = Counter(segment.speaker_id for segment in segment_results)
    segment_count_text = ", ".join(
        f"{speaker_id}: {count}" for speaker_id, count in sorted(segment_counts.items())
    )
    total_duration_ms = sum(
        segment.duration_ms + segment.gap_after_ms for segment in segment_results
    )

    print("Render complete.")
    print(f"Turns rendered: {len(turns)}")
    print(f"Segments per speaker: {segment_count_text or 'none'}")
    print(f"Total estimated duration: {_format_duration(total_duration_ms)}")

    # Rough estimate: ElevenLabs charges per character
    eleven_chars = sum(
        len(job.text)
        for turn in turns
        for job in _speech_jobs(turn, 0)
        if voices.get(turn.speaker_id, VoiceConfig(speaker_id=turn.speaker_id)).engine == "elevenlabs"
    )
    if eleven_chars > 0:
        print(f"ElevenLabs estimated characters: {eleven_chars}")

    print(f"Manifest: {manifest_path}")
    if als_path:
        print(f"ALS: {als_path}")
    if fcpxml_path:
        print(f"FCPXML: {fcpxml_path}")
    if logic_script:
        print(f"Logic AppleScript: {logic_script}")


def _generate_comparison_als_files(
    segment_results: list[SegmentResult],
    episode_out_dir: Path,
    episode_id: str,
    thresholds: list[float],
) -> None:
    print("\nGenerating comparison ALS files...")
    for threshold in thresholds:
        remeasured: list[SegmentResult] = []
        for segment in segment_results:
            audio, sr = soundfile.read(segment.wav_path, dtype="float32")
            if audio.ndim > 1:
                audio = audio.mean(axis=1)
            speech_duration_ms = _measure_speech_duration(audio, sr, threshold)
            remeasured.append(
                SegmentResult(
                    turn_index=segment.turn_index,
                    turn_id=segment.turn_id,
                    chunk_index=segment.chunk_index,
                    speaker_id=segment.speaker_id,
                    wav_path=segment.wav_path,
                    duration_ms=segment.duration_ms,
                    speech_duration_ms=speech_duration_ms,
                    sample_rate=segment.sample_rate,
                    gap_after_ms=segment.gap_after_ms,
                    checksum=segment.checksum,
                )
            )
        threshold_str = f"{threshold:g}"
        als_name = f"{episode_id}_t{threshold_str}.als"
        compare_als_path = generate_als(remeasured, episode_out_dir / als_name)
        print(f"  Threshold {threshold}: {compare_als_path.name}")


def _turns_with_segments(
    turns: list[Turn],
    segment_results: list[SegmentResult],
) -> list[Turn]:
    if not segment_results:
        return []

    result_ids = {segment.turn_id for segment in segment_results}
    return [turn for turn in turns if turn.turn_id in result_ids]


def _preview_text(text: str, max_length: int = 120) -> str:
    preview = " ".join(text.split())
    if len(preview) <= max_length:
        return preview
    return preview[: max_length - 3].rstrip() + "..."


def _turn_preview(turn: Turn) -> str:
    return (
        f"[{turn.timestamp_mmss}] {turn.display_name}: "
        f"{_preview_text(turn.clean_text)}"
    )


def _print_dry_run_summary(turns: list[Turn], manifest_path: Path) -> None:
    speakers: dict[str, str] = {}
    for turn in turns:
        speakers.setdefault(turn.speaker_id, turn.display_name)

    speaker_text = ", ".join(
        f"{display_name} ({speaker_id})"
        for speaker_id, display_name in speakers.items()
    )

    print("Dry run complete.")
    print(f"Total turns: {len(turns)}")
    print(f"Speakers found: {speaker_text or 'none'}")
    if turns:
        print(f"First turn: {_turn_preview(turns[0])}")
        print(f"Last turn: {_turn_preview(turns[-1])}")
    print(f"Manifest: {manifest_path}")


def _print_change_report(turns: list[Turn], changed_ids: list[str]) -> None:
    if not changed_ids:
        print("No changes detected. All turns match the previous render.")
        return
    print(f"Detected {len(changed_ids)} changed turn(s):")
    current_by_id = {t.turn_id: t for t in turns}
    for tid in changed_ids:
        turn = current_by_id.get(tid)
        if turn:
            print(f"  Turn {turn.turn_index}: {_turn_preview(turn)}")
        else:
            print(f"  Turn {tid}: [deleted from transcript]")


def _run_migration(
    episode_out_dir: Path,
    turns: list[Turn],
    previous_state: RenderState | None,
) -> None:
    """Rename old turn-index WAV files to new turn-id format and save v2.0 state."""
    from voice_pipeline.render_state import _is_v1_state

    state_path = episode_out_dir / "render_state.json"
    if not state_path.exists():
        raise SystemExit("No render_state.json found. Nothing to migrate.")

    raw_data = json.loads(state_path.read_text(encoding="utf-8"))
    if not _is_v1_state(raw_data):
        print("Render state is already v2.0 or newer. No migration needed.")
        return

    print("Migrating v1.0 → v2.0...")

    # Map current turn_index -> turn_id
    index_to_turn = {t.turn_index: t for t in turns}

    renamed = 0
    skipped = 0
    for turn in turns:
        speaker_dir = episode_out_dir / "Samples" / "Processed" / turn.speaker_id
        if not speaker_dir.exists():
            continue

        # Find old-format files for this turn_index
        old_pattern = f"turn_{turn.turn_index:04d}_chunk_*.wav"
        for old_path in sorted(speaker_dir.glob(old_pattern)):
            # Extract chunk index from filename
            stem = old_path.stem
            if "_chunk_" not in stem:
                continue
            chunk_part = stem.split("_chunk_")[1]
            try:
                chunk_index = int(chunk_part)
            except ValueError:
                continue

            new_name = f"{turn.turn_id}_chunk_{chunk_index:04d}.wav"
            new_path = speaker_dir / new_name

            if new_path.exists():
                # Collision: new-format file already exists
                skipped += 1
                continue

            old_path.rename(new_path)
            renamed += 1

    # Save migrated render state
    if previous_state is not None:
        save_render_state(episode_out_dir, previous_state)
        print(f"Migrated render state saved (schema 2.0).")

    print(f"Migration complete: {renamed} file(s) renamed, {skipped} skipped.")


def main() -> None:
    require_apple_silicon()
    logging.basicConfig(
        level=logging.INFO,
        format="%(levelname)s:%(name)s:%(message)s",
    )

    parser = _build_arg_parser()
    args = parser.parse_args()

    transcript_path = args.transcript.expanduser().resolve()
    if not transcript_path.exists():
        raise SystemExit(f"Transcript not found: {transcript_path}")

    episode_id = args.episode_id or transcript_path.stem
    output_root = args.out_dir.expanduser().resolve()
    episode_out_dir = output_root / episode_id
    sample_budget = _sample_budget_ms(args.sample_seconds)

    turns = tokenize_markup(parse_transcript(transcript_path))
    turns = _filter_turns(turns, args.max_turns)

    voices_path = Path(__file__).with_name("voices.yaml")
    voices = load_voices(voices_path)
    _validate_voices(turns, voices)

    # Load previous render state for change detection and position preservation
    previous_state = load_render_state_with_migration(episode_out_dir, turns)

    if args.detect_changes:
        if previous_state is None:
            print("No previous render state found. Cannot detect changes.")
            return
        changed = detect_changed_turns(turns, previous_state)
        _print_change_report(turns, changed)
        return

    if args.migrate:
        _run_migration(episode_out_dir, turns, previous_state)
        return

    if args.dry_run:
        manifest_path = write_manifest(
            episode_id=episode_id,
            source_file=str(transcript_path),
            model_id=args.model,
            engine=args.engine,
            turns=turns,
            segments=[],
            voices=voices,
            output_path=episode_out_dir,
        )
        _print_dry_run_summary(turns, manifest_path)
        return

    compare_thresholds: list[float] | None = None
    if args.compare_thresholds is not None:
        if args.compare_thresholds == "default":
            compare_thresholds = [0.02, 0.03, 0.04, 0.05, 0.06, 0.08]
        else:
            compare_thresholds = [
                float(x.strip()) for x in args.compare_thresholds.split(",")
            ]

    # Determine which turns to regenerate
    regenerate_turn_indices: set[int] = set()
    regenerate_turn_ids: set[str] = set()
    if args.regenerate_turns is not None:
        regenerate_turn_indices = set(parse_turn_spec(args.regenerate_turns))
        # Map turn indices to turn IDs for internal use
        index_to_turn = {t.turn_index: t for t in turns}
        for idx in regenerate_turn_indices:
            turn = index_to_turn.get(idx)
            if turn is not None:
                regenerate_turn_ids.add(turn.turn_id)
        # Delete existing WAVs for turns being regenerated
        for turn in turns:
            if turn.turn_id in regenerate_turn_ids:
                _delete_partial_turn_chunks(turn, episode_out_dir)

    # Precision insert: only synthesize new/modified turns
    use_precision_insert = False
    if args.precision_insert:
        if previous_state is None:
            raise SystemExit(
                "--precision-insert requires a previous render state. "
                "Run a full render first, or use --migrate if upgrading from v1.0."
            )
        plan = plan_precision_insert(turns, previous_state)
        use_precision_insert = True
        # Delete files for deleted turns
        old_id_to_speaker = {fp.turn_id: fp.speaker_id for fp in previous_state.turns}
        for deleted_id in plan.deleted_ids:
            speaker_id = old_id_to_speaker.get(deleted_id)
            if speaker_id is not None:
                _delete_turn_chunks_by_id(deleted_id, speaker_id, episode_out_dir)
        # Mark inserted and modified turns for regeneration
        for turn in plan.inserted + plan.modified:
            regenerate_turn_ids.add(turn.turn_id)
            _delete_partial_turn_chunks(turn, episode_out_dir)
        if plan.inserted or plan.modified or plan.deleted_ids:
            print(
                f"Precision insert: {len(plan.inserted)} new, {len(plan.modified)} modified, "
                f"{len(plan.deleted_ids)} deleted, {len(plan.unchanged)} unchanged."
            )
        else:
            print("Precision insert: no changes detected. Nothing to do.")
            return

    resume = _prepare_episode_output(episode_out_dir, args.overwrite)
    # If regenerating specific turns or doing precision insert, force resume mode
    if regenerate_turn_indices or use_precision_insert:
        resume = True

    # Build position map from previous render state to preserve unchanged segment positions
    # For precision insert, we recompute positions from scratch (no position map)
    position_map: dict[tuple[str, int], int] | None = None
    if previous_state is not None and not use_precision_insert:
        position_map = build_position_map(previous_state)

    # Build engine cache from unique engines required by voices.yaml
    engines: dict[str, TTSEngine] = {}
    for voice in voices.values():
        if voice.engine not in engines:
            engines[voice.engine] = _engine_for_key(
                voice.engine, args.model, trim_edges=not args.no_trim
            )

    segment_results = asyncio.run(
        render_loop(
            turns=turns,
            engines=engines,
            voices=voices,
            out_dir=episode_out_dir,
            episode_id=episode_id,
            gap_ms=args.gap_ms,
            resume=resume,
            sample_budget_ms=sample_budget,
            speech_threshold=args.speech_threshold,
            trim_edges=not args.no_trim,
            regenerate_turn_indices=regenerate_turn_indices,
            regenerate_turn_ids=regenerate_turn_ids,
        )
    )

    manifest_turns = (
        _turns_with_segments(turns, segment_results)
        if sample_budget is not None
        else turns
    )

    als_path = episode_out_dir / f"{episode_id}.als"
    if not args.skip_als:
        generate_als(segment_results, als_path, position_map=position_map)
    else:
        print("Skipping ALS generation (--skip-als).")

    fcpxml_path = episode_out_dir / f"{episode_id}.fcpxml"
    if args.fcpxml:
        generate_fcpxml(segment_results, fcpxml_path, position_map=position_map, episode_id=episode_id)
        print(f"FCPXML generated: {fcpxml_path}")

    logic_script: str | None = None
    if args.logic or args.logic_dry_run:
        try:
            script, output = export_to_logic(
                segment_results,
                position_map=position_map,
                project_name=episode_id,
                dry_run=args.logic_dry_run,
            )
            if args.logic_dry_run:
                script_path = episode_out_dir / f"{episode_id}_logic_export.applescript"
                script_path.write_text(script, encoding="utf-8")
                logic_script = str(script_path)
                print(f"Logic AppleScript written to: {logic_script}")
            else:
                logic_script = "executed via osascript"
                if output:
                    print(f"Logic export output: {output}")
        except RuntimeError as e:
            print(f"Logic export failed: {e}")

    manifest_path = write_manifest(
        episode_id=episode_id,
        source_file=str(transcript_path),
        model_id=args.model,
        engine=args.engine,
        turns=manifest_turns,
        segments=segment_results,
        voices=voices,
        output_path=episode_out_dir,
        default_gap_ms=args.gap_ms,
        position_map=position_map,
    )
    _print_completion_summary(
        manifest_turns,
        segment_results,
        manifest_path,
        als_path if not args.skip_als else None,
        fcpxml_path if args.fcpxml else None,
        logic_script,
        voices,
    )

    # Save render state for future change detection and selective regeneration
    segment_positions: list[SegmentPosition] = []
    manifest_data = json.loads(manifest_path.read_text(encoding="utf-8"))
    for turn in manifest_data["turns"]:
        for seg in turn["segments"]:
            segment_positions.append(
                SegmentPosition(
                    turn_id=turn["turn_id"],
                    chunk_index=seg["chunk_index"],
                    start_ms=seg["start_ms"],
                    duration_ms=seg["duration_ms"],
                    speech_duration_ms=seg["speech_duration_ms"],
                    gap_after_ms=seg["gap_after_ms"],
                )
            )
    render_state = RenderState(
        schema_version="2.0",
        source_file=str(transcript_path),
        source_hash=compute_source_hash(transcript_path),
        rendered_at=datetime.utcnow().isoformat() + "Z",
        turns=[compute_turn_fingerprint(t) for t in turns],
        segments=segment_positions,
        als_path=str(als_path) if not args.skip_als else None,
        fcpxml_path=str(fcpxml_path) if args.fcpxml else None,
        ableton_project_als_path=str(args.ableton_project_als) if args.ableton_project_als else None,
    )
    save_render_state(episode_out_dir, render_state)

    # Copy ALS to Ableton project if configured
    ableton_als_path = args.ableton_project_als
    if ableton_als_path and not args.skip_als:
        ableton_als_path = ableton_als_path.expanduser().resolve()
        ableton_als_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(als_path, ableton_als_path)
        print(f"Copied ALS to Ableton project: {ableton_als_path}")

    if compare_thresholds and not args.skip_als:
        _generate_comparison_als_files(
            segment_results, episode_out_dir, episode_id, compare_thresholds
        )


if __name__ == "__main__":
    main()
