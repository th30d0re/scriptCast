import asyncio
import json
import sys
from pathlib import Path

import pytest

import voice_pipeline.__main__ as cli
from voice_pipeline.markup import tokenize_markup
from voice_pipeline.models import SegmentResult, Turn, VoiceConfig


class _FakeEngine:
    @property
    def sample_rate(self) -> int:
        return 24000

    async def load(self) -> None:
        return None

    async def synthesize_chunk(
        self,
        text: str,
        voice_config: VoiceConfig,
    ) -> list[float]:
        del text, voice_config
        return [0.1, 0.1, 0.1]


class _EmptyAudioEngine(_FakeEngine):
    async def synthesize_chunk(
        self,
        text: str,
        voice_config: VoiceConfig,
    ) -> list[float]:
        del text, voice_config
        return []


async def _fake_process_segment(
    audio: object,
    source_rate: int,
    target_rate: int,
    output_path: Path,
    turn_index: int,
    chunk_index: int,
    speaker_id: str,
    gap_after_ms: int,
) -> SegmentResult:
    wav_path = (
        output_path
        / "Samples"
        / "Processed"
        / speaker_id
        / f"turn_{turn_index:04d}_chunk_{chunk_index:04d}.wav"
    )
    wav_path.parent.mkdir(parents=True, exist_ok=True)
    wav_path.write_bytes(b"fake wav")
    return SegmentResult(
        turn_index=turn_index,
        chunk_index=chunk_index,
        speaker_id=speaker_id,
        wav_path=wav_path,
        duration_ms=1000,
        sample_rate=target_rate,
        gap_after_ms=gap_after_ms,
        checksum=f"{turn_index}-{chunk_index}",
    )


def test_sample_seconds_cli_uses_rendered_timeline_and_reaches_ep0_speakers(
    tmp_path,
    monkeypatch,
    capsys,
) -> None:
    episode_id = "ep0_sample"
    transcript_path = Path("Architecting_the_operation/podcasts/ATO_EP0.md")
    monkeypatch.setattr(cli, "require_apple_silicon", lambda: None)
    monkeypatch.setattr(cli, "_engine_for_key", lambda _key, _model: _FakeEngine())
    monkeypatch.setattr(cli, "process_segment", _fake_process_segment)
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "voice_pipeline",
            "--transcript",
            str(transcript_path),
            "--episode-id",
            episode_id,
            "--out-dir",
            str(tmp_path),
            "--sample-seconds",
            "60",
            "--gap-ms",
            "250",
            "--overwrite",
        ],
    )

    cli.main()
    capsys.readouterr()

    manifest_path = tmp_path / episode_id / "episode_manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    speakers_with_segments = {
        turn["speaker_id"] for turn in manifest["turns"] if turn["segments"]
    }
    emitted_ms = sum(
        segment["duration_ms"] + segment["gap_after_ms"]
        for turn in manifest["turns"]
        for segment in turn["segments"]
    )

    assert {
        "emmanuel_theodore",
        "ai_1",
        "ai_2",
    }.issubset(speakers_with_segments)
    assert emitted_ms == 60_000


def test_render_loop_uses_contiguous_speech_chunk_indexes_for_markup(
    tmp_path,
    monkeypatch,
) -> None:
    turn = Turn(
        turn_index=0,
        speaker_id="ai_1",
        display_name="AI 1",
        timestamp_mmss="00:00",
        timestamp_ms=0,
        raw_text="[tone:soft] hello [pause:500ms] world",
        clean_text="[tone:soft] hello [pause:500ms] world",
        line_span=(1, 2),
    )
    tokenize_markup([turn])
    voices = {
        "ai_1": VoiceConfig(
            speaker_id="ai_1",
            kokoro_voice="af_heart",
            lang_code="a",
            speed=1.0,
        )
    }
    monkeypatch.setattr(cli, "process_segment", _fake_process_segment)

    segments = asyncio.run(
        cli.render_loop(
            turns=[turn],
            engine=_FakeEngine(),
            voices=voices,
            out_dir=tmp_path,
            episode_id="episode",
            gap_ms=250,
        )
    )

    assert [segment.chunk_index for segment in segments] == [0, 1]
    assert [segment.gap_after_ms for segment in segments] == [500, 250]
    assert [segment.wav_path.name for segment in segments] == [
        "turn_0000_chunk_0000.wav",
        "turn_0000_chunk_0001.wav",
    ]


def test_dry_run_cli_prints_turn_summary(tmp_path, monkeypatch, capsys) -> None:
    transcript_path = tmp_path / "episode.md"
    transcript_path.write_text(
        "\n".join(
            [
                "Emmanuel Theodore (00:00)",
                "Hello there.",
                "",
                "AI 1 (00:03)",
                "A short reply.",
            ]
        ),
        encoding="utf-8",
    )
    monkeypatch.setattr(cli, "require_apple_silicon", lambda: None)
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "voice_pipeline",
            "--transcript",
            str(transcript_path),
            "--episode-id",
            "dry_run",
            "--out-dir",
            str(tmp_path / "outputs"),
            "--dry-run",
        ],
    )

    cli.main()

    output = capsys.readouterr().out
    assert "Dry run complete." in output
    assert "Total turns: 2" in output
    assert "Speakers found: Emmanuel Theodore (emmanuel_theodore), AI 1 (ai_1)" in output
    assert "First turn: [00:00] Emmanuel Theodore: Hello there." in output
    assert "Last turn: [00:03] AI 1: A short reply." in output
    assert "Manifest:" in output


def test_render_loop_rejects_empty_synthesized_audio(
    tmp_path,
) -> None:
    turn = Turn(
        turn_index=0,
        speaker_id="ai_1",
        display_name="AI 1",
        timestamp_mmss="00:00",
        timestamp_ms=0,
        raw_text="hello",
        clean_text="hello",
        line_span=(1, 2),
    )
    tokenize_markup([turn])
    voices = {
        "ai_1": VoiceConfig(
            speaker_id="ai_1",
            kokoro_voice="af_heart",
            lang_code="a",
            speed=1.0,
        )
    }

    with pytest.raises(RuntimeError, match="zero-duration segment emission"):
        asyncio.run(
            cli.render_loop(
                turns=[turn],
                engine=_EmptyAudioEngine(),
                voices=voices,
                out_dir=tmp_path,
                episode_id="episode",
                gap_ms=250,
            )
        )


def test_standalone_pause_turn_is_preserved_in_manifest_timeline(
    tmp_path,
    monkeypatch,
    capsys,
) -> None:
    transcript_path = tmp_path / "episode.md"
    transcript_path.write_text(
        "\n".join(
            [
                "AI 1 (00:00)",
                "hello",
                "",
                "AI 1 (00:02)",
                "[pause:500ms]",
                "",
                "AI 1 (00:03)",
                "world",
            ]
        ),
        encoding="utf-8",
    )
    monkeypatch.setattr(cli, "require_apple_silicon", lambda: None)
    monkeypatch.setattr(cli, "_engine_for_key", lambda _key, _model: _FakeEngine())
    monkeypatch.setattr(cli, "process_segment", _fake_process_segment)
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "voice_pipeline",
            "--transcript",
            str(transcript_path),
            "--episode-id",
            "pause_turn",
            "--out-dir",
            str(tmp_path),
            "--gap-ms",
            "250",
            "--overwrite",
        ],
    )

    cli.main()
    capsys.readouterr()

    manifest_path = tmp_path / "pause_turn" / "episode_manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    first_turn, pause_turn, third_turn = manifest["turns"]

    assert first_turn["start_ms"] == 0
    assert first_turn["end_ms"] == 1000
    assert first_turn["segments"][0]["gap_after_ms"] == 250

    assert pause_turn["segments"] == []
    assert pause_turn["start_ms"] == 1250
    assert pause_turn["end_ms"] == 2000

    assert third_turn["start_ms"] == 2000
    assert third_turn["end_ms"] == 3000
