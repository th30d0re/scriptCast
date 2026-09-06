"""Recompute an episode script's timestamps from measured audio.

Header timestamps in `Architecting_the_operation/podcasts/*.md` are source
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

    python3 tools/retime_script.py Architecting_the_operation/podcasts/ATO_EP01_authors_preface.md \
        --manifest outputs/ATO_EP01_local/episode_manifest.json \
        --shotlist Architecting_the_operation/video/ATO_EP01_shotlist.md \
        --apply
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from voice_pipeline.markup import tokenize_markup
from voice_pipeline.parser import parse_transcript

_HEADER_RE = re.compile(r"^(?P<name>.+?) \((?P<ts>\d{1,2}:\d{2})\)\s*$")
_ANCHOR_RE = re.compile(r"`(?P<name>[^`(]+?) \((?P<ts>\d{1,2}:\d{2})\)`")
_HOLD_RE = re.compile(r"(?P<lead>\*\*Hold:\*\* through )(?P<ts>\d{1,2}:\d{2})")
_DEFAULT_WPS = 2.55
_RATES_PATH = Path(__file__).resolve().parent.parent / "voice_pipeline" / "speaker_rates.json"


def _mmss(ms: int) -> str:
    total = ms // 1000
    return f"{total // 60:02d}:{total % 60:02d}"


def _to_ms(mmss: str) -> int:
    m, s = mmss.split(":")
    return int(m) * 60_000 + int(s) * 1_000


def _turn_speech(turn) -> tuple[int, int]:
    """(word count, silence ms) for one tokenized turn."""
    words = 0
    silence = 0
    for chunk in turn.markup_chunks:
        if chunk.kind == "speech":
            words += len((chunk.text or "").split())
        elif chunk.kind == "silence":
            silence += chunk.duration_ms or 0
    return words, silence


def _load_measured(manifest_path: Path | None) -> dict[str, int]:
    if manifest_path is None or not manifest_path.exists():
        return {}
    manifest = json.loads(manifest_path.read_text())
    return {
        t["turn_id"]: int(t["end_ms"]) - int(t["start_ms"])
        for t in manifest.get("turns", [])
        if t.get("end_ms") is not None and t.get("start_ms") is not None
    }


def _load_known_rates() -> dict[str, float]:
    """Speaker rates measured on earlier episodes."""
    if not _RATES_PATH.exists():
        return {}
    try:
        return {k: float(v) for k, v in json.loads(_RATES_PATH.read_text()).items()}
    except (ValueError, TypeError):
        return {}


def _save_known_rates(rates: dict[str, float]) -> None:
    """Carry this episode's measured rates forward.

    A script with no audio yet has nothing to calibrate against, and the same
    three voices read every episode. Persisting what the last render measured
    means a brand-new script gets timestamps within a second of where its audio
    will land, instead of a generic words-per-second guess.
    """
    known = _load_known_rates()
    known.update({k: round(v, 4) for k, v in rates.items()})
    _RATES_PATH.write_text(json.dumps(known, indent=1, sort_keys=True) + "\n")


def _calibrate(turns, measured: dict[str, int]) -> dict[str, float]:
    """Words per second per speaker, from turns whose audio exists."""
    totals: dict[str, list[int]] = {}
    for turn in turns:
        if turn.turn_id not in measured:
            continue
        words, silence = _turn_speech(turn)
        speech_ms = measured[turn.turn_id] - silence
        if words < 5 or speech_ms <= 0:
            continue
        bucket = totals.setdefault(turn.speaker_id, [0, 0])
        bucket[0] += words
        bucket[1] += speech_ms
    return {sid: w / (ms / 1000.0) for sid, (w, ms) in totals.items() if ms}


def retime(script: Path, manifest: Path | None, gap_ms: int):
    turns = tokenize_markup(parse_transcript(script))
    measured = _load_measured(manifest)
    rates = _calibrate(turns, measured)
    if rates:
        _save_known_rates(rates)
    # A speaker with no audio in this episode still has a rate from the last
    # one, since the same voices read every episode.
    rates = {**_load_known_rates(), **rates}
    fallback = (
        sum(rates.values()) / len(rates) if rates else _DEFAULT_WPS
    )

    cursor = 0
    rows = []
    estimated = 0
    for turn in turns:
        words, silence = _turn_speech(turn)
        if turn.turn_id in measured:
            duration = measured[turn.turn_id]
        else:
            estimated += 1
            wps = rates.get(turn.speaker_id, fallback)
            duration = int(round(words / wps * 1000)) + silence
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
    print("rate wps: " + ", ".join(f"{k}={v:.2f}" for k, v in sorted(rates.items()))
          + f"  fallback={fallback:.2f}")
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
