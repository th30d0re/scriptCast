import React from 'react';
import {Flicker} from '../motion/Flicker';
import type {Timing} from '../motion/core';
import {color} from '../theme/tokens';

const clamp = (value:number) => Math.max(0, Math.min(1, value));

export function NumberBadge({value, active=1, progress, startFrame, durationFrames}:{value:string; active?:number} & Timing) {
  const badge = <div aria-hidden="true" style={{width:48,height:48,borderRadius:24,display:'grid',placeItems:'center',font:'500 24px "IBM Plex Mono"',color:color('color/fill/gold'),border:`2px solid ${color('color/fill/gold')}`,opacity:active}}>{value}</div>;
  const hasTiming = progress !== undefined || startFrame !== undefined || durationFrames !== undefined;
  return hasTiming ? <Flicker progress={progress} startFrame={startFrame} durationFrames={durationFrames} style={{position:'relative',inset:'auto',width:48,height:48,flex:'0 0 48px'}}>{badge}</Flicker> : badge;
}

export function PersonIcon({tone='color/text/soft'}:{tone?:string}) {
  return <svg width="48" height="48" viewBox="0 0 48 48" aria-hidden="true">
    <circle cx="24" cy="14" r="8" fill="none" stroke={color(tone)} strokeWidth="2.5"/>
    <path d="M9 42 Q12 25 24 25 Q36 25 39 42" fill="none" stroke={color(tone)} strokeWidth="2.5" strokeLinecap="round"/>
  </svg>;
}

export function ActionIcon() {
  return <svg width="48" height="48" viewBox="0 0 48 48" aria-hidden="true">
    <path d="M7 24 H38 M28 13 L39 24 L28 35" fill="none" stroke={color('color/fill/gold')} strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"/>
  </svg>;
}

export function SystemIcon({tone='color/text/soft'}:{tone?:string}) {
  return <svg width="48" height="48" viewBox="0 0 48 48" aria-hidden="true">
    {[10,24,38].map((x,index)=><circle key={x} cx={x} cy={index===1?10:34} r="5" fill={color('color/bg/field')} stroke={color(tone)} strokeWidth="2"/>)}
    <path d="M13 31 L21 14 M27 14 L35 31 M15 34 H33" fill="none" stroke={color(tone)} strokeWidth="2"/>
  </svg>;
}

export function WordIcon() {
  return <svg width="48" height="48" viewBox="0 0 48 48" aria-hidden="true">
    <path d="M6 10 H42 V34 H25 L16 42 V34 H6 Z" fill="none" stroke={color('color/text/soft')} strokeWidth="2.5" strokeLinejoin="round"/>
    <path d="M13 19 H35 M13 26 H29" stroke={color('color/fill/gold')} strokeWidth="2"/>
  </svg>;
}

export function HeartIcon() {
  return <svg width="48" height="48" viewBox="0 0 48 48" aria-hidden="true">
    <path d="M24 40 C18 34 7 27 7 17 C7 7 19 5 24 13 C29 5 41 7 41 17 C41 27 30 34 24 40 Z" fill="none" stroke={color('color/marker/race')} strokeWidth="2.5"/>
  </svg>;
}

export function CausalArrow({x1,x2,y,p=1,accent='color/fill/gold',width=3,id}:{x1:number;x2:number;y:number;p?:number;accent?:string;width?:number;id?:string}) {
  const end=x1+(x2-x1)*clamp(p);
  return <g data-causal-arrow={id}>
    <line x1={x1} y1={y} x2={end} y2={y} stroke={color(accent)} strokeWidth={width} strokeLinecap="round"/>
    {p>.88&&<path d={`M ${end-12} ${y-8} L ${end} ${y} L ${end-12} ${y+8}`} fill="none" stroke={color(accent)} strokeWidth={width} strokeLinecap="round" strokeLinejoin="round"/>}
  </g>;
}

export function ScanLines({x,y,w,h,progress:raw}:{x:number;y:number;w:number;h:number;progress:number}) {
  const p=clamp(raw);
  return <g aria-hidden="true" opacity={p}>
    {Array.from({length:7},(_,index)=>{
      const yy=y+index*h/6;
      return <line key={index} x1={x} y1={yy} x2={x+w*p} y2={yy} stroke={color('color/fill/gold')} strokeWidth="1" opacity={.25+index*.06}/>;
    })}
  </g>;
}
