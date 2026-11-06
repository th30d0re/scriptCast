"""E1: transcribe on-screen text in each frame with a local VLM (mlx-vlm).

Usage: .venv-critic/bin/python experiments/e1_ocr_vlm/run_vlm.py MODEL_ID OUT.jsonl [--max-side 1280]
Writes one JSON object per frame: path, text, seconds, load_seconds (first row only).
"""
import argparse, json, time
from pathlib import Path

from mlx_vlm import load, generate
from mlx_vlm.prompt_utils import apply_chat_template
from mlx_vlm.utils import load_config
from PIL import Image

PROMPT = (
    "Transcribe every piece of text visible in this video frame, exactly as written, "
    "including small labels, axis ticks, captions and equations. One line per text block, "
    "top to bottom, left to right. Output only the text, nothing else."
)

ap = argparse.ArgumentParser()
ap.add_argument("model")
ap.add_argument("out")
ap.add_argument("--max-side", type=int, default=1280)
args = ap.parse_args()

frames = sorted((Path(__file__).parent / "frames").glob("*.png"))
t0 = time.time()
model, processor = load(args.model)
config = load_config(args.model)
load_s = time.time() - t0
prompt = apply_chat_template(processor, config, PROMPT, num_images=1)
tmp = Path(__file__).parent / "_resized.png"

with open(args.out, "w") as fh:
    for i, f in enumerate(frames):
        im = Image.open(f).convert("RGB")
        im.thumbnail((args.max_side, args.max_side))
        im.save(tmp)
        t = time.time()
        res = generate(model, processor, prompt, [str(tmp)], max_tokens=400, temperature=0.0, verbose=False)
        text = res.text if hasattr(res, "text") else str(res)
        row = {"path": str(f), "text": text, "seconds": round(time.time() - t, 2)}
        if i == 0:
            row["load_seconds"] = round(load_s, 1)
        fh.write(json.dumps(row) + "\n")
        fh.flush()
        print(f.name, row["seconds"], flush=True)
tmp.unlink(missing_ok=True)
