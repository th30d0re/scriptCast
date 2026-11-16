"""Pre-fill router labels from the dispatch log, review notes and Stage briefs.

Every row gets a guessed label and `reviewed: false`. The operator corrects
them in review.py. Re-running keeps rows already reviewed (matched by id).
"""
import json, re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(__file__).parent / "labels.jsonl"
LOG = Path.home() / "Library/Mobile Documents/iCloud~md~obsidian/Documents/Root/Orchestration/dispatch-log.jsonl"

LANES = ["tsx_scene", "figcraft_layout", "svg_asset", "on_screen_text", "audit_sweep", "research"]


def guess(text: str, kind: str = "") -> dict:
    t = text.lower()
    if kind in ("research",) or re.search(r"research|fact-check|measure .* teacher", t):
        lane = "research"
    elif kind in ("review", "verify") or re.search(r"audit|verif|sweep", t):
        lane = "audit_sweep"
    elif re.search(r"svg|icon|vector", t):
        lane = "svg_asset"
    elif re.search(r"layout|gap|crowd|overlap|safe-area|figcraft", t):
        lane = "figcraft_layout"
    elif re.search(r"caption|text|typo|wording|label", t):
        lane = "on_screen_text"
    else:
        lane = "tsx_scene"
    tier = "mechanical" if re.search(r"gap|lift|nudge|rename|typo", t) else (
        "judgment" if re.search(r"spike|architecture|audit|engine|calibrat", t) else "standard")
    action = "resume_builder" if re.search(r"\bfix|resume|\b\db\b|phase \d+b", t) else "spawn"
    ctx = "chapter" if re.search(r"yt0\d|episode|scene|cue", t) else "none"
    return {"lane": lane, "tier": tier, "session_action": action, "context_source": ctx}


rows = []
for i, line in enumerate(LOG.read_text().splitlines()):
    r = json.loads(line)
    text = f"[{r['project']}] {r['kind']}: {r['task']}"
    rows.append({"id": f"log-{i:02d}", "source": "dispatch-log", "card": text,
                 "ran_with": f"{r['agent']} {r['model']}", "outcome": r.get("verdict"),
                 "note": r.get("note", ""), **guess(text, r["kind"])})

for i, n in enumerate(json.loads((ROOT / "review/notes.json").read_text())):
    text = f"[{n['scene']} / {n['element']} / {n['cue']}] {n['note']}"
    rows.append({"id": f"note-{i:02d}", "source": "review/notes.json", "card": text,
                 "ran_with": "", "outcome": n.get("status"), "note": "", **guess(text)})

for p in sorted((ROOT / "docs/briefs").glob("*.md")):
    if re.search(r"findings|lastmsg", p.name):
        continue
    body = p.read_text()
    title = next((l.lstrip("# ").strip() for l in body.splitlines() if l.startswith("#")), p.stem)
    first = " ".join(l.strip() for l in body.splitlines()[1:12] if l.strip() and not l.startswith("#"))[:300]
    text = f"[brief {p.name}] {title}. {first}"
    rows.append({"id": f"brief-{p.stem}", "source": f"docs/briefs/{p.name}", "card": text,
                 "ran_with": "", "outcome": "", "note": "", **guess(text, "implement")})

kept = {}
if OUT.exists():
    for l in OUT.read_text().splitlines():
        r = json.loads(l)
        if r.get("reviewed"):
            kept[r["id"]] = r
with OUT.open("w") as fh:
    for r in rows:
        fh.write(json.dumps(kept.get(r["id"], {**r, "reviewed": False})) + "\n")
print(f"{len(rows)} rows -> {OUT} ({len(kept)} reviewed rows kept)")
