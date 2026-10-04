"""Plain-text report over review_log.jsonl and accepted_hearings.yaml.

Shows false-failure rate per episode, repeat-offender script words, and
before/after failure counts for words with an accepted hearing (using
timestamps on stress.json entries when present).
"""
from __future__ import annotations

import argparse
import json
import re
from collections import Counter, defaultdict
from pathlib import Path

from scriptcast.review import (
    ACCEPTED_HEARINGS,
    load_accepted_hearings,
    load_reviews,
)

_PUNCT_RE = re.compile(r"[^a-z0-9\s]")
_WS_RE = re.compile(r"\s+")


def _words(text: str) -> list[str]:
    text = _PUNCT_RE.sub(" ", text.lower())
    return [w for w in _WS_RE.sub(" ", text).strip().split() if w]


def _parse_ts(value: object) -> str | None:
    if value is None:
        return None
    text = str(value).strip()
    return text or None


def _load_stress_entries(project_dir: Path, episode: str | None) -> list[dict]:
    """Collect stress.json turn entries under outputs/, optionally filtered."""
    root = Path(project_dir).resolve()
    # project_dir is the voices folder; stress.json usually sits under
    # <project-root>/outputs/<episode>/. Walk a few anchors and dedupe paths.
    stress_paths: list[Path] = []
    seen: set[Path] = set()
    for anchor in (root, root.parent, root.parent.parent):
        outputs = anchor / "outputs"
        if not outputs.is_dir():
            continue
        for path in outputs.glob("**/stress.json"):
            resolved = path.resolve()
            if resolved in seen:
                continue
            ep = resolved.parent.name
            if episode and ep != episode:
                continue
            seen.add(resolved)
            stress_paths.append(resolved)
    entries: list[dict] = []
    for path in stress_paths:
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if not isinstance(data, list):
            continue
        ep = path.parent.name
        for item in data:
            if isinstance(item, dict):
                row = dict(item)
                row.setdefault("_episode", ep)
                row.setdefault("_stress_path", str(path))
                entries.append(row)
    return entries


def _hearing_added_at(project_dir: Path) -> dict[str, str]:
    """Map word → timestamp when its accepted hearing was added (YAML comments)."""
    path = Path(project_dir) / ACCEPTED_HEARINGS
    if not path.is_file():
        return {}
    current_word: str | None = None
    pending_ts: str | None = None
    out: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("#"):
            if stripped.startswith("# review "):
                pending_ts = stripped[len("# review ") :].strip()
            continue
        if stripped.endswith(":") and not stripped.startswith("-"):
            current_word = stripped[:-1].strip()
            pending_ts = None
            continue
        if stripped.startswith("-") and current_word:
            if pending_ts and current_word not in out:
                out[current_word] = pending_ts
            pending_ts = None
    return out


def render_report(project_dir: Path, episode: str | None = None) -> str:
    reviews = load_reviews(project_dir)
    if episode:
        reviews = [r for r in reviews if r.get("episode") == episode]
    lines: list[str] = []
    lines.append(f"review-stats · {project_dir}")
    if episode:
        lines.append(f"episode filter: {episode}")
    lines.append(f"reviews: {len(reviews)}")

    by_episode: dict[str, list[dict]] = defaultdict(list)
    for record in reviews:
        by_episode[str(record.get("episode") or "?")].append(record)

    lines.append("")
    lines.append("False-failure rate (failures later marked recognizer_misheard)")
    if not by_episode:
        lines.append("  (no reviews)")
    for ep in sorted(by_episode):
        rows = by_episode[ep]
        misheard = sum(1 for r in rows if r.get("outcome") == "recognizer_misheard")
        total = len(rows)
        rate = (misheard / total) if total else 0.0
        lines.append(
            f"  {ep}: {misheard}/{total} = {rate:.1%} "
            f"(of reviewed failures marked recognizer_misheard)"
        )

    word_episodes: dict[str, set[str]] = defaultdict(set)
    word_counts: Counter[str] = Counter()
    for record in reviews:
        if record.get("outcome") != "recognizer_misheard":
            continue
        ep = str(record.get("episode") or "?")
        for word in _words(str(record.get("script_text") or "")):
            word_episodes[word].add(ep)
            word_counts[word] += 1
    offenders = sorted(
        (
            (word, len(eps), word_counts[word])
            for word, eps in word_episodes.items()
            if len(eps) >= 2 or word_counts[word] >= 2
        ),
        key=lambda row: (-row[1], -row[2], row[0]),
    )
    lines.append("")
    lines.append("Repeat-offender words")
    if not offenders:
        lines.append("  (none)")
    else:
        for word, n_eps, n_hits in offenders[:40]:
            lines.append(f"  {word}: {n_hits} review(s) across {n_eps} episode(s)")

    accepted = load_accepted_hearings(project_dir)
    added_at = _hearing_added_at(project_dir)
    stress = _load_stress_entries(project_dir, episode)
    lines.append("")
    lines.append("Accepted hearings · failures before vs after")
    if not accepted:
        lines.append("  (no accepted hearings)")
    else:
        for word in sorted(accepted):
            ts = added_at.get(word)
            before = after = unknown = 0
            for entry in stress:
                if not entry.get("failed"):
                    continue
                expected = " ".join(_words(str(entry.get("expected") or "")))
                if word not in expected.split():
                    continue
                entry_ts = _parse_ts(entry.get("timestamp") or entry.get("verified_at"))
                if ts and entry_ts:
                    if entry_ts < ts:
                        before += 1
                    else:
                        after += 1
                else:
                    unknown += 1
            hearings = ", ".join(sorted(accepted[word]))
            stamp = ts or "unknown"
            lines.append(
                f"  {word} ← [{hearings}] added {stamp}: "
                f"before {before} · after {after}"
                + (f" · undated {unknown}" if unknown else "")
            )

    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("project_dir", type=Path)
    parser.add_argument("--episode", default=None, help="Limit to one episode id")
    args = parser.parse_args(argv)
    print(render_report(args.project_dir, args.episode), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
