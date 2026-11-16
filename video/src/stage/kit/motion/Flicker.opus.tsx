import React from 'react';
import {color} from '../theme/tokens';
import {flickerLevel, lightStyle, powerDownState, useTiming, type Box, type Timing} from './core';

type FlickerBase = Timing & React.PropsWithChildren<{
  /** Pulse pattern seed. Same seed, same pulses, on every render. */
  seed?:number;
  /** 0 is a plain fade, 1 the full set of uneven pulses and overshoot. */
  intensity?:number;
  glowToken?:string;
  /** The wrapper fills its positioned parent by default; override for inline use. */
  style?:React.CSSProperties;
}>;

export type FlickerProps = FlickerBase;
export const FLICKER_DEFAULT_FRAMES = 18;
export const POWER_DOWN_DEFAULT_FRAMES = 20;

/**
 * Powers its children on like a HUD tube catching: a few uneven dim pulses, a bright ignition that overshoots
 * with a gold bloom, one small sag, then steady. Renders nothing before it starts.
 */
export function Flicker({seed = 1, intensity = 1, glowToken = 'color/fill/gold', style, children, ...timing}:FlickerProps) {
  const {t, d} = useTiming(timing, FLICKER_DEFAULT_FRAMES);
  const started = timing.progress !== undefined ? timing.progress > 0 : t > 0;
  if (!started) return null;
  const level = flickerLevel(seed, t, d, intensity);
  if (level <= 0) return null;
  return <div data-bakeoff="opus/Flicker" style={{position:'absolute', inset:0, ...style, ...lightStyle(level, color(glowToken))}}>{children}</div>;
}

export type PowerDownProps = FlickerBase & {
  /** The region that collapses. The CRT squeeze centres on it; without it the wrapper's centre is used. */
  box?:Box;
  /** 'crt' squeezes to a bright line and pinches to a point; 'flicker' dies out in place with no squeeze. */
  mode?:'crt'|'flicker';
};

/** The matching power-down: a flare, two uneven dips, then the content collapses to a line and out. */
export function PowerDown({seed = 1, intensity = 1, glowToken = 'color/fill/gold', box, mode = 'crt', style, children, ...timing}:PowerDownProps) {
  const {t, d} = useTiming(timing, POWER_DOWN_DEFAULT_FRAMES);
  const s = powerDownState(seed, t, d, intensity);
  if (s.level <= 0 || (mode === 'crt' && (s.sx <= 0 || s.sy <= 0))) return null;
  const origin = box ? `${box.x + box.w/2}px ${box.y + box.h/2}px` : '50% 50%';
  const level = mode === 'crt' ? s.level : s.sy < 1 ? s.level*s.sy : s.level;
  return <div data-bakeoff="opus/PowerDown" style={{position:'absolute', inset:0, ...style, transformOrigin:origin, transform:mode === 'crt' && (s.sx < 1 || s.sy < 1) ? `scale(${s.sx.toFixed(4)}, ${s.sy.toFixed(4)})` : undefined, ...lightStyle(level, color(glowToken))}}>{children}</div>;
}
