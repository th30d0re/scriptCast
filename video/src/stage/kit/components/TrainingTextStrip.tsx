import React from 'react';
import {color} from '../theme/tokens';

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
  const progress=Math.max(0,Math.min(1,rawProgress)),visibleCount=Math.ceil(tokens.length*progress);
  return <div data-shared="TrainingTextStrip" data-pattern-key={patternKey} style={{position:'relative',width:'100%',minHeight:92,boxSizing:'border-box',overflow:'hidden',display:'flex',alignItems:'center',gap:12,padding:'16px 20px',background:color('color/fill/panel-render'),border:`1px solid ${color('color/line/hairline')}`}}>
    {tokens.slice(0,visibleCount).map((token,index)=>{
      const active=emphasized(token,index,emphasis);
      return <span key={`${patternKey}-${index}`} data-pattern-token={active?'emphasized':'plain'} style={{flex:'0 0 auto',padding:'8px 12px',font:'400 22px "IBM Plex Mono"',color:active?color('color/fill/gold'):color('color/text/soft'),background:active?color('color/fill/gold-wash'):'transparent',border:`1px solid ${active?color('color/line/hairline-strong'):'transparent'}`}}>{token}</span>;
    })}
  </div>;
}

