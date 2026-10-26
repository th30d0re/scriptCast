"""Opt-in JSON modes for the voice and clip tools."""

from __future__ import annotations

import json
import sys
from io import StringIO
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

import numpy
import soundfile
import yaml

from scriptcast.pronunciation import HeteronymReading
from scriptcast.tools.audition_voice import PASSAGES

CLIPS = """\
# Clip registry.
# Second clip comment.
clips:
  speech:
    source: archive/sources/speech.m4a
    start: 1.0
    end: 2.0
    citation: "A book"
    # note under speech
"""

PRONUNCIATIONS = """\
# Spellings handed to the engine.
# Second pronunciation comment.

record:
  default: old   # the noun
"""


def _run(main, argv: list[str]) -> tuple[int, str, str]:
    stdout = StringIO()
    stderr = StringIO()
    with redirect_stdout(stdout), redirect_stderr(stderr):
        old = sys.argv
        sys.argv = argv
        try:
            code = main()
        finally:
            sys.argv = old
    return code, stdout.getvalue(), stderr.getvalue()


def _whisper(monkeypatch, transcribe) -> None:
    monkeypatch.setitem(sys.modules, "mlx_whisper", type("M", (), {
        "transcribe": staticmethod(transcribe),
    })())


def _wav(path: Path, seconds: float, amplitude: float = 0.25, rate: int = 24000) -> None:
    audio = numpy.full(int(seconds * rate), amplitude, dtype="float32")
    soundfile.write(str(path), audio, rate)


def _project(tmp_path: Path, monkeypatch) -> None:
    (tmp_path / "scriptcast.toml").write_text("# test\n", encoding="utf-8")
    monkeypatch.setenv("SCRIPTCAST_PROJECT", str(tmp_path))


def _clip_words(path, **kwargs):
    return {"segments": [{"words": [
        {"word": " Hello", "start": 1.0, "end": 1.5},
        {"word": " world", "start": 1.5, "end": 2.0},
        {"word": " .", "start": 2.0, "end": 2.1},
        {"word": " later", "start": 4.0, "end": 4.4},
    ]}]}


def test_clip_without_json_matches_legacy_text(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.make_clip import main

    _project(tmp_path, monkeypatch)
    source = tmp_path / "source.wav"
    _wav(source, 5)
    _whisper(monkeypatch, _clip_words)
    code, out, _err = _run(main, [
        "scriptcast-clip", str(source), "--id", "excerpt",
        "--from", "Hello", "--to", "world",
    ])
    assert code == 0
    registry = tmp_path / "archive" / "clips.yaml"
    start = max(0.0, 1.0 - 0.12)
    end = 2.0 + 0.12
    expected = (
        f"registered excerpt: {start:.2f}s to {end:.2f}s ({end - start:.1f}s) in {registry}\n"
        "\nScript turn (check the transcript against the recording):\n"
        "\n"
        "Archive (00:00)\n"
        "[clip:excerpt] Hello world\n"
    )
    assert out == expected
    data = {"clips": {"excerpt": {
        "source": "archive/sources/excerpt.wav",
        "start": round(float(start), 2),
        "end": round(float(end), 2),
        "citation": "",
        "origin_url": "",
        "content_note": "",
    }}}
    assert registry.read_text(encoding="utf-8") == yaml.safe_dump(data, sort_keys=True, allow_unicode=True)


def test_clip_words_json_shape(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.make_clip import main

    _project(tmp_path, monkeypatch)
    source = tmp_path / "source.wav"
    _wav(source, 3)
    _whisper(monkeypatch, _clip_words)
    code, out, _err = _run(main, ["scriptcast-clip", str(source), "--words-json", "--model", "small"])
    assert code == 0
    payload = json.loads(out)
    assert list(payload) == ["source", "duration", "words"]
    assert payload["source"] == str(source.resolve())
    assert payload["duration"] == 3.0
    assert payload["words"] == [
        {"i": 0, "word": "Hello", "start": 1.0, "end": 1.5},
        {"i": 1, "word": "world", "start": 1.5, "end": 2.0},
        {"i": 2, "word": "later", "start": 4.0, "end": 4.4},
    ]
    assert not (tmp_path / "archive" / "clips.yaml").exists()
    assert out.count("\n") == 1


def test_clip_json_span_is_exact_and_keeps_comments(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.make_clip import main

    _project(tmp_path, monkeypatch)
    (tmp_path / "archive").mkdir()
    registry = tmp_path / "archive" / "clips.yaml"
    registry.write_text(CLIPS, encoding="utf-8")
    source = tmp_path / "source.wav"
    _wav(source, 6)
    _whisper(monkeypatch, _clip_words)
    code, out, _err = _run(main, [
        "scriptcast-clip", str(source), "--id", "exact",
        "--start", "1.5", "--end", "3.25", "--pad", "0.5",
        "--speaker", "Narrator", "--json",
    ])
    assert code == 0
    payload = json.loads(out)
    assert payload["id"] == "exact"
    assert payload["registry"] == str(registry.resolve())
    assert payload["entry"]["start"] == 1.5
    assert payload["entry"]["end"] == 3.25
    assert payload["turn"] == "Narrator (00:00)\n[clip:exact] world"
    text = registry.read_text(encoding="utf-8")
    assert "# Clip registry." in text
    assert "# note under speech" in text
    loaded = yaml.safe_load(text)
    assert loaded["clips"]["exact"]["start"] == 1.5
    assert loaded["clips"]["exact"]["end"] == 3.25
    assert loaded["clips"]["speech"]["citation"] == "A book"


def test_clip_json_collision_and_replace(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.make_clip import main

    _project(tmp_path, monkeypatch)
    (tmp_path / "archive").mkdir()
    registry = tmp_path / "archive" / "clips.yaml"
    registry.write_text(CLIPS, encoding="utf-8")
    source = tmp_path / "source.wav"
    _wav(source, 6)
    _whisper(monkeypatch, _clip_words)
    before = registry.read_bytes()
    code, out, _err = _run(main, [
        "scriptcast-clip", str(source), "--id", "speech",
        "--start", "1.5", "--end", "3.25", "--json",
    ])
    assert code == 1
    assert json.loads(out)["error"]
    assert registry.read_bytes() == before
    code, out, _err = _run(main, [
        "scriptcast-clip", str(source), "--id", "speech",
        "--start", "1.5", "--end", "3.25", "--json", "--replace",
    ])
    assert code == 0
    text = registry.read_text(encoding="utf-8")
    assert "# note under speech" in text
    assert yaml.safe_load(text)["clips"]["speech"]["end"] == 3.25


def _reference_segments(path, **kwargs):
    return {"segments": [
        {"start": 0.0, "end": 4.0, "text": " Alpha sentence."},
        {"start": 4.0, "end": 9.5, "text": " Beta sentence."},
        {"start": 9.5, "end": 14.0, "text": " Gamma sentence."},
    ], "text": "Alpha sentence. Beta sentence. Gamma sentence."}


def _install_reference_audio(monkeypatch, seconds: float = 15, amplitude: float = 0.25) -> None:
    def _to_wav(source: Path, dest: Path) -> None:
        _wav(dest, seconds, amplitude)

    monkeypatch.setattr("scriptcast.tools.make_reference._to_wav", _to_wav)


def test_reference_without_json_matches_legacy_text(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.make_reference import main

    _project(tmp_path, monkeypatch)
    _install_reference_audio(monkeypatch)
    _whisper(monkeypatch, _reference_segments)
    source = tmp_path / "raw.wav"
    _wav(source, 1, amplitude=0.0)
    code, out, _err = _run(main, ["scriptcast-reference", str(source), "--name", "host"])
    assert code == 0
    wav = tmp_path / "voices" / "candidates" / "host.wav"
    info = soundfile.info(str(wav))
    assert info.frames == int(9.5 * 24000)
    expected = (
        f"wrote {wav}  9.50s  from 0.0s to 9.5s of the recording\n"
        "peak was 0.250, normalised to 0.92\n"
        "transcript: Alpha sentence. Beta sentence.\n"
        "\nRead that against the clip before rendering with it. "
        "The text has to match the audio word for word.\n"
    )
    assert out == expected
    texts = json.loads((tmp_path / "voices" / "candidates" / "reference_texts.json").read_text())
    assert texts == {"host": "Alpha sentence. Beta sentence."}


def test_reference_candidates_json_shape(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.make_reference import main

    _project(tmp_path, monkeypatch)
    _install_reference_audio(monkeypatch)
    _whisper(monkeypatch, _reference_segments)
    source = tmp_path / "raw.wav"
    source.write_bytes(b"raw")
    code, out, _err = _run(main, [
        "scriptcast-reference", str(source), "--candidates-json", "--seconds", "9",
    ])
    assert code == 0
    payload = json.loads(out)
    assert payload["source"] == str(source.resolve())
    assert payload["candidates"][0]["rank"] == 1
    assert payload["candidates"][0]["start"] == 0.0
    assert payload["candidates"][0]["end"] == 9.5
    assert payload["candidates"][0]["text"] == "Alpha sentence. Beta sentence."
    assert payload["candidates"][0]["score"] == round(1.0 / 1.5, 4)
    assert [item["rank"] for item in payload["candidates"]] == list(range(1, len(payload["candidates"]) + 1))
    assert not (tmp_path / "voices" / "candidates" / "host.wav").exists()


def test_reference_json_span_is_exact(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.make_reference import main

    _project(tmp_path, monkeypatch)
    _install_reference_audio(monkeypatch, seconds=5, amplitude=0.5)

    def transcribe(path, **kwargs):
        assert kwargs.get("word_timestamps") is True
        return {"segments": [{"start": 0.0, "end": 5.0, "text": " Hello there later.", "words": [
            {"word": " Hello", "start": 0.1, "end": 0.4},
            {"word": " there", "start": 1.0, "end": 1.4},
            {"word": " later", "start": 3.0, "end": 3.4},
        ]}]}

    _whisper(monkeypatch, transcribe)
    source = tmp_path / "raw.wav"
    source.write_bytes(b"raw")
    code, out, _err = _run(main, [
        "scriptcast-reference", str(source), "--name", "host",
        "--start", "0.5", "--end", "2.0", "--json",
    ])
    assert code == 0
    payload = json.loads(out)
    wav = tmp_path / "voices" / "candidates" / "host.wav"
    assert payload["wav"] == str(wav.resolve())
    assert payload["text"] == "there"
    assert payload["duration"] == 1.5
    assert payload["peak"] == 0.5
    assert soundfile.info(str(wav)).frames == int(2.0 * 24000) - int(0.5 * 24000)


def test_reference_candidates_clamped_and_outside_span_rejected(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.make_reference import main

    _project(tmp_path, monkeypatch)
    _install_reference_audio(monkeypatch, seconds=10, amplitude=0.4)

    def transcribe(path, **kwargs):
        return {"segments": [
            {"start": -0.4, "end": 4.0, "text": " Alpha sentence."},
            {"start": 4.0, "end": 10.5, "text": " Beta sentence."},
        ]}

    _whisper(monkeypatch, transcribe)
    source = tmp_path / "raw.wav"
    source.write_bytes(b"raw")
    code, out, _err = _run(main, [
        "scriptcast-reference", str(source), "--candidates-json", "--seconds", "9",
    ])
    assert code == 0
    payload = json.loads(out)
    by_text = {item["text"]: item for item in payload["candidates"]}
    opened = by_text["Alpha sentence."]
    assert opened["start"] == 0.0
    assert opened["end"] == 4.0
    whole = by_text["Alpha sentence. Beta sentence."]
    assert whole["start"] == 0.0
    assert whole["end"] == 10.0
    assert whole["score"] == round(1.0 / (1.0 + abs(10.0 - 9.0)), 4)
    tail = by_text["Beta sentence."]
    assert tail["start"] == 4.0
    assert tail["end"] == 10.0
    assert tail["score"] == round(1.0 / (1.0 + abs(6.0 - 9.0)), 4)
    for item in payload["candidates"]:
        assert 0.0 <= item["start"] <= item["end"] <= 10.0
    assert payload["candidates"][0]["text"] == "Alpha sentence. Beta sentence."
    assert not (tmp_path / "voices" / "candidates" / "host.wav").exists()

    code, out, _err = _run(main, [
        "scriptcast-reference", str(source), "--name", "host",
        "--start", "0", "--end", "10.5", "--json",
    ])
    assert code == 1
    assert json.loads(out) == {"error": "span extends outside the recording"}
    assert not (tmp_path / "voices" / "candidates" / "host.wav").exists()

    code, out, _err = _run(main, [
        "scriptcast-reference", str(source), "--name", "host",
        "--start", "-0.2", "--end", "3", "--json",
    ])
    assert code == 1
    assert json.loads(out)["error"] == "span extends outside the recording"
    assert not (tmp_path / "voices" / "candidates").exists()


def test_audition_without_json_matches_legacy_text(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.engine import ENGINE_REGISTRY
    from scriptcast.tools.audition_voice import main

    reference = tmp_path / "ref.wav"
    _wav(reference, 0.2, amplitude=0.1)

    class FakeEngine:
        sample_rate = 24000

        def __init__(self, model_id: str) -> None:
            self.model_id = model_id

        async def synthesize_chunk(self, text: str, config) -> numpy.ndarray:
            return numpy.zeros(2400, dtype="float32")

    monkeypatch.setitem(ENGINE_REGISTRY, "mlx_chatterbox", FakeEngine)

    def transcribe(path, **kwargs):
        index = int(Path(path).stem.rsplit("_", 1)[-1])
        return {"text": PASSAGES[index], "segments": []}

    _whisper(monkeypatch, transcribe)
    code, out, _err = _run(main, ["scriptcast-audition", str(reference), "--speaker", "guest"])
    assert code == 0
    expected = (
        f"\n{'reference':<34} {'temp':>5} {'mean':>7} {'min':>7} {'clean':>7}\n"
        f"{'ref.wav':<34} {0.6:>5} {1.0:>7.4f} {1.0:>7.4f} {len(PASSAGES):>3}/{len(PASSAGES)}\n"
    )
    assert out == expected


def test_audition_json_mirrors_score_fields(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.engine import ENGINE_REGISTRY
    from scriptcast.tools.audition_voice import main

    reference = tmp_path / "ref.wav"
    _wav(reference, 0.2, amplitude=0.1)
    keep = tmp_path / "takes"

    class FakeEngine:
        sample_rate = 24000

        def __init__(self, model_id: str) -> None:
            pass

        async def synthesize_chunk(self, text: str, config) -> numpy.ndarray:
            return numpy.zeros(2400, dtype="float32")

    monkeypatch.setitem(ENGINE_REGISTRY, "mlx_chatterbox", FakeEngine)

    def transcribe(path, **kwargs):
        index = int(Path(path).stem.rsplit("_", 1)[-1])
        heard = PASSAGES[index]
        if index == 0:
            heard = "nope"
        return {"text": heard, "segments": []}

    _whisper(monkeypatch, transcribe)
    code, out, err = _run(main, ["scriptcast-audition", str(reference), "--json"])
    assert code == 1
    assert json.loads(out) == {"error": "--json requires --keep"}
    code, out, err = _run(main, [
        "scriptcast-audition", str(reference), "--json", "--keep", str(keep),
        "--speaker", "guest", "--engine", "mlx_chatterbox",
    ])
    assert code == 0
    assert "reference" in err
    payload = json.loads(out)
    assert payload["speaker"] == "guest"
    assert payload["engine"] == "mlx_chatterbox"
    candidate = payload["candidates"][0]
    assert candidate["reference"] == str(reference.resolve())
    assert candidate["mean"] == candidate["min"] or candidate["min"] < candidate["mean"]
    assert "wer" not in candidate
    assert {"temperature", "mean", "min", "clean", "of", "takes"} <= set(candidate)
    take = candidate["takes"][0]
    assert take["wav"].startswith(str(keep.resolve()))
    assert take["text"] == PASSAGES[0]
    assert take["heard"] == "nope"
    assert "score" in take and "run" in take and "wer" not in take
    assert (keep / "scores.json").is_file()


def _calibrate_fakes(monkeypatch, tmp_path: Path) -> None:
    from scriptcast.engine import ENGINE_REGISTRY

    _project(tmp_path, monkeypatch)
    voices = tmp_path / "voices.yaml"
    voices.write_text(
        "speakers:\n  host:\n    name: Host\n    engine: omnivoice\n"
        "    reference_audio: ref.wav\n    reference_text: hello\n",
        encoding="utf-8",
    )
    (tmp_path / "pronunciations.yaml").write_text(PRONUNCIATIONS, encoding="utf-8")

    class FakeEngine:
        sample_rate = 24000

        def __init__(self, model_id: str) -> None:
            pass

        async def synthesize_chunk(self, text: str, config) -> numpy.ndarray:
            return numpy.zeros(2400, dtype="float32")

    monkeypatch.setitem(ENGINE_REGISTRY, "omnivoice", FakeEngine)
    monkeypatch.setattr(
        "scriptcast.tools.calibrate_pronunciation._heteronym_table",
        lambda: {"record": {"default": "abc", "verb": "def"}},
    )
    monkeypatch.setattr(
        "scriptcast.tools.calibrate_pronunciation._script_sentences",
        lambda word, reading, limit, scripts_dir: ["It includes one specific historical record."],
    )

    def find_heteronyms(sentence: str):
        index = sentence.lower().find("record")
        if index < 0:
            return []
        return [HeteronymReading(
            word=sentence[index:index + 6], start=index, end=index + 6,
            tag="NN", key="default", phonemes="abc", alternatives=("def",), contrast="stress",
        )]

    monkeypatch.setattr("scriptcast.tools.calibrate_pronunciation.find_heteronyms", find_heteronyms)

    def check_segment(path, sentence, words):
        verdict = "ok" if "reckerd" in Path(path).name else "wrong"
        return [{"word": "record", "reading": "default", "verdict": verdict}]

    monkeypatch.setattr("scriptcast.tools.calibrate_pronunciation.check_segment", check_segment)
    _whisper(monkeypatch, lambda path, **kwargs: {"segments": [{"words": []}], "text": ""})


def test_calibrate_without_json_matches_legacy_text(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.calibrate_pronunciation import main

    _calibrate_fakes(monkeypatch, tmp_path)
    code, out, _err = _run(main, [
        "scriptcast-calibrate", "record", "--reading", "default",
        "--candidates", "reckerd", "--takes", "1", "--sentences", "1",
        "--speaker", "host",
    ])
    assert code == 0
    out_dir = tmp_path / "outputs" / "pronunciation_calibration" / "record_default"
    wrong = {"ok": 0, "unclear": 0, "wrong": 1, "garbled": 0, "unaligned": 0}
    right = {"ok": 1, "unclear": 0, "wrong": 0, "garbled": 0, "unaligned": 0}
    expected = (
        f"  {'record':<16} ok 0/1  {wrong}\n"
        f"  {'reckerd':<16} ok 1/1  {right}\n"
        f"\nbest: 'reckerd' at 100% ok. Audio in {out_dir}\n"
    )
    assert out == expected
    assert "reckerd" not in (tmp_path / "pronunciations.yaml").read_text(encoding="utf-8")


def test_calibrate_json_write_keeps_comments(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.calibrate_pronunciation import main

    _calibrate_fakes(monkeypatch, tmp_path)
    pronunciations = tmp_path / "pronunciations.yaml"
    before = pronunciations.read_text(encoding="utf-8")
    keep = tmp_path / "takes"
    code, out, _err = _run(main, [
        "scriptcast-calibrate", "record", "--reading", "default",
        "--candidates", "reckerd", "--takes", "1", "--sentences", "1",
        "--speaker", "host", "--json", "--keep", str(keep),
    ])
    assert code == 0
    payload = json.loads(out)
    assert payload["word"] == "record"
    assert payload["reading"] == "default"
    assert payload["best"] == "reckerd"
    assert payload["written"] is False
    assert payload["candidates"][0]["spelling"] == "reckerd"
    assert payload["candidates"][0]["score"] == 1.0
    assert payload["candidates"][0]["takes"][0]["ok"] is True
    assert payload["candidates"][0]["takes"][0]["wav"].endswith(".wav")
    assert pronunciations.read_text(encoding="utf-8") == before

    code, out, _err = _run(main, [
        "scriptcast-calibrate", "record", "--reading", "default",
        "--candidates", "reckerd", "--takes", "1", "--sentences", "1",
        "--speaker", "host", "--json", "--keep", str(keep), "--write",
    ])
    assert code == 0
    payload = json.loads(out)
    assert payload["written"] is True
    text = pronunciations.read_text(encoding="utf-8")
    assert "# Spellings handed to the engine." in text
    assert "# the noun" in text
    assert "reckerd" in text


def test_stress_without_json_is_one_object_per_line(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.stress_check import main

    wav = tmp_path / "take.wav"
    wav.write_bytes(b"RIFF")
    entry = {
        "word": "record", "reading": "default", "expected": "abc",
        "contrast": "stress", "verdict": "ok", "heard": "r eh",
    }
    monkeypatch.setattr("scriptcast.tools.stress_check.check_segment", lambda *args, **kwargs: [entry])
    _whisper(monkeypatch, lambda path, **kwargs: {"segments": [{"words": []}], "text": ""})
    code, out, _err = _run(main, ["scriptcast-stress", str(wav), "--text", "historical record"])
    assert code == 0
    assert out == json.dumps(entry, ensure_ascii=False) + "\n"


def test_stress_json_shape(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.stress_check import main

    wav = tmp_path / "take.wav"
    wav.write_bytes(b"RIFF")
    entry = {
        "word": "record", "reading": "default", "expected": "abc",
        "contrast": "stress", "verdict": "wrong", "heard": "r eh",
    }
    monkeypatch.setattr("scriptcast.tools.stress_check.check_segment", lambda *args, **kwargs: [entry])
    _whisper(monkeypatch, lambda path, **kwargs: {"segments": [{"words": []}], "text": ""})
    code, out, _err = _run(main, ["scriptcast-stress", str(wav), "--text", "historical record", "--json"])
    assert code == 0
    assert json.loads(out) == {
        "heteronyms": [{"word": "record", "expected": "abc", "heard": "r eh", "ok": False}],
    }


def _outlier_fixture(tmp_path: Path, monkeypatch, end_ms: int) -> tuple[Path, str, str]:
    from scriptcast.parser import parse_transcript

    _project(tmp_path, monkeypatch)
    text = " ".join(["alpha"] * 20)
    script = tmp_path / "episode.md"
    script.write_text(f"Host (00:00)\n{text}\n", encoding="utf-8")
    turn = parse_transcript(script)[0]
    episode = tmp_path / "episode"
    episode.mkdir(exist_ok=True)
    (episode / "episode_manifest.json").write_text(json.dumps({"turns": [{
        "turn_id": turn.turn_id,
        "turn_index": 12,
        "speaker_id": "host",
        "start_ms": 0,
        "end_ms": end_ms,
    }]}), encoding="utf-8")
    (tmp_path / "speaker_rates.json").write_text(json.dumps({
        "host": {"ms_per_word": 400.0, "ms_per_mark": 0.0},
    }), encoding="utf-8")
    return episode, str(script), text


def test_outliers_without_json_matches_legacy_text(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.check_render_outliers import main

    episode, script, text = _outlier_fixture(tmp_path, monkeypatch, 20000)
    code, out, _err = _run(main, [
        "scriptcast-outliers", str(episode), "--transcript", script,
    ])
    assert code == 1
    snippet = text[:60]
    expected = (
        "1 turns >= 20 words | median 2.50 | p95 2.50 | max 2.50\n"
        f"  LONG  turn  12 {'host':<18} {20.0:6.1f}s for {20:>3} words  ratio 2.50  "
        f"{json.loads((episode / 'episode_manifest.json').read_text())['turns'][0]['turn_id']}  {snippet}\n"
        "\n1 turn(s) to review. Re-synthesize with:\n"
        "  --regenerate-turns 12\n"
    )
    assert out == expected

    episode, script, _text = _outlier_fixture(tmp_path, monkeypatch, 8000)
    code, out, _err = _run(main, [
        "scriptcast-outliers", str(episode), "--transcript", script,
    ])
    assert code == 0
    assert out == (
        "1 turns >= 20 words | median 1.00 | p95 1.00 | max 1.00\n"
        "check-render-outliers: nothing flagged.\n"
    )


def test_outliers_json_shape(tmp_path: Path, monkeypatch) -> None:
    from scriptcast.tools.check_render_outliers import main

    episode, script, text = _outlier_fixture(tmp_path, monkeypatch, 20000)
    turn_id = json.loads((episode / "episode_manifest.json").read_text())["turns"][0]["turn_id"]
    code, out, err = _run(main, [
        "scriptcast-outliers", str(episode), "--transcript", script, "--json", "--ratio", "1.5",
    ])
    assert code == 0
    assert "LONG" in err
    assert "turn(s) to review" not in out
    payload = json.loads(out)
    assert payload == {"outliers": [{
        "turn_index": 12,
        "turn_id": turn_id,
        "speaker_id": "host",
        "predicted_s": 8.0,
        "actual_s": 20.0,
        "ratio": 2.5,
        "reason": f"ratio 2.50 for 20 words: {text[:60]}",
    }]}
