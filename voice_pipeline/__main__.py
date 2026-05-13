"""Entry point for the local voice pipeline."""

from __future__ import annotations

import argparse
import asyncio
import hashlib
import logging
import shutil
import time
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

import numpy
import soundfile

from voice_pipeline.als_generator import generate_als
from voice_pipeline.engine import ENGINE_REGISTRY, TTSEngine
from voice_pipeline.manifest import write_manifest
from voice_pipeline.markup import tokenize_markup
from voice_pipeline.models import SegmentResult, Turn, VoiceConfig
from voice_pipeline.parser import parse_transcript
from voice_pipeline.platform_check import require_apple_silicon
from voice_pipeline.post_processor import process_segment
from voice_pipeline.voices import load_voices

_DEFAULT_MODEL = "prince-canuma/Kokoro-82M"
_DEFAULT_ENGINE = "mlx_kokoro"
_TARGET_SAMPLE_RATE = 48000


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
        help="HuggingFace model repo ID",
    )
    parser.add_argument(
        "--engine",
        type=str,
        default=_DEFAULT_ENGINE,
        help="Engine registry key",
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
    turn_index: int,
    chunk_index: int,
) -> Path:
    return (
        output_path
        / "Samples"
        / "Processed"
        / speaker_id
        / f"turn_{turn_index:04d}_chunk_{chunk_index:04d}.wav"
    )


def _segment_from_wav(
    wav_path: Path,
    turn_index: int,
    chunk_index: int,
    speaker_id: str,
    gap_after_ms: int,
) -> SegmentResult:
    info = soundfile.info(wav_path)
    duration_ms = int(info.frames / info.samplerate * 1000)
    checksum = hashlib.sha256(wav_path.read_bytes()).hexdigest()
    return SegmentResult(
        turn_index=turn_index,
        chunk_index=chunk_index,
        speaker_id=speaker_id,
        wav_path=wav_path,
        duration_ms=duration_ms,
        sample_rate=int(info.samplerate),
        gap_after_ms=gap_after_ms,
        checksum=checksum,
    )


def _load_completed_turn_segments(
    turn: Turn,
    jobs: list[_SpeechJob],
    output_path: Path,
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
                turn.turn_index,
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
            chunk_index,
            turn.speaker_id,
            gap_after_ms,
        )
        for chunk_index, gap_after_ms, wav_path in expected_paths
    ]


def _delete_partial_turn_chunks(turn: Turn, output_path: Path) -> None:
    speaker_dir = output_path / "Samples" / "Processed" / turn.speaker_id
    if not speaker_dir.exists():
        return

    for wav_path in speaker_dir.glob(f"turn_{turn.turn_index:04d}_chunk_*.wav"):
        wav_path.unlink()


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
    engine: TTSEngine,
    voices: dict[str, VoiceConfig],
    out_dir: Path,
    episode_id: str,
    gap_ms: int,
    resume: bool = False,
    sample_budget_ms: int | None = None,
) -> list[SegmentResult]:
    del episode_id
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

        if resume:
            existing_segments = _load_completed_turn_segments(turn, jobs, out_dir)
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
                voices[turn.speaker_id],
            )
            raw_audio_array = numpy.asarray(raw_audio)
            if raw_audio_array.size == 0:
                raise RuntimeError(
                    "Synthesis produced empty audio after trimming for "
                    f"turn {turn.turn_index}, chunk {job.chunk_index}; "
                    "aborting to avoid zero-duration segment emission."
                )
            segment_result = await process_segment(
                raw_audio_array,
                engine.sample_rate,
                _TARGET_SAMPLE_RATE,
                out_dir,
                turn.turn_index,
                job.chunk_index,
                turn.speaker_id,
                job.gap_after_ms,
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


def _engine_for_key(engine_key: str, model_id: str) -> TTSEngine:
    engine_class = ENGINE_REGISTRY.get(engine_key)
    if engine_class is None:
        available = ", ".join(sorted(ENGINE_REGISTRY))
        raise SystemExit(
            f"Unknown engine {engine_key!r}. Available engine keys: {available}"
        )

    try:
        return engine_class(model_id)
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
    als_path: Path,
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
    print(f"Manifest: {manifest_path}")
    print(f"ALS: {als_path}")


def _turns_with_segments(
    turns: list[Turn],
    segment_results: list[SegmentResult],
) -> list[Turn]:
    if not segment_results:
        return []

    last_turn_index = max(segment.turn_index for segment in segment_results)
    return [turn for turn in turns if turn.turn_index <= last_turn_index]


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

    resume = _prepare_episode_output(episode_out_dir, args.overwrite)
    engine = _engine_for_key(args.engine, args.model)
    segment_results = asyncio.run(
        render_loop(
            turns=turns,
            engine=engine,
            voices=voices,
            out_dir=episode_out_dir,
            episode_id=episode_id,
            gap_ms=args.gap_ms,
            resume=resume,
            sample_budget_ms=sample_budget,
        )
    )

    manifest_turns = (
        _turns_with_segments(turns, segment_results)
        if sample_budget is not None
        else turns
    )
    als_path = generate_als(segment_results, episode_out_dir / f"{episode_id}.als")
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
    )
    _print_completion_summary(
        manifest_turns,
        segment_results,
        manifest_path,
        als_path,
    )


if __name__ == "__main__":
    main()
