import React, {useEffect, useRef, useState} from 'react';
import '@fontsource/ibm-plex-sans/400.css';
import '@fontsource/ibm-plex-sans/500.css';
import '@fontsource/ibm-plex-sans/600.css';
import '@fontsource/ibm-plex-mono/400.css';
import '@fontsource/ibm-plex-mono/500.css';
// react-dom/client types are supplied locally for this dependency-light workspace.
import {createRoot} from 'react-dom/client';
import {ManualClock} from '../src/stage/clock';
import {StageContent, sceneWarnings} from '../src/stage/StageEpisode';
import {clamp} from '../src/stage/motion';
import {ReviewChrome, ReviewHitSurface, ReviewSession} from '../src/stage/review/ReviewMode';
import type {StagePlan} from '../src/stage/types';
import './style.css';
function Player({plan}: {plan: StagePlan}) {
  const [mode,setMode] = useState<'scroll'|'play'>('scroll'), [ms,setMs] = useState(plan.range.start_ms), [playing,setPlaying] = useState(false), [pps,setPps] = useState(120), [width,setWidth] = useState(window.innerWidth), [audioError,setAudioError] = useState(''), [review,setReviewOn] = useState(false);
  const audio = useRef<HTMLAudioElement>(null), track = useRef<HTMLDivElement>(null);
  const start = plan.range.start_ms, end = plan.range.end_ms, duration = end-start;
  const shots = plan.segments.flatMap(s => s.shots), gutter = NAV_W + (review ? 304 : 0), scale = Math.max(1, width-gutter)/plan.width, viewportHeight = plan.height*scale;
  const seek = (value: number) => {const time = clamp(value,start,end-0.001); setMs(time); if (audio.current) audio.current.currentTime = time/1000; if (mode === 'scroll' && track.current) window.scrollTo({top:track.current.offsetTop+(time-start)*pps/1000});};
  const setReview = (on: boolean) => {if (on) audio.current?.pause(); setReviewOn(on);};
  const toggle = () => {const el=audio.current; if (!el || !plan.audio) return; if (el.paused) {if(ms>=end-1) el.currentTime=start/1000; void el.play().catch(e=>setAudioError(String(e)));} else el.pause();};
  useEffect(() => {const resize=()=>setWidth(window.innerWidth); window.addEventListener('resize',resize); return ()=>window.removeEventListener('resize',resize);},[]);
  useEffect(() => {
    if(mode !== 'scroll') return;
    const scroll=()=>{if(track.current) setMs(clamp(start+(window.scrollY-track.current.offsetTop)*1000/pps,start,end-0.001));};
    window.addEventListener('scroll',scroll,{passive:true}); scroll(); return ()=>window.removeEventListener('scroll',scroll);
  },[mode,pps,start,end]);
  useEffect(() => {
    if(mode!=='play') return;
    let handle=0;
    const tick=()=>{const el=audio.current; if(el && !el.paused) {if(el.currentTime*1000>=end){el.pause();el.currentTime=end/1000;setMs(end-0.001);} else setMs(Math.max(start,el.currentTime*1000));} handle=requestAnimationFrame(tick);};
    handle=requestAnimationFrame(tick); return ()=>cancelAnimationFrame(handle);
  },[mode,start,end]);
  useEffect(() => {const key=(e:KeyboardEvent)=>{if(mode!=='play' || /INPUT|BUTTON|SELECT|TEXTAREA/.test((e.target as HTMLElement).tagName))return; if(e.code==='Space'){e.preventDefault();toggle();} if(e.code==='ArrowRight'||e.code==='ArrowLeft'){e.preventDefault();seek(ms+(e.code==='ArrowRight'?5000:-5000));}};window.addEventListener('keydown',key);return()=>window.removeEventListener('keydown',key);});
  function switchMode(next:'scroll'|'play') {audio.current?.pause(); if(audio.current) audio.current.currentTime=ms/1000; setMode(next); if(next==='play') window.scrollTo(0,0); if(next==='scroll') setTimeout(()=>{if(track.current)window.scrollTo(0,track.current.offsetTop+(ms-start)*pps/1000);},0);}
  const warnings=[...sceneWarnings(plan),...(audioError?[audioError]:[])];
  return <ReviewSession plan={plan} ms={ms} seek={seek} enabled={review} onEnabled={setReview}><header><strong>{plan.episode}</strong><button aria-pressed={mode==='scroll'} onClick={()=>switchMode('scroll')}>Scroll</button><button aria-pressed={mode==='play'} onClick={()=>switchMode('play')}>Play</button><button aria-pressed={review} title="Review (R)" onClick={()=>setReview(!review)}>Review</button>{mode==='scroll' && <label>px / second <input aria-label="Scroll speed" type="number" min="20" max="500" value={pps} onChange={e=>setPps(clamp(Number(e.target.value),20,500))}/></label>}<span>{(ms/1000).toFixed(1)}s</span></header>
  <audio ref={audio} src={plan.audio?`/${plan.audio}`:undefined} preload="metadata" onLoadedMetadata={()=>{if(audio.current)audio.current.currentTime=start/1000;}} onPlay={()=>setPlaying(true)} onPause={()=>setPlaying(false)} onError={()=>plan.audio&&setAudioError('Episode audio could not be loaded.')} onEnded={()=>setPlaying(false)}/>
  <div ref={track} style={{height:mode==='scroll'?duration*pps/1000+viewportHeight:viewportHeight}}><div className="viewport" style={{height:viewportHeight,marginLeft:gutter,position:mode==='scroll'?'sticky':'relative',top:0}}><div style={{position:'relative',width:plan.width,height:plan.height,transform:`scale(${scale})`,transformOrigin:'top left'}}><ManualClock ms={ms} fps={plan.fps}><StageContent plan={plan}/></ManualClock><ReviewHitSurface /></div></div></div>
  <nav aria-label="Shots" style={{right:review?304:0,width:NAV_W}}>{shots.map(s=><button key={s.shot_id} title={String(s.props.title??s.shot_id)} aria-current={ms>=s.start_ms&&ms<s.end_ms?'step':undefined} onClick={()=>seek(s.start_ms)}>{String(s.props.title??s.shot_id)}</button>)}</nav>
  {mode==='play' && <footer><button disabled={!plan.audio} onClick={toggle}>{playing?'Pause':'Play'}</button><input aria-label="Episode position" type="range" min={start} max={end-0.001} step="1" value={ms} list="shots" onChange={e=>seek(Number(e.target.value))}/><datalist id="shots">{shots.map(s=><option key={s.shot_id} value={s.start_ms} label={s.shot_id}/>)}</datalist></footer>}
  {warnings.length>0&&<details className="warnings"><summary>{warnings.length} notices</summary>{warnings.map((w,i)=><p key={i}>{w}</p>)}</details>}
  <ReviewChrome /></ReviewSession>;
}
const NAV_W = 96;
function App(){const [plan,setPlan]=useState<StagePlan|null>(null),[error,setError]=useState('');useEffect(()=>{const url=new URLSearchParams(location.search).get('plan')??'/stage/default.json';fetch(url).then(r=>{if(!r.ok)throw new Error(`Plan HTTP ${r.status}`);return r.json();}).then(setPlan).catch(e=>setError(String(e)));},[]);return error?<p role="alert">{error}</p>:plan?<Player plan={plan}/>:<p>Loading Stage…</p>;}
createRoot(document.getElementById('root')!).render(<App/>);
