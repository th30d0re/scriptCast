import json
from pathlib import Path
import sys

import pytest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from scriptcast.router.cue_card import MAX_TOKENS, CardTooLong, CueCard, render, render_fit
from scriptcast.tools import stage_route as sr

sys.path.insert(0, str(ROOT / "experiments" / "router_eval"))
import eval as router_eval  # noqa: E402


def stub_backend(answers=None, flip=False):
    """Fake laya: fixed answers, reversed order optionally disagrees on lane."""
    def backend(cards, questions, model):
        assert "session_action" not in questions
        out = []
        for _ in cards:
            fwd = {"lane": {"type": "choice", "choice": "tsx_scene", "answer_confidence": 0.6,
                            "probabilities": {"tsx_scene": 0.6, "audit_sweep": 0.4}},
                   "tier": {"type": "score", "score": 1.0, "probabilities": {"0": 0.2, "1": 0.7, "2": 0.1}},
                   "context_source": {"type": "choice", "choice": "none", "answer_confidence": 0.5,
                                      "probabilities": {"none": 0.5, "chapter": 0.5}}}
            rev = json.loads(json.dumps(fwd))
            if flip:
                rev["lane"] = {"type": "choice", "choice": "audit_sweep", "answer_confidence": 0.7,
                               "probabilities": {"tsx_scene": 0.3, "audit_sweep": 0.7}}
            out.append({"fwd": fwd, "rev": rev, "ms": 40.0, "tokens": 50})
        return {"version": "stub", "load_ms": 1.0, "results": out}
    return backend


def broken_backend(cards, questions, model):
    raise RuntimeError(".venv-router not found")


@pytest.fixture
def cfg(tmp_path):
    c = sr.load_config()
    c["notebooks_path"] = tmp_path / "absent.toml"
    return c


# ---- card budget

def test_card_renders_fields():
    t = render(CueCard(scene="APA", element_id="tray", cue="t21", turn=21, note="tray too high",
                       defect_category="layout", owned_files=["stage/a/APA.tsx"]))
    assert "APA / tray / t21" in t and "turn 21" in t and "(layout)" in t and "APA.tsx" in t


def test_card_too_long_raises_and_fit_truncates():
    card = CueCard(scene="S", note="word " * 2000)
    with pytest.raises(CardTooLong):
        render(card)
    text, n, truncated = render_fit(card)
    assert truncated and n <= MAX_TOKENS and text.endswith("...")


def test_card_with_custom_counter():
    with pytest.raises(CardTooLong):
        render(CueCard(note="a b c"), count=lambda s: 401)


# ---- rules table

@pytest.mark.parametrize("card,lane,tier,action", [
    (CueCard(scene="APA", note="tray sits 40 px too high", defect_category="layout",
             owned_files=["stage/min3_7/a/APA.tsx"], has_live_session=True), "figcraft_layout", "standard", "resume_builder"),
    (CueCard(scene="APA", note="fix the bounce timing", owned_files=["stage/a/APA.tsx"]), "tsx_scene", "standard", "spawn"),
    (CueCard(scene="APA", note="redraw", owned_files=["assets/person.svg"]), "svg_asset", "standard", "spawn"),
    (CueCard(scene="APA", note="caption uses antithesis", defect_category="content"), "on_screen_text", "standard", "spawn"),
    (CueCard(note="all scenes, equation numbers vs aux", kind="check"), "audit_sweep", "standard", "resume_checker"),
    (CueCard(note="10 fact-check questions", kind="research"), "research", "standard", "spawn"),
    (CueCard(scene="APA", note="open a clearer gap beneath the rule"), "figcraft_layout", "mechanical", "spawn"),
    (CueCard(note="stage-layout spike for FigCraft", kind="implement"), "figcraft_layout", "judgment", "spawn"),
    (CueCard(note="layout tweak", owned_files=["figcraft/APA.json"]), "figcraft_layout", "standard", "spawn"),
])
def test_rules(cfg, card, lane, tier, action):
    r = sr.apply_rules(card, render(card), cfg["rules"])
    assert (r["lane"], r["tier"], r["session_action"]) == (lane, tier, action)


def test_live_session_only_resumes_fixes(cfg):
    c = CueCard(note="new scene", kind="implement", has_live_session=True)
    assert sr.apply_rules(c, render(c), cfg["rules"])["session_action"] == "spawn"


def test_context_routing(cfg):
    for ek, want in (("premiere", "book"), ("special", "book"), ("series", "chapter"), ("", "none")):
        c = CueCard(scene="S", note="tray", episode_kind=ek)
        assert sr.apply_rules(c, render(c), cfg["rules"])["context_source"] == want


def test_no_rule_raises(cfg):
    rules = dict(cfg["rules"], lane=[{"id": "X", "value": "research", "kind": ["research"]}])
    with pytest.raises(sr.NoRule):
        sr.apply_rules(CueCard(note="x"), "x", rules)


def test_notebook_resolution(cfg, tmp_path):
    em = tmp_path / "episode_map.json"
    em.write_text(json.dumps([{"kind": "series", "episode": 0, "premiere": True, "notebook": "BOOK"},
                              {"kind": "series", "episode": 5, "notebook": "EP5"}]))
    cfg["rules"] = dict(cfg["rules"], notebooks=dict(cfg["rules"]["notebooks"], episode_map=str(em)))
    assert sr.episode_kind_from_map(0, cfg) == "premiere"
    assert sr.episode_kind_from_map(5, cfg) == "series"
    assert sr.resolve_notebook("chapter", CueCard(episode=5), cfg) == "EP5"
    assert sr.resolve_notebook("book", CueCard(), cfg) == "d3f54f50-61a5-465e-a94a-55d17326c0e2"
    assert sr.resolve_notebook("none", CueCard(), cfg) is None
    nb = tmp_path / "notebooks.toml"
    nb.write_text('[[notebook]]\nkey = "chapter"\nid = "R2EP5"\nepisode = 5\nkind = "book"\n')
    cfg["notebooks_path"] = nb
    assert sr.resolve_notebook("chapter", CueCard(episode=5), cfg) == "R2EP5"


def test_tier_target(cfg):
    assert sr.tier_target("tsx_scene", "standard", cfg["tiers"])["model"] == "gpt-5.6-sol"
    assert sr.tier_target("tsx_scene", "mechanical", cfg["tiers"])["model"] == "gpt-5.6-luna"
    assert sr.tier_target("svg_asset", "mechanical", cfg["tiers"])["agent"] == "agy"


# ---- shadow + log schema

LOG_KEYS = {"ts", "card", "card_text", "card_sha", "card_tokens", "card_truncated", "rules",
            "laya", "decided_by", "decision", "override", "dispatch_ref"}


def test_route_with_stub_logs_shadow(cfg, tmp_path):
    log = tmp_path / "router_log.jsonl"
    rc = sr.main(["--note", "tray too high", "--scene", "APA", "--file", "stage/a/APA.tsx",
                  "--log", str(log)], backend=stub_backend())
    assert rc == 0
    rec = json.loads(log.read_text())
    assert LOG_KEYS <= set(rec)
    assert rec["laya"]["status"] == "ok"
    assert rec["laya"]["lane"]["fwd"][0] == "tsx_scene" and not rec["laya"]["lane"]["abstain"]
    assert rec["laya"]["tier"]["avg"][0] == "standard"
    assert set(rec["decided_by"].values()) == {"rules"}
    assert "session_action" not in rec["laya"]


def test_order_disagreement_abstains(cfg):
    rec = sr.route(CueCard(note="x", scene="S"), cfg, stub_backend(flip=True))
    assert rec["laya"]["lane"]["abstain"]


def test_laya_unavailable_falls_back(cfg, tmp_path, capsys):
    log = tmp_path / "l.jsonl"
    assert sr.main(["--note", "tray", "--scene", "S", "--log", str(log)], backend=broken_backend) == 0
    assert "laya: unavailable" in capsys.readouterr().out
    rec = json.loads(log.read_text())
    assert rec["laya"]["status"] == "unavailable" and rec["decision"]["lane"]


def test_override_records_label(cfg, tmp_path):
    log = tmp_path / "l.jsonl"
    sr.main(["--note", "tray", "--scene", "S", "--log", str(log), "--no-laya",
             "--override", "lane=svg_asset tier=judgment"])
    rec = json.loads(log.read_text())
    assert rec["decision"]["lane"] == "svg_asset" and rec["decided_by"]["lane"] == "override"
    assert rec["label"]["tier"] == "judgment" and rec["label"]["session_action"] == "spawn"


def test_note_id_from_episode(tmp_path):
    (tmp_path / "review").mkdir()
    (tmp_path / "review" / "notes.json").write_text(json.dumps(
        [{"id": "n1", "scene": "APA", "element": "tray", "cue": "t21", "note": "lift the tray"}]))
    assert sr.main(["--note", "n1", "--scene", "APA", "--episode", str(tmp_path), "--no-laya"]) == 0
    rec = json.loads((tmp_path / "router_log.jsonl").read_text())
    assert rec["card"]["element_id"] == "tray" and rec["decision"]["tier"] == "mechanical"


# ---- eval

def test_eval_with_stub(cfg):
    rows = [{"id": "a", "source": "dispatch-log", "card": "[P] research: fact-check pass",
             "lane": "research", "tier": "standard", "session_action": "spawn", "context_source": "none"},
            {"id": "b", "source": "review/notes.json", "card": "[APA / tray / t21] open a gap",
             "lane": "figcraft_layout", "tier": "mechanical", "session_action": "spawn", "context_source": "none"}]
    rep = router_eval.evaluate(rows, cfg, stub_backend())
    assert rep["questions"]["lane"]["rules_acc"] == 1.0
    assert rep["questions"]["lane"]["laya_avg_acc"] == 0.0
    assert rep["laya"]["p50_ms"] == 40.0
    assert rep["questions"]["session_action"]["laya"].startswith("rules-only")
