import React from 'react';
import {color} from '../theme/tokens';
import type {PersonGroup} from '../components/types';
import type {PersonMarkProps} from '../primitives/PersonMark';
import {clamp01, cubicOut, lightStyle, span, useTiming, type Timing} from './core';

export type StickPose = 'stand'|'open'|'raise';

export type StickFigureProps = PersonMarkProps & Timing & {
  pose?:StickPose;
  /** Set false to show the figure fully drawn regardless of timing (the default when no timing prop is given). */
  animate?:boolean;
};

// Same colour mapping as PersonMark so a StickFigure can stand in for one.
export const stickGroupColor = (group:PersonGroup) => group === 'ingroup'
  ? color('color/fill/gold')
  : group === 'outgroup' ? color('color/marker/race') : color('color/text/muted');

/** Strokes in a 44 × 58 box, the PersonMark footprint. Order is the draw-on order. */
const POSES:Record<StickPose, {arms:string; legs:string}> = {
  stand:{arms:'M22 23L13 34M22 23L31 34', legs:'M22 36L15 53M22 36L29 53'},
  open:{arms:'M22 24L9 21M22 24L35 21', legs:'M22 36L13 53M22 36L31 53'},
  raise:{arms:'M22 23L13 34M22 23L31 19L33 7', legs:'M22 36L15 53M22 36L29 53'},
};
const TORSO = 'M22 18.5V36';
export const STICK_PHASES = {head:[0, .3], torso:[.22, .46], arms:[.4, .7], legs:[.6, .92], settle:[.92, 1]} as const;
export const STICK_DEFAULT_FRAMES = 20;

/**
 * A person glyph — head, torso, two arms, two legs — in clean square-jointed strokes. Drop-in for PersonMark
 * (same x, y, group, emphasis, scale; centred on x, y), with optional poses and a stroke-by-stroke draw-on.
 */
export function StickFigure({x, y, group, emphasis:rawEmphasis, scale, pose = 'stand', animate, ...timing}:StickFigureProps) {
  const timed = animate ?? (timing.progress !== undefined || timing.startFrame !== undefined);
  const {p} = useTiming(timing, STICK_DEFAULT_FRAMES);
  const q = timed ? p : 1;
  if (q <= 0) return null;
  const emphasis = clamp01(rawEmphasis), size = 44*Math.max(.2, scale);
  const P = STICK_PHASES, part = (w:readonly [number, number]) => cubicOut(span(q, w[0], w[1]));
  const head = part(P.head), torso = part(P.torso), arms = part(P.arms), legs = part(P.legs);
  // Settles from a short warm bloom as the last leg lands; at rest the level is exactly 1.
  const bloom = q >= 1 ? 1 : 1 + .45*span(q, P.legs[0], P.legs[1])*(1 - span(q, P.settle[0], P.settle[1]));
  const ink = stickGroupColor(group);
  const width = 3 + 1.2*emphasis;
  // Heavier emphasis fills the head in, matching PersonMark's solid head at full emphasis.
  const headFill = clamp01((emphasis - .45)/.55);
  // Paired limbs grow together: each subpath gets its own normalized dash.
  const seg = (d:string, k:number, key:string) => k > 0 && d.split('M').filter(Boolean).map((sub, i) =>
    <path key={`${key}${i}`} d={`M${sub}`} pathLength={1} strokeDasharray={`${k} 1`} stroke="currentColor" strokeWidth={width} strokeLinecap="round" strokeLinejoin="round"/>);
  return <div data-bakeoff="opus/StickFigure" data-group={group} style={{position:'absolute', left:x, top:y, width:size, height:size*1.3, transform:'translate(-50%,-50%)', opacity:.42 + .58*emphasis, color:ink}}>
    <svg width="100%" height="100%" viewBox="0 0 44 58" fill="none" aria-hidden="true" overflow="visible" style={lightStyle(bloom, ink)}>
      {head > 0 && <circle cx="22" cy="11" r="6.4" transform="rotate(-90 22 11)" pathLength={1} strokeDasharray={head >= 1 ? undefined : `${head} 1`} stroke="currentColor" strokeWidth={width} fill="currentColor" fillOpacity={headFill*head}/>}
      {seg(TORSO, torso, 'torso')}
      {seg(POSES[pose].arms, arms, 'arms')}
      {seg(POSES[pose].legs, legs, 'legs')}
    </svg>
  </div>;
}
