import React from 'react';
import {color} from '../theme/tokens';
import {Trace} from './Trace';
import {clamp01, cubicInOut, flickerLevel, lightStyle, pointAt, polylineLength, span, stroke as strokeWidthOf, useTiming, type Point, type Timing} from './core';

export type DrawLineProps = Timing & {
  /** Two points for a straight line, more for a polyline. Scene coordinates. */
  points:readonly Point[];
  strokeToken?:string;
  glowToken?:string;
  strokeWidth?:number;
  /** Hot tail length in px. */
  tail?:number;
  /** A small square terminal at the start that flickers on before the line leaves it. */
  origin?:boolean;
  /** Vertex ticks that light as the head passes each corner of a polyline. */
  nodes?:boolean;
  end?:'none'|'arrow'|'terminal';
  seed?:number;
};

export const DRAW_LINE_PHASES = {origin:[0, .14], draw:[.08, .82], settle:[.82, 1]} as const;
export const DRAW_LINE_DEFAULT_FRAMES = 30;

/** Frame offsets (relative to the start) at which the head reaches each interior vertex. For sound cues. */
export function drawLineVertexFrames(points:readonly Point[], durationFrames = DRAW_LINE_DEFAULT_FRAMES) {
  const total = polylineLength(points), out:number[] = [];
  let run = 0;
  for (let i = 1; i < points.length - 1; i++) {
    run += Math.hypot(points[i][0] - points[i-1][0], points[i][1] - points[i-1][1]);
    // Invert cubicInOut numerically: find the first frame whose eased draw reaches this vertex.
    for (let f = 0; f <= durationFrames; f++) {
      if (cubicInOut(span(f/durationFrames, DRAW_LINE_PHASES.draw[0], DRAW_LINE_PHASES.draw[1]))*total >= run) { out.push(f); break; }
    }
  }
  return out;
}

/** A straight line or polyline that draws on with a bright head that dims to the settled stroke behind it. */
export function DrawLine({points, strokeToken = 'color/line/wire', glowToken = 'color/fill/gold', strokeWidth, tail = 90, origin = true, nodes = true, end = 'none', seed = 3, ...timing}:DrawLineProps) {
  const {p, t, d} = useTiming(timing, DRAW_LINE_DEFAULT_FRAMES);
  if (p <= 0 || points.length < 2) return null;
  const width = strokeWidth ?? strokeWidthOf('stroke/thin');
  const P = DRAW_LINE_PHASES;
  const drawn = cubicInOut(span(p, P.draw[0], P.draw[1]));
  const settle = span(p, P.settle[0], P.settle[1]);
  const total = polylineLength(points), head = drawn*total;
  const ink = color(strokeToken), glow = color(glowToken);
  const originLevel = origin ? flickerLevel(seed, t, Math.max(4, P.origin[1]*d)) : 0;
  const xs = points.map(q => q[0]), ys = points.map(q => q[1]), pad = 40;
  const x0 = Math.min(...xs) - pad, y0 = Math.min(...ys) - pad, vw = Math.max(...xs) - x0 + pad, vh = Math.max(...ys) - y0 + pad;
  // Distance along the path to each vertex, to light node ticks as the head passes.
  const marks:number[] = [0];
  for (let i = 1; i < points.length; i++) marks.push(marks[i-1] + Math.hypot(points[i][0] - points[i-1][0], points[i][1] - points[i-1][1]));
  const arrived = drawn >= 1;
  const last = pointAt(points, total);
  const tick = width*1.9;
  return <svg data-bakeoff="opus/DrawLine" style={{position:'absolute', left:x0, top:y0, overflow:'visible', pointerEvents:'none'}} width={vw} height={vh} viewBox={`${x0} ${y0} ${vw} ${vh}`} fill="none">
    <Trace points={points} drawn={drawn} heat={1 - settle} cool={.62 + .38*settle} width={width} strokeToken={strokeToken} glowToken={glowToken} tail={tail}/>
    {nodes && points.slice(1, -1).map(([vx, vy], i) => {
      const passed = head - marks[i + 1];
      if (passed < 0) return null;
      // Each vertex flares for ~20 px of head travel, then holds as a quiet tick.
      const level = 1 + .8*Math.exp(-passed/20);
      return <rect key={i} x={vx - tick} y={vy - tick} width={tick*2} height={tick*2} stroke={ink} strokeWidth={width*.6} fill={color('color/bg/field')} style={lightStyle(level, glow)}/>;
    })}
    {origin && originLevel > 0 && <rect x={points[0][0] - tick*1.2} y={points[0][1] - tick*1.2} width={tick*2.4} height={tick*2.4} fill={ink} style={lightStyle(originLevel, glow)}/>}
    {arrived && end !== 'none' && (() => {
      const [ex, ey] = last.p, a = last.angle, land = clamp01((p - P.draw[1])/.06);
      const level = 1 + .7*(1 - settle);
      if (end === 'terminal') return <rect x={ex - tick*1.2} y={ey - tick*1.2} width={tick*2.4} height={tick*2.4} stroke={ink} strokeWidth={width*.7} fill={color('color/bg/field')} opacity={land} style={lightStyle(level, glow)}/>;
      const L = 12 + 2*width, S = 8 + width;
      const c = Math.cos(a), s = Math.sin(a);
      const pt = (along:number, across:number) => `${(ex + c*along - s*across).toFixed(2)} ${(ey + s*along + c*across).toFixed(2)}`;
      return <path d={`M${pt(-L*land, -S*land)}L${pt(0, 0)}L${pt(-L*land, S*land)}`} stroke={ink} strokeWidth={width} strokeLinecap="round" strokeLinejoin="round" style={lightStyle(level, glow)}/>;
    })()}
  </svg>;
}
