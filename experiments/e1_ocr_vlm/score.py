"""E1 scorer: compare a VLM transcription against Apple Vision OCR per frame.

recall  = share of Vision words (conf >= 50, len >= 3) that appear in the VLM text
extra   = VLM words (len >= 3) absent from all Vision words (any conf): candidate
          hallucinations OR things Vision missed; inspect by hand.
Usage: python3 score.py vlm.jsonl [vlm2.jsonl ...]
"""
import json, re, statistics, sys
from pathlib import Path

HERE = Path(__file__).parent
norm = lambda s: re.findall(r"[a-z0-9]{3,}", s.lower())
vision = {Path(r["path"]).name: r for r in map(json.loads, open(HERE / "vision.jsonl"))}

for f in sys.argv[1:]:
    rows = [json.loads(l) for l in open(f)]
    recalls, extras, secs, extra_words = [], [], [], []
    for r in rows:
        v = vision[Path(r["path"]).name]
        conf = set(w for x in v["words"] if x["conf"] >= 50 for w in norm(x["text"]))
        allv = set(w for x in v["words"] for w in norm(x["text"]))
        got = set(norm(r["text"]))
        recalls.append(len(conf & got) / len(conf) if conf else 1.0)
        ex = got - allv
        extras.append(len(ex))
        extra_words.append((Path(r["path"]).name, sorted(ex)[:12]))
        secs.append(r["seconds"])
    print(f"== {f}  load {rows[0].get('load_seconds')}s  frames {len(rows)}")
    print(f"recall mean {statistics.mean(recalls):.2f}  min {min(recalls):.2f}")
    print(f"extra words/frame mean {statistics.mean(extras):.1f}")
    print(f"sec/frame median {statistics.median(secs):.1f}  max {max(secs):.1f}")
    for name, ex in extra_words:
        if ex:
            print("  ", name, ex)
