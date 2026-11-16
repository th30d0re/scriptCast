"""E2: ask a local VLM to QA each frame. Usage: .venv-critic/bin/python run_qa.py MODEL OUT.jsonl [--max-side 1280]

Each frame is shown with its expected on-screen text (the scene-spec stand-in: Apple Vision
words at conf >= 50 from the approved frame). The model returns JSON:
{"verdict": "pass"|"fail", "defects": [{"category", "evidence", "region": [x, y, w, h]}]}.
"""
import argparse, json, re, time
from pathlib import Path

from mlx_vlm import load, generate
from mlx_vlm.prompt_utils import apply_chat_template
from mlx_vlm.utils import load_config
from PIL import Image

HERE = Path(__file__).parent
PROMPT = """You are the visual QA checker for an explainer video. Inspect this frame for rendering defects.
Defect categories:
- Layout: elements overlapping each other, text drawn on top of other text, misaligned blocks.
- Rendering: text or shapes cut off / clipped mid-word, partially drawn elements.
- Appearance: colours that break the palette (the palette is navy background, cream/gold text, muted red accents).
- State: expected text missing, or text duplicated somewhere it does not belong.
Expected on-screen text (from the scene spec, may be partial):
{expected}
Reply with JSON only, no prose:
{{"verdict": "pass" or "fail", "defects": [{{"category": "...", "evidence": "...", "region": [x, y, w, h]}}]}}
region is normalised 0-1 (fraction of width/height). Use "pass" with an empty list if the frame is clean."""


def expected_text():
    rows = [json.loads(l) for l in (HERE.parent / "e1_ocr_vlm/vision.jsonl").read_text().splitlines() if l.strip()]
    return {Path(r["path"]).name: " ".join(w["text"] for w in r["words"] if w["conf"] >= 50) for r in rows}


def parse(text):
    m = re.search(r"\{.*\}", text, re.S)
    if not m:
        return None
    try:
        return json.loads(m.group(0))
    except json.JSONDecodeError:
        return None


ap = argparse.ArgumentParser()
ap.add_argument("model"); ap.add_argument("out"); ap.add_argument("--max-side", type=int, default=1280); ap.add_argument("--reference", action="store_true", help="also show the approved still (image 1) before the candidate (image 2)")
args = ap.parse_args()
truth = [json.loads(l) for l in (HERE / "truth.jsonl").read_text().splitlines() if l.strip()]
exp = expected_text()
t0 = time.time(); model, processor = load(args.model); config = load_config(args.model); load_s = time.time() - t0
tmp = HERE / "_resized.png"
with open(args.out, "w") as fh:
    for i, t in enumerate(truth):
        im = Image.open(HERE / "set" / t["frame"]).convert("RGB"); im.thumbnail((args.max_side, args.max_side)); im.save(tmp)
        text = PROMPT.format(expected=exp[Path(t["frame"]).name][:1500] or "(none)")
        images = [str(tmp)]
        if args.reference:
            ref = Image.open(HERE / "set/clean" / Path(t["frame"]).name).convert("RGB"); ref.thumbnail((args.max_side, args.max_side))
            ref.save(HERE / "_ref.png"); images = [str(HERE / "_ref.png"), str(tmp)]
            text = ("Image 1 is the APPROVED reference render of this frame. Image 2 is the NEW render to check. "
                    "Any visible difference in image 2 from image 1 is a defect; report regions in image 2.\n") + text
        prompt = apply_chat_template(processor, config, text, num_images=len(images))
        s = time.time()
        res = generate(model, processor, prompt, images, max_tokens=500, temperature=0.0, verbose=False)
        raw = getattr(res, "text", res)
        row = dict(frame=t["frame"], raw=raw, parsed=parse(raw), seconds=round(time.time() - s, 2))
        if i == 0: row["load_seconds"] = round(load_s, 1)
        fh.write(json.dumps(row) + "\n"); fh.flush()
        print(i, t["frame"], row["seconds"], (row["parsed"] or {}).get("verdict"), flush=True)
