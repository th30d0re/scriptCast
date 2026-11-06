import {Frame} from './Frame';
import {MathText} from './Math';
import {FRAME_BEATS} from './motion';
import {palette as p} from './theme';

export type EquationProps = {headline: string; latex: string; number: string | number; terms: {latex: string; gloss: string; color?: string}[]; reading: string; sources: string; qrUrl?: string; progress?: number; durationInFrames?: number};
// The active term walks the terms as the card plays: progress 0 to 1 spans the whole card.
export const activeTerm = (progress: number, count: number) => Math.min(count - 1, Math.floor(Math.max(0, progress) * count));
// One term per 60 beats (two seconds), or the card's own length when the plan supplies it.
export const equationBeats = (props: EquationProps) => Math.max(FRAME_BEATS, props.durationInFrames ?? props.terms.length * 60);

export function EquationCard(props: EquationProps) {
  const active = activeTerm(props.progress ?? 1, props.terms.length);
  return <Frame {...props} beats={equationBeats(props)} qr={Boolean(props.qrUrl)}>{scale => <div style={{display: 'flex', flexDirection: 'column', gap: 20 * scale}}>
    <div style={{fontSize: 28 * scale, color: p.gold}}>Equation {props.number}</div>
    <div style={{fontSize: 48 * scale, textAlign: 'center'}}><MathText latex={props.latex} display /></div>
    {props.terms.map((term, i) => <div key={i} data-term-active={i === active} style={{display: 'flex', alignItems: 'baseline', gap: 24 * scale, padding: 10 * scale, borderLeft: `4px solid ${i === active ? term.color ?? p.gold : p.navy2}`, background: i === active ? p.navy2 : 'transparent', fontSize: 30 * scale}}>
      <span style={{color: term.color ?? p.gold}}><MathText latex={term.latex} /></span><span>{term.gloss}</span>
    </div>)}
    <div style={{fontSize: 34 * scale, lineHeight: 1.25}}>{props.reading}</div>
  </div>}</Frame>;
}
