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
  return <div data-shared="TrainingTextStrip" data-pattern-key={patternKey} style={{position:'relative',width:'100%',minHeight:120,boxSizing:'border-box',overflow:'hidden'}}>
    <div aria-hidden="true" style={{position:'absolute',left:4,top:0,padding:'2px 8px',font:'500 16px "IBM Plex Mono"',letterSpacing:1.1,color:color('color/fill/gold'),border:`1px solid ${color('color/line/hairline-strong')}`,background:color('color/fill/panel-render'),opacity:progress}}>PATTERN</div>
    <FitBox text={text} x={4} y={32} family="mono" size={22} weight={400} lineHeight={1.3} padX="spacing/md" padY="spacing/sm" textToken="color/text/soft" strokeToken="color/line/hairline" fillToken="color/fill/panel-render" accentToken="color/fill/gold" corners="tl-br" seed={31} intensity={.7} progress={progress}/>
    {tokens.slice(0,visibleCount).map((token,index)=>{
      const active=emphasized(token,index,emphasis);
      const preceding=visible.slice(0,index).join('   ');
      if (!active) return null;
      return <span key={`${patternKey}-${index}`} data-pattern-token="emphasized" aria-hidden="true" style={{position:'absolute',left:20 + ([...preceding].length + (index ? 3 : 0))*13.2,top:70,width:[...token].length*13.2,height:3,background:color('color/fill/gold')}}/>;
    })}
  </div>;
}
