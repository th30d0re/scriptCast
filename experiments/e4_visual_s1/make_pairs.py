"""Create a deterministic ~2k System-1 crop-pair feature set from E1 frames."""
from __future__ import annotations
import json, random, sys
from io import BytesIO
from pathlib import Path
import numpy as np
from PIL import Image

HERE = Path(__file__).parent; ROOT = Path("/Users/emmanuel/Dev/Tools/scriptCast")
sys.path.insert(0, str(HERE.parent.parent))
from scriptcast.critic.s1 import features
sys.path.insert(0, str(HERE.parent / "e2_visual_qa"))
from mutate import lines, mutate

def crop(im, b):
    p=4; return im.crop((max(0,b['x']-p),max(0,b['y']-p),min(im.width,b['x']+b['w']+p),min(im.height,b['y']+b['h']+p)))
def overlap(a, r, W, H):
    ax,ay,aw,ah=a['x']/W,a['y']/H,a['w']/W,a['h']/H; bx,by,bw,bh=r
    return max(0,min(ax+aw,bx+bw)-max(ax,bx))*max(0,min(ay+ah,by+bh)-max(ay,by)) / max(1e-9,aw*ah)
def benign(im, rng):
    kind=rng.randrange(3)
    if kind == 0: return im.transform(im.size, Image.Transform.AFFINE, (1,0,rng.choice([-1,1]),0,1,rng.choice([-1,1])))
    if kind == 1:
        f=BytesIO(); im.save(f, 'JPEG', quality=90); return Image.open(BytesIO(f.getvalue())).convert('RGB')
    gamma=rng.choice([.96,1.04]); a=np.asarray(im,dtype=float)/255; return Image.fromarray(np.uint8(np.clip(a**gamma,0,1)*255))
def main():
    out=HERE/'pairs.npz'; meta=HERE/'pairs.jsonl'; rng=random.Random(41)
    e1=ROOT/'experiments/e1_ocr_vlm'; rows=[json.loads(x) for x in (e1/'vision.jsonl').read_text().splitlines() if x]
    xs=[]; ys=[]; frames=[]; records=[]
    # Every mutation has all line crops: positives are only crops hit by its known region.
    for seed in range(80):
      for index,row in enumerate(rows):
        im=Image.open(e1/'frames'/Path(row['path']).name).convert('RGB'); boxes=lines(row['words'])
        if not boxes: continue
        local=random.Random(seed*1000+index); changed,region=mutate(im.copy(), ['overlap','clip','wrong_colour','remove','duplicate'][seed%5], boxes, local)
        for b in boxes:
          y=int(overlap(b,region,im.width,im.height) > .08)
          xs.append([features(crop(im,b),crop(changed,b)).get(k,0.) for k in ('ssim','mean_abs_diff','edge_diff','palette_distance')]); ys.append(y); frames.append(index)
          records.append({'source_frame':Path(row['path']).name,'seed':seed,'label':y,'kind':'defect'})
          if len(xs)>=2000: break
        # Benign pairs explicitly teach render tolerance.
        for b in boxes[:2]:
          xs.append([features(crop(im,b),crop(benign(im,local),b)).get(k,0.) for k in ('ssim','mean_abs_diff','edge_diff','palette_distance')]); ys.append(0); frames.append(index); records.append({'source_frame':Path(row['path']).name,'seed':seed,'label':0,'kind':'benign'})
        if len(xs)>=2000: break
      if len(xs)>=2000: break
    np.savez_compressed(out, X=np.asarray(xs,np.float32), y=np.asarray(ys,np.int8), frame=np.asarray(frames,np.int16), names=np.asarray(['ssim','mean_abs_diff','edge_diff','palette_distance']))
    meta.write_text(''.join(json.dumps(r)+'\n' for r in records))
    print(f'{len(xs)} pairs; positives={sum(ys)}; {out}')
if __name__ == '__main__': main()
