import csv
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATH = ROOT / "experiments/svg_offload/judge.py"
spec = importlib.util.spec_from_file_location("svg_judge", PATH)
judge = importlib.util.module_from_spec(spec); sys.modules[spec.name] = judge; spec.loader.exec_module(judge)


def plan():
    return {"episode_dir": "/live/stage", "cues": {"S1.cue": 2_000}, "segments": [{"shots": [{"shot_id": "S1", "start_ms": 1_000, "end_ms": 6_000}]}]}


def test_cue_fraction_adds_settle_and_clamps():
    assert judge.cue_fraction(plan(), "S1", "cue") == .38
    assert judge.cue_fraction({**plan(), "cues": {"cue": 9_000}}, "S1", "cue") == .98


def test_rewrite_plan_changes_only_episode_dir(tmp_path):
    source = tmp_path / "live.json"; source.write_text(json.dumps(plan()))
    out = tmp_path / "nested/plan.json"
    got = judge.rewrite_plan(source, out, Path("/candidate/stage"))
    assert got["episode_dir"] == "/candidate/stage"
    assert json.loads(out.read_text())["cues"] == plan()["cues"]


def test_empty_diff_is_skipped_without_runner(tmp_path):
    cell = "one-B-worker"; (tmp_path / "out" / cell).mkdir(parents=True)
    (tmp_path / "out" / cell / "tsx.diff").write_text("")
    row = {"cue": "one", "condition": "B", "worker": "worker", "shot": "S1", "cue_id": "cue"}
    result = judge.judge_row(row, tmp_path, tmp_path / "missing-plan.json", dry_run=False,
                             runner=lambda *a, **kw: (_ for _ in ()).throw(AssertionError("runner called")))
    assert result["status"] == "no_change"


def test_csv_shape_with_fake_runner(tmp_path, monkeypatch):
    exp = tmp_path / "exp"; (exp / "out/one-B-worker").mkdir(parents=True)
    (exp / "out/one-B-worker/tsx.diff").write_text("diff")
    with (exp / "results.csv").open("w", newline="") as f:
        csv.DictWriter(f, fieldnames=["cue", "scene", "shot", "cue_id", "condition", "worker"]).writeheader()
        csv.DictWriter(f, fieldnames=["cue", "scene", "shot", "cue_id", "condition", "worker"]).writerow({"cue":"one", "scene":"Scene", "shot":"S1", "cue_id":"cue", "condition":"B", "worker":"worker"})
    live = tmp_path / "live.json"; live.write_text(json.dumps(plan()))
    monkeypatch.setattr(judge, "LIVE_PLAN", live); monkeypatch.setattr(judge, "BOOK", tmp_path / "book")
    def fake(command, **kwargs):
        if "still" in command:
            from PIL import Image
            out = Path(command[command.index("-o") + 1]); out.parent.mkdir(parents=True, exist_ok=True); Image.new("RGB", (1, 1)).save(out)
        return subprocess.CompletedProcess(command, 0, "", "")
    assert judge.main(["--exp-dir", str(exp), "--skip-critic"], runner=fake) == 0
    rows = list(csv.DictReader((exp / "judge/judge.csv").open()))
    assert list(rows[0]) == judge.JUDGE_HEADER
    assert rows[0]["status"] == "ok" and rows[0]["render_ok"] == "True"
