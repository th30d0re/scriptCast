import json
from pathlib import Path
import pytest
from scriptcast.tools.stage_plan import build_plan, normalize_words, resolve_cue, compile_plan

@pytest.fixture
def data():
    manifest = {'turns':[{'turn_id':'a','turn_index':0,'start_ms':10000,'end_ms':14000}, {'turn_id':'b','turn_index':1,'start_ms':15000,'end_ms':19000}, {'turn_id':'c','turn_index':2,'start_ms':20000,'end_ms':24000}]}
    words = {'turns':{t['turn_id']:[{'text':w,'start_ms':i*300,'end_ms':i*300+200} for i,w in enumerate(['“Tripartite','MODEL,”','Episode','episode!'])] for t in manifest['turns']}}
    spec = {'episode':'test','shots':[{'shot_id':f'S{i}','turn_id':t['turn_id'],'scene':'Demo','instance':'same' if i<2 else 'next','cues':{'shared':{'word':'episode'},f'unique{i}':{'ms':0}}} for i,t in enumerate(manifest['turns'])]}
    return spec,manifest,words

@pytest.mark.parametrize('cue,expected', [({'word':'TRIPARTITE'},10000),({'word':'tripartite model'},10000),({'word':'tripartite model','at':'end'},10500),({'word':'episode','occurrence':2},10900),({'word':'episode','at':'end','offset_ms':-120},10680),({'ms':9000},19000)])
def test_cues(data,cue,expected):
    _,manifest,raw=data
    words,_=normalize_words(manifest,raw)
    assert resolve_cue(cue,manifest['turns'][0],words['a'])==expected

def test_missing(data):
    _,manifest,raw=data
    words,_=normalize_words(manifest,raw)
    with pytest.raises(ValueError,match='Missing word.*tokens: tripartite model episode episode'):
        resolve_cue({'word':'absent'},manifest['turns'][0],words['a'])

def test_plan(data):
    plan=build_plan(*data)
    assert len(plan['segments'])==2
    assert len(plan['segments'][0]['shots'])==2
    assert plan['segments'][0]['shots'][0]['end_ms']==15000
    assert plan['range']=={'start_ms':10000,'end_ms':24800}
    assert 'shared' not in plan['cues']
    assert plan['cues']['S0.shared']==10600
    assert plan['cues']['unique0']==10000

def test_range(data):
    plan=build_plan(*data,shot_range='S1..S1')
    assert plan['range']=={'start_ms':15000,'end_ms':20000}
    assert plan['duration_ms']==24800
    assert len(plan['segments'][0]['shots'])==1
    with pytest.raises(ValueError,match='Invalid shot range'):build_plan(*data,shot_range='S2..S0')

def test_normalization(data):
    _,manifest,raw=data
    relative,mode=normalize_words(manifest,raw)
    assert mode=='relative'
    absolute,mode=normalize_words(manifest,{'turns':relative})
    assert mode=='absolute'
    assert relative==absolute

def test_demo():
    path=Path(__file__).parents[1]/'video/episodes/_demo/stage.yaml'
    plan=compile_plan(path)
    assert plan['cues']['portugal']==3400
    assert len(plan['segments'])==2

def test_real():
    root=Path('/Users/emmanuel/Documents/Theory/TheOriginalPower/outputs/YT02v2_ov')
    if not all((root/n).exists() for n in ('episode_manifest.json','word_timings.json')):pytest.skip('Real episode not present')
    manifest=json.loads((root/'episode_manifest.json').read_text())
    timings=json.loads((root/'word_timings.json').read_text())
    words,mode=normalize_words(manifest,timings)
    assert mode=='relative'
    starts={t['turn_id']:t['start_ms'] for t in manifest['turns']}
    assert words['a57c5a84a6c02ce8'][0]['start_ms']==starts['a57c5a84a6c02ce8']
    first=manifest['turns'][0]['turn_id']
    eleven=next(w for w in words[first] if w['text'].lower().strip('.,')=='eleven')
    spec={'episode':'YT02v2_ov','shots':[{'shot_id':'S00-01','turn_id':first,'scene':'Title','cues':{'slam':{'word':'eleven'},'early':{'word':'eleven','offset_ms':-120}}}]}
    plan=build_plan(spec,manifest,timings)
    assert plan['cues']['slam']==eleven['start_ms']
    assert plan['cues']['early']==eleven['start_ms']-120

def test_cli_dry_runs(data,tmp_path,capsys,monkeypatch):
    from scriptcast.tools.stage import main
    plan=build_plan(*data)
    (tmp_path/'stage.yaml').write_text('episode: test')
    plan['episode_dir']=str(tmp_path)
    source=tmp_path/'plan.json';source.write_text(json.dumps(plan))
    assert main(['render',str(source),'-o',str(tmp_path/'out.mp4'),'--dry-run'])==0
    output=capsys.readouterr().out
    assert 'STAGE_EPISODE_DIR=' in output and 'src/stage/index.ts StageEpisode' in output
    assert main(['render',str(source),'--per-shot',str(tmp_path),'--dry-run'])==0
    output=capsys.readouterr().out
    assert 'S0.mp4' in output and '--frames=0-149' in output and '--muted' in output
    assert main(['still',str(source),'--shot','S0','--at','0.5','-o',str(tmp_path/'x.png'),'--dry-run'])==0
    assert '--frame=75' in capsys.readouterr().out

def test_reject_cli_appends_a_review_record(data, tmp_path, capsys):
    from scriptcast.tools.stage import main
    plan = build_plan(*data)
    (tmp_path/'stage.yaml').write_text('episode: test')
    plan['episode_dir'] = str(tmp_path)
    source = tmp_path/'plan.json'
    source.write_text(json.dumps(plan))
    still = tmp_path/'S0.png'
    still.write_bytes(b'png fixture')
    assert main(['reject', str(source), '--shot', 'S0', '--reason', 'text overlaps', '--still', str(still), '--frame', '44']) == 0
    record = json.loads((tmp_path/'review/rejections.jsonl').read_text())
    assert record['shot'] == 'S0'
    assert record['still_path'] == str(still.resolve())
    assert record['frame'] == 44
    assert record['reason'] == 'text overlaps'
    assert record['ts'].endswith('Z')

def test_rejection_writer_rejects_blank_reason(tmp_path):
    from scriptcast.tools.stage import append_rejection
    with pytest.raises(ValueError, match='reason'):
        append_rejection(tmp_path, shot='S0', still_path=tmp_path/'S0.png', frame=1, reason='  ')

def test_staging_audio(tmp_path):
    import shutil
    demo=Path(__file__).parents[1]/'video/episodes/_demo'
    for name in ('stage.yaml','episode_manifest.json','word_timings.json'):
        shutil.copyfile(demo/name,tmp_path/name)
    (tmp_path/'demo.mp3').write_bytes(b'synthetic audio fixture')
    public=tmp_path/'public'
    plan=compile_plan(tmp_path/'stage.yaml',public_dir=public)
    assert plan['audio']=='stage/demo/demo.mp3'
    assert (public/plan['audio']).read_bytes()==b'synthetic audio fixture'

def test_chunk_dry_run(data,tmp_path,capsys):
    from scriptcast.tools.stage import main
    plan=build_plan(*data)
    (tmp_path/'stage.yaml').write_text('episode: test')
    plan['episode_dir']=str(tmp_path)
    plan['audio']='stage/test/test.mp3'
    source=tmp_path/'plan.json';source.write_text(json.dumps(plan))
    assert main(['render',str(source),'-o',str(tmp_path/'out.mp4'),'--chunks','2','--dry-run'])==0
    output=capsys.readouterr().out
    assert '--frames=0-221' in output and '--frames=222-443' in output
    assert '-ss 10.0' in output and '-t 14.8' in output
