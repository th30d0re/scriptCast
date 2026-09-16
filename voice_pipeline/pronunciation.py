"""Heteronyms: words spelled alike that are pronounced by part of speech.

"The historical record" wants REC-ord; "we record it" wants re-CORD. OmniVoice
reads plain text and picks a stress on its own, and in practice it picks the
wrong one often enough to hear across an episode. It takes no phoneme input, so
the only lever is the spelling it is handed.

Two halves live here:

* **Deciding the reading.** misaki, Kokoro's grapheme-to-phoneme engine, tags
  each word with spaCy and chooses the reading its lexicon lists for that part
  of speech. `find_heteronyms` exposes that decision with character offsets, and
  the render verifier checks the audio against it.
* **Steering the engine.** `pronunciations.yaml` maps a word and a reading to a
  spelling that OmniVoice pronounces the intended way, found by rendering
  candidates and checking them (`tools/calibrate_pronunciation.py`). The
  spelling reaches the engine only. The script, captions, and verification keep
  the real word.

Kokoro is excluded: it already speaks misaki's phonemes, and a respelling would
make it worse.
"""
from __future__ import annotations

import hashlib
import re
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path

import yaml

RESPELLINGS_PATH = Path(__file__).with_name("pronunciations.yaml")

# Engines that read plain text and choose stress themselves.
RESPELLING_ENGINES = frozenset({"omnivoice", "mlx_chatterbox", "elevenlabs", "mlx_dia"})

_STRESS = str.maketrans("", "", "ˈˌ")
_MISAKI_VOWELS = set("aeiouæɑɒɔəɛɜɪʊʌᵊᵻAIOWYɚɐ")
_WORD_RE = re.compile(r"[A-Za-z']+")


@dataclass(frozen=True)
class HeteronymReading:
    """One occurrence of a heteronym and the reading context calls for."""

    word: str
    start: int
    end: int
    tag: str
    key: str
    phonemes: str
    alternatives: tuple[str, ...]
    # "stress" when another reading puts the primary stress on a different
    # syllable (REC-ord, re-CORD), "vowel" when the stressed vowel changes (lives,
    # read), "minor" when only an unstressed vowel or a voicing differs
    # (deliberate, use). Minor pairs vary between speakers and are not checked.
    contrast: str = "minor"


@lru_cache(maxsize=1)
def _g2p():
    from misaki import en

    return en.G2P(trf=False, british=False)


@lru_cache(maxsize=1)
def _heteronym_table() -> dict[str, dict[str, str]]:
    """Words whose readings differ in sound, not only in stress marks.

    misaki lists many function words ("that", "this") with a stressed and an
    unstressed form. Those carry no risk and cannot be heard apart reliably, so
    only entries whose readings differ once stress marks are removed count.
    """
    lexicon = _g2p().lexicon
    table: dict[str, dict[str, str]] = {}
    for source in (lexicon.golds, lexicon.silvers):
        for word, entry in source.items():
            if word in table or not isinstance(entry, dict):
                continue
            readings = {k: v for k, v in entry.items() if isinstance(v, str) and v}
            if len({v.translate(_STRESS) for v in readings.values()}) > 1:
                table[word] = readings
    return table


def _primary(phonemes: str) -> tuple[int, str]:
    """Syllable index carrying primary stress, and its vowel."""
    nucleus, previous_vowel, marked = -1, False, False
    for symbol in phonemes:
        if symbol == "ˈ":
            marked = True
            continue
        is_vowel = symbol in _MISAKI_VOWELS
        if is_vowel and not previous_vowel:
            nucleus += 1
            if marked:
                return nucleus, symbol
        previous_vowel = is_vowel
    return -1, ""


def _contrast(chosen: str, alternatives: tuple[str, ...]) -> str:
    here = _primary(chosen)
    kinds = set()
    for other in alternatives:
        there = _primary(other)
        if here[0] < 0 or there[0] < 0:
            continue
        if here[0] != there[0]:
            kinds.add("stress")
        elif here[1] != there[1]:
            kinds.add("vowel")
    return "stress" if "stress" in kinds else "vowel" if "vowel" in kinds else "minor"


def is_heteronym(word: str) -> bool:
    return word.lower() in _heteronym_table()


def find_heteronyms(text: str) -> list[HeteronymReading]:
    """Every heteronym in `text`, with the reading misaki picks from context."""
    table = _heteronym_table()
    if not any(w.lower() in table for w in _WORD_RE.findall(text)):
        return []

    _, tokens = _g2p()(text)
    found: list[HeteronymReading] = []
    cursor = 0
    for token in tokens:
        start = text.find(token.text, cursor)
        if start < 0:
            continue
        end = start + len(token.text)
        cursor = end
        readings = table.get(token.text.lower())
        if not readings or not token.phonemes:
            continue
        chosen = token.phonemes.translate(_STRESS)
        # Several keys can share one reading ("read" lists ADJ, VBD, VBN and
        # VBP all as ɹˈɛd). Name it by DEFAULT when that matches, else by the
        # token's own tag, so pronunciations.yaml has one stable key to use.
        matching = sorted(k for k, v in readings.items() if v.translate(_STRESS) == chosen)
        if not matching:
            continue
        key = ("DEFAULT" if "DEFAULT" in matching
               else token.tag if token.tag in matching else matching[0])
        alternatives = tuple(
            sorted({v for v in readings.values() if v.translate(_STRESS) != chosen})
        )
        found.append(
            HeteronymReading(
                word=token.text,
                start=start,
                end=end,
                tag=token.tag,
                key=key.lower(),
                phonemes=token.phonemes,
                alternatives=alternatives,
                contrast=_contrast(token.phonemes, alternatives),
            )
        )
    return found


def load_respellings(path: Path = RESPELLINGS_PATH) -> dict[str, dict[str, str]]:
    if not path.exists():
        return {}
    data = yaml.safe_load(path.read_text()) or {}
    return {
        word.lower(): {k.lower(): str(v) for k, v in (readings or {}).items()}
        for word, readings in data.items()
    }


def respell(text: str, respellings: dict[str, dict[str, str]] | None = None) -> str:
    """Replace heteronyms with the spelling calibrated for their reading."""
    if respellings is None:
        respellings = load_respellings()
    if not respellings or not any(
        w.lower() in respellings for w in _WORD_RE.findall(text)
    ):
        return text

    out: list[str] = []
    cursor = 0
    for reading in find_heteronyms(text):
        spelling = respellings.get(reading.word.lower(), {}).get(reading.key)
        if not spelling:
            continue
        if reading.word[:1].isupper():
            spelling = spelling[:1].upper() + spelling[1:]
        out.append(text[cursor : reading.start])
        out.append(spelling)
        cursor = reading.end
    out.append(text[cursor:])
    return "".join(out)


def speech_text_for(engine: str, text: str) -> str:
    """The text an engine should be handed for `text`."""
    if engine not in RESPELLING_ENGINES:
        return text
    return respell(text)


def speech_text_hash(engine: str, texts: list[str]) -> str:
    """Identity of the respelled text, or "" when nothing was respelled.

    Stored beside the turn's text hash, so adding or changing a respelling
    re-renders exactly the turns it touches and leaves every other turn alone.
    """
    if engine not in RESPELLING_ENGINES:
        return ""
    respelled = [respell(t) for t in texts]
    if respelled == texts:
        return ""
    return hashlib.sha256("\n".join(respelled).encode("utf-8")).hexdigest()[:16]
