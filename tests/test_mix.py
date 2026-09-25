from types import SimpleNamespace as NS
from pathlib import Path

import pytest

from scriptcast.tools.mix import build_command, plan_mix

TURNS = [NS(turn_index=0, clean_text="First line here."),
         NS(turn_index=1, clean_text="It took sixty-nine days after the fact."),
         NS(turn_index=2, clean_text="Interesting.")]
MANIFEST = {"turns": [{"turn_index": 0, "start_ms": 0, "end_ms": 2000},
                      {"turn_index": 1, "start_ms": 2300, "end_ms": 6000},
                      {"turn_index": 2, "start_ms": 6300, "end_ms": 7000}]}
ASR = [{"word": "69", "start": 3.0, "end": 3.4}, {"word": "days", "start": 3.4, "end": 3.7}]


def test_plan_places_overlay_after_word_and_gain_on_turn():
    config = {"overlays": [{"file": "n.wav", "turn_contains": "days after the fact", "after_word": "^69$", "delay_ms": 100, "gain_db": -20}],
              "gains": [{"turn_equals": "interesting.", "gain_db": 5}]}
    plan = plan_mix(config, TURNS, MANIFEST, ASR, Path("/proj"))
    assert plan["overlays"] == [{"file": "/proj/n.wav", "at_ms": 3500, "gain_db": -20.0}]
    assert plan["gains"] == [{"start_s": 6.3, "end_s": 7.0, "gain_db": 5.0}]


def test_ambiguous_or_missing_rule_raises():
    with pytest.raises(ValueError):
        plan_mix({"gains": [{"turn_contains": "zzz", "gain_db": 1}]}, TURNS, MANIFEST, ASR, Path("."))
    with pytest.raises(ValueError):
        plan_mix({"overlays": [{"file": "n.wav", "turn_contains": "sixty", "after_word": "^nope$", "gain_db": -9}]}, TURNS, MANIFEST, ASR, Path("."))


def test_command_has_overlay_delay_gain_and_limiter():
    plan = {"overlays": [{"file": "/p/n.wav", "at_ms": 3500, "gain_db": -20.0}], "gains": [{"start_s": 6.3, "end_s": 7.0, "gain_db": 5.0}]}
    cmd = build_command(Path("a.mp3"), plan, Path("o.mp3"))
    graph = cmd[cmd.index("-filter_complex") + 1]
    assert "adelay=3500|3500" in graph and "volume=-20.0dB" in graph
    assert "volume=5.0dB:enable='between(t,6.300,7.000)'" in graph and "alimiter" in graph
