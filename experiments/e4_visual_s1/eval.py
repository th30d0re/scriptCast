"""Evaluate S1 on E2 and list S1 scores beside E3 author judgements."""
from __future__ import annotations
import csv,json,sys,time
from pathlib import Path
HERE=Path(__file__).parent; ROOT=Path('/Users/emmanuel/Dev/Tools/scriptCast'); sys.path.insert(0,str(HERE.parent.parent))
from scriptcast.critic.s1 import judge_frame
sys.path.insert(0,str(HERE.parent/'e2_visual_qa')); from mutate import lines
def iou(a,b):
 ax,ay,aw,ah=a; bx,by,bw,bh=b; z=max(0,min(ax+aw,bx+bw)-max(ax,bx))*max(0,min(ay+ah,by+bh)-max(ay,by)); return z/(aw*ah+bw*bh-z+1e-9)
def main():
 e1=ROOT/'experiments/e1_ocr_vlm'; lookup={Path((r:=json.loads(x))['path']).name:lines(r['words']) for x in (e1/'vision.jsonl').read_text().splitlines() if x}
 e2=ROOT/'experiments/e2_visual_qa'; truth=[json.loads(x) for x in (e2/'truth.jsonl').read_text().splitlines() if x]
 recall=ff=loc=defects=clean=0; timings=[]
 for t in truth:
  name=Path(t['frame']).name; boxes=[dict(x=b['x']/1920,y=b['y']/1080,w=b['w']/1920,h=b['h']/1080,element_id=str(i)) for i,b in enumerate(lookup[name])]
  start=time.perf_counter(); result=judge_frame(e2/'set/clean'/name,e2/'set'/t['frame'],boxes); timings.append((time.perf_counter()-start)*1000/max(len(boxes),1))
  failed=result['verdict']=='fail'
  if t['defect']:
   defects+=1; recall+=failed
   top=result['elements'][0]['element_id'] if result['elements'] else '-'; b=boxes[int(top)] if top!='-' else {'x':0,'y':0,'w':0,'h':0}; loc+=bool(iou([b[k] for k in ('x','y','w','h')],t['region'])>.05)
  else: clean+=1; ff+=failed
 print('E2 frame_recall=%.3f false_fail=%.3f localisation=%.3f ms_element=%.3f' % (recall/max(defects,1),ff/max(clean,1),loc/max(defects,1),sum(timings)/max(len(timings),1)))
 ranks={r['cue']:r['top'] for r in csv.DictReader((ROOT/'experiments/svg_offload/author_rank.csv').open())}; judge=ROOT/'experiments/svg_offload/judge'
 print('\ncue\tworker\tauthor_top\tp_defect')
 for new in sorted(judge.glob('*/new.png')):
  cue=new.parent.name.rsplit('-B-',1)[0]; ref=judge/'ref'/f'{cue}.png'
  if not ref.exists(): continue
  r=judge_frame(ref,new,[{'x':0,'y':0,'w':1,'h':1,'element_id':'frame'}]); print(f'{cue}\t{new.parent.name.split("-B-",1)[1]}\t{ranks.get(cue,"")}\t{r["elements"][0]["p_defect"]:.3f}')
if __name__=='__main__': main()
