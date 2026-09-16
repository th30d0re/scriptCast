"""The stress judge, on phoneme strings the recognizer produced from real renders.

Each case is a take whose reading is known: the respelling it was rendered
from, or Emmanuel's ear. They pin the calibration so a cost change that breaks
it shows up here instead of in an episode.
"""
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "tools"))

from stress_check import from_espeak, judge  # noqa: E402

NOUN = "ɹˈɛkəɹd"
VERB = "ɹəkˈɔɹd"


@pytest.mark.parametrize(
    "heard, expected, other, verdict",
    [
        ("ɹ ɪ k əl ɹ ɛ k ɚ d", NOUN, VERB, "ok"),   # "historical reckerd"
        ("k əl ɹ æ k ɚ", NOUN, VERB, "ok"),         # plain noun, clean take
        ("ɹ ɛ k ɚ", NOUN, VERB, "ok"),
        ("ɹ iː k ɔːɹ", VERB, NOUN, "ok"),            # "ree-CORD"
        ("i ɹ ɪ k ɔːɹ", VERB, NOUN, "ok"),           # plain verb, clean take
        ("ɹ iː k ɔːɹ", NOUN, VERB, "wrong"),         # verb reading where a noun belongs
        ("ɹ ɛ k ɚ", VERB, NOUN, "wrong"),
        ("k u b e k oː d", NOUN, VERB, "garbled"),   # the take Emmanuel caught at 1:27
        ("k ɔ b a k ɔ", NOUN, VERB, "garbled"),
        ("ɹ ɛ k ɔːɹ", NOUN, VERB, "unclear"),        # both vowels full: not flagged
    ],
)
def test_judge(heard, expected, other, verdict):
    assert judge(from_espeak(heard), expected, (other,))["verdict"] == verdict
