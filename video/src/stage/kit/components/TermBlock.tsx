import React from 'react';
import {color} from '../theme/tokens';
import type {Accent, TermBlockGlyph, TermState} from './types';
import {DrawRect} from '../motion/DrawRect';
import {clamp01, cubicOut, type Timing, useTiming} from '../motion/core';

export type TermBlockProps = Timing & {
  term: string;
  summary?: string;
  glyph: TermBlockGlyph;
  state: TermState;
  accent: Accent;
  unit?: string;
};

const stateAmount = (state: TermState) => typeof state === 'number'
  ? Math.max(0, Math.min(1, state))
  : state === 'hidden' ? 0 : state === 'dim' ? .42 : state === 'active' ? .82 : 1;

export function TermBlock({term, summary, glyph, state, accent, unit, progress, startFrame, durationFrames}: TermBlockProps) {
  const amount=stateAmount(state), selected=state === 'selected';
  const duration=durationFrames??36, {p:timingProgress}=useTiming({progress,startFrame,durationFrames:duration},duration);
  const ruleProgress=cubicOut(clamp01((timingProgress-.64)/.36));
  const ref=React.useRef<HTMLDivElement>(null),[size,setSize]=React.useState({w:0,h:0});
  React.useLayoutEffect(()=>{
    const element=ref.current;
    if (!element) return;
    const measure=()=>setSize({w:element.clientWidth,h:element.clientHeight});
    measure();
    const observer=new ResizeObserver(measure); observer.observe(element);
    return ()=>observer.disconnect();
  },[]);
  return <div ref={ref} data-shared="TermBlock" data-term={term} data-state={String(state)} style={{position:'relative', width:'100%', height:'100%', minHeight:142, boxSizing:'border-box', overflow:'hidden', opacity:amount, transform:`translateY(${12*(1-amount)}px)`}}>
    {size.w>0&&size.h>0&&<DrawRect box={{x:0,y:0,w:size.w,h:size.h}} progress={progress} startFrame={startFrame} durationFrames={duration} corners={selected?'tr-bl':'tl-br'} strokeToken={selected?accent:'color/line/hairline'} fillToken="color/fill/panel-render" glowToken="color/fill/gold" strokeWidth={selected?2:undefined} seed={89}/>}
    <span aria-hidden="true" style={{position:'absolute',left:92,top:14,right:20,height:1,transformOrigin:'left center',transform:`scaleX(${ruleProgress})`,background:color(accent)}}/>
    {unit&&<span aria-hidden="true" style={{position:'absolute',left:24,bottom:8,right:20,height:1,transformOrigin:'left center',transform:`scaleX(${ruleProgress})`,background:color(accent)}}/>}
    <span aria-hidden="true" style={{position:'absolute', left:0, top:12, bottom:12, width:4, background:color(accent)}}/>
    <div style={{position:'absolute', left:24, top:22, width:52, height:52, display:'grid', placeItems:'center', color:color(accent)}}>{glyph}</div>
    <div style={{position:'absolute', left:92, top:18, right:20, fontSize:30, lineHeight:1.15, fontWeight:600, color:color('color/text/primary')}}>{term}</div>
    {summary && <div style={{position:'absolute', left:92, top:60, right:20, fontSize:28, lineHeight:1.25, color:color('color/text/soft')}}>{summary}</div>}
    {unit && <div data-term-unit={unit} style={{position:'absolute', left:24, bottom:14, font:'400 20px "IBM Plex Mono"', color:color('color/text/muted')}}>{unit}</div>}
  </div>;
}
