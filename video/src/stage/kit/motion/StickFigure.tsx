import React from 'react';
import {color, skinToneColor, needsFigureRim, stickGroupColor} from '../theme/tokens';
import type {PersonGroup} from '../components/types';
import type {PersonMarkProps} from '../primitives/PersonMark';
import {clamp01, cubicOut, lightStyle, span, useTiming, type Timing} from './core';
import {useKitSounds, type KitSoundCueProps} from './KitSound';
import {StickFigure as ArgonStickFigure} from './variants/argon/StickFigure';
import {StickFigure as AstraStickFigure} from './variants/astra/primitives';

export type StickPose = 'stand'|'open'|'raise';

export type StickFigureProps = PersonMarkProps & Timing & {
  variant?:'opus'|'argon'|'astra';
  pose?:StickPose;
  /** Set false to show the figure fully drawn regardless of timing (the default when no timing prop is given). */
  animate?:boolean;
} & KitSoundCueProps;

// Same colour mapping as PersonMark so a StickFigure can stand in for one:
// In-group is white (cream token), out-group is black (#0B0B0B), neutral stays muted grey.
export {stickGroupColor};

/** Strokes in a 44 × 58 box, the PersonMark footprint. Order is the draw-on order. */
const POSES:Record<StickPose, {arms:string; legs:string}> = {
  stand:{arms:'M22 23L13 34M22 23L31 34', legs:'M22 36L15 53M22 36L29 53'},
  open:{arms:'M22 24L9 21M22 24L35 21', legs:'M22 36L13 53M22 36L31 53'},
  raise:{arms:'M22 23L13 34M22 23L31 19L33 7', legs:'M22 36L15 53M22 36L29 53'},
};
const TORSO = 'M22 18.5V36';
import {STICK_DEFAULT_FRAMES, STICK_PHASES} from './stickTiming';
export {STICK_DEFAULT_FRAMES, STICK_PHASES};

/**
 * A person glyph — head, torso, two arms, two legs — in clean square-jointed strokes. Drop-in for PersonMark
 * (same x, y, group, emphasis, scale; centred on x, y), with optional poses and a stroke-by-stroke draw-on.
 */
export function StickFigure({variant='opus', x, y, group, emphasis:rawEmphasis, scale, pose = 'stand', animate, tone, beat, beatCue, ...timing}:StickFigureProps) {
  const audio = useKitSounds('StickFigure',variant,timing.progress === undefined && (animate ?? timing.startFrame !== undefined) ? timing.startFrame : undefined,undefined,beat,beatCue);
  if (variant === 'argon') return <><ArgonStickFigure x={x} y={y} group={group} emphasis={rawEmphasis} scale={scale} animate={animate} tone={tone} {...timing}/>{audio}</>;
  if (variant === 'astra') return <><AstraStickFigure x={x} y={y} group={group} emphasis={rawEmphasis} scale={scale} tone={tone} {...timing}/>{audio}</>;
  const timed = animate ?? (timing.progress !== undefined || timing.startFrame !== undefined);
  const {p} = useTiming(timing, STICK_DEFAULT_FRAMES);
  const q = timed ? p : 1;
  if (q <= 0) return <>{audio}</>;
  const emphasis = clamp01(rawEmphasis), size = 44*Math.max(.2, scale);
  const P = STICK_PHASES, part = (w:readonly [number, number]) => cubicOut(span(q, w[0], w[1]));
  const head = part(P.head), torso = part(P.torso), arms = part(P.arms), legs = part(P.legs);
  // Settles from a short warm bloom as the last leg lands; at rest the level is exactly 1.
  const bloom = q >= 1 ? 1 : 1 + .45*span(q, P.legs[0], P.legs[1])*(1 - span(q, P.settle[0], P.settle[1]));
  const ink = tone !== undefined ? skinToneColor(tone) : stickGroupColor(group);
  const needsRim = needsFigureRim(tone, group);
  // Dark figures get a heavier core so the black reads as a solid body inside its light rim.
  const width = 3 + 1.2*emphasis + (needsRim ? 1.4 : 0);
  const rimColor = color('color/line/hairline-strong');
  const rimWidth = 0.9;
  const totalRimWidth = width + rimWidth * 2;
  // Heavier emphasis fills the head in, matching PersonMark's solid head at full emphasis.
  const headFill = clamp01((emphasis - .45)/.55);
  // Paired limbs grow together: each subpath gets its own normalized dash.
  const seg = (d:string, k:number, key:string, strokeColor:string, strokeW:number) => k > 0 && d.split('M').filter(Boolean).map((sub, i) =>
    <path key={`${key}${i}`} d={`M${sub}`} pathLength={1} strokeDasharray={`${k} 1`} stroke={strokeColor} strokeWidth={strokeW} strokeLinecap="round" strokeLinejoin="round"/>);
  return <><div data-bakeoff="opus/StickFigure" data-shared="PersonMark" data-group={group} style={{position:'absolute', left:x, top:y, width:size, height:size*1.3, transform:'translate(-50%,-50%)', opacity:.42 + .58*emphasis, color:ink}}>
    <svg width="100%" height="100%" viewBox="0 0 44 58" fill="none" aria-hidden="true" overflow="visible" style={lightStyle(bloom, ink)}>
      {needsRim && head > 0 && <circle cx="22" cy="11" r="6.4" transform="rotate(-90 22 11)" pathLength={1} strokeDasharray={head >= 1 ? undefined : `${head} 1`} stroke={rimColor} strokeWidth={totalRimWidth} fill="none"/>}
      {needsRim && seg(TORSO, torso, 'torso-rim', rimColor, totalRimWidth)}
      {needsRim && seg(POSES[pose].arms, arms, 'arms-rim', rimColor, totalRimWidth)}
      {needsRim && seg(POSES[pose].legs, legs, 'legs-rim', rimColor, totalRimWidth)}
      {head > 0 && <circle cx="22" cy="11" r="6.4" transform="rotate(-90 22 11)" pathLength={1} strokeDasharray={head >= 1 ? undefined : `${head} 1`} stroke={ink} strokeWidth={width} fill={ink} fillOpacity={headFill*head}/>}
      {seg(TORSO, torso, 'torso', ink, width)}
      {seg(POSES[pose].arms, arms, 'arms', ink, width)}
      {seg(POSES[pose].legs, legs, 'legs', ink, width)}
    </svg>
  </div>{audio}</>;
}
