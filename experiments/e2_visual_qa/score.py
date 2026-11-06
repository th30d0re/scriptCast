"""E2 scoring: detection recall on mutated, false-fail rate on clean, localisation, parse rate, speed."""
import json, sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).parent
truth = {t["frame"]: t for t in map(json.loads, (HERE / "truth.jsonl").read_text().splitlines())}


def overlaps(a, b):
    ax, ay, aw, ah = a; bx, by, bw, bh = b
    return ax < bx + bw and bx < ax + aw and ay < by + bh and by < ay + ah


for path in [p for p in sys.argv[1:] if not p.endswith("truth.jsonl")]:
    rows = [json.loads(l) for l in Path(path).read_text().splitlines() if l.strip()]
    parsed = sum(r["parsed"] is not None for r in rows)
    clean = [r for r in rows if truth[r["frame"]]["defect"] is None]
    mut = [r for r in rows if truth[r["frame"]]["defect"]]
    fails = lambda r: (r["parsed"] or {}).get("verdict") == "fail"
    ff = sum(map(fails, clean)) / len(clean)
    rec = sum(map(fails, mut)) / len(mut)
    loc = 0; per = defaultdict(lambda: [0, 0])
    for r in mut:
        k = truth[r["frame"]]["defect"]; per[k][1] += 1
        if fails(r):
            per[k][0] += 1
            regs = [d.get("region") for d in (r["parsed"] or {}).get("defects", []) if isinstance(d, dict)]
            if any(isinstance(g, list) and len(g) == 4 and all(isinstance(v, (int, float)) for v in g) and overlaps(g, truth[r["frame"]]["region"]) for g in regs):
                loc += 1
    secs = sorted(r["seconds"] for r in rows)
    print(f"{Path(path).stem}: parse {parsed}/{len(rows)}  recall {rec:.2f}  false-fail {ff:.2f}  localised {loc}/{len(mut)}  "
          f"p50 {secs[len(secs)//2]:.1f}s  load {rows[0].get('load_seconds')}s")
    print("   per type: " + "  ".join(f"{k} {a}/{b}" for k, (a, b) in sorted(per.items())))
