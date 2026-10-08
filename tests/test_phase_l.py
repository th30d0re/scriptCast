"""scriptcast-project show, validate, and comment-preserving edit."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from scriptcast.archive import load_registry
from scriptcast.config_edit import insert_at, locate_block
from scriptcast.pronunciation import load_respellings
from scriptcast.project import Project
from scriptcast.review import load_accepted_hearings
from scriptcast.tools.project_config import main
from scriptcast.voices import load_voices

VOICES = """\
# Voices for the fixture.
# Second header comment.
speakers:
  aisha:
    name: "Aisha"
    engine: "archive"
  # comment before host
  host:
    name: "Host"
    engine: "kokoro"
    kokoro_voice: "af_heart"
    lang_code: "a"
    # host stays verbatim
"""

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
  other:
    source: archive/sources/other.m4a
    start: 3
    end: 4.5
    origin_url: "https://example.test/other"
"""

PRONUNCIATIONS = """\
# Spellings handed to the engine.
# Second pronunciation comment.
record:
  default: reckerd   # the noun
  verb: ree-CORD

"4473":
  always: forty-four seventy-three

lead:
  always: led
"""

HEARINGS = """\
# Accepted hearings: treat these Whisper hearings as matching the script word.
# Comments under each word cite the review timestamp that added the hearing.
# Third comment line.
# Fourth comment line.

bonded:
  # review 2026-10-05T01:33:56Z
  - about

in:
  # review 2026-10-05T01:33:56Z
  - it

ab:
  # review 2026-10-05T02:00:00Z
  - xyzzy

pati:
  # review 2026-10-04T12:00:00Z
  - potty
  # review 2026-10-04T13:00:00Z
  - poti
"""

TOML = """\
# scriptCast project configuration.
# Every key is optional.

voices = "voices.yaml"  # voices file
# clips stay put
clips = "archive/clips.yaml"
tail_ms = 800   # keep the decay
"""

RATES = """\
{
  "aisha": 2.9,
  "host": {
    "ms_per_word": 320.0,
    "ms_per_mark": 10
  }
}
"""


def _write(root: Path) -> None:
    (root / "voices.yaml").write_text(VOICES, encoding="utf-8")
    (root / "archive").mkdir()
    (root / "archive" / "clips.yaml").write_text(CLIPS, encoding="utf-8")
    (root / "archive" / "sources").mkdir()
    (root / "archive" / "sources" / "speech.m4a").write_bytes(b"clip")
    (root / "pronunciations.yaml").write_text(PRONUNCIATIONS, encoding="utf-8")
    (root / "accepted_hearings.yaml").write_text(HEARINGS, encoding="utf-8")
    (root / "speaker_rates.json").write_text(RATES, encoding="utf-8")
    (root / "scriptcast.toml").write_text(TOML, encoding="utf-8")


def _digest(path: Path) -> str:
    data = path.read_bytes() if path.is_file() else b""
    return hashlib.sha256(data).hexdigest()


def _run(args: list[str]) -> tuple[int, str, str]:
    from io import StringIO
    from contextlib import redirect_stderr, redirect_stdout

    stdout = StringIO()
    stderr = StringIO()
    with redirect_stdout(stdout), redirect_stderr(stderr):
        code = main(args)
    return code, stdout.getvalue(), stderr.getvalue()


def _outside(before: str, start: int, end: int) -> bytes:
    lines = before.splitlines(keepends=True)
    return ("".join(lines[:start]) + "".join(lines[end:])).encode()


def _assert_outside(before: str, after: str, start: int, end: int) -> None:
    lines = before.splitlines(keepends=True)
    prefix = "".join(lines[:start]).encode()
    suffix = "".join(lines[end:]).encode()
    blob = after.encode()
    assert blob[: len(prefix)] == prefix
    assert len(blob) >= len(prefix) + len(suffix)
    if suffix:
        assert blob[-len(suffix) :] == suffix


def _edit(root: Path, ops: list[dict], kinds: list[str]) -> tuple[int, str, str]:
    ops_path = root / "ops.json"
    ops_path.write_text(json.dumps(ops), encoding="utf-8")
    args = ["edit", "--ops", str(ops_path), "--root", str(root)]
    paths = {
        "voices": root / "voices.yaml",
        "clips": root / "archive" / "clips.yaml",
        "pronunciations": root / "pronunciations.yaml",
        "accepted_hearings": root / "accepted_hearings.yaml",
        "toml": root / "scriptcast.toml",
    }
    for kind in kinds:
        args.extend(["--expect-sha256", f"{kind}={_digest(paths[kind])}"])
    return _run(args)


def test_show_shape_missing_file_and_risky(tmp_path: Path):
    _write(tmp_path)
    code, out, err = _run(["show", "--json", "--root", str(tmp_path)])
    assert code == 0, err
    assert err == ""
    report = json.loads(out)
    assert report["schema"] == 1
    assert report["root"] == str(tmp_path.resolve())
    assert report["toml"]["values"]["voices"] == "voices.yaml"
    assert report["toml"]["values"]["tail_ms"] == 800
    assert report["toml"]["unknown_keys"] == []
    assert report["toml"]["problems"] == []
    aisha = next(item for item in report["voices"]["speakers"] if item["id"] == "aisha")
    assert aisha["name"] == "Aisha"
    assert aisha["engine"] == "archive"
    assert aisha["fields"]["engine"] == "archive"
    assert aisha["problems"] == []
    speech = next(item for item in report["clips"]["clips"] if item["id"] == "speech")
    assert speech["start"] == 1.0
    assert speech["end"] == 2.0
    assert speech["citation"] == "A book"
    assert speech["origin_url"] is None
    assert speech["content_note"] is None
    assert speech["source_exists"] is True
    other = next(item for item in report["clips"]["clips"] if item["id"] == "other")
    assert other["source_exists"] is False
    words = {item["word"]: item for item in report["pronunciations"]["entries"]}
    assert words["record"]["readings"]["default"] == "reckerd"
    assert words["4473"]["readings"]["always"] == "forty-four seventy-three"
    hearings = {item["word"]: item for item in report["accepted_hearings"]["entries"]}
    assert hearings["bonded"]["heard"] == ["about"]
    assert hearings["bonded"]["review_ts"] == "2026-10-05T01:33:56Z"
    assert hearings["bonded"]["risky"] is False
    assert hearings["in"]["risky"] is True
    assert hearings["ab"]["risky"] is True
    assert hearings["pati"]["review_ts"] is None
    assert report["speaker_rates"]["rates"]["aisha"] == 2.9
    assert report["speaker_rates"]["rates"]["host"]["ms_per_word"] == 320.0

    missing = tmp_path / "missing"
    missing.mkdir()
    (missing / "scriptcast.toml").write_text('voices = "no-such.yaml"\n', encoding="utf-8")
    code, out, err = _run(["show", "--json", "--root", str(missing)])
    assert code == 0, err
    report = json.loads(out)
    assert report["voices"]["missing"] is True
    assert report["voices"]["speakers"] == []
    assert report["voices"]["problems"] == []
    assert report["clips"]["missing"] is True
    assert report["accepted_hearings"]["missing"] is True
    assert str(report["accepted_hearings"]["path"]).endswith("accepted_hearings.yaml")


def test_validate_exits_on_loader_problem(tmp_path: Path):
    _write(tmp_path)
    code, _out, err = _run(["validate", "--json", "--root", str(tmp_path)])
    assert code == 0, err
    voices = tmp_path / "voices.yaml"
    voices.write_text(
        "speakers:\n  aisha:\n    engine: omnivoice\n    name: Aisha\n    reference_audio: a.wav\n",
        encoding="utf-8",
    )
    code, out, _err = _run(["validate", "--json", "--root", str(tmp_path)])
    assert code == 1
    report = json.loads(out)
    message = "Speaker 'aisha' (engine='omnivoice') is missing required field: reference_text"
    assert message in report["voices"]["problems"]
    assert report["voices"]["speakers"][0]["problems"] == [message]

    (tmp_path / "scriptcast.toml").write_text(TOML + "not_a_real_key = 1\n", encoding="utf-8")
    code, out, _err = _run(["validate", "--json", "--root", str(tmp_path)])
    assert code == 1
    report = json.loads(out)
    assert report["toml"]["unknown_keys"] == ["not_a_real_key"]
    assert "unknown key" in report["toml"]["problems"][0]


def test_every_op_preserves_outside_bytes(tmp_path: Path):
    _write(tmp_path)
    voices = (tmp_path / "voices.yaml").read_text(encoding="utf-8")
    start, end = locate_block(voices, ["speakers"], "aisha")
    code, out, err = _edit(
        tmp_path,
        [{
            "file": "voices",
            "op": "set_speaker",
            "id": "aisha",
            "fields": {"engine": "archive", "name": "Aisha", "note": "z-last"},
        }],
        ["voices"],
    )
    assert code == 0, err
    updated = (tmp_path / "voices.yaml").read_text(encoding="utf-8")
    _assert_outside(voices, updated, start, end)
    assert _outside(voices, start, end) == _outside_region_bytes(voices, updated, start, end)
    assert '    name: "Aisha"\n' in updated
    assert '    engine: "archive"\n' in updated
    assert '    note: "z-last"\n' in updated
    assert updated.index('name: "Aisha"') < updated.index('engine: "archive"') < updated.index('note: "z-last"')
    assert "# comment before host" in updated
    assert '# host stays verbatim' in updated
    show = json.loads(out)
    assert show["voices"]["speakers"][0]["fields"]["note"] == "z-last"

    before = updated
    start, _end = locate_block(before, ["speakers"], "aisha")
    code, _out, err = _edit(
        tmp_path,
        [{"file": "voices", "op": "rename_speaker", "id": "aisha", "to": "aisha_b"}],
        ["voices"],
    )
    assert code == 0, err
    renamed = (tmp_path / "voices.yaml").read_text(encoding="utf-8")
    before_lines = before.splitlines(keepends=True)
    renamed_lines = renamed.splitlines(keepends=True)
    assert before_lines[:start] + before_lines[start + 1 :] == renamed_lines[:start] + renamed_lines[start + 1 :]
    assert "aisha_b:" in renamed_lines[start]
    assert 'note: "z-last"' in renamed

    code, _out, err = _edit(
        tmp_path,
        [{"file": "voices", "op": "delete_speaker", "id": "aisha_b"}],
        ["voices"],
    )
    assert code == 0, err
    deleted = (tmp_path / "voices.yaml").read_text(encoding="utf-8")
    block_start, block_end = locate_block(renamed, ["speakers"], "aisha_b")
    _assert_outside(renamed, deleted, block_start, block_end)
    assert "aisha_b" not in deleted
    assert "kokoro_voice: \"af_heart\"" in deleted

    guest_before = deleted
    at = insert_at(guest_before, ["speakers"])
    code, _out, err = _edit(
        tmp_path,
        [{
            "file": "voices",
            "op": "set_speaker",
            "id": "guest",
            "fields": {"engine": "archive", "name": "Guest"},
        }],
        ["voices"],
    )
    assert code == 0, err
    with_guest = (tmp_path / "voices.yaml").read_text(encoding="utf-8")
    _assert_outside(guest_before, with_guest, at, at)
    assert load_voices(tmp_path / "voices.yaml")["guest"].name == "Guest"

    clips = (tmp_path / "archive" / "clips.yaml").read_text(encoding="utf-8")
    start, end = locate_block(clips, ["clips"], "speech")
    code, _out, err = _edit(
        tmp_path,
        [{
            "file": "clips",
            "op": "set_clip",
            "id": "speech",
            "fields": {"source": "archive/sources/speech.m4a", "start": 1.5, "end": 2.5, "citation": "B"},
        }],
        ["clips"],
    )
    assert code == 0, err
    clips_after = (tmp_path / "archive" / "clips.yaml").read_text(encoding="utf-8")
    _assert_outside(clips, clips_after, start, end)
    assert "origin_url: \"https://example.test/other\"" in clips_after
    assert load_registry(tmp_path / "archive" / "clips.yaml")["speech"]["citation"] == "B"

    code, _out, err = _edit(
        tmp_path,
        [{"file": "clips", "op": "delete_clip", "id": "other"}],
        ["clips"],
    )
    assert code == 0, err
    clips_deleted = (tmp_path / "archive" / "clips.yaml").read_text(encoding="utf-8")
    other_start, other_end = locate_block(clips_after, ["clips"], "other")
    _assert_outside(clips_after, clips_deleted, other_start, other_end)
    assert "other:" not in clips_deleted

    pronunciations = (tmp_path / "pronunciations.yaml").read_text(encoding="utf-8")
    start, end = locate_block(pronunciations, [], "lead")
    code, _out, err = _edit(
        tmp_path,
        [{
            "file": "pronunciations",
            "op": "set_entry",
            "word": "lead",
            "readings": {"always": "led", "noun": "ledd"},
        }],
        ["pronunciations"],
    )
    assert code == 0, err
    pronunciations_after = (tmp_path / "pronunciations.yaml").read_text(encoding="utf-8")
    _assert_outside(pronunciations, pronunciations_after, start, end)
    assert 'reckerd   # the noun' in pronunciations_after
    assert '"4473":' in pronunciations_after
    loaded = load_respellings(tmp_path / "pronunciations.yaml")
    assert loaded["lead"]["noun"] == "ledd"
    assert loaded["4473"]["always"] == "forty-four seventy-three"

    code, _out, err = _edit(
        tmp_path,
        [{"file": "pronunciations", "op": "delete_entry", "word": "record"}],
        ["pronunciations"],
    )
    assert code == 0, err
    pronunciations_deleted = (tmp_path / "pronunciations.yaml").read_text(encoding="utf-8")
    record_start, record_end = locate_block(pronunciations_after, [], "record")
    _assert_outside(pronunciations_after, pronunciations_deleted, record_start, record_end)
    assert "record:" not in pronunciations_deleted
    assert load_respellings(tmp_path / "pronunciations.yaml")["lead"]["always"] == "led"

    hearings = (tmp_path / "accepted_hearings.yaml").read_text(encoding="utf-8")
    start, end = locate_block(hearings, [], "pati")
    code, _out, err = _edit(
        tmp_path,
        [{"file": "accepted_hearings", "op": "delete", "word": "Pati", "heard": "Potty!"}],
        ["accepted_hearings"],
    )
    assert code == 0, err
    hearings_after = (tmp_path / "accepted_hearings.yaml").read_text(encoding="utf-8")
    prefix = "".join(hearings.splitlines(keepends=True)[:start]).encode()
    assert hearings_after.encode().startswith(prefix)
    assert "# review 2026-10-04T13:00:00Z" in hearings_after
    assert "poti" in hearings_after
    assert "potty" not in hearings_after
    assert "# Fourth comment line." in hearings_after
    assert load_accepted_hearings(tmp_path)["pati"] == {"poti"}

    code, _out, err = _edit(
        tmp_path,
        [{"file": "accepted_hearings", "op": "delete", "word": "ab", "heard": "xyzzy"}],
        ["accepted_hearings"],
    )
    assert code == 0, err
    hearings_word_gone = (tmp_path / "accepted_hearings.yaml").read_text(encoding="utf-8")
    word_start, word_end = locate_block(hearings_after, [], "ab")
    _assert_outside(hearings_after, hearings_word_gone, word_start, word_end)
    assert "\nab:" not in hearings_word_gone
    assert "bonded:" in hearings_word_gone
    assert "ab" not in load_accepted_hearings(tmp_path)

    toml = (tmp_path / "scriptcast.toml").read_text(encoding="utf-8")
    code, _out, err = _edit(
        tmp_path,
        [{"file": "toml", "op": "set", "key": "tail_ms", "value": 900}],
        ["toml"],
    )
    assert code == 0, err
    toml_after = (tmp_path / "scriptcast.toml").read_text(encoding="utf-8")
    assert 'voices = "voices.yaml"  # voices file' in toml_after
    assert "# clips stay put\n" in toml_after
    assert "tail_ms = 900   # keep the decay" in toml_after
    assert toml.split("tail_ms")[0] == toml_after.split("tail_ms")[0]
    assert Project(tmp_path).tail_ms == 900


def _outside_region_bytes(before: str, after: str, start: int, end: int) -> bytes:
    lines = before.splitlines(keepends=True)
    prefix = "".join(lines[:start]).encode()
    suffix = "".join(lines[end:]).encode()
    blob = after.encode()
    return blob[: len(prefix)] + blob[len(blob) - len(suffix) :]


def test_invalid_engine_is_rejected_and_file_is_unchanged(tmp_path: Path):
    _write(tmp_path)
    before = (tmp_path / "voices.yaml").read_bytes()
    code, out, err = _edit(
        tmp_path,
        [{
            "file": "voices",
            "op": "set_speaker",
            "id": "aisha",
            "fields": {"engine": "omnivoice", "name": "Aisha", "reference_audio": "a.wav"},
        }],
        ["voices"],
    )
    assert code == 1
    assert out == ""
    assert err.strip() == (
        "voices: Speaker 'aisha' (engine='omnivoice') is missing required field: reference_text"
    )
    assert (tmp_path / "voices.yaml").read_bytes() == before


def test_stale_hash_exits_3(tmp_path: Path):
    _write(tmp_path)
    before = {path: path.read_bytes() for path in _tracked(tmp_path)}
    ops = tmp_path / "ops.json"
    ops.write_text(json.dumps([
        {"file": "voices", "op": "delete_speaker", "id": "aisha"},
    ]), encoding="utf-8")
    code, out, err = _run([
        "edit", "--ops", str(ops), "--root", str(tmp_path),
        "--expect-sha256", "voices=" + "0" * 64,
    ])
    assert code == 3
    assert out == ""
    assert "voices: sha256 mismatch" in err
    for path, data in before.items():
        assert path.read_bytes() == data


def test_multi_file_batch_is_atomic(tmp_path: Path):
    _write(tmp_path)
    before = {path: path.read_bytes() for path in _tracked(tmp_path)}
    code, out, err = _edit(
        tmp_path,
        [
            {
                "file": "pronunciations",
                "op": "set_entry",
                "word": "lead",
                "readings": {"always": "ledd"},
            },
            {
                "file": "clips",
                "op": "set_clip",
                "id": "fresh",
                "fields": {"source": "archive/sources/speech.m4a", "start": 0, "end": 1},
            },
            {
                "file": "voices",
                "op": "set_speaker",
                "id": "aisha",
                "fields": {"engine": "omnivoice", "name": "Aisha", "reference_audio": "a.wav"},
            },
        ],
        ["pronunciations", "clips", "voices"],
    )
    assert code == 1
    assert out == ""
    assert "reference_text" in err
    assert err.startswith("voices:")
    for path, data in before.items():
        assert path.read_bytes() == data


def test_unknown_toml_key_is_rejected(tmp_path: Path):
    _write(tmp_path)
    before = (tmp_path / "scriptcast.toml").read_bytes()
    code, out, err = _edit(
        tmp_path,
        [{"file": "toml", "op": "set", "key": "not_a_key", "value": "x"}],
        ["toml"],
    )
    assert code == 1
    assert out == ""
    assert "op[0]:" in err
    assert "unknown key" in err
    assert (tmp_path / "scriptcast.toml").read_bytes() == before


def test_loaders_read_every_edited_file(tmp_path: Path):
    _write(tmp_path)
    code, out, err = _edit(
        tmp_path,
        [
            {
                "file": "voices",
                "op": "set_speaker",
                "id": "guest",
                "fields": {"name": "Guest", "engine": "archive"},
            },
            {
                "file": "clips",
                "op": "set_clip",
                "id": "fresh",
                "fields": {
                    "source": "archive/sources/speech.m4a",
                    "start": 0.0,
                    "end": 1.0,
                    "citation": "C",
                },
            },
            {
                "file": "pronunciations",
                "op": "set_entry",
                "word": "rho",
                "readings": {"always": "roe"},
            },
            {
                "file": "accepted_hearings",
                "op": "delete",
                "word": "in",
                "heard": "it",
            },
            {"file": "toml", "op": "set", "key": "speech_threshold", "value": 0.05},
        ],
        ["voices", "clips", "pronunciations", "accepted_hearings", "toml"],
    )
    assert code == 0, err
    report = json.loads(out)
    assert any(item["id"] == "guest" for item in report["voices"]["speakers"])
    voices = load_voices(tmp_path / "voices.yaml")
    assert "guest" in voices and voices["host"].engine == "kokoro"
    registry = load_registry(tmp_path / "archive" / "clips.yaml")
    assert registry["fresh"]["citation"] == "C"
    assert registry["speech"]["source"].endswith("speech.m4a")
    respellings = load_respellings(tmp_path / "pronunciations.yaml")
    assert respellings["rho"]["always"] == "roe"
    assert respellings["lead"]["always"] == "led"
    accepted = load_accepted_hearings(tmp_path)
    assert "in" not in accepted
    assert accepted["bonded"] == {"about"}
    assert Project(tmp_path).speech_threshold == 0.05


def test_added_key_keeps_double_quoted_siblings(tmp_path: Path):
    voices = """\
speakers:
  aisha:
    name: "Aisha"
    engine: "omnivoice"
    reference_audio: "voices/candidates/aisha_sentence_ref.wav"
    reference_text: "Elite Obscuration, and the Square Ceiling."
    character_profile: "Co-narrator, mathematical and empirical throughline"
"""
    (tmp_path / "voices.yaml").write_text(voices, encoding="utf-8")
    code, _out, err = _edit(
        tmp_path,
        [{
            "file": "voices",
            "op": "set_speaker",
            "id": "aisha",
            "fields": {
                "name": "Aisha",
                "engine": "omnivoice",
                "reference_audio": "voices/candidates/aisha_sentence_ref.wav",
                "reference_text": "Elite Obscuration, and the Square Ceiling.",
                "character_profile": "Co-narrator, mathematical and empirical throughline",
                "speed": 1.04,
            },
        }],
        ["voices"],
    )
    assert code == 0, err
    updated = (tmp_path / "voices.yaml").read_text(encoding="utf-8")
    assert updated == voices + '    speed: 1.04\n'
    assert 'name: "Aisha"' in updated
    assert 'engine: "omnivoice"' in updated
    assert 'reference_audio: "voices/candidates/aisha_sentence_ref.wav"' in updated
    assert 'reference_text: "Elite Obscuration, and the Square Ceiling."' in updated
    assert 'character_profile: "Co-narrator, mathematical and empirical throughline"' in updated

    clips = """\
clips:
  speech:
    source: "archive/sources/speech.m4a"
    start: 1.0
    end: 2.0
    citation: "A book"
"""
    (tmp_path / "archive").mkdir()
    (tmp_path / "archive" / "clips.yaml").write_text(clips, encoding="utf-8")
    code, _out, err = _edit(
        tmp_path,
        [{
            "file": "clips",
            "op": "set_clip",
            "id": "speech",
            "fields": {
                "source": "archive/sources/speech.m4a",
                "start": 1.0,
                "end": 2.0,
                "citation": "A book",
                "content_note": "quiet",
            },
        }],
        ["clips"],
    )
    assert code == 0, err
    assert (tmp_path / "archive" / "clips.yaml").read_text(encoding="utf-8") == clips + '    content_note: "quiet"\n'

    pronunciations = """\
lead:
  always: "led"
"""
    (tmp_path / "pronunciations.yaml").write_text(pronunciations, encoding="utf-8")
    code, _out, err = _edit(
        tmp_path,
        [{
            "file": "pronunciations",
            "op": "set_entry",
            "word": "lead",
            "readings": {"always": "led", "noun": "ledd"},
        }],
        ["pronunciations"],
    )
    assert code == 0, err
    assert (tmp_path / "pronunciations.yaml").read_text(encoding="utf-8") == pronunciations + '  noun: "ledd"\n'


def test_show_fields_hold_every_key_and_round_trip(tmp_path: Path):
    voices = """\
speakers:
  aisha:
    name: "Aisha"
    engine: "archive"
    note: "keep-me"
"""
    clips = """\
clips:
  speech:
    source: "archive/sources/speech.m4a"
    start: 1.0
    end: 2.0
    citation: "A book"
    linger: 12
    speaker: "Lee Atwater"
    transcript_source: "The published excerpt."
"""
    pronunciations = """\
lead:
  always: "led"
  noun: "ledd"
"""
    (tmp_path / "voices.yaml").write_text(voices, encoding="utf-8")
    (tmp_path / "archive").mkdir()
    (tmp_path / "archive" / "clips.yaml").write_text(clips, encoding="utf-8")
    (tmp_path / "pronunciations.yaml").write_text(pronunciations, encoding="utf-8")
    code, out, err = _run(["show", "--json", "--root", str(tmp_path)])
    assert code == 0, err
    report = json.loads(out)
    speaker = report["voices"]["speakers"][0]
    assert speaker["fields"] == {"name": "Aisha", "engine": "archive", "note": "keep-me"}
    clip = report["clips"]["clips"][0]
    assert clip["source"] == "archive/sources/speech.m4a"
    assert clip["fields"] == {
        "source": "archive/sources/speech.m4a",
        "start": 1.0,
        "end": 2.0,
        "citation": "A book",
        "linger": 12,
        "speaker": "Lee Atwater",
        "transcript_source": "The published excerpt.",
    }
    entry = report["pronunciations"]["entries"][0]
    assert entry["fields"] == {"always": "led", "noun": "ledd"}
    assert entry["readings"] == entry["fields"]

    fields = dict(speaker["fields"])
    fields["speed"] = 1.04
    code, _out, err = _edit(
        tmp_path,
        [{"file": "voices", "op": "set_speaker", "id": "aisha", "fields": fields}],
        ["voices"],
    )
    assert code == 0, err
    updated = (tmp_path / "voices.yaml").read_text(encoding="utf-8")
    import difflib
    diff = [
        line for line in difflib.unified_diff(voices.splitlines(), updated.splitlines(), lineterm="")
        if line.startswith(("+", "-")) and not line.startswith(("+++", "---"))
    ]
    assert diff == ["+    speed: 1.04"]

    code, _out, err = _edit(
        tmp_path,
        [{"file": "clips", "op": "set_clip", "id": "speech", "fields": clip["fields"]}],
        ["clips"],
    )
    assert code == 0, err
    assert (tmp_path / "archive" / "clips.yaml").read_text(encoding="utf-8") == clips
    assert 'linger: 12' in clips
    assert 'speaker: "Lee Atwater"' in clips
    assert 'transcript_source: "The published excerpt."' in clips


def test_init_writes_header_and_refuses_a_second_time(tmp_path: Path):
    code, out, err = _run(["init", "--root", str(tmp_path)])
    assert code == 0, err
    header = "# scriptCast project settings. Every key is optional; see the README Configuration table.\n"
    assert (tmp_path / "scriptcast.toml").read_text(encoding="utf-8") == header
    report = json.loads(out)
    assert report["schema"] == 1
    assert "missing" not in report["toml"]
    assert report["toml"]["values"] == {}
    assert report["voices"]["missing"] is True
    before = (tmp_path / "scriptcast.toml").read_bytes()
    code, out, err = _run(["init", "--root", str(tmp_path)])
    assert code == 2
    assert out == ""
    assert "already exists" in err
    assert (tmp_path / "scriptcast.toml").read_bytes() == before

    from io import StringIO
    from contextlib import redirect_stdout
    help_out = StringIO()
    try:
        with redirect_stdout(help_out):
            main(["edit", "--help"])
    except SystemExit as exc:
        assert exc.code == 0
    text = help_out.getvalue()
    assert "complete mapping" in text
    assert "byte-identical" in text


def _tracked(root: Path) -> list[Path]:
    return [
        root / "voices.yaml",
        root / "archive" / "clips.yaml",
        root / "pronunciations.yaml",
        root / "accepted_hearings.yaml",
        root / "scriptcast.toml",
        root / "speaker_rates.json",
    ]
