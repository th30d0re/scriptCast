import React from 'react';
import {color} from '../theme/tokens';
import {DrawRect} from '../motion/DrawRect';

export type CitationCardProps = {
  authors: string;
  year: string | number;
  title: string;
  progress: number;
  compact?: boolean;
  startFrame?: number;
  durationFrames?: number;
};

export function CitationCard({authors, year, title, progress:rawProgress, compact=false, startFrame, durationFrames}: CitationCardProps) {
  const progress=Math.max(0,Math.min(1,rawProgress));
  // The final frame is deliberately a plain card: no inherited transform, fade, or draw-envelope state.
  const settled=progress>=.999, frameProgress=settled?1:progress, textOpacity=settled?1:Math.max(0,Math.min(1,(progress-.18)/.82));
  const ref=React.useRef<HTMLDivElement>(null),[size,setSize]=React.useState({w:0,h:0});
  React.useLayoutEffect(()=>{
    const element=ref.current;
    if (!element) return;
    const measure=()=>setSize({w:element.clientWidth,h:element.clientHeight});
    measure();
    const observer=new ResizeObserver(measure); observer.observe(element);
    return ()=>observer.disconnect();
  },[]);
  return <div ref={ref} data-shared="CitationCard" data-compact={compact?'true':'false'} style={{position:'relative', width:'100%', height:'100%', minHeight:compact?128:206, boxSizing:'border-box', overflow:'hidden', opacity:settled?1:progress, transform:settled?'none':`translateX(${20*(1-progress)}px)`, padding:compact?'20px 26px':'30px 34px'}}>
    {size.w>0&&size.h>0&&<DrawRect box={{x:0,y:0,w:size.w,h:size.h}} progress={frameProgress} startFrame={startFrame} durationFrames={durationFrames} corners="tl-br" strokeToken="color/line/hairline" fillToken="color/fill/panel-render" glowToken="color/fill/gold" seed={47}/>}
    <div style={{font:'500 20px "IBM Plex Mono"', letterSpacing:1, color:color('color/fill/gold'), opacity:textOpacity}}>{year}</div>
    <div style={{marginTop:compact?8:14, fontSize:compact?28:36, lineHeight:1.15, fontWeight:600, color:color('color/text/primary'), opacity:textOpacity}}>{authors}</div>
    <div style={{marginTop:compact?8:16, fontSize:28, lineHeight:1.3, fontStyle:'italic', color:color('color/text/soft'), opacity:textOpacity}}>{title}</div>
  </div>;
}
