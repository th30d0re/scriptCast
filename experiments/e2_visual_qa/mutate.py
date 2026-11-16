"""E2: build the visual-QA set. 30 approved YT02 frames (from E1) + one mutated copy each.

Mutations use Apple Vision word boxes (E1 vision.jsonl) so each defect has a known region.
Types rotate per frame: overlap, clip, wrong_colour, remove, duplicate. Seeded, deterministic.
Writes set/clean/*.png, set/mutated/*.png and truth.jsonl.
"""
import json, random
from pathlib import Path
from PIL import Image, ImageOps

HERE = Path(__file__).parent
E1 = HERE.parent / "e1_ocr_vlm"
TYPES = ["overlap", "clip", "wrong_colour", "remove", "duplicate"]


def lines(words):
    """Group confident words into line boxes (same y within half a height)."""
    out = []
    for w in sorted((w for w in words if w["conf"] >= 50 and w["h"] >= 16), key=lambda w: (w["y"], w["x"])):
        for L in out:
            if abs(L["y"] - w["y"]) < w["h"] / 2 and w["x"] - (L["x"] + L["w"]) < 3 * w["h"]:
                x2 = max(L["x"] + L["w"], w["x"] + w["w"]); y2 = max(L["y"] + L["h"], w["y"] + w["h"])
                L["x"], L["y"] = min(L["x"], w["x"]), min(L["y"], w["y"]); L["w"], L["h"] = x2 - L["x"], y2 - L["y"]
                break
        else:
            out.append(dict(x=w["x"], y=w["y"], w=w["w"], h=w["h"]))
    return [L for L in out if L["w"] >= 40]


def bg(im, b):
    x, y = max(0, b["x"] - 6), max(0, b["y"] - 6)
    return im.getpixel((x, y))


def mutate(im, kind, boxes, rng):
    W, H = im.size
    b = rng.choice(boxes)
    pad = 4
    box = (b["x"] - pad, b["y"] - pad, b["x"] + b["w"] + pad, b["y"] + b["h"] + pad)
    patch = im.crop(box)
    if kind == "overlap":
        others = [o for o in boxes if o is not b] or [b]
        o = rng.choice(others)
        dx, dy = o["x"] + o["w"] // 3, o["y"] + o["h"] // 3
        im.paste(patch, (dx, dy)); region = (dx, dy, patch.width, patch.height)
    elif kind == "clip":
        cut = b["x"] + b["w"] // 2
        im.paste(bg(im, b), (cut, box[1], box[2], box[3])); region = (cut, box[1], box[2] - cut, box[3] - box[1])
    elif kind == "wrong_colour":
        rgb = patch.convert("RGB"); r, g, bl = rgb.split()
        im.paste(Image.merge("RGB", (bl, r, g)), box[:2]); region = (box[0], box[1], patch.width, patch.height)
    elif kind == "remove":
        im.paste(bg(im, b), box); region = (box[0], box[1], patch.width, patch.height)
    else:  # duplicate
        for _ in range(50):
            dx, dy = rng.randrange(40, W - patch.width - 40), rng.randrange(40, H - patch.height - 40)
            if not any(dx < o["x"] + o["w"] and o["x"] < dx + patch.width and dy < o["y"] + o["h"] and o["y"] < dy + patch.height for o in boxes):
                break
        im.paste(patch, (dx, dy)); region = (dx, dy, patch.width, patch.height)
    return im, [round(region[0] / W, 4), round(region[1] / H, 4), round(region[2] / W, 4), round(region[3] / H, 4)]


def main():
    rng = random.Random(2)
    (HERE / "set/clean").mkdir(parents=True, exist_ok=True); (HERE / "set/mutated").mkdir(parents=True, exist_ok=True)
    truth = []
    rows = [json.loads(l) for l in (E1 / "vision.jsonl").read_text().splitlines() if l.strip()]
    for i, r in enumerate(rows):
        src = E1 / "frames" / Path(r["path"]).name
        boxes = lines(r["words"])
        im = Image.open(src).convert("RGB")
        im.save(HERE / "set/clean" / src.name)
        truth.append(dict(frame=f"clean/{src.name}", defect=None))
        if not boxes:
            continue
        kind = TYPES[i % len(TYPES)]
        m, region = mutate(im.copy(), kind, boxes, rng)
        m.save(HERE / "set/mutated" / src.name)
        truth.append(dict(frame=f"mutated/{src.name}", defect=kind, region=region))
    (HERE / "truth.jsonl").write_text("".join(json.dumps(t) + "\n" for t in truth))
    print(len(truth), "frames;", sum(t["defect"] is not None for t in truth), "mutated")


if __name__ == "__main__":
    main()
