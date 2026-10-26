"""Flag turns whose rendered audio does not match the text it was given.

MLX Chatterbox occasionally degenerates on a turn: it finishes the sentence and
keeps going, babbling or repeating until it hits the generation limit. The clip
is valid audio, the manifest is consistent, and the Ableton set looks correct,
so nothing downstream notices. Only a listener does.

The signal is the duration ratio. The project's speaker-rates file holds each
speaker's measured milliseconds per word and per punctuation mark, fitted by
`scriptcast-retime`, so a turn that runs far longer than its own text
predicts is suspect. Short turns carry fixed breath and pacing overhead that does
not scale with length, so only turns above `--min-words` are ranked.

Punctuation is in the model because it costs real time. A comma-heavy list is
read with a pause at every mark, and a words-only model calls that turn too long
when it is simply being read correctly.

A note on a signal that does NOT work, so nobody adds it back. `speech_duration_ms`
equalling `duration_ms` looks like "the clip was still talking when generation
stopped", but `_measure_speech_duration` adds `--tail-ms` (150 by default) and
clamps to the clip length. The two fields therefore match on any clip whose speech
ends within 150ms of the file end, which is most of them.

    scriptcast-outliers outputs/<episode_id> \
        --transcript scripts/<episode>.md
"""
from __future__ import annotations

import argparse
import json
import logging
import re
import statistics
import sys
from pathlib import Path

from scriptcast.markup import tokenize_markup
from scriptcast.parser import parse_transcript
from scriptcast.project import current
from scriptcast.tool_json import JsonArgumentParser, emit, run_json

_MARK_RE = re.compile(r"[.,;:]")
_DEFAULT = (392.0, 0.0)  # ms per word, ms per mark


def _speech(turn) -> tuple[int, int, int]:
    words = marks = silence = 0
    for chunk in turn.markup_chunks:
        if chunk.kind == "speech":
            text = chunk.text or ""
            words += len(text.split())
            marks += len(_MARK_RE.findall(text))
        elif chunk.kind == "silence":
            silence += chunk.duration_ms or 0
    return words, marks, silence


def _load_rates() -> dict[str, tuple[float, float]]:
    rates_path = current().speaker_rates
    if not rates_path.exists():
        return {}
    raw = json.loads(rates_path.read_text())
    out: dict[str, tuple[float, float]] = {}
    for speaker, value in raw.items():
        if isinstance(value, dict):
            out[speaker] = (float(value["ms_per_word"]), float(value.get("ms_per_mark", 0.0)))
        else:
            out[speaker] = (1000.0 / float(value), 0.0)
    return out


def main() -> int:
    ap = JsonArgumentParser(json_flags=("--json",))
    ap.add_argument("episode_dir", type=Path)
    ap.add_argument("--transcript", type=Path, required=True)
    ap.add_argument("--min-words", type=int, default=20,
                    help="Turns shorter than this are exempt from the ratio test.")
    ap.add_argument("--ratio", type=float, default=1.5,
                    help="Flag a turn running this many times its predicted length.")
    ap.add_argument("--json", action="store_true",
                    help="Print one JSON object of flagged turns and exit 0.")
    args = ap.parse_args()

    logging.disable(logging.WARNING)
    return run_json(bool(args.json), lambda: _run(args))


def _run(args: argparse.Namespace) -> int:
    manifest = json.loads((args.episode_dir / "episode_manifest.json").read_text())
    by_id = {t.turn_id: t for t in tokenize_markup(parse_transcript(args.transcript))}
    rates = _load_rates()

    ranked: list[tuple[float, dict]] = []

    for turn in manifest["turns"]:
        source = by_id.get(turn["turn_id"])
        if source is None:
            continue
        words, marks, silence = _speech(source)
        actual = turn["end_ms"] - turn["start_ms"] - silence

        if words < args.min_words or actual <= 0:
            continue
        per_word, per_mark = rates.get(turn["speaker_id"], _DEFAULT)
        predicted = words * per_word + marks * per_mark
        ranked.append((actual / predicted,
                       {"turn": turn["turn_index"], "speaker": turn["speaker_id"],
                        "id": turn["turn_id"], "actual": actual, "words": words,
                        "predicted": predicted,
                        "text": (source.clean_text or "")[:60]}))

    ranked.sort(key=lambda r: -r[0])
    log = sys.stderr if args.json else sys.stdout
    if ranked and not args.json:
        values = sorted(r[0] for r in ranked)
        print(f"{len(ranked)} turns >= {args.min_words} words | "
              f"median {statistics.median(values):.2f} | "
              f"p95 {values[min(len(values) - 1, int(len(values) * 0.95))]:.2f} | "
              f"max {values[-1]:.2f}")

    long_turns = [(r, d) for r, d in ranked if r >= args.ratio]
    if args.json:
        if ranked:
            values = sorted(r[0] for r in ranked)
            print(f"{len(ranked)} turns >= {args.min_words} words | "
                  f"median {statistics.median(values):.2f} | "
                  f"p95 {values[min(len(values) - 1, int(len(values) * 0.95))]:.2f} | "
                  f"max {values[-1]:.2f}", file=log)
        for ratio, d in long_turns:
            print(f"  LONG  turn {d['turn']:>3} {d['speaker']:<18} {d['actual']/1000:6.1f}s "
                  f"for {d['words']:>3} words  ratio {ratio:.2f}  {d['id']}  {d['text']}", file=log)
        emit({
            "outliers": [
                {
                    "turn_index": d["turn"],
                    "turn_id": d["id"],
                    "speaker_id": d["speaker"],
                    "predicted_s": round(d["predicted"] / 1000, 2),
                    "actual_s": round(d["actual"] / 1000, 2),
                    "ratio": round(ratio, 2),
                    "reason": (
                        f"ratio {ratio:.2f} for {d['words']} words: {d['text']}"
                    ),
                }
                for ratio, d in long_turns
            ],
        })
        return 0
    for ratio, d in long_turns:
        print(f"  LONG  turn {d['turn']:>3} {d['speaker']:<18} {d['actual']/1000:6.1f}s "
              f"for {d['words']:>3} words  ratio {ratio:.2f}  {d['id']}  {d['text']}")
    flagged = {d["turn"] for _, d in long_turns}
    if flagged:
        print(f"\n{len(flagged)} turn(s) to review. Re-synthesize with:")
        print(f"  --regenerate-turns {','.join(str(t) for t in sorted(flagged))}")
        return 1
    print("check-render-outliers: nothing flagged.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
