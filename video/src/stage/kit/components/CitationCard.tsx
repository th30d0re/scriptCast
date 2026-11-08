import React from 'react';
import {color} from '../theme/tokens';

export type CitationCardProps = {
  authors: string;
  year: string | number;
  title: string;
  progress: number;
  compact?: boolean;
};

export function CitationCard({authors, year, title, progress:rawProgress, compact=false}: CitationCardProps) {
  const progress=Math.max(0,Math.min(1,rawProgress));
  return <div data-shared="CitationCard" data-compact={compact?'true':'false'} style={{position:'relative', width:'100%', height:'100%', minHeight:compact?128:206, boxSizing:'border-box', overflow:'hidden', opacity:progress, transform:`translateX(${20*(1-progress)}px)`, padding:compact?'20px 26px':'30px 34px', background:color('color/fill/panel-render'), border:`1px solid ${color('color/line/hairline')}`}}>
    <div style={{font:'500 20px "IBM Plex Mono"', letterSpacing:1, color:color('color/fill/gold')}}>{year}</div>
    <div style={{marginTop:compact?8:14, fontSize:compact?28:36, lineHeight:1.15, fontWeight:600, color:color('color/text/primary')}}>{authors}</div>
    <div style={{marginTop:compact?8:16, fontSize:28, lineHeight:1.3, fontStyle:'italic', color:color('color/text/soft')}}>{title}</div>
  </div>;
}
