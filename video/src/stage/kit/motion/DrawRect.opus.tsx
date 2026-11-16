import React from 'react';
import {color} from '../theme/tokens';
import {notchPath} from '../primitives/notch';
import {Trace} from './Trace';
import {clamp01, cubicInOut, flickerLevel, lightStyle, span, stroke as strokeWidthOf, useTiming, type Box, type Point, type Timing} from './core';

export type RectCorners = 'tl-br'|'tr-bl';
export type NotchSide = 'top'|'bottom'|'both';

export type DrawRectProps = Timing & {
  box:Box;
  /** Which opposite pair of corners the draw starts from. The other pair is where the edges meet. */
  corners?:RectCorners;
  strokeToken?:string;
  fillToken?:string;
  /** Glow colour for the hot heads and the ignition overshoot. */
  glowToken?:string;
  strokeWidth?:number;
  /** Adds the kit's five-tier notch, centred on the chosen edge(s); it grows in after the edges close. */
  notch?:NotchSide|false;
  /** Fill flicker seed; change it per box so neighbouring boxes do not pulse in step. */
  seed?:number;
  /** 0 fades the fill in, 1 is the full CRT flicker. */
  intensity?:number;
  /** Small L brackets on the start corners that light before the edges run. */
  brackets?:boolean;
};

/** Phase windows as fractions of the run. Shared with sounds.ts so cues land on the same frames. */
export const DRAW_RECT_PHASES = {
  brackets:[0, .1],
  edges:[.06, .6],
  meet:.6,
  notch:[.6, .84],
  fill:[.62, 1],
} as const;
export const DRAW_RECT_DEFAULT_FRAMES = 36;

const NOTCH_SPAN = 348; // notchPath's wire runs 786..1134 around x = 960.

export function notchScale(w:number) { return Math.min(1, w*.42/NOTCH_SPAN); }

function Notch({box, side, p, ink, width}:{box:Box; side:'top'|'bottom'; p:number; ink:string; width:number}) {
  const s = notchScale(box.w), cx = box.x + box.w/2, y = side === 'top' ? box.y : box.y + box.h;
  return <path d={notchPath(y, side === 'top' ? 1 : -1, p)} transform={`translate(${cx} ${y}) scale(${s}) translate(-960 ${-y})`} vectorEffect="non-scaling-stroke" stroke={ink} strokeWidth={width} strokeLinecap="round" strokeLinejoin="round" fill="none"/>;
}

/**
 * A rectangle that draws itself: two opposite corners light, four edges run along the sides at right angles,
 * meet and close at the other two corners, then the fill flickers on.
 */
export function DrawRect({box, corners = 'tl-br', strokeToken = 'color/line/wire', fillToken = 'color/surface/1', glowToken = 'color/fill/gold', strokeWidth, notch = false, seed = 7, intensity = 1, brackets = true, ...timing}:DrawRectProps) {
  const {p, t, d} = useTiming(timing, DRAW_RECT_DEFAULT_FRAMES);
  if (p <= 0) return null;
  const width = strokeWidth ?? strokeWidthOf('stroke/chart');
  const {x, y, w, h} = box;
  const TL:Point = [x, y], TR:Point = [x + w, y], BR:Point = [x + w, y + h], BL:Point = [x, y + h];
  const [a, b, m1, m2] = corners === 'tl-br' ? [TL, BR, TR, BL] : [TR, BL, TL, BR];
  // Each start corner sends one horizontal and one vertical edge. Speeds differ so all four land on the same frame.
  const lines:Point[][] = [[a, [m1[0], a[1]]], [a, [a[0], m2[1]]], [b, [m2[0], b[1]]], [b, [b[0], m1[1]]]];
  const P = DRAW_RECT_PHASES;
  const e = cubicInOut(span(p, P.edges[0], P.edges[1]));
  const closed = p >= P.meet;
  const meetT = P.meet*d;
  // Heads cool over six frames after the meet; the whole frame flashes once as it closes.
  const sinceMeet = t - meetT;
  // Both decay to exactly rest by p = 1, so the finished state carries no residual glow however short the run.
  const rest = 1 - span(p, P.meet, 1);
  const heat = closed ? clamp01(1 - sinceMeet/6)*rest : 1;
  const flash = closed ? 1 + .55*Math.exp(-sinceMeet/2.2)*rest : 1;
  const fillLevel = flickerLevel(seed, t - P.fill[0]*d, (P.fill[1] - P.fill[0])*d, intensity);
  const notchP = span(p, P.notch[0], P.notch[1]);
  const ink = color(strokeToken), glow = color(glowToken);
  // The notch's closely stacked tiers need a finer pen than the frame so they
  // remain individually legible at the top edge.
  const notchWidth = Math.min(width, strokeWidthOf('stroke/chart')*.7);
  const sides:('top'|'bottom')[] = notch === 'both' ? ['top', 'bottom'] : notch ? [notch] : [];
  const pad = 40;
  const bracketL = Math.min(14, w*.2, h*.2);
  const bracketLevel = brackets ? flickerLevel(seed + 101, t - P.brackets[0]*d, Math.max(4, (P.brackets[1] - P.brackets[0])*d), intensity) : 0;
  const bracket = ([cx, cy]:Point) => {
    const sx = cx === x ? 1 : -1, sy = cy === y ? 1 : -1, o = 5;
    return `M${cx - sx*o} ${cy - sy*o + sy*bracketL}V${cy - sy*o}H${cx - sx*o + sx*bracketL}`;
  };
  // Straight edges keep a gap where a notch sits once the frame has closed, so notchPath(0) continues the wire exactly.
  const s = notchScale(w), half = NOTCH_SPAN/2*s, cx = x + w/2;
  const closedEdges = () => {
    const segs:string[] = [];
    const run = (y0:number, notched:boolean) => notched ? `M${x} ${y0}H${cx - half}M${cx + half} ${y0}H${x + w}` : `M${x} ${y0}H${x + w}`;
    segs.push(run(y, sides.includes('top')), run(y + h, sides.includes('bottom')), `M${x} ${y}V${y + h}M${x + w} ${y}V${y + h}`);
    return segs.join('');
  };
  return <svg data-bakeoff="opus/DrawRect" style={{position:'absolute', left:x - pad, top:y - pad, overflow:'visible', pointerEvents:'none'}} width={w + 2*pad} height={h + 2*pad} viewBox={`${x - pad} ${y - pad} ${w + 2*pad} ${h + 2*pad}`} fill="none">
    {fillLevel > 0 && <rect x={x} y={y} width={w} height={h} fill={color(fillToken)} style={lightStyle(fillLevel, glow)}/>}
    {brackets && bracketLevel > 0 && <path d={bracket(a) + bracket(b)} stroke={ink} strokeWidth={width} strokeLinecap="square" style={lightStyle(bracketLevel*(closed ? Math.max(.45, 1 - sinceMeet/12) : 1), glow)}/>}
    {!closed
      ? lines.map((pts, i) => <Trace key={i} points={pts} drawn={e} width={width} strokeToken={strokeToken} glowToken={glowToken} tail={Math.min(56, Math.max(w, h)*.25)}/>)
      : <g style={lightStyle(flash, glow)}>
          <path d={closedEdges()} stroke={ink} strokeWidth={width} strokeLinecap="square"/>
          {sides.map(side => <Notch key={side} box={box} side={side} p={notchP} ink={ink} width={notchWidth}/>)}
          {heat > 0 && [m1, m2].map(([mx, my], i) => <rect key={i} x={mx - width*1.6} y={my - width*1.6} width={width*3.2} height={width*3.2} fill={ink} opacity={heat} style={{filter:`brightness(1.8) drop-shadow(0 0 ${2 + 8*heat}px ${glow})`}}/>)}
        </g>}
  </svg>;
}
