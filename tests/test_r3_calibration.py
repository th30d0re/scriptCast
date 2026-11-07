from experiments.r3_calibration.collect import deduplicate, shot_name
from experiments.r3_calibration.run import metrics, thresholds


def test_thresholds_cover_requested_sweep():
    assert thresholds() == [.1, .2, .3, .4, .5, .6, .7, .8, .9]


def test_metrics_report_calibration_quantities():
    got = metrics([
        {"label": "bad", "verdict": "fail", "tier2_ran": True, "latency_ms": 10},
        {"label": "bad", "verdict": "pass", "tier2_ran": False, "latency_ms": 20},
        {"label": "good", "verdict": "fail", "tier2_ran": True, "latency_ms": 30},
        {"label": "good", "verdict": "pass", "tier2_ran": False, "latency_ms": 40},
    ])
    assert got == {"n": 4, "bad_n": 2, "good_n": 2, "recall": .5, "false_fail": .5,
                   "tier2_call_rate": .5, "latency_ms_mean": 25.0}


def test_manifest_helpers_are_conservative():
    row = {"still_path": "/x/a.png", "reference_path": "/x/a.png", "label": "good", "shot": "a", "source": "one"}
    assert shot_name(__import__("pathlib").Path("S01-02.png")) == "S01-02"
    assert deduplicate([row, dict(row, source="two")]) == [row]
