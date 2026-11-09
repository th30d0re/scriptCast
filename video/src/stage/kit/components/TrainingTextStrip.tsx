import React from 'react';
import {color} from '../theme/tokens';
import {FitBox} from '../motion/FitBox';

export type TrainingTextStripProps = {
  tokens: readonly string[];
  patternKey: string;
  progress: number;
  emphasis: number | string | readonly string[];
};

const emphasized=(token:string,index:number,emphasis:TrainingTextStripProps['emphasis'])=>typeof emphasis==='number'
  ? index===emphasis
  : typeof emphasis==='string' ? token===emphasis : emphasis.includes(token);

export function TrainingTextStrip({tokens, patternKey, progress:rawProgress, emphasis}: TrainingTextStripProps) {
  const progress=Math.max(0,Math.min(1,rawProgress)),visibleCount=Math.ceil(tokens.length*progress),visible=tokens.slice(0,visibleCount),text=visible.join('   ');
  return <div data-shared="TrainingTextStrip" data-pattern-key={patternKey} style={{position:'relative',width:'100%',minHeight:92,boxSizing:'border-box',overflow:'hidden'}}>
    <FitBox text={text} x={4} y={8} family="mono" size={22} weight={400} lineHeight={1.3} padX="spacing/md" padY="spacing/sm" textToken="color/text/soft" strokeToken="color/line/hairline" fillToken="color/fill/panel-render" accentToken="color/fill/gold" tag="PATTERN" corners="tl-br" seed={31} intensity={.7} progress={progress}/>
    {tokens.slice(0,visibleCount).map((token,index)=>{
      const active=emphasized(token,index,emphasis);
      const preceding=visible.slice(0,index).join('   ');
      return <span key={`${patternKey}-${index}`} data-pattern-token={active?'emphasized':'plain'} style={{position:'absolute',left:20 + ([...preceding].length + (index ? 3 : 0))*13.2,top:16,padding:'0 2px',font:'400 22px "IBM Plex Mono"',lineHeight:1.3,color:active?color('color/fill/gold'):'transparent',background:active?color('color/fill/gold-wash'):'transparent',border:`1px solid ${active?color('color/line/hairline-strong'):'transparent'}`}}>{token}</span>;
    })}
  </div>;
}
