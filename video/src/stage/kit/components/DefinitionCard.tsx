import React from 'react';
import {color} from '../theme/tokens';
import {FitBox} from '../motion/FitBox';
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
  return <div data-shared="DefinitionCard" data-definition-term={term} data-progress={progress.toFixed(3)} style={{position:'relative', width:'100%', height:'100%', minHeight:220, boxSizing:'border-box', overflow:'hidden', opacity:progress, transform:`translateY(${18*(1-progress)}px)`, padding:'30px 34px 28px 38px'}}>
    <FitBox text={term} x={22} y={18} size={38} weight={600} lineHeight={1.1} padX="spacing/md" padY="spacing/sm" textToken={accent} strokeToken={accent} fillToken="color/fill/panel-render" accentToken={accent} tag="TERM" corners="tl-br" seed={17} intensity={.8} progress={progress}/>
    <div aria-hidden="true" style={{height:42}}/>
    <div style={{marginTop:18, fontSize:28, lineHeight:1.34, color:color('color/text/primary')}}>{definition}</div>
    <div style={{marginTop:22, display:'flex', justifyContent:'space-between', gap:24, font:'400 20px "IBM Plex Mono"', lineHeight:1.3, color:color('color/text/muted')}}>
      <span>{source}</span>
      {paraphrased && <span data-paraphrased="true">PARAPHRASED</span>}
    </div>
  </div>;
}
