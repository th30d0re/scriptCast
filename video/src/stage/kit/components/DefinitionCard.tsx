import React from 'react';
import {color} from '../theme/tokens';
import type {Accent} from './types';

export type DefinitionCardProps = {
  term: string;
  definition: string;
  source: string;
  paraphrased: boolean;
  progress: number;
  accent: Accent;
};

export function DefinitionCard({term, definition, source, paraphrased, progress:rawProgress, accent}: DefinitionCardProps) {
  const progress=Math.max(0,Math.min(1,rawProgress));
  return <div data-shared="DefinitionCard" data-definition-term={term} data-progress={progress.toFixed(3)} style={{position:'relative', width:'100%', height:'100%', minHeight:220, boxSizing:'border-box', overflow:'hidden', opacity:progress, transform:`translateY(${18*(1-progress)}px)`, padding:'30px 34px 28px 38px', background:color('color/fill/panel-render'), border:`1px solid ${color('color/line/hairline')}`}}>
    <span aria-hidden="true" style={{position:'absolute', left:0, top:0, bottom:0, width:5, background:color(accent), transform:`scaleY(${progress})`, transformOrigin:'center bottom'}}/>
    <div style={{fontSize:38, lineHeight:1.1, fontWeight:600, color:color(accent)}}>{term}</div>
    <div style={{marginTop:18, fontSize:28, lineHeight:1.34, color:color('color/text/primary')}}>{definition}</div>
    <div style={{marginTop:22, display:'flex', justifyContent:'space-between', gap:24, font:'400 20px "IBM Plex Mono"', lineHeight:1.3, color:color('color/text/muted')}}>
      <span>{source}</span>
      {paraphrased && <span data-paraphrased="true">PARAPHRASED</span>}
    </div>
  </div>;
}

