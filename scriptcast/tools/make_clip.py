"""Cut an archival excerpt by naming its first and last words.

    scriptcast-clip ~/Downloads/speech_1963.m4a --id speech_1963 \
        --from "the first words of the excerpt" --to "the last words of it" \
        --citation "Author, A. (Year). Title. Publisher." \
        --url https://example.com/... --content-note "Contains strong language."

Transcribes the source with Whisper word timings, finds the two phrases,
copies the source into the project's clip sources directory, writes the
cut into the clip registry, and prints the script turn to paste. Read the
printed transcript against the recording and correct it by hand: it becomes the
caption and the text verification checks the clip against.
"""
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path

import soundfile
import yaml

from scriptcast.project import current
from scriptcast.tool_json import JsonArgumentParser, emit, fail, flag_present, run_json

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


def _words(result: dict) -> list[dict]:
    return [w for seg in result["segments"] for w in seg.get("words", []) if _norm(w["word"])]


def _duration(path: Path, words: list[dict]) -> float:
    try:
        return float(soundfile.info(str(path)).duration)
    except Exception:
        pass
    probe = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration",
         "-of", "default=noprint_wrappers=1:nokey=1", str(path)],
        capture_output=True, text=True,
    )
    if probe.returncode == 0 and probe.stdout.strip():
        return float(probe.stdout.strip())
    if words:
        return float(words[-1]["end"])
    return 0.0


def _transcribe(source: Path, model: str) -> dict:
    import mlx_whisper

    return mlx_whisper.transcribe(
        str(source), word_timestamps=True, verbose=False, path_or_hf_repo=_MODELS[model],
    )


def _write_registry(registry_path: Path, data: dict, json_mode: bool, clip_id: str, entry: dict) -> None:
    registry_path.parent.mkdir(parents=True, exist_ok=True)
    if not json_mode:
        registry_path.write_text(yaml.safe_dump(data, sort_keys=True, allow_unicode=True))
        return
    from scriptcast.config_edit import apply_ops

    text = registry_path.read_text(encoding="utf-8") if registry_path.exists() else ""
    updated = apply_ops({"clips": text}, [{
        "op": "set_clip",
        "file": "clips",
        "id": clip_id,
        "fields": entry,
    }])
    registry_path.write_text(updated["clips"], encoding="utf-8")


def main() -> int:
    words_mode = flag_present("--words-json")
    span_mode = flag_present("--start") or flag_present("--end")
    ap = JsonArgumentParser(json_flags=("--json", "--words-json"))
    ap.add_argument("source", type=Path)
    ap.add_argument("--id", required=not words_mode)
    ap.add_argument("--from", dest="start_phrase", required=not words_mode and not span_mode)
    ap.add_argument("--to", dest="end_phrase", required=not words_mode and not span_mode)
    ap.add_argument("--speaker", default=None, help="Display name for the script turn.")
    ap.add_argument("--citation", default="")
    ap.add_argument("--url", default="")
    ap.add_argument("--content-note", default="")
    ap.add_argument("--pad", type=float, default=0.12)
    ap.add_argument("--model", choices=sorted(_MODELS), default="medium")
    ap.add_argument("--words-json", action="store_true",
                    help="Print word timings as one JSON object and do not write the registry.")
    ap.add_argument("--json", action="store_true",
                    help="Register the clip and print one JSON object.")
    ap.add_argument("--replace", action="store_true",
                    help="With --json, replace an id that already exists.")
    ap.add_argument("--start", type=float, default=None,
                    help="Start in seconds. With --end, an alternative to --from and --to.")
    ap.add_argument("--end", type=float, default=None,
                    help="End in seconds. With --start, an alternative to --from and --to.")
    args = ap.parse_args()
    json_mode = bool(args.words_json or args.json)
    return run_json(json_mode, lambda: _run(args))


def _run(args: argparse.Namespace) -> int:
    if args.words_json and args.json:
        return fail("--words-json and --json are separate modes")
    if args.words_json:
        result = _transcribe(args.source, args.model)
        words = _words(result)
        payload = {
            "source": str(args.source.resolve()),
            "duration": round(_duration(args.source, words), 2),
            "words": [
                {
                    "i": index,
                    "word": word["word"].strip(),
                    "start": round(float(word["start"]), 2),
                    "end": round(float(word["end"]), 2),
                }
                for index, word in enumerate(words)
            ],
        }
        emit(payload)
        return 0

    explicit = args.start is not None or args.end is not None
    phrased = bool(args.start_phrase or args.end_phrase)
    if explicit and phrased:
        message = "pass either --from/--to or --start/--end, not both"
        if args.json:
            return fail(message)
        raise SystemExit(message)
    if explicit and (args.start is None or args.end is None):
        message = "--start and --end are both required"
        if args.json:
            return fail(message)
        raise SystemExit(message)
    if explicit and args.end <= args.start:
        message = "--end must be greater than --start"
        if args.json:
            return fail(message)
        raise SystemExit(message)

    result = _transcribe(args.source, args.model)
    words = _words(result)
    if explicit:
        start = round(float(args.start), 2)
        end = round(float(args.end), 2)
        picked = [w for w in words if float(w["end"]) > start and float(w["start"]) < end]
        transcript = " ".join(w["word"].strip() for w in picked)
    else:
        first, _ = _find(words, args.start_phrase)
        _, last = _find(words, args.end_phrase, after=first)
        start = max(0.0, words[first]["start"] - args.pad)
        end = words[last]["end"] + args.pad
        transcript = " ".join(w["word"].strip() for w in words[first : last + 1])

    project = current()
    sources_dir = project.clip_sources
    registry_path = project.clips

    sources_dir.mkdir(parents=True, exist_ok=True)
    stored = sources_dir / f"{args.id}{args.source.suffix.lower()}"
    if args.source.resolve() != stored.resolve():
        shutil.copy2(args.source, stored)

    data = yaml.safe_load(registry_path.read_text()) if registry_path.exists() else None
    data = data or {"clips": {}}
    clips = data.setdefault("clips", {})
    if args.json and args.id in clips and not args.replace:
        return fail(f"clip {args.id!r} already exists; pass --replace to overwrite it")
    entry = {
        "source": str(stored.relative_to(project.root)),
        "start": round(float(start), 2),
        "end": round(float(end), 2),
        "citation": args.citation,
        "origin_url": args.url,
        "content_note": args.content_note,
    }
    clips[args.id] = entry
    _write_registry(registry_path, data, args.json, args.id, entry)

    speaker = args.speaker or "Archive"
    turn = f"{speaker} (00:00)\n[clip:{args.id}] {transcript}"
    if args.json:
        print(
            f"registered {args.id}: {start:.2f}s to {end:.2f}s ({end - start:.1f}s) in {registry_path}",
            file=sys.stderr,
        )
        emit({
            "id": args.id,
            "registry": str(registry_path.resolve()),
            "entry": entry,
            "turn": turn,
        })
        return 0

    print(f"registered {args.id}: {start:.2f}s to {end:.2f}s ({end - start:.1f}s) in {registry_path}")
    print("\nScript turn (check the transcript against the recording):\n")
    print(turn)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
