from PIL import Image, ImageDraw
from scriptcast.critic.s1 import features, score, judge_frame

def test_identical_feature_math():
    im=Image.new('RGB',(20,20),'white')
    f=features(im,im)
    assert f['ssim'] > .999 and f['mean_abs_diff'] == 0 and f['edge_diff'] == 0 and f['palette_distance'] == 0

def test_changed_crop_scores_higher():
    a=Image.new('RGB',(30,30),'white'); b=a.copy(); ImageDraw.Draw(b).rectangle((5,5,24,24),fill='black')
    assert score(features(a,b)) > score(features(a,a))

def test_judge_frame_shape(tmp_path):
    a=Image.new('RGB',(30,30),'white'); b=a.copy(); ImageDraw.Draw(b).rectangle((2,2,15,15),fill='black')
    ap=tmp_path/'a.png'; bp=tmp_path/'b.png'; a.save(ap); b.save(bp)
    result=judge_frame(ap,bp,[{'element_id':'left','x':0,'y':0,'w':.6,'h':.6},{'element_id':'right','x':.6,'y':.6,'w':.3,'h':.3}])
    assert set(result)=={'verdict','elements'} and result['elements'][0]['element_id']=='left'
