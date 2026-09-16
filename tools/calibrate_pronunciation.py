"""Find a spelling that makes the TTS engine say a heteronym the intended way.

Renders the word inside real script sentences, once as spelled and once per
candidate respelling, several takes each, and judges every take with the
phoneme stress check. The engine is stochastic, so one good take proves
nothing; the rate across takes is what counts.

    python3 tools/calibrate_pronunciation.py record --reading default \
        --candidates reckerd,wreck-urd --takes 6
    python3 tools/calibrate_pronunciation.py record --reading verb \
        --candidates ree-CORD --takes 6 --write

Readings use misaki's lexicon keys ("default", "verb", "noun", ...). Run with
no --reading to list the keys a word has. `--write` records the winner in
voice_pipeline/pronunciations.yaml when it beats the plain spelling. Audio is
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

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
sys.path.insert(0, str(Path(__file__).resolve().parent))

import soundfile
import yaml

from stress_check import check_segment
from voice_pipeline.engine import ENGINE_REGISTRY
from voice_pipeline.pronunciation import (
    RESPELLINGS_PATH,
    _heteronym_table,
    find_heteronyms,
)
from voice_pipeline.voices import load_voices

_ROOT = Path(__file__).resolve().parent.parent
_SCRIPTS = _ROOT / "Architecting_the_operation" / "podcasts"
_OUT = _ROOT / "outputs" / "pronunciation_calibration"
_SENTENCE_RE = re.compile(r"[^.!?]*[.!?]")


def _script_sentences(word: str, reading: str, limit: int) -> list[str]:
    """Sentences from the episode scripts where `word` takes `reading`."""
    found: list[str] = []
    for path in sorted(_SCRIPTS.glob("ATO_EP0[1-9]*.md")):
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


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("word")
    ap.add_argument("--reading", default=None)
    ap.add_argument("--candidates", default="",
                    help="Comma-separated respellings to try. The plain word is always tried.")
    ap.add_argument("--takes", type=int, default=5)
    ap.add_argument("--sentences", type=int, default=2)
    ap.add_argument("--speaker", default="emmanuel_theodore")
    ap.add_argument("--voices", type=Path, default=_ROOT / "voice_pipeline" / "voices.omnivoice.yaml")
    ap.add_argument("--write", action="store_true")
    args = ap.parse_args()
    logging.disable(logging.WARNING)

    word = args.word.lower()
    readings = _heteronym_table().get(word)
    if not readings:
        print(f"{word!r} is not a heteronym in misaki's lexicon.")
        return 1
    if args.reading is None:
        for key, phonemes in readings.items():
            print(f"  {key.lower():<8} {phonemes}")
        return 0
    reading = args.reading.lower()

    sentences = _script_sentences(word, reading, args.sentences)
    if not sentences:
        print(f"no script sentence uses {word!r} as {reading!r}.")
        return 1

    import mlx_whisper

    voice = load_voices(args.voices)[args.speaker]
    if voice.engine != "omnivoice":
        print(f"calibration drives OmniVoice; {args.speaker} uses {voice.engine!r}.")
        return 1
    engine = ENGINE_REGISTRY["omnivoice"]("k2-fsa/OmniVoice")
    spellings = [word] + [c.strip() for c in args.candidates.split(",") if c.strip()]
    out_dir = _OUT / f"{word}_{reading}"
    out_dir.mkdir(parents=True, exist_ok=True)

    async def render(text: str, path: Path) -> None:
        audio = await engine.synthesize_chunk(text, voice)
        soundfile.write(str(path), audio, engine.sample_rate)

    rows = []
    for spelling in spellings:
        tally = {"ok": 0, "unclear": 0, "wrong": 0, "garbled": 0, "unaligned": 0}
        for s_index, sentence in enumerate(sentences):
            spoken = _swap(sentence, word, reading, spelling)
            for take in range(args.takes):
                path = out_dir / f"{re.sub(r'[^A-Za-z]+', '_', spelling)}_s{s_index}_t{take}.wav"
                asyncio.run(render(spoken, path))
                result = mlx_whisper.transcribe(
                    str(path), word_timestamps=True, verbose=False,
                    path_or_hf_repo="mlx-community/whisper-small.en-mlx")
                words = [w for seg in result["segments"] for w in seg.get("words", [])]
                for entry in check_segment(path, sentence, words):
                    if entry["word"].lower() == word and entry["reading"] == reading:
                        tally[entry["verdict"]] += 1
        total = sum(tally.values())
        rows.append((spelling, tally["ok"] / total if total else 0.0, tally, total))
        print(f"  {spelling:<16} ok {tally['ok']}/{total}  {tally}", flush=True)

    rows.sort(key=lambda r: (-r[1], r[0] != word))
    best = rows[0]
    print(f"\nbest: {best[0]!r} at {best[1]:.0%} ok. Audio in {out_dir}")
    plain = next(r for r in rows if r[0] == word)
    if args.write:
        if best[0] == word or best[1] <= plain[1]:
            print("plain spelling is as good as any candidate; nothing written.")
            return 0
        data = yaml.safe_load(RESPELLINGS_PATH.read_text()) or {}
        data.setdefault(word, {})[reading] = best[0]
        header = RESPELLINGS_PATH.read_text().split("\n\n")[0] if RESPELLINGS_PATH.exists() else ""
        body = yaml.safe_dump(data, sort_keys=True, allow_unicode=True)
        RESPELLINGS_PATH.write_text((header + "\n\n" if header.startswith("#") else "") + body)
        print(f"wrote {word}.{reading} = {best[0]!r} to {RESPELLINGS_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
