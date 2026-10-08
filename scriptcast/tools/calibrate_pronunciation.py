"""Find a spelling that makes the TTS engine say a heteronym the intended way.

Renders the word inside real script sentences, once as spelled and once per
candidate respelling, several takes each, and judges every take with the
phoneme stress check. The engine is stochastic, so one good take proves
nothing; the rate across takes is what counts.

    scriptcast-calibrate record --reading default \
        --candidates reckerd,wreck-urd --takes 6
    scriptcast-calibrate record --reading verb \
        --candidates ree-CORD --takes 6 --write

Readings use misaki's lexicon keys ("default", "verb", "noun", ...). Run with
no --reading to list the keys a word has. `--write` records the winner in
the project's pronunciations file when it beats the plain spelling. Audio is
kept under outputs/pronunciation_calibration/ for listening.

What spells well: single lowercase words ("reckerd") held up; hyphens and
capitals that split a word ("RECK-erd") made Whisper hear two words and the
engine hesitate.
"""
from __future__ import annotations

import argparse
import asyncio
import logging
import re
import sys
from pathlib import Path

import soundfile
import yaml

from scriptcast.engine import ENGINE_REGISTRY
from scriptcast.project import current
from scriptcast.pronunciation import (
    _heteronym_table,
    find_heteronyms,
    respellings_path,
)
from scriptcast.tool_json import JsonArgumentParser, emit, fail, run_json
from scriptcast.tools.stress_check import check_segment
from scriptcast.voices import load_voices

_SENTENCE_RE = re.compile(r"[^.!?]*[.!?]")


def _script_sentences(word: str, reading: str, limit: int, scripts_dir: Path) -> list[str]:
    """Sentences from the project scripts where `word` takes `reading`."""
    found: list[str] = []
    for path in sorted(scripts_dir.rglob("*.md")):
        for line in path.read_text().splitlines():
            if word not in line.lower() or re.match(r"^[A-Z][A-Za-z ]+ \(\d+:\d{2}\)$", line):
                continue
            for sentence in _SENTENCE_RE.findall(line):
                sentence = sentence.strip()
                if not 6 <= len(sentence.split()) <= 30:
                    continue
                if any(r.word.lower() == word and r.key == reading
                       for r in find_heteronyms(sentence)):
                    found.append(sentence)
                    if len(found) >= limit:
                        return found
    return found


def _swap(sentence: str, word: str, reading: str, spelling: str) -> str:
    out, cursor = [], 0
    for r in find_heteronyms(sentence):
        if r.word.lower() != word or r.key != reading:
            continue
        piece = spelling[:1].upper() + spelling[1:] if r.word[:1].isupper() else spelling
        out += [sentence[cursor:r.start], piece]
        cursor = r.end
    return "".join(out) + sentence[cursor:]


def _write_respelling(word: str, reading: str, spelling: str, json_mode: bool) -> Path:
    out_path = respellings_path()
    if not json_mode:
        data: dict = {}
        header = ""
        if out_path.exists():
            data = yaml.safe_load(out_path.read_text()) or {}
            header = out_path.read_text().split("\n\n")[0]
        data.setdefault(word, {})[reading] = spelling
        body = yaml.safe_dump(data, sort_keys=True, allow_unicode=True)
        out_path.write_text((header + "\n\n" if header.startswith("#") else "") + body)
        return out_path
    from scriptcast.config_edit import apply_ops

    text = out_path.read_text(encoding="utf-8") if out_path.exists() else ""
    loaded = yaml.safe_load(text) if text.strip() else None
    loaded = loaded or {}
    if not isinstance(loaded, dict):
        raise SystemExit("pronunciations file must be a mapping")
    current_readings = loaded.get(word) or {}
    if not isinstance(current_readings, dict):
        current_readings = {}
    readings = {str(key): str(value) for key, value in current_readings.items()}
    readings[reading] = spelling
    out_path.parent.mkdir(parents=True, exist_ok=True)
    updated = apply_ops({"pronunciations": text}, [{
        "op": "set_entry",
        "file": "pronunciations",
        "word": word,
        "readings": readings,
    }])
    out_path.write_text(updated["pronunciations"], encoding="utf-8")
    return out_path


def main() -> int:
    ap = JsonArgumentParser(json_flags=("--json",))
    ap.add_argument("word")
    ap.add_argument("--reading", default=None)
    ap.add_argument("--candidates", default="",
                    help="Comma-separated respellings to try. The plain word is always tried.")
    ap.add_argument("--takes", type=int, default=5)
    ap.add_argument("--sentences", type=int, default=2)
    ap.add_argument("--speaker", required=True,
                    help="Speaker id from the voices file; must use the omnivoice engine.")
    ap.add_argument("--voices", type=Path, default=None,
                    help="Voices YAML. Defaults to the project's voices file.")
    ap.add_argument("--scripts", type=Path, default=None,
                    help="Directory of markdown scripts to mine for sentences. "
                         "Defaults to the project's scripts directory.")
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--json", action="store_true",
                    help="Print one JSON object. Requires --keep so the takes stay on disk.")
    ap.add_argument("--keep", type=Path, default=None,
                    help="With --json, directory that keeps the rendered takes.")
    args = ap.parse_args()
    logging.disable(logging.WARNING)
    return run_json(bool(args.json), lambda: _run(args))


def _run(args: argparse.Namespace) -> int:

    project = current()
    scripts_dir = args.scripts or project.scripts

    if args.json and (args.reading is None or args.keep is None):
        return fail("--json requires --reading and --keep")

    word = args.word.lower()
    readings = _heteronym_table().get(word)
    if not readings:
        message = f"{word!r} is not a heteronym in misaki's lexicon."
        if args.json:
            return fail(message)
        print(message)
        return 1
    if args.reading is None:
        for key, phonemes in readings.items():
            print(f"  {key.lower():<8} {phonemes}")
        return 0
    reading = args.reading.lower()

    sentences = _script_sentences(word, reading, args.sentences, scripts_dir)
    if not sentences:
        message = f"no script sentence uses {word!r} as {reading!r}."
        if args.json:
            return fail(message)
        print(message)
        return 1

    import mlx_whisper

    voice = load_voices(args.voices or project.voices)[args.speaker]
    if voice.engine != "omnivoice":
        message = f"calibration drives OmniVoice; {args.speaker} uses {voice.engine!r}."
        if args.json:
            return fail(message)
        print(message)
        return 1
    engine = ENGINE_REGISTRY["omnivoice"]("k2-fsa/OmniVoice")
    spellings = [word] + [c.strip() for c in args.candidates.split(",") if c.strip()]
    if args.json:
        out_dir = args.keep
    else:
        out_dir = project.outputs / "pronunciation_calibration" / f"{word}_{reading}"
    out_dir.mkdir(parents=True, exist_ok=True)
    log = sys.stderr if args.json else sys.stdout

    async def render(text: str, path: Path) -> None:
        audio = await engine.synthesize_chunk(text, voice)
        soundfile.write(str(path), audio, engine.sample_rate)

    rows = []
    take_rows: dict[str, list[dict]] = {}
    for spelling in spellings:
        tally = {"ok": 0, "unclear": 0, "wrong": 0, "garbled": 0, "unaligned": 0}
        takes = []
        for s_index, sentence in enumerate(sentences):
            spoken = _swap(sentence, word, reading, spelling)
            for take in range(args.takes):
                path = out_dir / f"{re.sub(r'[^A-Za-z]+', '_', spelling)}_s{s_index}_t{take}.wav"
                asyncio.run(render(spoken, path))
                result = mlx_whisper.transcribe(
                    str(path), word_timestamps=True, verbose=False,
                    path_or_hf_repo="mlx-community/whisper-small.en-mlx")
                words = [w for seg in result["segments"] for w in seg.get("words", [])]
                matched = None
                for entry in check_segment(path, sentence, words):
                    if entry["word"].lower() == word and entry["reading"] == reading:
                        tally[entry["verdict"]] += 1
                        if matched is None:
                            matched = entry["verdict"] == "ok"
                takes.append({
                    "wav": str(path.resolve()),
                    "sentence": spoken,
                    "ok": bool(matched),
                })
        total = sum(tally.values())
        rows.append((spelling, tally["ok"] / total if total else 0.0, tally, total))
        take_rows[spelling] = takes
        print(f"  {spelling:<16} ok {tally['ok']}/{total}  {tally}", file=log, flush=True)

    rows.sort(key=lambda r: (-r[1], r[0] != word))
    best = rows[0]
    print(f"\nbest: {best[0]!r} at {best[1]:.0%} ok. Audio in {out_dir}", file=log)
    plain = next(r for r in rows if r[0] == word)
    written = False
    if args.write:
        if best[0] == word or best[1] <= plain[1]:
            print("plain spelling is as good as any candidate; nothing written.", file=log)
        else:
            out_path = _write_respelling(word, reading, best[0], bool(args.json))
            written = True
            print(f"wrote {word}.{reading} = {best[0]!r} to {out_path}", file=log)
    if args.json:
        emit({
            "word": word,
            "reading": reading,
            "candidates": [
                {
                    "spelling": spelling,
                    "score": score,
                    "takes": take_rows[spelling],
                }
                for spelling, score, _tally, _total in rows
            ],
            "best": best[0],
            "written": written,
        })
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
