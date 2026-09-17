"""Recompute an episode script's timestamps from measured audio.

Header timestamps in the markdown scripts are source
references and the join key between a script and its shot list. They are not
what the renderer uses for spacing, but they have to stay monotonic and honest,
and hand-maintaining them after every insert is how collisions get introduced.

This tool derives them instead. Every turn's duration comes from the rendered
manifest when that turn's audio already exists (`turn_id` is a hash of speaker
and text, so it survives a timestamp change), and from a words-per-second rate
calibrated on that same manifest when the turn is new or edited. Positions
accumulate with a fixed inter-turn gap, matching how the renderer lays out the
timeline.

A shot list can be remapped in the same pass: anchors snap to the real turn they
name, and hold times ride a piecewise-linear old-to-new map.

    scriptcast-retime scripts/<episode>.md \
        --manifest outputs/<episode_id>/episode_manifest.json \
        --shotlist video/<episode>_shotlist.md \
        --apply
"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

from scriptcast.markup import tokenize_markup
from scriptcast.parser import parse_transcript
from scriptcast.project import current

_MARK_RE = re.compile(r"[.,;:]")
_HEADER_RE = re.compile(r"^(?P<name>.+?) \((?P<ts>\d{1,2}:\d{2})\)\s*$")
_ANCHOR_RE = re.compile(r"`(?P<name>[^`(]+?) \((?P<ts>\d{1,2}:\d{2})\)`")
_HOLD_RE = re.compile(r"(?P<lead>\*\*Hold:\*\* through )(?P<ts>\d{1,2}:\d{2})")
_DEFAULT_WPS = 2.55


def _rates_path() -> Path:
    """The project's speaker-rates file, resolved when called."""
    return current().speaker_rates


def _mmss(ms: int) -> str:
    total = ms // 1000
    return f"{total // 60:02d}:{total % 60:02d}"


def _to_ms(mmss: str) -> int:
    m, s = mmss.split(":")
    return int(m) * 60_000 + int(s) * 1_000


def _turn_speech(turn) -> tuple[int, int, int]:
    """(word count, punctuation marks, silence ms) for one tokenized turn.

    Punctuation is counted because it costs real time. A comma-heavy list reads
    with a pause at every mark, so a words-only model calls it over-long when it
    is simply being read correctly.
    """
    words = 0
    marks = 0
    silence = 0
    for chunk in turn.markup_chunks:
        if chunk.kind == "speech":
            text = chunk.text or ""
            words += len(text.split())
            marks += len(_MARK_RE.findall(text))
        elif chunk.kind == "silence":
            silence += chunk.duration_ms or 0
    return words, marks, silence


def _load_measured(manifest_path: Path | None) -> dict[str, int]:
    if manifest_path is None or not manifest_path.exists():
        return {}
    manifest = json.loads(manifest_path.read_text())
    return {
        t["turn_id"]: int(t["end_ms"]) - int(t["start_ms"])
        for t in manifest.get("turns", [])
        if t.get("end_ms") is not None and t.get("start_ms") is not None
    }


def _load_known_rates() -> dict[str, tuple[float, float]]:
    """Per-speaker (ms per word, ms per mark) measured on earlier episodes."""
    if not _rates_path().exists():
        return {}
    try:
        raw = json.loads(_rates_path().read_text())
    except ValueError:
        return {}
    known: dict[str, tuple[float, float]] = {}
    for speaker, value in raw.items():
        if isinstance(value, dict):
            known[speaker] = (
                float(value["ms_per_word"]), float(value.get("ms_per_mark", 0.0))
            )
        else:  # older files stored a bare words-per-second figure
            known[speaker] = (1000.0 / float(value), 0.0)
    return known


def _save_known_rates(rates: dict[str, tuple[float, float]]) -> None:
    """Carry this episode's measured coefficients forward.

    A script with no audio yet has nothing to calibrate against, and the same
    voices read every episode. Persisting what the last render measured means a
    brand-new script gets timestamps close to where its audio will actually
    land, instead of a generic words-per-second guess.
    """
    known = {
        speaker: {"ms_per_word": round(w, 2), "ms_per_mark": round(m, 2)}
        for speaker, (w, m) in _load_known_rates().items()
    }
    known.update(
        {
            speaker: {"ms_per_word": round(w, 2), "ms_per_mark": round(m, 2)}
            for speaker, (w, m) in rates.items()
        }
    )
    _rates_path().write_text(json.dumps(known, indent=1, sort_keys=True) + "\n")


_MIN_FIT_TURNS = 15


def _calibrate(turns, measured: dict[str, int]) -> dict[str, tuple[float, float]]:
    """Least-squares (ms per word, ms per mark) per speaker.

    Duration is modelled as `a*words + b*marks` with no intercept and fitted on
    turns whose audio exists. Fitting the two together matters: hold the word
    rate fixed and any pause allowance is absorbed into it, which is how a
    words-only model ends up flagging a correctly-read list as too long. A
    speaker with too few measured turns falls back to a words-only rate.
    """
    samples: dict[str, list[tuple[int, int, int]]] = {}
    for turn in turns:
        if turn.turn_id not in measured:
            continue
        words, marks, silence = _turn_speech(turn)
        speech_ms = measured[turn.turn_id] - silence
        if words < 5 or speech_ms <= 0:
            continue
        samples.setdefault(turn.speaker_id, []).append((speech_ms, words, marks))

    fitted: dict[str, tuple[float, float]] = {}
    for speaker, rows in samples.items():
        total_ms = sum(d for d, _, _ in rows)
        total_words = sum(w for _, w, _ in rows)
        if len(rows) < _MIN_FIT_TURNS:
            fitted[speaker] = (total_ms / total_words, 0.0)
            continue
        sxx = sum(w * w for _, w, _ in rows)
        smm = sum(m * m for _, _, m in rows)
        sxm = sum(w * m for _, w, m in rows)
        sxy = sum(w * d for d, w, _ in rows)
        smy = sum(m * d for d, _, m in rows)
        det = sxx * smm - sxm * sxm
        if det <= 0:
            fitted[speaker] = (total_ms / total_words, 0.0)
            continue
        per_word = (smm * sxy - sxm * smy) / det
        per_mark = (sxx * smy - sxm * sxy) / det
        # A negative coefficient means the fit is not describing speech.
        if per_word <= 0 or per_mark < 0:
            fitted[speaker] = (total_ms / total_words, 0.0)
            continue
        fitted[speaker] = (per_word, per_mark)
    return fitted


def retime(script: Path, manifest: Path | None, gap_ms: int):
    turns = tokenize_markup(parse_transcript(script))
    measured = _load_measured(manifest)
    rates = _calibrate(turns, measured)
    if rates:
        _save_known_rates(rates)
    # A speaker with no audio in this episode still has coefficients from the
    # last one, since the same voices read every episode.
    rates = {**_load_known_rates(), **rates}
    if rates:
        fallback = (
            sum(w for w, _ in rates.values()) / len(rates),
            sum(m for _, m in rates.values()) / len(rates),
        )
    else:
        fallback = (1000.0 / _DEFAULT_WPS, 0.0)

    cursor = 0
    rows = []
    estimated = 0
    for turn in turns:
        words, marks, silence = _turn_speech(turn)
        if turn.turn_id in measured:
            duration = measured[turn.turn_id]
        else:
            estimated += 1
            per_word, per_mark = rates.get(turn.speaker_id, fallback)
            duration = int(round(words * per_word + marks * per_mark)) + silence
        rows.append(
            {
                "turn_index": turn.turn_index,
                "display_name": turn.display_name,
                "speaker_id": turn.speaker_id,
                "old": turn.timestamp_mmss,
                "new": _mmss(cursor),
                "old_ms": turn.timestamp_ms,
                "new_ms": cursor,
                "duration_ms": duration,
                "source": "measured" if turn.turn_id in measured else "estimated",
                "line": turn.line_span[0],
            }
        )
        cursor += duration + gap_ms

    return rows, cursor, estimated, rates, fallback


def rewrite_script(script: Path, rows) -> int:
    lines = script.read_text().splitlines(keepends=True)
    changed = 0
    for row in rows:
        index = row["line"] - 1
        match = _HEADER_RE.match(lines[index].rstrip("\n"))
        if match is None or match.group("name") != row["display_name"]:
            raise SystemExit(
                f"line {row['line']} is not the expected header for turn "
                f"{row['turn_index']}: {lines[index]!r}"
            )
        if match.group("ts") != row["new"]:
            lines[index] = f"{row['display_name']} ({row['new']})\n"
            changed += 1
    script.write_text("".join(lines))
    return changed


def _interpolate(old_ms: int, rows) -> int:
    """Map a point on the old timeline onto the new one, piecewise-linear."""
    prev = None
    for row in rows:
        if row["old_ms"] <= old_ms:
            prev = row
        else:
            nxt = row
            if prev is None:
                return old_ms
            span_old = nxt["old_ms"] - prev["old_ms"]
            span_new = nxt["new_ms"] - prev["new_ms"]
            if span_old <= 0:
                return prev["new_ms"]
            frac = (old_ms - prev["old_ms"]) / span_old
            return int(prev["new_ms"] + frac * span_new)
    if prev is None:
        return old_ms
    return prev["new_ms"] + (old_ms - prev["old_ms"])


def rewrite_shotlist(
    shotlist: Path, rows, tolerance_s: int = 30
) -> tuple[int, list[str], list[str]]:
    """Remap anchors and hold times onto the new timeline.

    An anchor is a join key: a speaker name plus that turn's timestamp. It is
    remapped only when it resolves to a real turn by that speaker within
    `tolerance_s`. Anything further away is a broken reference that predates
    this run, so it is left alone and reported rather than snapped to whatever
    turn happens to be nearest.
    """
    text = shotlist.read_text()
    warnings: list[str] = []
    unresolved: list[str] = []
    by_speaker: dict[str, list] = {}
    for row in rows:
        by_speaker.setdefault(row["display_name"], []).append(row)

    def fix_anchor(match: re.Match[str]) -> str:
        name = match.group("name")
        old_ms = _to_ms(match.group("ts"))
        candidates = by_speaker.get(name)
        if not candidates:
            unresolved.append(f"`{name} ({match.group('ts')})` names an unknown speaker")
            return match.group(0)
        best = min(candidates, key=lambda r: abs(r["old_ms"] - old_ms))
        drift = abs(best["old_ms"] - old_ms) // 1000
        if drift > tolerance_s:
            unresolved.append(
                f"`{name} ({match.group('ts')})` matches no turn within "
                f"{tolerance_s}s (nearest is {best['old']}, {drift}s away)"
            )
            return match.group(0)
        if drift:
            warnings.append(
                f"`{name} ({match.group('ts')})` -> {best['new']} "
                f"(was {best['old']}, {drift}s drift)"
            )
        return f"`{name} ({best['new']})`"

    def fix_hold(match: re.Match[str]) -> str:
        return match.group("lead") + _mmss(_interpolate(_to_ms(match.group("ts")), rows))

    text, anchors = _ANCHOR_RE.subn(fix_anchor, text)
    text, holds = _HOLD_RE.subn(fix_hold, text)
    shotlist.write_text(text)
    return anchors + holds, warnings, unresolved


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("script", type=Path)
    ap.add_argument("--manifest", type=Path, default=None,
                    help="Rendered episode_manifest.json for measured durations.")
    ap.add_argument("--gap-ms", type=int, default=350)
    ap.add_argument("--shotlist", type=Path, default=None)
    ap.add_argument("--apply", action="store_true",
                    help="Write the files. Without it, report only.")
    ap.add_argument("--map-out", type=Path, default=None,
                    help="Write the old-to-new mapping as JSON.")
    ap.add_argument("--map-in", type=Path, default=None,
                    help="Remap a shot list from a mapping written by an "
                         "earlier run, after the script itself was rewritten.")
    ap.add_argument("--tolerance-s", type=int, default=30,
                    help="How far an anchor may sit from its turn and still "
                         "count as naming it.")
    args = ap.parse_args()

    if args.map_in:
        rows = json.loads(args.map_in.read_text())
        if not args.shotlist:
            raise SystemExit("--map-in only makes sense with --shotlist")
        count, warnings, unresolved = rewrite_shotlist(
            args.shotlist, rows, args.tolerance_s
        )
        print(f"shotlist: rewrote {count} references")
        for warning in warnings:
            print(f"  moved: {warning}")
        for item in unresolved:
            print(f"  UNRESOLVED: {item}")
        return 1 if unresolved else 0

    rows, total_ms, estimated, rates, fallback = retime(
        args.script, args.manifest, args.gap_ms
    )
    moved = [r for r in rows if r["old"] != r["new"]]

    print(f"turns: {len(rows)}  measured: {len(rows) - estimated}  estimated: {estimated}")
    print("rates: " + ", ".join(
        f"{k}={w:.0f}ms/word+{m:.0f}ms/mark" for k, (w, m) in sorted(rates.items())
    ))
    print(f"runtime: {_mmss(total_ms)}  timestamps changed: {len(moved)}")
    for row in moved[:5]:
        print(f"  turn {row['turn_index']:>3} {row['display_name']}: {row['old']} -> {row['new']}")
    if len(moved) > 5:
        print(f"  ... and {len(moved) - 5} more")

    if args.map_out:
        args.map_out.write_text(json.dumps(rows, indent=1))

    if not args.apply:
        print("dry run; pass --apply to write")
        return 0

    print(f"script: rewrote {rewrite_script(args.script, rows)} headers")
    if args.shotlist:
        count, warnings, unresolved = rewrite_shotlist(
            args.shotlist, rows, args.tolerance_s
        )
        print(f"shotlist: rewrote {count} references")
        for warning in warnings:
            print(f"  moved: {warning}")
        for item in unresolved:
            print(f"  UNRESOLVED: {item}")
        return 1 if unresolved else 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
