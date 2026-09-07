"""Transcribe every rendered segment and compare it to the text it was given.

Duration checks miss the failure that matters. MLX Chatterbox can produce a clip
of exactly the right length that says the wrong thing: a mangled opening word, a
swallowed phrase, a stretch of noise where a name should be. The WAV is valid,
the manifest is consistent, the Ableton set looks correct, and only a listener
notices.

This runs the audio back through Whisper on device and scores what came out
against what went in. It is the only check in the pipeline that verifies content
rather than shape.

Two stages, because accuracy and speed pull against each other. The sweep uses
whisper-tiny.en at roughly a quarter-second per segment, which is fast enough to
run over a whole episode. Anything it flags is worth re-checking with a larger
model before re-synthesizing, since tiny mishears proper nouns on its own.

    python3 tools/verify_render.py outputs/ATO_EP02_local \
        --transcript Architecting_the_operation/podcasts/ATO_EP02_preface.md

    python3 tools/verify_render.py outputs/ATO_EP02_local \
        --transcript ... --model small --turns 1,7,41

Scores are word-sequence similarity after normalization, so they carry ASR error
as well as synthesis error. Read the printed pairs; do not regenerate on the
number alone.
"""
from __future__ import annotations

import argparse
import difflib
import json
import logging
import re
import statistics
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from voice_pipeline.markup import tokenize_markup
from voice_pipeline.parser import parse_transcript

_MODELS = {
    "tiny": "mlx-community/whisper-tiny.en-mlx",
    "small": "mlx-community/whisper-small.en-mlx",
    "medium": "mlx-community/whisper-medium.en-mlx",
}
_PUNCT_RE = re.compile(r"[^a-z0-9\s]")
_WS_RE = re.compile(r"\s+")
# Scripts spell numbers out because the TTS reads digits inconsistently, and
# Whisper writes them back as digits. Dropping both sides keeps a date-heavy
# turn from scoring like a broken one.
_NUMBER_WORDS = {
    "zero","one","two","three","four","five","six","seven","eight","nine","ten",
    "eleven","twelve","thirteen","fourteen","fifteen","sixteen","seventeen",
    "eighteen","nineteen","twenty","thirty","forty","fourty","fifty","sixty",
    "seventy","eighty","ninety","hundred","thousand","million","billion",
    "forties","fifties","sixties","seventies","eighties","nineties",
}


def _normalize(text: str) -> list[str]:
    text = _PUNCT_RE.sub(" ", text.lower())
    words = _WS_RE.sub(" ", text).strip().split()
    return [w for w in words if w not in _NUMBER_WORDS and not w.isdigit()]


def _sounds_alike(want: str, got: str, floor: float = 0.75) -> bool:
    """Whether two spans differ only in how the transcript spelled them.

    Whisper splits compounds the script writes closed ("out-group" comes back as
    "outgroup", "runtimes" as "run times"), and it renders spoken symbols as the
    nearest English word: the script's "Psi" is transcribed "sigh". Comparing
    the two with spaces removed catches those without excusing a span that is
    genuinely different audio.
    """
    a = want.replace(" ", "")
    b = got.replace(" ", "")
    if not a or not b:
        return False
    return difflib.SequenceMatcher(None, a, b).ratio() >= floor


def _score(expected: str, heard: str) -> tuple[float, int]:
    """(overall word similarity, longest run of consecutive words mangled).

    The run length is the signal that matters. A whole-turn ratio barely moves
    when one word out of twenty comes out wrong, and one wrong word is exactly
    what a listener hears as a glitch. Episode 2's opening had "Prejudice plus
    power" render as something no model could read as "prejudice", and it still
    scored 0.909 overall.

    Runs that only differ in spelling are not counted, so a correctly-read line
    does not keep failing on the transcriber's habits.
    """
    a, b = _normalize(expected), _normalize(heard)
    if not a:
        return 1.0, 0
    matcher = difflib.SequenceMatcher(None, a, b)
    worst = 0
    for tag, i1, i2, j1, j2 in matcher.get_opcodes():
        if tag == "equal":
            continue
        if _sounds_alike(" ".join(a[i1:i2]), " ".join(b[j1:j2])):
            continue
        worst = max(worst, i2 - i1, j2 - j1)
    # The ratio ignores spelling differences for the same reason.
    despaced = difflib.SequenceMatcher(None, "".join(a), "".join(b)).ratio()
    return max(matcher.ratio(), despaced), worst


def _diff(expected: str, heard: str, width: int = 3) -> str:
    """The runs that differ, so the failure mode is visible."""
    a, b = _normalize(expected), _normalize(heard)
    out = []
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b).get_opcodes():
        if tag == "equal":
            continue
        want = " ".join(a[i1:i2]) or "-"
        got = " ".join(b[j1:j2]) or "-"
        if _sounds_alike(want, got):
            continue
        if len(want.split()) > 12:
            want = " ".join(want.split()[:12]) + " ..."
        if len(got.split()) > 12:
            got = " ".join(got.split()[:12]) + " ..."
        out.append(f"[{want} -> {got}]")
        if len(out) >= width:
            break
    return " ".join(out)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("episode_dir", type=Path)
    ap.add_argument("--transcript", type=Path, required=True)
    ap.add_argument("--model", choices=sorted(_MODELS), default="tiny")
    ap.add_argument("--threshold", type=float, default=0.90,
                    help="Flag turns whose overall word similarity is below this.")
    ap.add_argument("--max-run", type=int, default=2,
                    help="Flag turns with a mangled run longer than this many "
                         "consecutive words, whatever the overall score.")
    ap.add_argument("--turns", type=str, default=None,
                    help="Comma-separated turn indices to check instead of all.")
    ap.add_argument("--json-out", type=Path, default=None)
    args = ap.parse_args()

    logging.disable(logging.WARNING)
    import mlx_whisper  # imported here so --help works without the model stack

    manifest = json.loads((args.episode_dir / "episode_manifest.json").read_text())
    script_turns = tokenize_markup(parse_transcript(args.transcript))
    by_id = {t.turn_id: t for t in script_turns}
    wanted = (
        {int(x) for x in args.turns.split(",")} if args.turns else None
    )

    # Turn indices reported here are the SCRIPT's, because that is what
    # `--regenerate-turns` consumes. The manifest carries its own indices and
    # the two diverge the moment a turn is added to or removed from the script,
    # which silently regenerates the wrong turns if the manifest's numbering is
    # passed along instead.
    orphans = [t for t in manifest["turns"] if t["turn_id"] not in by_id]
    if orphans:
        print(f"note: {len(orphans)} manifest turn(s) are no longer in the script; "
              f"the manifest is stale. Re-render or relayout to drop them.")

    results = []
    for turn in manifest["turns"]:
        source = by_id.get(turn["turn_id"])
        if source is None:
            continue
        if wanted is not None and source.turn_index not in wanted:
            continue
        expected = " ".join(
            c.text or "" for c in source.markup_chunks if c.kind == "speech"
        ).strip()
        if not expected:
            continue
        heard = " ".join(
            mlx_whisper.transcribe(
                str(args.episode_dir / seg["segment_wav"]),
                path_or_hf_repo=_MODELS[args.model],
                verbose=False,
            )["text"].strip()
            for seg in turn["segments"]
        )
        ratio, worst_run = _score(expected, heard)
        results.append(
            {
                "turn": source.turn_index,
                "manifest_turn": turn["turn_index"],
                "speaker": turn["speaker_id"],
                "turn_id": turn["turn_id"],
                "score": round(ratio, 3),
                "worst_run": worst_run,
                "expected": expected,
                "heard": heard,
            }
        )

    if not results:
        print("no turns matched")
        return 0

    scores = sorted(r["score"] for r in results)
    print(f"{len(results)} turns transcribed with whisper-{args.model}.en | "
          f"median {statistics.median(scores):.3f} | "
          f"p05 {scores[max(0, int(len(scores) * 0.05))]:.3f} | min {scores[0]:.3f}")

    flagged = sorted(
        (r for r in results
         if r["score"] < args.threshold or r["worst_run"] > args.max_run),
        key=lambda r: r["score"],
    )
    for r in flagged:
        print(f"\n  {r['score']:.3f}  run {r['worst_run']:>2}  turn {r['turn']:>3}  "
              f"{r['speaker']}  {r['turn_id']}")
        print(f"      want: {r['expected'][:150]}")
        print(f"      got : {r['heard'][:150]}")
        d = _diff(r["expected"], r["heard"])
        if d:
            print(f"      diff: {d}")

    if args.json_out:
        args.json_out.write_text(json.dumps(results, indent=1))

    if flagged:
        print(f"\n{len(flagged)} turn(s) flagged (score < {args.threshold} or a mangled "
              f"run longer than {args.max_run} words). Re-check with a larger model "
              f"before regenerating:")
        print(f"  --model small --turns {','.join(str(r['turn']) for r in flagged)}")
        return 1
    print(f"verify-render: nothing flagged.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
