"""Byte-preserving structural edits and editor metadata for scripts."""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import re
import tempfile

from scriptcast.archive import load_registry
from scriptcast.parser import _SPEAKER_RE, _TAG_RE, _speaker_id, _timestamp_to_ms
from scriptcast.project import Project
from scriptcast.voices import load_voices


class StaleScriptError(ValueError):
    """The caller edited a script version other than the one on disk."""


class OperationsError(ValueError):
    """One or more operations failed validation."""

    def __init__(self, errors: list[str]) -> None:
        super().__init__("\n".join(errors))
        self.errors = errors


@dataclass(frozen=True)
class _TurnBlock:
    data: bytes
    header_line: int

    def _header(self) -> tuple[str, bytes]:
        line = self.data.splitlines(keepends=True)[0]
        if line.endswith(b"\r\n"):
            return line[:-2].decode("utf-8"), b"\r\n"
        if line.endswith(b"\n"):
            return line[:-1].decode("utf-8"), b"\n"
        return line.decode("utf-8"), b""

    def match(self):
        header, _ = self._header()
        match = _SPEAKER_RE.match(header)
        if match is None:  # Blocks are created only from matching headers.
            raise ValueError(f"invalid speaker header on line {self.header_line}")
        return match

    @property
    def speaker_id(self) -> str:
        return _speaker_id(self.match().group("display_name").strip())

    @property
    def timestamp_ms(self) -> int:
        return _timestamp_to_ms(self.match().group("timestamp"))

    @property
    def body(self) -> bytes:
        first = self.data.find(b"\n")
        return b"" if first < 0 else self.data[first + 1:]

    def with_display_name(self, display_name: str) -> "_TurnBlock":
        header, ending = self._header()
        match = self.match()
        start, end = match.span("display_name")
        rewritten = (header[:start] + display_name + header[end:]).encode("utf-8") + ending
        old_header_length = len(self.data.splitlines(keepends=True)[0])
        return _TurnBlock(rewritten + self.data[old_header_length:], self.header_line)


def _split_script(original: bytes) -> tuple[bytes, list[_TurnBlock], bytes]:
    if b"\r" in original.replace(b"\r\n", b""):
        raise ValueError("Unsupported bare CR line endings; use LF or CRLF")
    # Decode once so invalid UTF-8 is rejected before any write.
    original.decode("utf-8")
    lines = original.splitlines(keepends=True)
    header_offsets: list[tuple[int, int]] = []
    offset = 0
    for line_number, raw_line in enumerate(lines, start=1):
        content = raw_line.removesuffix(b"\n").removesuffix(b"\r").decode("utf-8")
        if _SPEAKER_RE.match(content):
            header_offsets.append((offset, line_number))
        offset += len(raw_line)
    if not header_offsets:
        raise ValueError("No speaker turns found in transcript")
    preamble = original[:header_offsets[0][0]]
    blocks = []
    for index, (start, line_number) in enumerate(header_offsets):
        end = header_offsets[index + 1][0] if index + 1 < len(header_offsets) else len(original)
        blocks.append(_TurnBlock(original[start:end], line_number))
    ending = b"\r\n" if b"\r\n" in original else b"\n"
    return preamble, blocks, ending


def _index(value: object, length: int, field: str, *, allow_end: bool = False) -> int:
    if type(value) is not int:
        raise ValueError(f"{field} must be an integer")
    limit = length if allow_end else length - 1
    if value < 0 or value > limit:
        raise ValueError(f"{field} {value} is out of range")
    return value


def _display_name(speaker_id: object, voices: dict) -> tuple[str, object]:
    if not isinstance(speaker_id, str) or not speaker_id:
        raise ValueError("speaker_id must be a non-empty string")
    voice = voices.get(speaker_id)
    if voice is None:
        raise ValueError(f"unknown speaker_id {speaker_id!r}")
    return voice.name or speaker_id.replace("_", " ").title(), voice


def _required(operation: dict, field: str) -> object:
    if field not in operation:
        raise ValueError(f"missing {field}")
    return operation[field]


_CLIP_AT_START_RE = re.compile(r"^\[clip:([^\]]+)\]")


def _validate_archive(body: bytes, voice: object, registry: dict) -> None:
    if getattr(voice, "engine", None) != "archive":
        return
    text = body.decode("utf-8").lstrip()
    match = _CLIP_AT_START_RE.match(text)
    if match is None:
        raise ValueError("archive speaker body must start with a [clip:id] tag")
    clip_id = match.group(1)
    if clip_id not in registry:
        raise ValueError(f"archive clip {clip_id!r} is not registered")


def _format_timestamp(milliseconds: int) -> str:
    seconds = milliseconds // 1000
    minutes, seconds = divmod(seconds, 60)
    if minutes > 99:
        raise ValueError("computed timestamp exceeds parser limit 99:59")
    return f"{minutes:02d}:{seconds:02d}"


def _insert_timestamp(value: object, blocks: list[_TurnBlock], position: int) -> str:
    if value is not None:
        if not isinstance(value, str) or re.fullmatch(r"\d{1,2}:\d{2}", value) is None:
            raise ValueError("timestamp must be MM:SS or null")
        if int(value.rsplit(":", 1)[1]) > 59:
            raise ValueError("timestamp seconds must be between 00 and 59")
        return value
    if position == 0:
        return "00:00"
    if position == len(blocks):
        return _format_timestamp(blocks[-1].timestamp_ms + 5_000)
    return _format_timestamp((blocks[position - 1].timestamp_ms + blocks[position].timestamp_ms) // 2)


def _new_block(display_name: str, timestamp: str, body: str, ending: bytes) -> _TurnBlock:
    normalized = body.replace("\r\n", "\n").replace("\r", "\n")
    if any(_SPEAKER_RE.match(line) for line in normalized.split("\n")):
        raise ValueError("body must not contain a speaker header line")
    body_bytes = normalized.replace("\n", ending.decode("ascii")).encode("utf-8")
    header = f"{display_name} ({timestamp})".encode("utf-8")
    return _TurnBlock(header + ending + body_bytes + ending + ending, 0)


def _apply_one(
    operation: object, blocks: list[_TurnBlock], ending: bytes, voices: dict, registry: dict,
) -> None:
    if not isinstance(operation, dict):
        raise ValueError("operation must be an object")
    kind = operation.get("op")
    if kind == "insert":
        position = _index(_required(operation, "position"), len(blocks), "position", allow_end=True)
        display_name, voice = _display_name(_required(operation, "speaker_id"), voices)
        body = _required(operation, "body")
        if not isinstance(body, str) or not body.strip():
            raise ValueError("body must be a non-empty string")
        if "\0" in body:
            raise ValueError("body must not contain NUL")
        timestamp = _insert_timestamp(_required(operation, "timestamp"), blocks, position)
        block = _new_block(display_name, timestamp, body, ending)
        _validate_archive(block.body, voice, registry)
        blocks.insert(position, block)
    elif kind == "delete":
        index = _index(_required(operation, "index"), len(blocks), "index")
        blocks.pop(index)
    elif kind == "move":
        index = _index(_required(operation, "index"), len(blocks), "index")
        destination = _index(_required(operation, "to"), len(blocks), "to")
        block = blocks.pop(index)
        blocks.insert(destination, block)
    elif kind == "set_speaker":
        index = _index(_required(operation, "index"), len(blocks), "index")
        display_name, voice = _display_name(_required(operation, "speaker_id"), voices)
        rewritten = blocks[index].with_display_name(display_name)
        _validate_archive(rewritten.body, voice, registry)
        blocks[index] = rewritten
    else:
        raise ValueError(f"unknown op {kind!r}")


def _assemble(preamble: bytes, blocks: list[_TurnBlock], ending: bytes) -> bytes:
    """Join exact blocks, supplying only a missing boundary after a former EOF block."""
    pieces = [preamble]
    for index, block in enumerate(blocks):
        if index and not blocks[index - 1].data.endswith(b"\n"):
            pieces.append(ending)
        pieces.append(block.data)
    return b"".join(pieces)


def edit_script(
    script: Path, operations_path: Path, expect_sha256: str, project: Project,
    voices_path: Path | None = None,
) -> None:
    """Validate ordered operations, then replace ``script`` atomically."""
    script = script.resolve()
    original = script.read_bytes()
    actual_hash = hashlib.sha256(original).hexdigest()
    if actual_hash != expect_sha256:
        raise StaleScriptError("stale script")
    try:
        operations = json.loads(operations_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"Cannot read operations {operations_path}: {exc}") from exc
    if not isinstance(operations, list):
        raise ValueError("operations file must contain a JSON list")

    preamble, blocks, ending = _split_script(original)
    voices = load_voices((voices_path or project.voices).resolve())
    registry = load_registry(project.clips)
    errors: list[str] = []
    for operation_index, operation in enumerate(operations):
        try:
            _apply_one(operation, blocks, ending, voices, registry)
        except (ValueError, UnicodeError) as exc:
            errors.append(f"op[{operation_index}]: {exc}")
    if errors:
        raise OperationsError(errors)

    edited = _assemble(preamble, blocks, ending)
    # Refuse to overwrite a concurrent change made after the initial hash check.
    if script.read_bytes() != original:
        raise StaleScriptError("stale script")
    mode = script.stat().st_mode
    temporary_path: Path | None = None
    try:
        with tempfile.NamedTemporaryFile(
            dir=script.parent, prefix=f".{script.name}.", suffix=".tmp", delete=False,
        ) as temporary:
            temporary_path = Path(temporary.name)
            temporary.write(edited)
            temporary.flush()
            os.fsync(temporary.fileno())
        os.chmod(temporary_path, mode)
        os.replace(temporary_path, script)
    finally:
        if temporary_path is not None:
            temporary_path.unlink(missing_ok=True)


def tags_json(script: Path, project: Project) -> dict:
    original = script.read_bytes()
    _, blocks, _ = _split_script(original)
    registry = load_registry(project.clips)
    voices = load_voices(project.voices)
    voices_by_display_id = {
        _speaker_id(voice.name or speaker_id.replace("_", " ").title()): voice
        for speaker_id, voice in voices.items()
    }
    problems: list[dict] = []
    for index, block in enumerate(blocks):
        body = block.body.decode("utf-8")
        clip_count = 0
        comments = [
            (match.start(), match.end())
            for match in re.finditer(r"<!--.*?-->", body, flags=re.DOTALL)
        ]
        offset = 0
        for line_offset, raw_line in enumerate(body.splitlines(keepends=True), start=1):
            line = raw_line.rstrip("\r\n")
            for match in _TAG_RE.finditer(line):
                start = offset + match.start()
                end = offset + match.end()
                if any(start < stop and begin < end for begin, stop in comments):
                    continue
                tag = match.group(1)
                messages: list[str] = []
                pause = re.fullmatch(r"\[pause:(\d+)(?:ms)?\]", tag)
                if pause:
                    milliseconds = int(pause.group(1))
                    if not 0 < milliseconds <= 10_000:
                        messages.append("pause must be between 1 and 10000 ms")
                elif tag in {"[beat]", "[emphasis]"}:
                    pass
                elif tag.startswith("[tone:"):
                    if re.fullmatch(r"\[tone:[^\]]+\]", tag) is None or not tag[6:-1].strip():
                        messages.append("tone must be non-empty")
                elif tag.startswith("[clip:"):
                    clip_id = tag[6:-1]
                    clip_count += 1
                    if not clip_id:
                        messages.append("clip id must be non-empty")
                    elif clip_id not in registry:
                        messages.append(f"clip {clip_id!r} is not registered")
                    if clip_count > 1:
                        messages.append("clip appears more than once in this turn")
                else:
                    messages.append("unrecognised tag; it will be spoken aloud")
                for message in messages:
                    problems.append({
                        "index": index,
                        "line": block.header_line + line_offset,
                        "tag": tag,
                        "message": message,
                    })
            offset += len(raw_line)
        voice = voices.get(block.speaker_id) or voices_by_display_id.get(block.speaker_id)
        if voice is not None and voice.engine == "archive" and clip_count == 0:
            problems.append({
                "index": index,
                "line": block.header_line,
                "tag": "[clip:id]",
                "message": "archive speaker turn must include a clip tag",
            })
    return {"script_sha256": hashlib.sha256(original).hexdigest(), "problems": problems}


def speakers_json(project: Project, voices_path: Path | None = None) -> dict:
    voices = load_voices((voices_path or project.voices).resolve())
    registry = load_registry(project.clips)
    return {
        "speakers": [
            {
                "id": speaker_id,
                "name": voice.name or speaker_id.replace("_", " ").title(),
                "engine": voice.engine,
                "has_expressive": bool(
                    voice.reference_audio_expressive and voice.reference_text_expressive
                ),
            }
            for speaker_id, voice in voices.items()
        ],
        "clips": [
            {
                "id": clip_id,
                "source": entry.get("source"),
                "start": entry.get("start"),
                "end": entry.get("end"),
                "citation": entry.get("citation", ""),
            }
            for clip_id, entry in registry.items()
        ],
    }
