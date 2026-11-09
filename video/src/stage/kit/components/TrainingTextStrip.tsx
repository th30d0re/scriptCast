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
  // FitBox gets the full text so its box is sized once and the text types inside it.
  const progress=Math.max(0,Math.min(1,rawProgress)),text=tokens.join(' '),underline=Math.max(0,Math.min(1,(progress-.85)/.15));
  return <div data-shared="TrainingTextStrip" data-pattern-key={patternKey} style={{position:'relative',width:'100%',minHeight:120,boxSizing:'border-box',overflow:'hidden'}}>
    <div aria-hidden="true" style={{position:'absolute',left:4,top:0,padding:'2px 8px',font:'500 16px "IBM Plex Mono"',letterSpacing:1.1,color:color('color/fill/gold'),border:`1px solid ${color('color/line/hairline-strong')}`,background:color('color/fill/panel-render'),opacity:progress}}>PATTERN</div>
    <FitBox text={text} x={4} y={32} family="mono" size={22} weight={400} lineHeight={1.3} padX="spacing/md" padY="spacing/sm" textToken="color/text/soft" strokeToken="color/line/hairline" fillToken="color/fill/panel-render" accentToken="color/fill/gold" corners="tl-br" seed={31} intensity={.7} progress={progress}/>
    {underline>0 && <div aria-hidden="true" style={{position:'absolute',left:30,top:42,font:'400 22px "IBM Plex Mono"',lineHeight:1.3,whiteSpace:'pre',color:'transparent'}}>
      {/* Invisible copy of the strip text: the underline rides the real glyph advances, then draws in once typing is done. */}
      {tokens.map((token,index)=><React.Fragment key={`${patternKey}-${index}`}>{index ? ' ' : ''}<span data-pattern-token={emphasized(token,index,emphasis) ? 'emphasized' : undefined} style={emphasized(token,index,emphasis) ? {paddingBottom:5,backgroundImage:`linear-gradient(${color('color/fill/gold')},${color('color/fill/gold')})`,backgroundRepeat:'no-repeat',backgroundPosition:'left bottom',backgroundSize:`${underline*100}% 3px`} : undefined}>{token}</span></React.Fragment>)}
    </div>}
  </div>;
}
