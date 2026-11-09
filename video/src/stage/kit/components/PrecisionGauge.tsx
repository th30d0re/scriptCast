import React from 'react';
import {color} from '../theme/tokens';
import {DrawRect} from '../motion/DrawRect';
import {clamp01, cubicOut, useTiming, type Timing} from '../motion/core';

export type PrecisionGaugeProps = Timing & {
  value: number;
  derived: boolean;
  label: string;
  lowLabel: string;
  highLabel: string;
};

export function PrecisionGauge({value:rawValue, derived, label, lowLabel, highLabel, progress, startFrame, durationFrames}: PrecisionGaugeProps) {
  // Untimed means fully shown (kit convention).
  if (progress === undefined && startFrame === undefined) progress = 1;
  const value=Math.max(0,Math.min(1,rawValue));
  const duration=durationFrames??40, {p:timingProgress}=useTiming({progress,startFrame,durationFrames:duration},duration);
  const sweep=cubicOut(clamp01((timingProgress-.64)/.36));
  const displayedValue=value*sweep;
  const ref=React.useRef<HTMLDivElement>(null),[size,setSize]=React.useState({w:0,h:0});
  React.useLayoutEffect(()=>{
    const element=ref.current;
    if (!element) return;
    const measure=()=>setSize({w:element.clientWidth,h:element.clientHeight});
    measure();
    const observer=new ResizeObserver(measure); observer.observe(element);
    return ()=>observer.disconnect();
  },[]);
  return <div ref={ref} data-shared="PrecisionGauge" data-gauge-value={value.toFixed(3)} style={{position:'relative',width:'100%',height:'100%',minWidth:300,minHeight:250,boxSizing:'border-box',padding:'30px 30px 24px'}}>
    {size.w>0&&size.h>0&&<DrawRect box={{x:0,y:0,w:size.w,h:size.h}} progress={progress} startFrame={startFrame} durationFrames={duration} corners="tl-br" strokeToken="color/line/hairline" fillToken="color/fill/panel-render" glowToken="color/fill/gold" seed={59}/>}
    <div style={{position:'relative',font:'500 20px "IBM Plex Mono"',letterSpacing:1,color:color('color/text/soft')}}>{label}</div>
    {derived && <div style={{position:'absolute',right:28,top:30,font:'400 20px "IBM Plex Mono"',color:color('color/text/accent')}}>DERIVED</div>}
    <div style={{position:'absolute',left:34,right:34,top:106,height:18,background:color('color/surface/well'),border:`1px solid ${color('color/line/hairline')}`}}>
      <div style={{height:'100%',width:`${displayedValue*100}%`,background:color('color/fill/gold'),transformOrigin:'left center'}}/>
      <div data-gauge-marker="true" style={{position:'absolute',left:`${displayedValue*100}%`,top:-12,width:3,height:42,background:color('color/text/accent'),transform:'translateX(-1px)'}}/>
    </div>
    <div style={{position:'absolute',left:30,top:144,font:'400 20px "IBM Plex Mono"',color:color('color/text/muted')}}>{lowLabel}</div>
    <div style={{position:'absolute',right:30,top:144,font:'400 20px "IBM Plex Mono"',color:color('color/text/muted')}}>{highLabel}</div>
    <div style={{position:'absolute',left:0,right:0,bottom:24,textAlign:'center',font:'500 32px "IBM Plex Mono"',color:color('color/text/primary')}}>{displayedValue.toFixed(2)}</div>
  </div>;
}
