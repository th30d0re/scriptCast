import json

from PIL import Image

from scriptcast.critic.grounding import ground_defects, pixel_diff
from scriptcast.tools.stage_critic import critic_stills, validate_reply


def _reply(defects):
    return json.dumps({"verdict": "fail" if defects else "pass", "defects": defects})


def _defect(element_id="title", category="Layout", region=[.12, .12, .2, .1], evidence="overlaps"):
    return {"category": category, "element_id": element_id, "evidence": evidence,
            "severity": "high", "region": region}


def _fixture(tmp_path):
    stills = tmp_path / "stills"; stills.mkdir()
    Image.new("RGB", (40, 20)).save(stills / "Scene_cue.png")
    plan = tmp_path / "stage_plan.json"
    plan.write_text(json.dumps({"width": 100, "height": 100, "frames": [{
        "scene": "Scene", "cue": "cue", "storyboard": "A title is visible.",
        "elements": [{"id": "title", "box": [10, 10, 40, 20]}],
    }]}))
    return stills, plan


def test_schema_validation():
    assert validate_reply(json.loads(_reply([_defect()]))) is not None
    assert validate_reply({"verdict": "wat", "defects": []}) is None
    assert validate_reply({"verdict": "pass", "defects": [], "extra": 1}) is None


def test_contract_error_after_two_bad_replies(tmp_path):
    stills, plan = _fixture(tmp_path)
    calls = []
    def runner(_):
        calls.append(1)
        return {"results": [{"file": "Scene_cue.png", "raw": "not json"}]}
    report = critic_stills(stills, plan, runner=runner)
    assert len(calls) == 2
    assert report["frames"][0]["status"] == "contract_error"
    assert (stills / "critic.json").is_file()


def test_grounding_uses_declared_ids_not_model_regions():
    kept, dropped = ground_defects([_defect("missing"), _defect(region=[.8, .8, .1, .1])],
                                   {"title": [.1, .1, .4, .2]}, "")
    assert len(kept) == 1 and dropped == 1
    assert kept[0]["region_agrees"] is False
    content = _defect(None, "Content", evidence='unexpected text "WRONG WORD"')
    kept, dropped = ground_defects([content], {}, "wrong word shown")
    assert kept == [dict(content, region_agrees=False)] and dropped == 0


def test_pixel_diff_and_grounding_drop_unchanged_declared_element(tmp_path):
    approved = tmp_path / "approved.png"
    candidate = tmp_path / "candidate.png"
    Image.new("RGB", (10, 10), "white").save(approved)
    Image.new("RGB", (10, 10), "white").save(candidate)
    box = [.2, .2, .4, .4]
    assert pixel_diff(approved, candidate, box) == 0.0
    kept, dropped = ground_defects([_defect(region=[.25, .25, .2, .1])], {"title": box}, "", approved, candidate)
    assert kept == [] and dropped == 1

    changed = Image.open(candidate)
    changed.putpixel((3, 3), (0, 0, 0))
    changed.save(candidate)
    assert pixel_diff(approved, candidate, box) > 0.01
    defect = _defect(region=[.25, .25, .2, .1])
    kept, dropped = ground_defects([defect], {"title": box}, "", approved, candidate)
    assert kept == [dict(defect, region_agrees=True)] and dropped == 0


def test_verdict_downgrades_when_only_defect_is_ungrounded(tmp_path):
    stills, plan = _fixture(tmp_path)
    def runner(_):
        return {"results": [{"file": "Scene_cue.png", "raw": _reply([_defect("missing")])}]}
    report = critic_stills(stills, plan, runner=runner)
    frame = report["frames"][0]
    assert frame["verdict"] == "pass"
    assert frame["ungrounded"] == 1
