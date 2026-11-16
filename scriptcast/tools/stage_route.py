"""scriptcast-stage-route: decide lane/tier/session/context for a Stage fix.

Rules decide (rules.toml); laya-mlx runs in shadow with forward and reversed
option order, and both answers are logged to router_log.jsonl. Laya lives in
.venv-router and loads lazily in a subprocess; when it is missing the record
says `laya: unavailable` and rules still decide.
"""
import argparse
from datetime import datetime, timezone
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib
from typing import Callable, Dict, List, Optional

from scriptcast.router.cue_card import CardTooLong, CueCard, render_fit

ROUTER_DIR = Path(__file__).resolve().parents[1] / "router"
QUESTIONS = ("lane", "tier", "session_action", "context_source")
SHADOW_QUESTIONS = ("lane", "tier", "context_source")  # session_action is rules-only
TIERS = ("mechanical", "standard", "judgment")


class NoRule(LookupError):
    pass


def load_config(router_dir: Path = ROUTER_DIR) -> dict:
    return {
        "rules": tomllib.loads((router_dir / "rules.toml").read_text()),
        "tiers": tomllib.loads((router_dir / "tiers.toml").read_text()),
        "questions": json.loads((router_dir / "questions.json").read_text()),
        "notebooks_path": router_dir / "notebooks.toml",
    }


# ---------------------------------------------------------------- rules

def _row_matches(row: dict, card: CueCard, text: str) -> bool:
    if "kind" in row and card.kind not in row["kind"]:
        return False
    if "defect_category" in row and card.defect_category not in row["defect_category"]:
        return False
    if "episode_kind" in row and card.episode_kind not in row["episode_kind"]:
        return False
    if "file_glob" in row and not any(
        fnmatch.fnmatch(f, g) or fnmatch.fnmatch(Path(f).name, g) or f"/{g.split('/')[0]}/" in f"/{f}"
        for f in card.owned_files for g in row["file_glob"]
    ):
        return False
    if "text" in row and not re.search(row["text"], text, re.I):
        return False
    return True


def _first(rows: List[dict], card: CueCard, text: str, name: str) -> dict:
    for row in rows:
        if _row_matches(row, card, text):
            return row
    raise NoRule(name)


def session_action(card: CueCard) -> str:
    if card.kind == "check":
        return "resume_checker"
    if card.kind == "fix" and card.has_live_session:
        return "resume_builder"
    return "spawn"


def apply_rules(card: CueCard, text: str, rules: dict) -> dict:
    lane = _first(rules["lane"], card, text, "lane")
    tier = _first(rules["tier"], card, text, "tier")
    ctx = _first(rules["context"], card, text, "context")
    return {
        "lane": lane["value"],
        "tier": tier["value"],
        "session_action": session_action(card),
        "context_source": ctx["value"],
        "rule_ids": [lane["id"], tier["id"], ctx["id"], "S1"],
    }



# ---------------------------------------------------------------- notebooks

def resolve_notebook(key: str, card: CueCard, cfg: dict) -> Optional[str]:
    """Map a context key to a notebook id (None for `none`)."""
    if key == "none":
        return None
    nb_path: Path = cfg["notebooks_path"]
    if nb_path.exists():  # written by R2
        data = tomllib.loads(nb_path.read_text())
        entries = data.get("notebook", data.get("notebooks", []))
        if isinstance(entries, dict):
            entries = [dict(v, key=k) for k, v in entries.items()]
        for e in entries:
            if key == "chapter" and card.episode is not None and e.get("episode") == card.episode:
                return e.get("id")
            if key != "chapter" and e.get("key") == key:
                return e.get("id")
    nbs = cfg["rules"]["notebooks"]
    if key in ("book", "persuasion"):
        return nbs.get(key)
    if key == "chapter" and card.episode is not None:
        p = Path(nbs.get("episode_map", ""))
        if p.exists():
            for e in json.loads(p.read_text()):
                if e.get("kind") == "series" and e.get("episode") == card.episode:
                    return e.get("notebook")
    return None


def episode_kind_from_map(episode: Optional[int], cfg: dict) -> str:
    if episode is None:
        return ""
    p = Path(cfg["rules"]["notebooks"].get("episode_map", ""))
    if not p.exists():
        return "series"
    for e in json.loads(p.read_text()):
        if e.get("episode") == episode:
            if e.get("kind") == "special":
                return "special"
            return "premiere" if e.get("premiere") else "series"
    return "series"


# ---------------------------------------------------------------- laya backend

def router_python() -> Optional[Path]:
    env = os.environ.get("SCRIPTCAST_ROUTER_PYTHON")
    if env:
        return Path(env)
    here = Path(__file__).resolve()
    for parent in here.parents:
        cand = parent / ".venv-router" / "bin" / "python"
        if cand.exists():
            return cand
    return None


Backend = Callable[[List[str], dict, str], dict]


def subprocess_backend(cards: List[str], questions: dict, model: str) -> dict:
    py = router_python()
    if py is None:
        raise RuntimeError(".venv-router not found")
    req = json.dumps({"model": model, "questions": questions, "cards": cards})
    proc = subprocess.run([str(py), str(ROUTER_DIR / "laya_runner.py")], input=req,
                          capture_output=True, text=True, timeout=600)
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip().splitlines()[-1] if proc.stderr.strip() else "laya failed")
    return json.loads(proc.stdout)


def shadow_questions(questions: dict) -> dict:
    return {q: {k: v for k, v in questions[q].items() if k != "rules_only"} for q in SHADOW_QUESTIONS}


def _choice(ans: dict):
    if ans.get("type") == "score":
        probs = ans.get("probabilities", {})
        idx = max(probs, key=probs.get) if probs else str(round(ans.get("score", 1)))
        return TIERS[int(idx)], float(probs.get(idx, ans.get("answer_confidence", 0.0)))
    return ans.get("choice"), float(ans.get("answer_confidence", ans.get("confidence", 0.0)))


def combine(fwd: dict, rev: dict, min_conf: float) -> dict:
    """Average forward and reversed probabilities; abstain on disagreement or low confidence."""
    out = {}
    for qid in SHADOW_QUESTIONS:
        if qid not in fwd or qid not in rev:
            continue
        f, r = _choice(fwd[qid]), _choice(rev[qid])
        pf, pr = fwd[qid].get("probabilities", {}), rev[qid].get("probabilities", {})
        avg = {k: (pf.get(k, 0.0) + pr.get(k, 0.0)) / 2 for k in set(pf) | set(pr)}
        if avg:
            best = max(avg, key=avg.get)
            label = TIERS[int(best)] if fwd[qid].get("type") == "score" else best
            a = [label, round(avg[best], 4)]
        else:
            a = [f[0], round(f[1], 4)]
        entry = {"fwd": [f[0], round(f[1], 4)], "rev": [r[0], round(r[1], 4)], "avg": a,
                 "abstain": f[0] != r[0] or a[1] < min_conf}
        if fwd[qid].get("type") == "score":
            entry["score"] = round((fwd[qid].get("score", 0) + rev[qid].get("score", 0)) / 2, 4)
        out[qid] = entry
    return out


# ---------------------------------------------------------------- route

def tier_target(lane: str, tier: str, tiers: dict) -> dict:
    return dict(tiers.get("lane", {}).get(lane, {}).get(tier) or tiers["default"][tier])


def route(card: CueCard, cfg: dict, backend: Optional[Backend] = subprocess_backend,
          override: Optional[Dict[str, str]] = None) -> dict:
    text, tokens, truncated = render_fit(card)
    rules = apply_rules(card, text, cfg["rules"])
    lcfg = cfg["rules"].get("laya", {})
    laya: dict
    if backend is None:
        laya = {"status": "unavailable", "error": "disabled"}
    else:
        try:
            res = backend([text], shadow_questions(cfg["questions"]), lcfg.get("model", "aac6fef/laya-mlx"))
            r0 = res["results"][0]
            laya = {"status": "ok", "model": lcfg.get("model"), "version": res.get("version"),
                    "ms": round(r0["ms"], 1), "load_ms": round(res.get("load_ms", 0.0), 1),
                    **combine(r0["fwd"], r0["rev"], lcfg.get("min_confidence", 0.15))}
            if r0.get("tokens"):
                tokens = r0["tokens"]
        except Exception as exc:  # degrade to rules, never block
            laya = {"status": "unavailable", "error": str(exc)[:200]}
    decision = {q: rules[q] for q in QUESTIONS}
    decided_by = {q: "rules" for q in QUESTIONS}
    promo = cfg["rules"].get("promotion", {})
    for q in SHADOW_QUESTIONS:
        if promo.get(q) == "laya" and laya.get("status") == "ok" and q in laya and not laya[q]["abstain"]:
            decision[q] = laya[q]["avg"][0]
            decided_by[q] = "laya"
    if override:
        for k, v in override.items():
            decision[k] = v
            decided_by[k] = "override"
    decision["notebook"] = resolve_notebook(decision["context_source"], card, cfg)
    decision["target"] = tier_target(decision["lane"], decision["tier"], cfg["tiers"])
    return {
        "ts": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "card": card.to_dict(),
        "card_text": text,
        "card_sha": hashlib.sha256(text.encode()).hexdigest()[:16],
        "card_tokens": tokens,
        "card_truncated": truncated,
        "rules": rules,
        "laya": laya,
        "decided_by": decided_by,
        "decision": decision,
        "override": override or None,
        "dispatch_ref": None,
    }


def append_log(record: dict, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a") as fh:
        fh.write(json.dumps(record, sort_keys=True) + "\n")


def _load_note(note: str, episode_dir: Optional[Path]) -> dict:
    """Resolve a note id against <episode>/review/notes.json; else treat as text."""
    if episode_dir:
        p = episode_dir / "review" / "notes.json"
        if p.exists():
            for n in json.loads(p.read_text()):
                if str(n.get("id")) == note:
                    return n
    return {"note": note}


def _parse_override(items: List[str]) -> Dict[str, str]:
    out = {}
    for item in items:
        for pair in item.split():
            k, _, v = pair.partition("=")
            if k not in QUESTIONS or not v:
                raise SystemExit(f"bad --override {pair!r}; use one of {QUESTIONS}=value")
            out[k] = v
    return out


def main(argv: Optional[List[str]] = None, backend: Optional[Backend] = subprocess_backend) -> int:
    ap = argparse.ArgumentParser(prog="scriptcast-stage-route", description=__doc__)
    ap.add_argument("--note", required=True, help="note id in review/notes.json, or note text")
    ap.add_argument("--scene", required=True)
    ap.add_argument("--cue", default="")
    ap.add_argument("--turn", type=int)
    ap.add_argument("--element", default="")
    ap.add_argument("--kind", default="fix", choices=["fix", "check", "implement", "research", "review"])
    ap.add_argument("--defect-category", default="")
    ap.add_argument("--file", action="append", default=[], dest="files")
    ap.add_argument("--live-session", action="store_true")
    ap.add_argument("--episode", type=Path, help="episode directory (router_log.jsonl lands here)")
    ap.add_argument("--episode-number", type=int)
    ap.add_argument("--log", type=Path)
    ap.add_argument("--no-laya", action="store_true")
    ap.add_argument("--override", action="append", default=[], help="lane=... tier=... (records a label)")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)

    cfg = load_config()
    n = _load_note(args.note, args.episode)
    card = CueCard(scene=args.scene, note=n.get("note", n.get("text", "")), kind=args.kind,
                   turn=args.turn, cue=args.cue or n.get("cue", ""),
                   element_id=args.element or n.get("element", n.get("element_id", "")),
                   defect_category=args.defect_category or n.get("category", ""),
                   owned_files=args.files, has_live_session=args.live_session,
                   episode=args.episode_number)
    card.episode_kind = episode_kind_from_map(card.episode, cfg)
    override = _parse_override(args.override)
    try:
        rec = route(card, cfg, None if args.no_laya else backend, override)
    except CardTooLong as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 3
    except NoRule as exc:
        print(f"error: no rule for {exc}; decide with --override\n{card.to_dict()}", file=sys.stderr)
        return 3
    if override:
        rec["label"] = {q: rec["decision"][q] for q in QUESTIONS}
    log = args.log or ((args.episode or Path.cwd()) / "router_log.jsonl")
    append_log(rec, log)
    if args.json:
        print(json.dumps(rec, indent=2))
    else:
        d = rec["decision"]
        t = d["target"]
        print(f"lane={d['lane']} tier={d['tier']} session={d['session_action']} "
              f"context={d['context_source']} notebook={d['notebook'] or '-'}")
        print(f"target: {t['agent']} {t['model']} {' '.join(t.get('flags', []))}".rstrip())
        lay = rec["laya"]
        if lay["status"] != "ok":
            print("laya: unavailable")
        else:
            print("laya: " + "  ".join(f"{q}={lay[q]['avg'][0]}({lay[q]['avg'][1]:.2f}"
                                       f"{',abstain' if lay[q]['abstain'] else ''})" for q in SHADOW_QUESTIONS))
    return 0


if __name__ == "__main__":
    sys.exit(main())
