import {useCurrentFrame, useVideoConfig} from 'remotion';
import {Frame} from './Frame';
import {MathText} from './Math';
import {palette as p} from '../theme';

export type EquationProps = {headline: string; latex: string; number: string | number; terms: {latex: string; gloss: string; color?: string}[]; reading: string; sources: string; qrUrl?: string; durationInFrames?: number};
export const activeTerm = (frame: number, duration: number, count: number) => Math.min(count - 1, Math.floor(Math.max(0, frame) * count / Math.max(1, duration)));

export function EquationCard(props: EquationProps) {
  let frame = 0, duration = 1;
  try {frame = useCurrentFrame(); duration = useVideoConfig().durationInFrames;} catch { /* Static markup. */ }
  duration = props.durationInFrames ?? duration;
  const active = activeTerm(frame, duration, props.terms.length);
  return <Frame {...props} qr={Boolean(props.qrUrl)}>{scale => <div style={{display: 'flex', flexDirection: 'column', gap: 20 * scale}}>
    <div style={{fontSize: 28 * scale, color: p.gold}}>Equation {props.number}</div>
    <div style={{fontSize: 48 * scale, textAlign: 'center'}}><MathText latex={props.latex} display /></div>
    {props.terms.map((term, i) => <div key={i} data-term-active={i === active} style={{display: 'flex', alignItems: 'baseline', gap: 24 * scale, padding: 10 * scale, borderLeft: `4px solid ${i === active ? term.color ?? p.gold : p.navy2}`, background: i === active ? p.navy2 : 'transparent', fontSize: 30 * scale}}>
      <span style={{color: term.color ?? p.gold}}><MathText latex={term.latex} /></span><span>{term.gloss}</span>
    </div>)}
    <div style={{fontSize: 34 * scale, lineHeight: 1.25}}>{props.reading}</div>
  </div>}</Frame>;
}
