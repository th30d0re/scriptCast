"""Check that rendered heteronyms were spoken in the reading the script needs.

Whisper cannot do this. It writes "record" whether it heard REC-ord or re-CORD,
so a wrong stress scores a perfect match. Stress changes the vowels, though: an
unstressed syllable shrinks toward "uh" and a stressed one keeps its full vowel.
REC-ord is /ɹɛkəɹd/ and re-CORD is /ɹəkɔɹd/. A phoneme recognizer hears that
difference, so each heteronym is cut out of the audio, transcribed as phonemes,
and compared with the reading misaki expects and with the readings it does not.

    scriptcast-stress outputs/<episode_id>/segments/x.wav \
        --text "It includes one specific historical record."

`tools/verify_render.py` calls `check_segment` for every turn that contains a
heteronym. The recognizer is facebook/wav2vec2-lv-60-espeak-cv-ft, about 1.2 GB,
downloaded on first use.
"""
from __future__ import annotations

import argparse
import difflib
import json
import re
from functools import lru_cache
from pathlib import Path

import numpy
import soundfile

from scriptcast.pronunciation import HeteronymReading, find_heteronyms

MODEL_ID = "facebook/wav2vec2-lv-60-espeak-cv-ft"
_RATE = 16000
_PAD_S = 0.2
# How much closer the audio must sit to one reading before it counts. Below
# this the verdict is "unclear" and nothing is flagged.
MARGIN = 0.6
# A heard word this far from the expected reading, and no closer to another, is
# not a stress error but a mangled word. Calibrated on hand-labelled renders:
# clean takes sat at 1.3 or below, the garbled "historical record" takes a
# listener heard sat above 3.
GARBLED = 2.5

# The recognizer covers dozens of languages. On a poor cut it reaches for
# Mandarin tones and German vowels; confining it to American English symbols
# keeps its best guess inside the sounds that could have been said.
_ENGLISH = {
    "p", "b", "t", "d", "k", "ɡ", "f", "v", "θ", "ð", "s", "z", "ʃ", "ʒ", "h", "m",
    "n", "ŋ", "l", "ɹ", "w", "j", "tʃ", "dʒ", "ɾ", "ʔ", "əl", "i", "iː", "ɪ", "ɛ",
    "æ", "ɑ", "ɑː", "ɔ", "ɔː", "ə", "ɚ", "ɜː", "ʊ", "u", "uː", "ʌ", "eɪ", "aɪ", "aʊ",
    "oʊ", "ɔɪ", "ɐ", "ᵻ", "oː", "ɑːɹ", "ɔːɹ", "oːɹ", "ɛɹ", "ɪɹ", "ʊɹ", "aɪɚ", "aɪə",
    "iə", "a", "e", "o", "<pad>",
}

_DIPHTHONGS = ("eɪ", "aɪ", "oʊ", "aʊ", "ɔɪ", "dʒ", "tʃ")
_MISAKI = {"A": "eɪ", "I": "aɪ", "O": "oʊ", "W": "aʊ", "Y": "ɔɪ", "ʤ": "dʒ", "ʧ": "tʃ"}
_FOLD = {
    "ɚ": ["ə", "ɹ"], "ɝ": ["ə", "ɹ"], "ɜ": ["ə"], "ɐ": ["ə"], "ᵊ": ["ə"],
    "ᵻ": ["ɪ"], "ɒ": ["ɑ"], "ɾ": ["t"], "ɡ": ["g"], "ɫ": ["l"], "e": ["eɪ"],
    "o": ["oʊ"], "r": ["ɹ"],
}
_VOWELS = {"i", "ɪ", "ɛ", "æ", "ə", "ʌ", "ɑ", "ɔ", "ʊ", "u", "eɪ", "aɪ", "oʊ", "aʊ", "ɔɪ", "a"}
_NEAR = {frozenset(p) for p in [("i", "ɪ"), ("ɪ", "ə"), ("ɛ", "æ"), ("ɑ", "ɔ"),
                                 ("oʊ", "ɔ"), ("ʊ", "u"), ("a", "ɑ"), ("a", "æ"),
                                 ("i", "ə")]}
# "ʌ" and "ə" are the same vowel stressed and unstressed, so they stay distinct:
# folding them together erases SUB-ject against sub-JECT.
# Word-final stops are often inaudible or swallowed by the next word, and the
# recognizer drops them. Missing one says nothing about stress.
_WEAK_FINAL = {"d", "t"}


def _units(text: str) -> list[str]:
    """Split a phoneme string into symbols, keeping diphthongs whole."""
    out, i = [], 0
    while i < len(text):
        pair = text[i : i + 2]
        if pair in _DIPHTHONGS:
            out.append(pair)
            i += 2
        else:
            out.append(text[i])
            i += 1
    return out


def normalize(symbols: list[str]) -> list[str]:
    folded: list[str] = []
    for symbol in symbols:
        folded.extend(_FOLD.get(symbol, [symbol]))
    return [s for s in folded if s.strip()]


def from_misaki(phonemes: str) -> list[str]:
    text = "".join(_MISAKI.get(c, c) for c in phonemes if c not in "ˈˌ")
    return normalize(_units(text))


def from_espeak(tokens: str) -> list[str]:
    units: list[str] = []
    for token in tokens.replace("ː", "").split():
        units.extend(_units(token))
    return normalize(units)


def _cost(a: str, b: str) -> float:
    if a == b:
        return 0.0
    if frozenset((a, b)) in _NEAR:
        return 0.5
    if a in _VOWELS and b in _VOWELS:
        return 0.8
    return 1.0


def distance(pattern: list[str], heard: list[str]) -> float:
    """Edit distance of `pattern` against its best-matching stretch of `heard`.

    The cut includes a little of the neighbouring words, so leading and
    trailing sounds in `heard` cost nothing.
    """
    if not pattern:
        return 0.0
    prev = [0.0] * (len(heard) + 1)
    for i, p in enumerate(pattern, 1):
        drop = 0.3 if i == len(pattern) and p in _WEAK_FINAL else 1.0
        cur = [prev[0] + drop] + [0.0] * len(heard)
        for j, h in enumerate(heard, 1):
            cur[j] = min(prev[j] + drop, cur[j - 1] + 1, prev[j - 1] + _cost(p, h))
        prev = cur
    return min(prev)


def judge(heard: list[str], expected: str, alternatives: tuple[str, ...]) -> dict:
    d_expected = distance(from_misaki(expected), heard)
    d_other = min(distance(from_misaki(a), heard) for a in alternatives)
    if d_expected + MARGIN <= d_other:
        verdict = "ok"
    elif d_other + MARGIN <= d_expected and d_other < GARBLED:
        verdict = "wrong"
    elif d_expected >= GARBLED and d_other >= GARBLED:
        verdict = "garbled"
    else:
        verdict = "unclear"
    return {"verdict": verdict, "d_expected": round(d_expected, 2),
            "d_other": round(d_other, 2)}


@lru_cache(maxsize=1)
def _recognizer():
    import torch
    from huggingface_hub import hf_hub_download
    from transformers import AutoModelForCTC, Wav2Vec2FeatureExtractor

    extractor = Wav2Vec2FeatureExtractor.from_pretrained(MODEL_ID)
    model = AutoModelForCTC.from_pretrained(MODEL_ID).eval()
    vocab = {v: k for k, v in json.loads(
        Path(hf_hub_download(MODEL_ID, "vocab.json")).read_text()).items()}
    blocked = torch.tensor([i for i, token in vocab.items() if token not in _ENGLISH])
    return torch, extractor, model, vocab, blocked


def recognize(audio: numpy.ndarray) -> str:
    """Phonemes for 16 kHz mono audio, as space-separated espeak symbols.

    Decoded by hand: the stock tokenizer insists on an espeak install that is
    only needed to turn text into phonemes, which this never does.
    """
    torch, extractor, model, vocab, blocked = _recognizer()
    values = extractor(audio, sampling_rate=_RATE, return_tensors="pt").input_values
    with torch.no_grad():
        logits = model(values).logits
        logits[..., blocked] = float("-inf")
        ids = logits.argmax(-1)[0].tolist()
    out, previous = [], None
    for i in ids:
        token = vocab.get(i)
        if i != previous and token not in (None, "<pad>", "<s>", "</s>", "<unk>"):
            out.append(token)
        previous = i
    return " ".join(out)


def _load_16k(path: Path) -> numpy.ndarray:
    from scipy.signal import resample_poly

    audio, rate = soundfile.read(str(path))
    if audio.ndim > 1:
        audio = audio.mean(axis=1)
    return resample_poly(audio, _RATE, rate) if rate != _RATE else audio


_TOKEN_RE = re.compile(r"[A-Za-z0-9']+")


def _norm(word: str) -> str:
    return re.sub(r"[^a-z0-9']", "", word.lower())


def check_segment(wav: Path, text: str, whisper_words: list[dict],
                  readings: list[HeteronymReading] | None = None) -> list[dict]:
    """Judge every heteronym in `text` against the audio in `wav`.

    `whisper_words` is Whisper's word list for the same audio, with `word`,
    `start` and `end`. A heteronym whose script word cannot be matched to a
    heard word is reported as "unaligned" and never flagged.
    """
    readings = find_heteronyms(text) if readings is None else readings
    if not readings:
        return []
    script = [(m.start(), m.end(), _norm(m.group())) for m in _TOKEN_RE.finditer(text)]
    heard_words = [w for w in whisper_words if _norm(w["word"])]
    matcher = difflib.SequenceMatcher(
        None, [s[2] for s in script], [_norm(w["word"]) for w in heard_words],
        autojunk=False,
    )
    mapping: dict[int, int] = {}
    for block in matcher.get_matching_blocks():
        for k in range(block.size):
            mapping[block.a + k] = block.b + k

    audio = None
    results = []
    for reading in readings:
        index = next((i for i, s in enumerate(script)
                      if s[0] <= reading.start < s[1]), None)
        entry = {"word": reading.word, "reading": reading.key,
                 "expected": reading.phonemes, "contrast": reading.contrast}
        if reading.contrast == "minor":
            entry["verdict"] = "minor"
            results.append(entry)
            continue
        if index is not None and (script[index][0], script[index][1]) != (reading.start, reading.end):
            # misaki split a longer word ("documented" into "document" + "ed").
            entry["verdict"] = "partial"
            results.append(entry)
            continue
        if index is None or index not in mapping:
            entry["verdict"] = "unaligned"
            results.append(entry)
            continue
        heard = heard_words[mapping[index]]
        if audio is None:
            audio = _load_16k(wav)
        a = max(0, int((heard["start"] - _PAD_S) * _RATE))
        b = min(len(audio), int((heard["end"] + _PAD_S) * _RATE))
        phones = recognize(audio[a:b]) if b - a > _RATE // 20 else ""
        entry.update({"at_s": round(heard["start"], 2), "heard": phones})
        entry.update(judge(from_espeak(phones), reading.phonemes, reading.alternatives))
        results.append(entry)
    return results


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("wav", type=Path)
    ap.add_argument("--text", required=True)
    args = ap.parse_args()

    import mlx_whisper

    result = mlx_whisper.transcribe(str(args.wav), word_timestamps=True, verbose=False,
                                    path_or_hf_repo="mlx-community/whisper-small.en-mlx")
    words = [w for s in result["segments"] for w in s.get("words", [])]
    for entry in check_segment(args.wav, args.text, words):
        print(json.dumps(entry, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
