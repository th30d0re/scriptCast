"""Cut an archival excerpt by naming its first and last words.

    python3 tools/make_clip.py ~/Downloads/atwater_1981.m4a --id atwater_1981 \
        --from "you start out in 1954" --to "a byproduct of them" \
        --citation "Perlstein, R. (2012). Exclusive: Lee Atwater's Infamous 1981 Interview on the Southern Strategy. The Nation." \
        --url https://www.thenation.com/... --content-note "Contains a racial slur."

Transcribes the source with Whisper word timings, finds the two phrases,
copies the source into Architecting_the_operation/archive/sources/, writes the
cut into the clip registry, and prints the script turn to paste. Read the
printed transcript against the recording and correct it by hand: it becomes the
caption and the text verification checks the clip against.
"""
from __future__ import annotations

import argparse
import re
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import yaml

from voice_pipeline.archive import REGISTRY_PATH, ROOT

_SOURCES = ROOT / "Architecting_the_operation" / "archive" / "sources"
_MODELS = {
    "small": "mlx-community/whisper-small.en-mlx",
    "medium": "mlx-community/whisper-medium.en-mlx",
}


def _norm(word: str) -> str:
    return re.sub(r"[^a-z0-9']", "", word.lower())


def _find(words: list[dict], phrase: str, after: int = 0) -> tuple[int, int]:
    target = [_norm(w) for w in phrase.split() if _norm(w)]
    heard = [_norm(w["word"]) for w in words]
    for i in range(after, len(heard) - len(target) + 1):
        if heard[i : i + len(target)] == target:
            return i, i + len(target) - 1
    raise SystemExit(f"phrase not found in the transcript: {phrase!r}")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("source", type=Path)
    ap.add_argument("--id", required=True)
    ap.add_argument("--from", dest="start_phrase", required=True)
    ap.add_argument("--to", dest="end_phrase", required=True)
    ap.add_argument("--speaker", default=None, help="Display name for the script turn.")
    ap.add_argument("--citation", default="")
    ap.add_argument("--url", default="")
    ap.add_argument("--content-note", default="")
    ap.add_argument("--pad", type=float, default=0.12)
    ap.add_argument("--model", choices=sorted(_MODELS), default="medium")
    args = ap.parse_args()

    import mlx_whisper

    result = mlx_whisper.transcribe(str(args.source), word_timestamps=True, verbose=False,
                                    path_or_hf_repo=_MODELS[args.model])
    words = [w for seg in result["segments"] for w in seg.get("words", []) if _norm(w["word"])]
    first, _ = _find(words, args.start_phrase)
    _, last = _find(words, args.end_phrase, after=first)
    start = max(0.0, words[first]["start"] - args.pad)
    end = words[last]["end"] + args.pad
    transcript = " ".join(w["word"].strip() for w in words[first : last + 1])

    _SOURCES.mkdir(parents=True, exist_ok=True)
    stored = _SOURCES / f"{args.id}{args.source.suffix.lower()}"
    if args.source.resolve() != stored.resolve():
        shutil.copy2(args.source, stored)

    data = yaml.safe_load(REGISTRY_PATH.read_text()) if REGISTRY_PATH.exists() else None
    data = data or {"clips": {}}
    data.setdefault("clips", {})[args.id] = {
        "source": str(stored.relative_to(ROOT)),
        "start": round(float(start), 2),
        "end": round(float(end), 2),
        "citation": args.citation,
        "origin_url": args.url,
        "content_note": args.content_note,
    }
    REGISTRY_PATH.parent.mkdir(parents=True, exist_ok=True)
    REGISTRY_PATH.write_text(yaml.safe_dump(data, sort_keys=True, allow_unicode=True))

    print(f"registered {args.id}: {start:.2f}s to {end:.2f}s ({end - start:.1f}s) in {REGISTRY_PATH}")
    print("\nScript turn (check the transcript against the recording):\n")
    print(f"{args.speaker or 'Archive'} (00:00)\n[clip:{args.id}] {transcript}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
