"""Per-project review log, accepted hearings, and Whisper vocabulary hints.

``project_dir`` is the folder that holds the configured voices file from
``scriptcast.toml`` (the same directory as voices/pronunciations). Review
artifacts live next to those files:

- ``review_log.jsonl`` — append-only reviewer decisions
- ``accepted_hearings.yaml`` — word → allowed Whisper hearings
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

import yaml

REVIEW_LOG = "review_log.jsonl"
ACCEPTED_HEARINGS = "accepted_hearings.yaml"
OUTCOMES = (
    "recognizer_misheard",
    "audio_wrong",
    "script_needs_correction",
)
_PUNCT_RE = re.compile(r"[^a-z0-9\s]")
_WS_RE = re.compile(r"\s+")


def project_dir_from_voices(voices_path: Path) -> Path:
    """Directory of the configured voices file — the project_dir for reviews."""
    return Path(voices_path).expanduser().resolve().parent


def _normalize_token(text: str) -> str:
    return _PUNCT_RE.sub("", text.lower())


def _normalize_words(text: str) -> list[str]:
    text = _PUNCT_RE.sub(" ", text.lower())
    return [w for w in _WS_RE.sub(" ", text).strip().split() if w]


def _utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def append_review(project_dir: Path, record: dict) -> Path:
    """Append one JSON line to review_log.jsonl. Returns the log path."""
    project_dir = Path(project_dir)
    project_dir.mkdir(parents=True, exist_ok=True)
    path = project_dir / REVIEW_LOG
    outcome = record.get("outcome")
    if outcome not in OUTCOMES:
        raise ValueError(f"outcome must be one of {OUTCOMES}, got {outcome!r}")
    line = {
        "turn_id": record["turn_id"],
        "episode": record["episode"],
        "span": {
            "start_token": int(record["span"]["start_token"]),
            "end_token": int(record["span"]["end_token"]),
        },
        "script_text": record["script_text"],
        "heard": dict(record["heard"]),
        "outcome": outcome,
        "timestamp": record.get("timestamp") or _utc_now(),
    }
    if record.get("reviewer_correction"):
        line["reviewer_correction"] = record["reviewer_correction"]
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(line, ensure_ascii=False) + "\n")
        handle.flush()
        os.fsync(handle.fileno())
    return path


def load_reviews(project_dir: Path) -> list[dict]:
    path = Path(project_dir) / REVIEW_LOG
    if not path.is_file():
        return []
    out = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        raw = raw.strip()
        if not raw:
            continue
        out.append(json.loads(raw))
    return out


def load_accepted_hearings(project_dir: Path) -> dict[str, set[str]]:
    """word → set of normalised heard tokens from accepted_hearings.yaml."""
    path = Path(project_dir) / ACCEPTED_HEARINGS
    if not path.is_file():
        return {}
    raw = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    if not isinstance(raw, dict):
        raise ValueError(f"{path}: expected a mapping of word to hearing lists")
    out: dict[str, set[str]] = {}
    for word, hearings in raw.items():
        key = _normalize_token(str(word))
        if not key:
            continue
        if isinstance(hearings, str):
            hearings = [hearings]
        if not isinstance(hearings, list):
            raise ValueError(f"{path}: {word!r} must map to a list of hearings")
        accepted = {_normalize_token(str(h)) for h in hearings if _normalize_token(str(h))}
        if accepted:
            out.setdefault(key, set()).update(accepted)
    return out


def _hearing_comments(path: Path) -> dict[tuple[str, str], str]:
    """Read (word, heard) → review timestamp from YAML comments."""
    if not path.is_file():
        return {}
    current_word: str | None = None
    pending_ts: str | None = None
    out: dict[tuple[str, str], str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("# review "):
            pending_ts = stripped[len("# review ") :].strip()
            continue
        if stripped.startswith("#"):
            continue
        if stripped.endswith(":") and not stripped.startswith("-"):
            current_word = _normalize_token(stripped[:-1])
            pending_ts = None
            continue
        if stripped.startswith("-") and current_word:
            heard = _normalize_token(stripped[1:].strip())
            if heard and pending_ts:
                out[(current_word, heard)] = pending_ts
            pending_ts = None
    return out


def add_accepted_hearing(
    project_dir: Path, word: str, heard: str, review_ts: str
) -> Path:
    """Add one accepted hearing and write the YAML atomically."""
    project_dir = Path(project_dir)
    project_dir.mkdir(parents=True, exist_ok=True)
    path = project_dir / ACCEPTED_HEARINGS
    word_n = _normalize_token(word)
    heard_n = _normalize_token(heard)
    if not word_n or not heard_n:
        raise ValueError("word and heard must contain alphanumeric characters")
    comments = _hearing_comments(path)
    comments[(word_n, heard_n)] = review_ts
    existing = yaml.safe_load(path.read_text(encoding="utf-8")) if path.is_file() else {}
    if existing is None:
        existing = {}
    if not isinstance(existing, dict):
        raise ValueError(f"{path}: expected a mapping")
    bucket = existing.setdefault(word_n, [])
    if not isinstance(bucket, list):
        bucket = [bucket]
        existing[word_n] = bucket
    if heard_n not in {_normalize_token(str(h)) for h in bucket}:
        bucket.append(heard_n)
    lines = [
        "# Accepted hearings: treat these Whisper hearings as matching the script word.",
        "# Comments under each word cite the review timestamp that added the hearing.",
        "",
    ]
    for key in sorted(existing):
        lines.append(f"{key}:")
        values = existing[key]
        if not isinstance(values, list):
            values = [values]
        for value in values:
            token = _normalize_token(str(value))
            if not token:
                continue
            stamp = comments.get((key, token))
            if stamp:
                lines.append(f"  # review {stamp}")
            lines.append(f"  - {token}")
        lines.append("")
    payload = "\n".join(lines).rstrip() + "\n"
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            dir=project_dir,
            prefix=f".{ACCEPTED_HEARINGS}.",
            suffix=".tmp",
            delete=False,
        ) as handle:
            temporary = Path(handle.name)
            handle.write(payload)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
        temporary = None
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)
    return path


def vocabulary_hints(project_dir: Path, limit: int = 40) -> str:
    """Distinct script words from recognizer_misheard reviews, newest first."""
    if limit <= 0:
        return ""
    words: list[str] = []
    seen: set[str] = set()
    for record in reversed(load_reviews(project_dir)):
        if record.get("outcome") != "recognizer_misheard":
            continue
        for token in _normalize_words(str(record.get("script_text") or "")):
            if token in seen:
                continue
            seen.add(token)
            words.append(token)
            if len(words) >= limit:
                return " ".join(words)
    return " ".join(words)


def _cmd_add(args: argparse.Namespace) -> int:
    project_dir = Path(args.project_dir)
    heard = {"whisper": args.heard_whisper}
    if args.heard_phonon:
        heard["phonon"] = args.heard_phonon
    timestamp = args.timestamp or _utc_now()
    record = {
        "turn_id": args.turn_id,
        "episode": args.episode,
        "span": {"start_token": args.start_token, "end_token": args.end_token},
        "script_text": args.script_text,
        "heard": heard,
        "outcome": args.outcome,
        "timestamp": timestamp,
    }
    if args.correction:
        record["reviewer_correction"] = args.correction
    path = append_review(project_dir, record)
    accepted = 0
    if args.outcome == "recognizer_misheard":
        for pair in args.accept or []:
            if "=" not in pair:
                print(f"scriptcast-review: bad --accept {pair!r} (want WORD=HEARD)",
                      file=sys.stderr)
                return 2
            word, heard_word = pair.split("=", 1)
            add_accepted_hearing(project_dir, word, heard_word, timestamp)
            accepted += 1
    print(f"appended review to {path}"
          + (f" · accepted {accepted} hearing(s)" if accepted else ""))
    print("Saved locally. Used by the next verification; does not retrain any model.")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Append a reviewer decision to the project review log."
    )
    sub = parser.add_subparsers(dest="command", required=True)
    add = sub.add_parser("add", help="Append one review decision")
    add.add_argument("project_dir", type=Path)
    add.add_argument("--turn-id", required=True)
    add.add_argument("--episode", required=True)
    add.add_argument("--start-token", type=int, required=True)
    add.add_argument("--end-token", type=int, required=True)
    add.add_argument("--script-text", required=True)
    add.add_argument("--heard-whisper", required=True)
    add.add_argument("--heard-phonon", default=None)
    add.add_argument("--outcome", required=True, choices=OUTCOMES)
    add.add_argument("--correction", default=None)
    add.add_argument("--timestamp", default=None)
    add.add_argument(
        "--accept",
        action="append",
        default=[],
        metavar="WORD=HEARD",
        help="Accept a hearing for recognizer_misheard (repeatable)",
    )
    args = parser.parse_args(argv)
    if args.command == "add":
        return _cmd_add(args)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
