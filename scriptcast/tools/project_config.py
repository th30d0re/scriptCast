"""Show, validate, and edit a scriptCast project's configuration files."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
import tempfile
import tomllib
from pathlib import Path

import yaml

from scriptcast.archive import load_registry
from scriptcast.config_edit import FILE_KINDS, EditError, apply_ops
from scriptcast.pronunciation import load_respellings
from scriptcast.project import CONFIG_NAME, DEFAULTS, Project, find_root
from scriptcast.review import (
    ACCEPTED_HEARINGS,
    _hearing_comments,
    _normalize_token,
    load_accepted_hearings,
    project_dir_from_voices,
)
from scriptcast.voices import load_voices

# Copied from ui/src/review.rs COMMON_WORDS.
# risky_accept: the word or the hearing is <= 2 characters, or is in this list.
COMMON_WORDS = frozenset(
    {
        "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "from",
        "had", "has", "have", "he", "her", "his", "i", "if", "in", "into", "is",
        "it", "its", "me", "my", "no", "not", "of", "on", "or", "our", "she",
        "so", "that", "the", "their", "them", "then", "there", "they", "this",
        "to", "up", "us", "was", "we", "were", "what", "when", "which", "who",
        "will", "with", "you", "your",
    }
)

_SPEAKER_ERR = re.compile(r"Speaker '([^']*)'")


def _risky(word: str, heard: str) -> bool:
    def common(token: str) -> bool:
        return len(token) <= 2 or token in COMMON_WORDS

    return common(word) or common(heard)


def _resolve(root: Path, value: object) -> Path:
    path = Path(str(value)).expanduser()
    return path if path.is_absolute() else root / path


def _jsonable(value: object) -> object:
    if isinstance(value, dict):
        return {str(key): _jsonable(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [_jsonable(item) for item in value]
    if isinstance(value, Path):
        return str(value)
    if isinstance(value, (str, int, float, bool)) or value is None:
        return value
    if hasattr(value, "isoformat"):
        return value.isoformat()
    return str(value)


def _read_toml(root: Path) -> tuple[dict, list[str], list[str], bool]:
    """Return raw values, unknown keys, problems, and whether the file is missing."""
    path = root / CONFIG_NAME
    if not path.is_file():
        return {}, [], [], True
    try:
        raw = tomllib.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        return {}, [], [str(exc)], False
    if not isinstance(raw, dict):
        return {}, [], [f"{path}: expected a table"], False
    unknown = sorted(set(raw) - set(DEFAULTS))
    problems: list[str] = []
    try:
        Project(root)
    except Exception as exc:
        problems.append(str(exc))
    return raw, unknown, problems, False


def resolve_paths(root: Path) -> dict[str, Path]:
    raw, _unknown, _problems, missing = _read_toml(root)
    merged = dict(DEFAULTS)
    if not missing:
        for key, value in raw.items():
            if key in DEFAULTS:
                merged[key] = value
    voices = _resolve(root, merged["voices"])
    return {
        "toml": root / CONFIG_NAME,
        "voices": voices,
        "pronunciations": _resolve(root, merged["pronunciations"]),
        "speaker_rates": _resolve(root, merged["speaker_rates"]),
        "clips": _resolve(root, merged["clips"]),
        "accepted_hearings": project_dir_from_voices(voices) / ACCEPTED_HEARINGS,
    }


def build_report(root: Path) -> dict:
    root = root.resolve()
    raw, unknown, toml_problems, toml_missing = _read_toml(root)
    paths = resolve_paths(root)
    toml: dict = {
        "path": str(paths["toml"].resolve()),
        "values": {},
        "unknown_keys": [],
        "problems": [],
    }
    if toml_missing:
        toml["missing"] = True
    else:
        toml["values"] = _jsonable(raw)
        toml["unknown_keys"] = unknown
        toml["problems"] = toml_problems
    return {
        "schema": 1,
        "root": str(root),
        "toml": toml,
        "voices": _voices_section(paths["voices"]),
        "clips": _clips_section(paths["clips"], root),
        "pronunciations": _pronunciations_section(paths["pronunciations"]),
        "accepted_hearings": _hearings_section(paths["accepted_hearings"]),
        "speaker_rates": _rates_section(paths["speaker_rates"]),
    }


def _missing(path: Path, list_key: str, empty: object) -> dict:
    report = {"path": str(path.resolve()), "missing": True, "problems": []}
    report[list_key] = empty
    return report


def _voices_section(path: Path) -> dict:
    if not path.is_file():
        return _missing(path, "speakers", [])
    problems: list[str] = []
    try:
        load_voices(path)
    except Exception as exc:
        problems.append(str(exc))
    try:
        data = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    except Exception as exc:
        if not problems:
            problems.append(str(exc))
        data = {}
    speakers_raw = data.get("speakers") if isinstance(data, dict) else None
    attributed = None
    if problems:
        match = _SPEAKER_ERR.search(problems[0])
        if match:
            attributed = match.group(1)
    speakers = []
    if isinstance(speakers_raw, dict):
        for speaker_id, raw_config in speakers_raw.items():
            fields = _jsonable(raw_config) if isinstance(raw_config, dict) else {}
            if not isinstance(fields, dict):
                fields = {}
            name = fields.get("name")
            engine = fields.get("engine")
            item_problems = list(problems) if attributed is not None and str(speaker_id) == attributed else []
            speakers.append(
                {
                    "id": str(speaker_id),
                    "name": name if isinstance(name, str) else (None if name is None else str(name)),
                    "engine": str(engine) if engine is not None else "kokoro",
                    "fields": fields,
                    "problems": item_problems,
                }
            )
    return {"path": str(path.resolve()), "speakers": speakers, "problems": problems}


def _opt_str(entry: dict, key: str) -> str | None:
    if key not in entry or entry[key] is None:
        return None
    return str(entry[key])


def _opt_float(entry: dict, key: str) -> float | None:
    if key not in entry or entry[key] is None:
        return None
    try:
        return float(entry[key])
    except (TypeError, ValueError):
        return None


def _source_exists(root: Path, source: str | None) -> bool:
    if not source:
        return False
    path = Path(source).expanduser()
    if not path.is_absolute():
        path = root / path
    return path.is_file()


def _clips_section(path: Path, root: Path) -> dict:
    if not path.is_file():
        return _missing(path, "clips", [])
    problems: list[str] = []
    registry: dict = {}
    try:
        registry = load_registry(path)
    except Exception as exc:
        problems.append(str(exc))
    clips = []
    if isinstance(registry, dict) and not problems:
        for clip_id, entry in registry.items():
            fields = entry if isinstance(entry, dict) else {}
            source = _opt_str(fields, "source")
            rendered = _jsonable(fields) if isinstance(fields, dict) else {}
            clips.append(
                {
                    "id": str(clip_id),
                    "source": source,
                    "start": _opt_float(fields, "start"),
                    "end": _opt_float(fields, "end"),
                    "citation": _opt_str(fields, "citation"),
                    "origin_url": _opt_str(fields, "origin_url"),
                    "content_note": _opt_str(fields, "content_note"),
                    "source_exists": _source_exists(root, source),
                    "fields": rendered if isinstance(rendered, dict) else {},
                    "problems": [],
                }
            )
    return {"path": str(path.resolve()), "clips": clips, "problems": problems}


def _pronunciations_section(path: Path) -> dict:
    if not path.is_file():
        return _missing(path, "entries", [])
    problems: list[str] = []
    try:
        load_respellings(path)
    except Exception as exc:
        problems.append(str(exc))
    entries = []
    if not problems:
        try:
            data = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
        except Exception as exc:
            data = {}
            problems.append(str(exc))
        if isinstance(data, dict):
            for word, readings in data.items():
                rendered = _jsonable(readings) if isinstance(readings, dict) else {}
                readings_obj = rendered if isinstance(rendered, dict) else {}
                entries.append(
                    {
                        "word": str(word),
                        "readings": readings_obj,
                        "fields": readings_obj,
                        "problems": [],
                    }
                )
    return {"path": str(path.resolve()), "entries": entries, "problems": problems}


def _review_ts(stamps: list[str | None]) -> str | None:
    if len(stamps) == 1:
        return stamps[0]
    if stamps and all(stamp == stamps[0] for stamp in stamps):
        return stamps[0]
    return None


def _hearings_section(path: Path) -> dict:
    if not path.is_file():
        return _missing(path, "entries", [])
    problems: list[str] = []
    try:
        load_accepted_hearings(path.parent)
    except Exception as exc:
        problems.append(str(exc))
    comments = _hearing_comments(path)
    entries = []
    try:
        data = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    except Exception as exc:
        if not problems:
            problems.append(str(exc))
        data = {}
    if isinstance(data, dict) and not problems:
        for word, hearings in data.items():
            if isinstance(hearings, str):
                heard = [hearings]
            elif isinstance(hearings, list):
                heard = ["" if item is None else str(item) for item in hearings]
            else:
                heard = []
            word_text = str(word)
            word_norm = _normalize_token(word_text)
            stamps = [
                comments.get((word_norm, _normalize_token(item)))
                for item in heard
            ]
            entries.append(
                {
                    "word": word_text,
                    "heard": heard,
                    "review_ts": _review_ts(stamps),
                    "risky": _risky(word_text, word_text) or any(_risky(word_text, item) for item in heard),
                }
            )
    return {"path": str(path.resolve()), "entries": entries, "problems": problems}


def _rates_section(path: Path) -> dict:
    if not path.is_file():
        return _missing(path, "rates", {})
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        return {"path": str(path.resolve()), "rates": {}, "problems": [str(exc)]}
    if not isinstance(data, dict):
        return {
            "path": str(path.resolve()),
            "rates": {},
            "problems": ["speaker rates must be a mapping"],
        }
    return {"path": str(path.resolve()), "rates": _jsonable(data), "problems": []}


def has_problem(report: dict) -> bool:
    toml = report["toml"]
    if toml.get("problems") or toml.get("unknown_keys"):
        return True
    for key in ("voices", "clips", "pronunciations", "accepted_hearings", "speaker_rates"):
        section = report[key]
        if section.get("problems"):
            return True
        for item_key in ("speakers", "clips", "entries"):
            for item in section.get(item_key, []) or []:
                if item.get("problems"):
                    return True
    return False


def _dump(report: dict) -> None:
    json.dump(report, sys.stdout, indent=2, ensure_ascii=False)
    sys.stdout.write("\n")


def _project_root(explicit: Path | None) -> Path | None:
    if explicit is None:
        return find_root()
    root = explicit.expanduser().resolve()
    if not root.is_dir():
        print(f"root does not exist: {root}", file=sys.stderr)
        return None
    return root


def _parse_expects(items: list[str]) -> dict[str, str] | None:
    expects: dict[str, str] = {}
    for item in items:
        if "=" not in item:
            print(f"--expect-sha256 {item!r} must be FILE=HEX", file=sys.stderr)
            return None
        kind, digest = item.split("=", 1)
        if kind in expects:
            print(f"duplicate --expect-sha256 for {kind}", file=sys.stderr)
            return None
        expects[kind] = digest.strip().lower()
    return expects


def _atomic_write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        handle = tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            newline="",
            dir=path.parent,
            prefix=f".{path.name}.",
            suffix=".tmp",
            delete=False,
        )
        temporary = Path(handle.name)
        with handle:
            handle.write(text)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
        temporary = None
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def _validate_pending(paths: dict[str, Path], updated: dict[str, str]) -> list[tuple[str, str]]:
    errors: list[tuple[str, str]] = []
    with tempfile.TemporaryDirectory(prefix="scriptcast-l-") as scratch:
        tmp = Path(scratch)
        checks = {
            "voices": lambda dest: load_voices(dest),
            "clips": lambda dest: load_registry(dest),
            "pronunciations": lambda dest: load_respellings(dest),
        }
        for kind, check in checks.items():
            if kind not in updated:
                continue
            dest = tmp / paths[kind].name
            dest.write_text(updated[kind], encoding="utf-8", newline="")
            try:
                check(dest)
            except Exception as exc:
                errors.append((kind, str(exc).replace(str(dest), str(paths[kind]))))
        if "accepted_hearings" in updated:
            hearings_dir = tmp / "hearings"
            hearings_dir.mkdir()
            dest = hearings_dir / ACCEPTED_HEARINGS
            dest.write_text(updated["accepted_hearings"], encoding="utf-8", newline="")
            try:
                load_accepted_hearings(hearings_dir)
            except Exception as exc:
                errors.append(("accepted_hearings", str(exc).replace(str(dest), str(paths["accepted_hearings"]))))
        if "toml" in updated:
            project = tmp / "project"
            project.mkdir()
            dest = project / CONFIG_NAME
            dest.write_text(updated["toml"], encoding="utf-8", newline="")
            try:
                Project(project)
            except Exception as exc:
                errors.append(("toml", str(exc).replace(str(dest), str(paths["toml"]))))
    return errors


def _touched_kinds(ops: list) -> list[str]:
    kinds: list[str] = []
    for index, op in enumerate(ops):
        if not isinstance(op, dict):
            raise EditError(index, "op must be an object")
        kind = op.get("file")
        if kind not in FILE_KINDS:
            raise EditError(index, f"unknown file {kind!r}")
        if kind not in kinds:
            kinds.append(str(kind))
    return kinds


def _cmd_edit(root: Path, ops_path: Path, expect_items: list[str]) -> int:
    expects = _parse_expects(expect_items)
    if expects is None:
        return 1
    try:
        ops = json.loads(ops_path.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"ops: {exc}", file=sys.stderr)
        return 1
    if not isinstance(ops, list):
        print("ops: expected a list", file=sys.stderr)
        return 1
    try:
        touched = _touched_kinds(ops)
    except EditError as exc:
        print(f"op[{exc.index}]: {exc.message}", file=sys.stderr)
        return 1
    missing = [kind for kind in touched if kind not in expects]
    if missing:
        for kind in missing:
            print(f"{kind}: expected sha256 was not provided", file=sys.stderr)
        return 1
    paths = resolve_paths(root)
    mismatches = []
    texts: dict[str, str] = {}
    for kind in touched:
        path = paths[kind]
        try:
            raw = path.read_bytes() if path.is_file() else b""
            texts[kind] = raw.decode("utf-8")
        except UnicodeError:
            print(f"{kind}: file is not utf-8", file=sys.stderr)
            return 1
        digest = hashlib.sha256(raw).hexdigest()
        if digest != expects[kind]:
            mismatches.append(f"{kind}: sha256 mismatch")
    if mismatches:
        for line in mismatches:
            print(line, file=sys.stderr)
        return 3
    try:
        updated = apply_ops(texts, ops)
    except EditError as exc:
        print(f"op[{exc.index}]: {exc.message}", file=sys.stderr)
        return 1
    pending = {kind: updated.get(kind, texts[kind]) for kind in touched}
    errors = _validate_pending(paths, pending)
    if errors:
        for kind, message in errors:
            print(f"{kind}: {message}", file=sys.stderr)
        return 1
    written: list[Path] = []
    originals = {paths[kind]: paths[kind].read_bytes() if paths[kind].is_file() else None for kind in touched}
    try:
        for kind in touched:
            _atomic_write(paths[kind], pending[kind])
            written.append(paths[kind])
    except Exception:
        for path in written:
            original = originals[path]
            if original is None:
                path.unlink(missing_ok=True)
            else:
                path.write_bytes(original)
        raise
    _dump(build_report(root))
    return 0


_COMPLETE_MAPPING = (
    "set_speaker, set_clip, and set_entry take the complete mapping: "
    "keys absent from the op are removed. Unchanged keys stay byte-identical."
)
_INIT_HEADER = (
    "# scriptCast project settings. Every key is optional; see the README Configuration table.\n"
)


def _cmd_init(root: Path) -> int:
    if root.exists() and not root.is_dir():
        print(f"root is not a directory: {root}", file=sys.stderr)
        return 1
    root.mkdir(parents=True, exist_ok=True)
    path = root / CONFIG_NAME
    if path.exists():
        print(f"{path}: scriptcast.toml already exists", file=sys.stderr)
        return 2
    _atomic_write(path, _INIT_HEADER)
    _dump(build_report(root))
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Read, validate, and edit scriptCast project configuration.",
        epilog=_COMPLETE_MAPPING,
    )
    sub = parser.add_subparsers(dest="command", required=True)

    show = sub.add_parser("show", help="Print the project configuration as JSON")
    show.add_argument("--json", action="store_true", required=True)
    show.add_argument("--root", type=Path)

    edit = sub.add_parser(
        "edit",
        help="Apply configuration edits atomically",
        description=_COMPLETE_MAPPING,
    )
    edit.add_argument("--ops", type=Path, required=True)
    edit.add_argument("--expect-sha256", action="append", default=[], metavar="FILE=HEX")
    edit.add_argument("--root", type=Path)

    init = sub.add_parser("init", help="Create scriptcast.toml when the project has none")
    init.add_argument("--root", type=Path, required=True)

    validate = sub.add_parser("validate", help="Print the project configuration and fail if it has problems")
    validate.add_argument("--json", action="store_true", required=True)
    validate.add_argument("--root", type=Path)

    args = parser.parse_args(argv)
    if args.command == "init":
        return _cmd_init(args.root.expanduser().resolve())
    root = _project_root(args.root)
    if root is None:
        return 1
    if args.command == "show":
        _dump(build_report(root))
        return 0
    if args.command == "validate":
        report = build_report(root)
        _dump(report)
        return 1 if has_problem(report) else 0
    if args.command == "edit":
        return _cmd_edit(root, args.ops, args.expect_sha256)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
