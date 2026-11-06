"""Runs inside .venv-router (laya-mlx 0.6.0). Imports nothing from scriptcast.

stdin: {"model": str, "questions": {...}, "cards": [str, ...]}
stdout: {"version": str, "load_ms": float,
         "results": [{"fwd": answers, "rev": answers, "ms": float, "tokens": int}]}
"rev" uses reversed option order; score levels are reversed and mapped back.
"""
import json
import sys
import time


def reverse_questions(qs):
    out = {}
    for qid, q in qs.items():
        q = dict(q)
        c = q["criteria"]
        q["criteria"] = dict(reversed(list(c.items()))) if isinstance(c, dict) else list(reversed(c))
        out[qid] = q
    return out


def unreverse_score(ans, n):
    ans = dict(ans)
    probs = ans.get("probabilities", {})
    ans["probabilities"] = {str(n - 1 - int(k)): v for k, v in probs.items()}
    if "score" in ans:
        ans["score"] = (n - 1) - ans["score"]
    return ans


def main():
    req = json.load(sys.stdin)
    import laya_mlx as laya

    t0 = time.perf_counter()
    agent = laya.load(req["model"])
    load_ms = (time.perf_counter() - t0) * 1000
    qs = req["questions"]
    rq = reverse_questions(qs)
    if req["cards"]:
        agent.predict(req["cards"][0], qs)  # warm-up, not timed
    results = []
    for card in req["cards"]:
        t = time.perf_counter()
        fwd = agent.predict(card, qs)
        rev = agent.predict(card, rq)
        ms = (time.perf_counter() - t) * 1000
        ra = rev["answers"]
        for qid, q in qs.items():
            if q["type"] == "score" and qid in ra:
                ra[qid] = unreverse_score(ra[qid], len(q["criteria"]))
        results.append({"fwd": fwd["answers"], "rev": ra, "ms": ms,
                        "tokens": fwd.get("usage", {}).get("input_tokens")})
    version = getattr(laya, "__version__", "0.6.0")
    json.dump({"version": version, "load_ms": load_ms, "results": results}, sys.stdout)


if __name__ == "__main__":
    main()
