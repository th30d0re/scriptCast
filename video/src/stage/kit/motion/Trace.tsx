import React from 'react';
import {color} from '../theme/tokens';
import {clamp01, pointAt, polylineLength, subPolyline, toPath, type Point} from './core';

export type TraceProps = {
  points:readonly Point[];
  /** Drawn fraction of the path, 0..1. */
  drawn:number;
  /** 0 hides the hot head, 1 shows it at full heat. Fade it once the trace has landed. */
  heat?:number;
  /** Opacity of the cooled line behind the head. Bring it to 1 once the trace settles. */
  cool?:number;
  strokeToken?:string;
  glowToken?:string;
  width:number;
  /** Length of the hot tail behind the head, in px. */
  tail?:number;
  cap?:'round'|'butt'|'square';
};

const TAIL_LAYERS = 6;

/**
 * A path that draws on with a hot leading head. The head is the stroke colour lifted by a brightness filter with a
 * gold glow; the tail is stacked partial strokes so heat falls off smoothly toward the cooled line.
 */
export function Trace({points, drawn, heat = 1, cool = .78, strokeToken = 'color/line/wire', glowToken = 'color/fill/gold', width, tail = 64, cap = 'square'}:TraceProps) {
  const total = polylineLength(points);
  const head = total*clamp01(drawn);
  if (head <= .01) return null;
  const ink = color(strokeToken);
  const tip = pointAt(points, head).p;
  const h = clamp01(heat);
  return <g>
    <path d={toPath(subPolyline(points, 0, head))} stroke={ink} strokeWidth={width} strokeOpacity={cool} strokeLinecap={cap} strokeLinejoin="miter" fill="none"/>
    {h > .01 && <g style={{filter:`brightness(${(1 + .9*h).toFixed(3)}) drop-shadow(0 0 ${(2 + 6*h).toFixed(2)}px ${color(glowToken)})`}} opacity={h}>
      {Array.from({length:TAIL_LAYERS}, (_, i) => {
        const from = Math.max(0, head - tail*(TAIL_LAYERS - i)/TAIL_LAYERS);
        return <path key={i} d={toPath(subPolyline(points, from, head))} stroke={ink} strokeWidth={width*(1 + .1*i)} strokeOpacity={.2} strokeLinecap="butt" fill="none"/>;
      })}
      <rect x={tip[0] - width*1.15} y={tip[1] - width*1.15} width={width*2.3} height={width*2.3} fill={ink}/>
    </g>}
  </g>;
}
