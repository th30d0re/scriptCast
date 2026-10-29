import {createContext, useContext, type CSSProperties, type ReactNode} from 'react';

// Card motion is written in beats: one beat is one frame at ENTRANCE_FPS. A card's
// `progress` (0 to 1) covers its whole entrance, so the beat is progress times the
// card's entrance length. No clock is read here; the caller supplies progress.
export const ENTRANCE_FPS = 30;

export const clamp01 = (x: number) => Math.min(1, Math.max(0, x));
export const easeOutCubic = (x: number) => 1 - Math.pow(1 - x, 3);

// Map x from [inMin, inMax] to [outMin, outMax], clamped at both ends, with optional easing.
export function interpolate(x: number, [inMin, inMax]: [number, number], [outMin, outMax]: [number, number], easing: (t: number) => number = t => t): number {
  const t = inMax === inMin ? (x >= inMax ? 1 : 0) : clamp01((x - inMin) / (inMax - inMin));
  return outMin + easing(t) * (outMax - outMin);
}

// Current beat of the card being drawn. Frame provides it; the default shows the finished card.
const BeatContext = createContext(Number.POSITIVE_INFINITY);
export const BeatProvider = BeatContext.Provider;

// Entrance value 0 to 1 over `dur` beats after `delay` beats.
export function useEnter(delay = 0, dur = 12): number {
  return interpolate(useContext(BeatContext), [delay, delay + dur], [0, 1], easeOutCubic);
}

// Opacity only: a transform would move the boxes Frame measures when it fits type to the card.
export function Reveal({delay = 0, dur = 12, children, style}: {delay?: number; dur?: number; children: ReactNode; style?: CSSProperties}) {
  const t = useEnter(delay, dur);
  return <div style={{opacity: t, ...style}}>{children}</div>;
}

// Beats Frame's own motion needs (headline rule 0..8, sources 6..14).
export const FRAME_BEATS = 14;
// Beats for a list whose item i reveals at i * step for `dur` beats.
export const staggerBeats = (count: number, step: number, dur: number) => Math.max(FRAME_BEATS, count > 0 ? (count - 1) * step + dur : 0);
