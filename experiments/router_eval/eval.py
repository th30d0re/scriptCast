"""Evaluate rules and laya shadow against labels.jsonl (reviewed rows only).

Per question: rules accuracy; laya accuracy (forward, reversed, averaged),
order-flip rate, abstention rate, ECE (10 bins, averaged answers), and
P50/P95 latency of one forward+reversed call.

  python experiments/router_eval/eval.py [--labels PATH] [--all] [--no-laya] [--json OUT]
"""
import argparse
import json
from pathlib import Path
import re
import statistics
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from scriptcast.router.cue_card import CueCard, render_fit  # noqa: E402
from scriptcast.tools import stage_route as sr  # noqa: E402

KIND_RE = re.compile(r"\]\s*(implement|review|research|fix|check|verify|audit)\s*:", re.I)


def card_from_label(row: dict) -> CueCard:
    """Labels hold free-text cards; kind comes from the '[project] kind:' prefix."""
    text = row["card"]
    m = KIND_RE.search(text)
    kind = m.group(1).lower() if m else ("fix" if row["source"].startswith("review/") else "implement")
    if kind in ("verify", "audit"):
        kind = "review"
    return CueCard(note=text, kind=kind)


def ece(conf_correct, bins=10):
    if not conf_correct:
        return None
    total, n = 0.0, len(conf_correct)
    for b in range(bins):
        lo, hi = b / bins, (b + 1) / bins
        sel = [(c, k) for c, k in conf_correct if lo < c <= hi or (b == 0 and c == 0)]
        if sel:
            total += len(sel) / n * abs(statistics.mean(c for c, _ in sel) - statistics.mean(k for _, k in sel))
    return round(total, 4)


def pct(xs, p):
    xs = sorted(xs)
    return round(xs[min(len(xs) - 1, int(round(p / 100 * (len(xs) - 1))))], 1) if xs else None


def evaluate(rows, cfg, backend=sr.subprocess_backend):
    cards = [card_from_label(r) for r in rows]
    texts = [render_fit(c)[0] for c in cards]
    rules = [sr.apply_rules(c, t, cfg["rules"]) for c, t in zip(cards, texts)]
    report = {"n": len(rows), "questions": {}, "laya": {"status": "unavailable"}}
    laya = None
    if backend is not None:
        try:
            res = backend(texts, sr.shadow_questions(cfg["questions"]), cfg["rules"]["laya"]["model"])
            min_conf = cfg["rules"]["laya"].get("min_confidence", 0.15)
            laya = [sr.combine(r["fwd"], r["rev"], min_conf) for r in res["results"]]
            ms = [r["ms"] for r in res["results"]]
            report["laya"] = {"status": "ok", "version": res.get("version"), "load_ms": round(res["load_ms"], 1),
                              "p50_ms": pct(ms, 50), "p95_ms": pct(ms, 95)}
        except Exception as exc:
            report["laya"] = {"status": "unavailable", "error": str(exc)[:200]}
    misses = []
    for q in sr.QUESTIONS:
        gold = [r[q] for r in rows]
        rp = [r[q] for r in rules]
        out = {"rules_acc": round(sum(a == b for a, b in zip(rp, gold)) / len(rows), 4)}
        misses += [(rows[i]["id"], q, rp[i], gold[i]) for i in range(len(rows)) if rp[i] != gold[i]]
        if laya and q in laya[0]:
            L = [x[q] for x in laya]
            acc = lambda key: round(sum(x[key][0] == g for x, g in zip(L, gold)) / len(rows), 4)
            out.update({
                "laya_fwd_acc": acc("fwd"), "laya_rev_acc": acc("rev"), "laya_avg_acc": acc("avg"),
                "flip_rate": round(sum(x["fwd"][0] != x["rev"][0] for x in L) / len(rows), 4),
                "abstain_rate": round(sum(x["abstain"] for x in L) / len(rows), 4),
                "ece": ece([(x["avg"][1], float(x["avg"][0] == g)) for x, g in zip(L, gold)]),
                "acc_when_not_abstain": (lambda kept: round(sum(x["avg"][0] == g for x, g in kept) / len(kept), 4)
                                         if kept else None)([(x, g) for x, g in zip(L, gold) if not x["abstain"]]),
            })
        elif q == "session_action":
            out["laya"] = "rules-only (D2)"
        report["questions"][q] = out
    report["rules_misses"] = misses
    return report


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--labels", type=Path, default=Path(__file__).parent / "labels.jsonl")
    ap.add_argument("--all", action="store_true", help="include unreviewed rows")
    ap.add_argument("--no-laya", action="store_true")
    ap.add_argument("--json", type=Path)
    a = ap.parse_args(argv)
    rows = [json.loads(l) for l in a.labels.read_text().splitlines() if l.strip()]
    if not a.all:
        rows = [r for r in rows if r.get("reviewed")]
    rep = evaluate(rows, sr.load_config(), None if a.no_laya else sr.subprocess_backend)
    print(json.dumps(rep, indent=2))
    if a.json:
        a.json.write_text(json.dumps(rep, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
