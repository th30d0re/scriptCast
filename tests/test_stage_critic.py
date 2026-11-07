import json

from PIL import Image, ImageDraw

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


def _reference(tmp_path, stills, changed=False):
    refs = tmp_path / "references"; refs.mkdir()
    image = Image.new("RGB", (40, 20), "black")
    if changed:
        ImageDraw.Draw(image).rectangle((4, 4, 25, 15), fill="white")
    image.save(refs / "Scene_cue.png")
    return refs


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


def test_unflagged_s1_frame_skips_tier2(tmp_path):
    stills, plan = _fixture(tmp_path)
    refs = _reference(tmp_path, stills)
    calls = []
    report = critic_stills(stills, plan, reference_dir=refs, runner=lambda request: calls.append(request) or {"results": []})
    frame = report["frames"][0]
    assert calls == []
    assert frame["verdict"] == "pass" and frame["tier2_ran"] is False
    assert frame["tier1"]["flagged"] is False and frame["tier1"]["elements"][0]["id"] == "title"


def test_flagged_s1_frame_runs_tier2_and_mentions_regions(tmp_path):
    stills, plan = _fixture(tmp_path)
    refs = _reference(tmp_path, stills, changed=True)
    calls = []
    def runner(request):
        calls.append(request)
        return {"results": [{"file": "Scene_cue.png", "raw": _reply([])}]}
    report = critic_stills(stills, plan, reference_dir=refs, runner=runner)
    assert len(calls) == 1
    assert "Check these regions" in calls[0]["items"][0]["prompt"]
    assert report["frames"][0]["tier2_ran"] is True


def test_tier2_contract_error_falls_back_to_s1(tmp_path):
    stills, plan = _fixture(tmp_path)
    refs = _reference(tmp_path, stills, changed=True)
    report = critic_stills(stills, plan, reference_dir=refs,
                           runner=lambda _: {"results": [{"file": "Scene_cue.png", "raw": "bad"}]})
    frame = report["frames"][0]
    assert frame["status"] == "s1_only" and frame["verdict"] == "fail" and frame["tier2_ran"] is True


def test_no_reference_still_runs_tier2(tmp_path):
    stills, plan = _fixture(tmp_path)
    calls = []
    def runner(request):
        calls.append(request)
        return {"results": [{"file": "Scene_cue.png", "raw": _reply([])}]}
    report = critic_stills(stills, plan, runner=runner)
    assert len(calls) == 1 and report["frames"][0]["tier2_ran"] is True
