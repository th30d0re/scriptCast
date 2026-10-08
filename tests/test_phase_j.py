from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

import scriptcast.__main__ as cli
from scriptcast.models import SegmentResult, VoiceConfig
from scriptcast.progress import Progress
from scriptcast.tools import repair_render


class _FakeEngine:
    sample_rate = 24_000

    async def load(self) -> None:
        return None

    async def synthesize_chunk(self, text, voice_config):
        del text, voice_config
        return [0.1, 0.1, 0.1]


async def _fake_process_segment(
    audio,
    source_rate,
    target_rate,
    output_path,
    turn_index,
    turn_id,
    chunk_index,
    speaker_id,
    gap_after_ms,
    speech_threshold=0.04,
    tail_ms=0,
    trim_edges=True,
):
    del audio, source_rate, speech_threshold, tail_ms, trim_edges
    wav = (
        output_path
        / "Samples"
        / "Processed"
        / speaker_id
        / f"{turn_id}_chunk_{chunk_index:04d}.wav"
    )
    wav.parent.mkdir(parents=True, exist_ok=True)
    wav.write_bytes(b"fake wav")
    return SegmentResult(
        turn_index=turn_index,
        turn_id=turn_id,
        chunk_index=chunk_index,
        speaker_id=speaker_id,
        wav_path=wav,
        duration_ms=1000,
        speech_duration_ms=850,
        sample_rate=target_rate,
        gap_after_ms=gap_after_ms,
        checksum=f"{turn_index}-{chunk_index}",
    )


def _voices():
    return {
        "host": VoiceConfig(
            speaker_id="host", kokoro_voice="am_adam", engine="omnivoice"
        ),
        "guest": VoiceConfig(
            speaker_id="guest", kokoro_voice="af_sky", engine="omnivoice"
        ),
    }


def _write_transcript(path: Path) -> None:
    path.write_text(
        "Host (00:00)\nWelcome to this small test episode today.\n\n"
        "Guest (00:03)\nThanks for having me.\n",
        encoding="utf-8",
    )


def _run_cli(monkeypatch, argv: list[str]) -> None:
    monkeypatch.setattr(cli, "require_apple_silicon", lambda: None)
    monkeypatch.setattr(sys, "argv", ["scriptcast", *argv])
    cli.main()


def test_dry_run_json_reports_preamble_and_unknown_speaker(
    tmp_path, monkeypatch
) -> None:
    transcript = tmp_path / "episode.md"
    transcript.write_text(
        "Production notes\n\nHost (00:00)\nWelcome to the show.\n\n"
        "Mystery Person (00:05)\nOne two three four five six seven eight nine ten eleven twelve thirteen.\n",
        encoding="utf-8",
    )
    voices = tmp_path / "voices.yaml"
    voices.write_text("unused", encoding="utf-8")
    report = tmp_path / "report.json"
    monkeypatch.setattr(cli, "load_voices", lambda path: {"host": _voices()["host"]})

    _run_cli(
        monkeypatch,
        [
            "--transcript", str(transcript), "--voices", str(voices),
            "--episode-id", "ep-json", "--dry-run", "--json-out", str(report),
        ],
    )

    data = json.loads(report.read_text(encoding="utf-8"))
    assert data["schema"] == 1
    assert data["episode_id"] == "ep-json"
    assert data["transcript"] == str(transcript.resolve())
    assert data["voices"] == str(voices.resolve())
    assert data["turns"][1]["first_words"] == (
        "One two three four five six seven eight nine ten eleven twelve"
    )
    assert data["speakers"][1]["engine"] is None
    assert data["warnings"] == [
        {
            "kind": "preamble_skipped",
            "line": 1,
            "speaker_id": None,
            "blocking": False,
            "message": "Non-speaker preamble line was skipped.",
        },
        {
            "kind": "unknown_speaker",
            "line": 6,
            "speaker_id": "mystery_person",
            "blocking": True,
            "message": "No voice config found for speaker: mystery_person",
        },
    ]


def test_dry_run_text_output_is_unchanged_without_json_out(
    tmp_path, monkeypatch, capsys
) -> None:
    transcript = tmp_path / "episode.md"
    _write_transcript(transcript)
    out_dir = tmp_path / "outputs"
    monkeypatch.setattr(cli, "load_voices", lambda path: _voices())

    _run_cli(
        monkeypatch,
        [
            "--transcript", str(transcript), "--episode-id", "legacy",
            "--out-dir", str(out_dir), "--dry-run",
        ],
    )

    assert capsys.readouterr().out == (
        "Dry run complete.\n"
        "Total turns: 2\n"
        "Speakers found: Host (host), Guest (guest)\n"
        "First turn: [00:00] Host: Welcome to this small test episode today.\n"
        "Last turn: [00:03] Guest: Thanks for having me.\n"
        f"Manifest: {out_dir.resolve() / 'legacy' / 'episode_manifest.json'}\n"
    )


def test_render_progress_has_turns_stages_and_done(tmp_path, monkeypatch) -> None:
    transcript = tmp_path / "episode.md"
    _write_transcript(transcript)
    progress_path = tmp_path / "progress.jsonl"
    monkeypatch.setattr(cli, "load_voices", lambda path: _voices())
    monkeypatch.setattr(cli, "_engine_for_key", lambda *args, **kwargs: _FakeEngine())
    monkeypatch.setattr(cli, "process_segment", _fake_process_segment)

    _run_cli(
        monkeypatch,
        [
            "--transcript", str(transcript), "--episode-id", "render",
            "--out-dir", str(tmp_path / "outputs"), "--overwrite",
            "--fcpxml", "--progress-jsonl", str(progress_path),
        ],
    )

    events = [json.loads(line) for line in progress_path.read_text().splitlines()]
    assert events[0]["event"] == "start"
    assert events[0]["tool"] == "render"
    assert [e["event"] for e in events].count("turn_start") == 2
    assert [e["event"] for e in events].count("turn_done") == 2
    assert [e["name"] for e in events if e["event"] == "stage"] == [
        "layout", "als", "fcpxml"
    ]
    assert events[-1]["event"] == "done"
    assert events[-1]["ok"] is True


def test_progress_writes_one_error_on_exception(tmp_path, monkeypatch) -> None:
    transcript = tmp_path / "episode.md"
    _write_transcript(transcript)
    progress_path = tmp_path / "progress.jsonl"
    monkeypatch.setattr(cli, "parse_transcript", lambda path: (_ for _ in ()).throw(RuntimeError("boom")))

    with pytest.raises(RuntimeError, match="boom"):
        _run_cli(
            monkeypatch,
            [
                "--transcript", str(transcript), "--dry-run",
                "--progress-jsonl", str(progress_path),
            ],
        )

    events = [json.loads(line) for line in progress_path.read_text().splitlines()]
    assert events == [{"event": "error", "message": "boom"}]


def test_progress_sigterm_writes_cancelled_and_exits_130(tmp_path) -> None:
    progress_path = tmp_path / "progress.jsonl"
    with pytest.raises(SystemExit) as exc:
        with Progress(progress_path) as progress:
            progress.start("render", 1)
            progress._cancel(15, None)
    assert exc.value.code == 130
    events = [json.loads(line) for line in progress_path.read_text().splitlines()]
    assert events[-1] == {"event": "error", "message": "cancelled"}
    assert [event["event"] for event in events].count("error") == 1


def test_resume_and_overwrite_are_distinct_and_mutually_exclusive(
    tmp_path, monkeypatch
) -> None:
    episode = tmp_path / "episode"
    episode.mkdir()
    marker = episode / "episode_manifest.json"
    marker.write_text("keep", encoding="utf-8")
    monkeypatch.setattr("builtins.input", lambda prompt: pytest.fail("prompted"))
    assert cli._prepare_episode_output(episode, False, resume_requested=True) is True
    assert marker.read_text() == "keep"
    assert cli._prepare_episode_output(episode, True) is False
    assert not marker.exists()
    with pytest.raises(SystemExit) as exc:
        cli._build_arg_parser().parse_args(
            ["--transcript", "x.md", "--resume", "--overwrite"]
        )
    assert exc.value.code == 2


def test_export_only_writes_exports_without_writing_wavs(tmp_path, monkeypatch) -> None:
    transcript = tmp_path / "episode.md"
    _write_transcript(transcript)
    out_dir = tmp_path / "outputs"
    episode = out_dir / "exported"
    monkeypatch.setattr(cli, "load_voices", lambda path: _voices())
    monkeypatch.setattr(cli, "_engine_for_key", lambda *args, **kwargs: _FakeEngine())
    monkeypatch.setattr(cli, "process_segment", _fake_process_segment)
    _run_cli(
        monkeypatch,
        [
            "--transcript", str(transcript), "--episode-id", "exported",
            "--out-dir", str(out_dir), "--overwrite", "--skip-als",
        ],
    )
    wavs_before = {p: p.read_bytes() for p in episode.rglob("*.wav")}

    _run_cli(
        monkeypatch,
        [
            "--export-only", "--episode-id", "exported", "--out-dir", str(out_dir),
            "--fcpxml",
        ],
    )

    assert (episode / "exported.als").exists()
    assert (episode / "exported.fcpxml").exists()
    assert {p: p.read_bytes() for p in episode.rglob("*.wav")} == wavs_before


def test_export_only_missing_manifest_exits_two(tmp_path, monkeypatch, capsys) -> None:
    with pytest.raises(SystemExit) as exc:
        _run_cli(
            monkeypatch,
            [
                "--export-only", "--episode-id", "missing",
                "--out-dir", str(tmp_path),
            ],
        )
    assert exc.value.code == 2
    assert "Existing manifest not found" in capsys.readouterr().err


def test_repair_progress_emits_pass_and_result(tmp_path, monkeypatch) -> None:
    failing = {
        "turn": 4,
        "turn_id": "turn-id",
        "speaker": "host",
        "score": 0.5,
        "worst_run": 3,
        "expected": "hello",
        "heard": "yellow",
        "failed": True,
    }
    answers = iter([[failing], []])
    monkeypatch.setattr(repair_render, "_verify", lambda *args, **kwargs: next(answers))
    monkeypatch.setattr(repair_render, "_regenerate", lambda *args, **kwargs: None)
    progress_path = tmp_path / "repair.jsonl"
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "scriptcast-repair", str(tmp_path), "--transcript", str(tmp_path / "e.md"),
            "--episode-id", "e", "--voices", str(tmp_path / "voices.yaml"),
            "--skip-relayout", "--progress-jsonl", str(progress_path),
        ],
    )

    assert repair_render.main() == 0
    events = [json.loads(line) for line in progress_path.read_text().splitlines()]
    assert [e["event"] for e in events].count("pass") == 1
    assert [e for e in events if e["event"] == "pass"][0]["failing"] == 1
    assert [e for e in events if e["event"] == "result"][-1] == {
        "event": "result", "passed": 1, "failed": 0
    }
