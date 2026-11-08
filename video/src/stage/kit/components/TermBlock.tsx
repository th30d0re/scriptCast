import React from 'react';
import {color} from '../theme/tokens';
import type {Accent, TermBlockGlyph, TermState} from './types';

export type TermBlockProps = {
  term: string;
  summary?: string;
  glyph: TermBlockGlyph;
  state: TermState;
  accent: Accent;
  unit?: string;
};

const stateAmount = (state: TermState) => typeof state === 'number'
  ? Math.max(0, Math.min(1, state))
  : state === 'hidden' ? 0 : state === 'dim' ? .42 : state === 'active' ? .82 : 1;

export function TermBlock({term, summary, glyph, state, accent, unit}: TermBlockProps) {
  const amount=stateAmount(state), selected=state === 'selected';
  return <div data-shared="TermBlock" data-term={term} data-state={String(state)} style={{position:'relative', width:'100%', height:'100%', minHeight:142, boxSizing:'border-box', overflow:'hidden', opacity:amount, transform:`translateY(${12*(1-amount)}px)`, background:color('color/fill/panel-render'), border:`${selected?2:1}px solid ${selected?color(accent):color('color/line/hairline')}`, boxShadow:selected?`inset 0 0 0 1px ${color('color/fill/gold-wash')}`:undefined}}>
    <span aria-hidden="true" style={{position:'absolute', left:0, top:12, bottom:12, width:4, background:color(accent)}}/>
    <div style={{position:'absolute', left:24, top:22, width:52, height:52, display:'grid', placeItems:'center', color:color(accent)}}>{glyph}</div>
    <div style={{position:'absolute', left:92, top:18, right:20, fontSize:30, lineHeight:1.15, fontWeight:600, color:color('color/text/primary')}}>{term}</div>
    {summary && <div style={{position:'absolute', left:92, top:60, right:20, fontSize:28, lineHeight:1.25, color:color('color/text/soft')}}>{summary}</div>}
    {unit && <div data-term-unit={unit} style={{position:'absolute', left:24, bottom:14, font:'400 20px "IBM Plex Mono"', color:color('color/text/muted')}}>{unit}</div>}
  </div>;
}
