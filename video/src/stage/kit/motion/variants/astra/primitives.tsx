import {useKitFrame} from '../../core';
import React, {type CSSProperties, type ReactNode} from 'react';
import {color, dimensions, Tokens, stickGroupColor, skinToneColor, needsFigureRim} from '../../../theme/tokens';
import {notchPath} from '../../../primitives/notch';
import type {PersonMarkProps} from '../../../primitives/PersonMark';
export type Box = {x:number;y:number;w:number;h:number};
export type Timing = {progress?:number;startFrame?:number;durationFrames?:number};
export type ColorToken = `color/${string}`;
export const clamp = (n:number)=>Math.max(0,Math.min(1,n));
export function useProgress({progress,startFrame=0,durationFrames=30}:Timing) {const frame=useKitFrame();return clamp(progress ?? (frame-startFrame)/Math.max(1,durationFrames));}
export function hash(seed:number,frame:number) {let n=Math.imul(seed^Math.floor(frame),0x45d9f3b);n=Math.imul(n^(n>>>16),0x45d9f3b);return ((n^(n>>>16))>>>0)/4294967295;}
export type DrawRectProps = Timing & {box:Box;stroke?:ColorToken;fill?:ColorToken;corner?:'top-left'|'top-right';notch?:boolean;seed?:number};
export function DrawRect({box,stroke='color/line/wire',fill='color/fill/panel-wash',corner='top-left',notch=false,seed=7,...timing}:DrawRectProps) {
 const p=useProgress(timing), q=clamp(p/.7), a=1-Math.pow(1-q,3), f=clamp((p-.7)/.3);const {x,y,w,h}=box;const notchScale=Math.min(.35,w/400);
 const paths=corner==='top-left'?[`M0 0H${w*a}`,`M0 0V${h*a}`,`M${w} ${h}H${w*(1-a)}`,`M${w} ${h}V${h*(1-a)}`]:[`M${w} 0H${w*(1-a)}`,`M${w} 0V${h*a}`,`M0 ${h}H${w*a}`,`M0 ${h}V${h*(1-a)}`];
 if(notch&&f>0)paths[0]=`M0 0H${w/2-173.941*notchScale}M${w/2+173.94*notchScale} 0H${w}`;
 return <svg style={{position:'absolute',left:x,top:y,overflow:'visible'}} width={w} height={h} viewBox={`0 0 ${w} ${h}`}><rect width={w} height={h} fill={color(fill)} opacity={f*(f<1&&hash(seed,Math.floor(p*30))<.22?.4:1)}/><g fill="none" stroke={color(stroke)} strokeWidth={dimensions['stroke/chart'].value}>{p>0&&paths.map((d,i)=><path key={i} d={d}/>)}{notch&&<g opacity={f} transform={`translate(${w/2-960*notchScale},0) scale(${notchScale})`}><path d={notchPath(0,1,f)} strokeWidth={1.6/notchScale}/></g>}</g>{p>0&&p<.7&&[[w*a,0],[0,h*a],[w*(1-a),h],[w,h*(1-a)]].map(([cx,cy],i)=><circle key={i} cx={corner==='top-right'?w-cx:cx} cy={cy} r={2.6} fill={color('color/line/wire')}/>)}</svg>;
}
export type Point={x:number;y:number};
export type DrawLineProps=Timing&{points:readonly Point[];stroke?:ColorToken;strokeWidth?:number};
export function DrawLine({points,stroke='color/line/wire',strokeWidth=dimensions['stroke/thin'].value,...timing}:DrawLineProps) {
 const p=useProgress(timing);if(points.length<2)return null;
 const lengths=points.slice(1).map((b,i)=>Math.hypot(b.x-points[i].x,b.y-points[i].y));const total=lengths.reduce((a,b)=>a+b,0), end=total*p;
 let distance=0; const segments=lengths.map((length,i)=>{const a=points[i],b=points[i+1],t=clamp((end-distance)/(length||1));distance+=length;return {a,b:{x:a.x+(b.x-a.x)*t,y:a.y+(b.y-a.y)*t},t};});
 const visible=segments.filter(s=>s.t>0);const head=visible[visible.length-1]?.b;
 return <svg width="100%" height="100%" style={{position:'absolute',inset:0,overflow:'visible',pointerEvents:'none'}}><g fill="none" strokeLinecap="round" strokeLinejoin="round">{segments.map((s,i)=>s.t>0&&<path key={i} d={`M${s.a.x} ${s.a.y}L${s.b.x} ${s.b.y}`} stroke={color(stroke)} strokeWidth={strokeWidth} opacity={.62}/>)}{head&&p<1&&<><circle cx={head.x} cy={head.y} r={10} fill={color(stroke)} opacity={.1}/><circle cx={head.x} cy={head.y} r={3.5} fill={color(stroke)}/></>}</g></svg>;
}
export type FlickerProps=Timing&{children:ReactNode;seed?:number;intensity?:number;direction?:'on'|'off';style?:CSSProperties};
export function flickerState(p:number,seed=7,intensity=1) {const t=clamp(p);if(t===0)return {opacity:0,brightness:1};if(t===1)return {opacity:1,brightness:1};const phase=Math.floor(t*30);const dip=(t>.12&&t<.23)||(t>.34&&t<.4)||(t>.55&&t<.59);return {opacity:dip?1-clamp(intensity)*(.8+.2*hash(seed,phase)):Math.min(1,t*7),brightness:1+clamp(intensity)*(1-t)*(.2+.35*hash(seed,phase))};}
export function Flicker({children,seed=7,intensity=1,direction='on',style,...timing}:FlickerProps) {const p=useProgress(timing),s=flickerState(direction==='off'?1-p:p,seed,intensity);return <div style={{...style,opacity:s.opacity,filter:`brightness(${s.brightness})`}}>{children}</div>;}
export type FitBoxProps=Timing&{text:string;x:number;y:number;fontSize?:number;padding?:'spacing/xs'|'spacing/sm'|'spacing/md'|'spacing/lg';seed?:number};
/** IBM Plex Mono has a 600/1000 em advance. Explicit newlines are measured independently. */
export function measureFitBox(text:string,fontSize=26,padding:FitBoxProps['padding']='spacing/md') {const lines=text.split('\n'),pad=dimensions[padding].value;return {w:Math.ceil(Math.max(...lines.map(l=>Array.from(l).length))*fontSize*.6+2*pad),h:Math.ceil(lines.length*fontSize*1.4+2*pad),pad};}
export function FitBox({text,x,y,fontSize=26,padding='spacing/md',seed=7,...timing}:FitBoxProps) {const p=useProgress(timing),{w,h,pad}=measureFitBox(text,fontSize,padding);return <><DrawRect box={{x,y,w,h}} progress={clamp(p/.7)} notch/><Flicker progress={clamp((p-.55)/.45)} seed={seed} style={{position:'absolute',left:x+pad,top:y+pad,fontFamily:Tokens.fonts.mono,fontSize,lineHeight:1.4,whiteSpace:'pre',color:color('color/text/primary')}}>{text}</Flicker></>;}
export type StickFigureProps=PersonMarkProps&Timing;
export function StickFigure({x,y,group,emphasis,scale,tone,...timing}:StickFigureProps) {
  const p=useProgress({...timing,progress:timing.progress??(timing.startFrame===undefined?1:undefined)});
  const s=Math.max(.2,scale);
  const e=clamp(emphasis);
  const ink = tone !== undefined ? skinToneColor(tone) : stickGroupColor(group);
  const strokeW = 2.2 + .6 * e;
  const needsRim = needsFigureRim(tone, group);
  const rimColor = color('color/line/hairline-strong');
  const rimW = strokeW + 2.8;
  const limbPaths = ['M22 16V34','M22 21L10 31','M22 21L34 31','M22 34L13 53','M22 34L31 53'];
  return (
    <svg style={{position:'absolute',left:x,top:y,transform:'translate(-50%,-50%)',overflow:'visible',opacity:.42+.58*e}} width={44*s} height={58*s} viewBox="0 0 44 58" fill="none" strokeLinecap="round" strokeLinejoin="round">
      {needsRim && (
        <g stroke={rimColor} strokeWidth={rimW} opacity={0.88}>
          <circle cx={22} cy={9} r={6} pathLength={1} strokeDasharray="1" strokeDashoffset={1-clamp(p*3)}/>
          {limbPaths.map((d,i)=><path key={`rim-${d}`} d={d} pathLength={1} strokeDasharray="1" strokeDashoffset={1-clamp((p-.12-i*.07)*2.5)}/>)}
        </g>
      )}
      <g stroke={ink} strokeWidth={strokeW}>
        <circle cx={22} cy={9} r={6} pathLength={1} strokeDasharray="1" strokeDashoffset={1-clamp(p*3)}/>
        {limbPaths.map((d,i)=><path key={d} d={d} pathLength={1} strokeDasharray="1" strokeDashoffset={1-clamp((p-.12-i*.07)*2.5)}/>)}
      </g>
    </svg>
  );
}
export type BootVariant='snap'|'scan'|'glitch';
export type BootInProps=Timing&{box:Box;children:ReactNode;variant?:BootVariant;seed?:number};
export function BootIn({box,children,variant='scan',seed=7,...timing}:BootInProps) {const p=useProgress(timing),borderEnd=variant==='snap'?.3:.48,reveal=clamp((p-.64)/.36);return <><DrawRect box={box} progress={clamp(p/borderEnd)} notch={variant!=='snap'}/><Flicker seed={seed} intensity={variant==='glitch'?1:.5} progress={clamp((p-borderEnd)/(.64-borderEnd))} style={{position:'absolute',left:box.x,top:box.y,width:box.w,height:box.h}}><div style={{width:'100%',height:'100%',clipPath:variant==='scan'?`inset(0 ${(1-reveal)*100}% 0 0)`:undefined,opacity:variant==='scan'?1:reveal,transform:`translateY(${variant==='snap'?(1-reveal)*6:0}px)`}}>{children}</div></Flicker>{variant==='scan'&&reveal>0&&reveal<1&&<div style={{position:'absolute',left:box.x+box.w*reveal,top:box.y+12,height:box.h-24,width:2,background:color('color/line/wire')}}/>}</>;}
