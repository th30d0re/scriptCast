import json

from PIL import Image

from scriptcast.tools import stage_dispatch as sd
from scriptcast.tools.stage_repair import repair_briefs, run_repairs


def fixture(tmp_path):
    episode = tmp_path / "yt02"; episode.mkdir()
    stills = tmp_path / "stills"; stills.mkdir(); Image.new("RGB", (10, 10)).save(stills / "Scene_cue.png")
    (stills / "Scene_cue.png.boxes.json").write_text(json.dumps({"marks": [{"id": "title", "box": {"x": 1, "y": 2, "w": 3, "h": 4}}]}))
    plan = tmp_path / "stage_plan.json"; plan.write_text(json.dumps({"segments": [{"scene": "Scene", "shots": [{"shot_id": "S01"}]}]}))
    report = tmp_path / "critic.json"; report.write_text(json.dumps({"frames": [{"file": "Scene_cue.png", "verdict": "fail", "defects": [{"element_id": "title", "evidence": "title overlaps chart"}]}]}))
    return episode, stills, plan, report


def test_report_only_builds_brief_from_marks_and_never_dispatches(tmp_path):
    ep, stills, plan, report = fixture(tmp_path)
    rows = run_repairs(ep, report, stills, plan, dispatch_main=lambda _: (_ for _ in ()).throw(AssertionError()))
    assert rows[0]["action"] == "report" and rows[0]["box"] == {"x": 1, "y": 2, "w": 3, "h": 4}
    assert rows[0]["shot"] == "S01"
    assert "shot S01" in repair_briefs(json.loads(report.read_text()), stills, plan)[0]["brief"]


def test_dispatch_resumes_live_builder_and_records_recritic_verdict(tmp_path):
    ep, stills, plan, report = fixture(tmp_path)
    path, key = sd.registry_path(ep), sd.registry_key(ep, "Scene", "builder")
    sd.registry_start(path, key, "codex", "gpt", "session-1", 0, 1, log_dir=tmp_path / "logs")
    calls = []
    def critic(*_, **__): return {"frames": [{"file": "Scene_cue.png", "verdict": "pass"}]}
    rows = run_repairs(ep, report, stills, plan, dispatch=True, dispatch_main=lambda args: calls.append(args) or 0, critic=critic)
    assert calls[0][0] == "resume" and "--session" not in calls[0]
    assert rows[0]["action"] == "resume" and rows[0]["before"] == "fail" and rows[0]["after"] == "pass"
    saved = (ep / "stage" / "review" / "repairs.jsonl").read_text()
    assert '"before": "fail"' in saved and '"after": "pass"' in saved


def test_dispatch_starts_when_builder_is_absent_and_limits_each_element_to_two(tmp_path):
    ep, stills, plan, report = fixture(tmp_path)
    calls = []
    def critic(*_, **__): return {"frames": [{"file": "Scene_cue.png", "verdict": "fail"}]}
    for _ in range(3):
        rows = run_repairs(ep, report, stills, plan, dispatch=True, dispatch_main=lambda args: calls.append(args) or 0, critic=critic)
    assert calls[0][0] == "start"
    assert [row["action"] for row in rows] == ["operator_queue"]
    assert len(calls) == 2


def test_stem_sidecar_and_s1_only_elements_build_top_three_briefs(tmp_path):
    ep, stills, plan, _ = fixture(tmp_path)
    exact = stills / "Scene_cue.png.boxes.json"
    exact.unlink()
    (stills / "Scene_cue.boxes.json").write_text(json.dumps({"marks": [
        {"id": "audit-beam", "box": {"x": 1, "y": 2, "w": 3, "h": 4}},
        {"id": "debate-funnel", "box": {"x": 5, "y": 6, "w": 7, "h": 8}},
        {"id": "third", "box": {"x": 9, "y": 10, "w": 11, "h": 12}},
        {"id": "below", "box": {"x": 0, "y": 0, "w": 1, "h": 1}},
    ]}))
    report = {"s1_threshold": .2, "frames": [{"file": "Scene_cue.png", "verdict": "fail",
        "defects": [], "tier1": {"flagged": True, "elements": [
            {"id": "audit-beam", "p_defect": .98}, {"id": "debate-funnel", "p_defect": .48},
            {"id": "third", "p_defect": .3}, {"id": "below", "p_defect": .1},
            {"id": "grid_0_0", "p_defect": .99}]}}]}
    rows = repair_briefs(report, stills, plan)
    assert [row["element_id"] for row in rows] == ["audit-beam", "debate-funnel", "third"]
    assert rows[0]["box"] == {"x": 1, "y": 2, "w": 3, "h": 4}
    assert "System 1 flagged this element (p=0.98)" in rows[0]["defect"]
