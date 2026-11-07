"""Train small L2 logistic regression with frame-level held-out split."""
from __future__ import annotations
import json, sys, time
from pathlib import Path
import numpy as np
from PIL import Image
HERE=Path(__file__).parent; sys.path.insert(0,str(HERE.parent.parent))
from scriptcast.critic.s1 import score, features
def main():
 d=np.load(HERE/'pairs.npz'); X,y,frame,names=d['X'],d['y'].astype(float),d['frame'],list(d['names'])
 test=(frame % 5)==0; train=~test; mean=X[train].mean(0); scale=X[train].std(0)+1e-7; Z=(X-mean)/scale
 # Class balanced weighted gradient descent: useful because most line crops are clean.
 w=np.zeros(Z.shape[1]); b=0.; pos=max(y[train].sum(),1); neg=max(train.sum()-pos,1); weights=np.where(y==1, .5/pos, .5/neg)*train.sum()
 for _ in range(2500):
  p=1/(1+np.exp(-np.clip(Z[train]@w+b,-40,40))); err=(p-y[train])*weights[train]
  w-=.08*((Z[train].T@err)/train.sum()+.02*w); b-=.08*err.mean()
 model={'intercept':float(b),'weights':{str(n):float(v) for n,v in zip(names,w)},'mean':{str(n):float(v) for n,v in zip(names,mean)},'scale':{str(n):float(v) for n,v in zip(names,scale)},'threshold':.5}
 (HERE.parent.parent/'scriptcast/critic/s1_weights.json').write_text(json.dumps(model,indent=2)+'\n')
 p=1/(1+np.exp(-np.clip(Z[test]@w+b,-40,40))); pred=p>=.5; truth=y[test]==1
 precision=(pred&truth).sum()/max(pred.sum(),1); recall=(pred&truth).sum()/max(truth.sum(),1); ff=(pred&~truth).sum()/max((~truth).sum(),1)
 a=Image.new('RGB',(120,32),'white'); c=a.copy(); c.putpixel((4,4),(0,0,0))
 start=time.perf_counter(); [features(a,c) for _ in range(500)]; ms=(time.perf_counter()-start)*1000/500
 print(json.dumps({'train':int(train.sum()),'test':int(test.sum()),'precision':round(float(precision),3),'recall':round(float(recall),3),'false_fail':round(float(ff),3),'ms_element':round(ms,4)}))
if __name__=='__main__': main()
