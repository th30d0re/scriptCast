"""Flag turns whose rendered audio does not match the text it was given.

MLX Chatterbox occasionally degenerates on a turn: it finishes the sentence and
keeps going, babbling or repeating until it hits the generation limit. The clip
is valid audio, the manifest is consistent, and the Ableton set looks correct,
so nothing downstream notices. Only a listener does.

The signal is the duration ratio. `voice_pipeline/speaker_rates.json` holds each
speaker's measured milliseconds per word and per punctuation mark, fitted by
`tools/retime_script.py`, so a turn that runs far longer than its own text
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

    python3 tools/check_render_outliers.py outputs/ATO_EP02_local \
        --transcript Architecting_the_operation/podcasts/ATO_EP02_preface.md
"""
from __future__ import annotations

import argparse
import json
import logging
import re
import statistics
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from voice_pipeline.markup import tokenize_markup
from voice_pipeline.parser import parse_transcript

_RATES_PATH = Path(__file__).resolve().parent.parent / "voice_pipeline" / "speaker_rates.json"
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
    if not _RATES_PATH.exists():
        return {}
    raw = json.loads(_RATES_PATH.read_text())
    out: dict[str, tuple[float, float]] = {}
    for speaker, value in raw.items():
        if isinstance(value, dict):
            out[speaker] = (float(value["ms_per_word"]), float(value.get("ms_per_mark", 0.0)))
        else:
            out[speaker] = (1000.0 / float(value), 0.0)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("episode_dir", type=Path)
    ap.add_argument("--transcript", type=Path, required=True)
    ap.add_argument("--min-words", type=int, default=20,
                    help="Turns shorter than this are exempt from the ratio test.")
    ap.add_argument("--ratio", type=float, default=1.5,
                    help="Flag a turn running this many times its predicted length.")
    args = ap.parse_args()

    logging.disable(logging.WARNING)
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
        ranked.append((actual / (words * per_word + marks * per_mark),
                       {"turn": turn["turn_index"], "speaker": turn["speaker_id"],
                        "id": turn["turn_id"], "actual": actual, "words": words,
                        "text": (source.clean_text or "")[:60]}))

    ranked.sort(key=lambda r: -r[0])
    if ranked:
        values = sorted(r[0] for r in ranked)
        print(f"{len(ranked)} turns >= {args.min_words} words | "
              f"median {statistics.median(values):.2f} | "
              f"p95 {values[min(len(values) - 1, int(len(values) * 0.95))]:.2f} | "
              f"max {values[-1]:.2f}")

    long_turns = [(r, d) for r, d in ranked if r >= args.ratio]
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
