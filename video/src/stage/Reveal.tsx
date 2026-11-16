import React from 'react';
import {useCue, useWords} from './plan';
import {useStageMs} from './clock';
import {cubicOut, progress} from './motion';
import {useTokens} from './tokens';
export function Reveal({cue, durationMs = 700, stagger = 0, rise = 24, children}: React.PropsWithChildren<{cue: string; durationMs?: number; stagger?: number; rise?: number}>) {
  const {since} = useCue(cue);
  return <>{React.Children.map(children, (child, i) => {const t = cubicOut(progress(since-i*stagger,0,durationMs)); return <div style={{opacity:t, transform:`translateY(${(1-t)*rise}px)`}}>{child}</div>;})}</>;
}
export function WordHighlight({turnId}: {turnId: string}) {
  const words = useWords(turnId), ms = useStageMs(), tokens = useTokens();
  return <span>{words.map((w,i) => <React.Fragment key={i}><span style={{color:ms >= w.start_ms && ms < w.end_ms ? tokens.colours.accent : tokens.colours.text}}>{w.text}</span>{i < words.length-1 ? ' ' : ''}</React.Fragment>)}</span>;
}
