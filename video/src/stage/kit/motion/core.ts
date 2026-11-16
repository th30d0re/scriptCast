// Shared timing, hashing and flicker envelopes for the opus bake-off primitives.
// Everything here is a pure function of its arguments: no Math.random, no Date, no CSS animation.
import {useContext} from 'react';
import {Internals, useCurrentFrame} from 'remotion';
import {ClockContext, NO_CLOCK} from '../../clock';
import {cubicInOut, cubicOut, progress as span} from '../../motion';
import {dimensions} from '../theme/tokens';

export type Box = {x:number; y:number; w:number; h:number};
export type Point = readonly [number, number];

/** Either drive an animation with `progress` (0..1), or let it run off the frame clock from `startFrame` for `durationFrames`. */
export type Timing = {progress?:number; startFrame?:number; durationFrames?:number};

export const clamp01 = (v:number) => Number.isFinite(v) ? Math.max(0, Math.min(1, v)) : 0;
export const mix = (a:number, b:number, t:number) => a + (b - a)*t;
export {span, cubicInOut, cubicOut};
export const stroke = (name:'stroke/wire'|'stroke/thin'|'stroke/chart'|'stroke/accent'|'stroke/rim') => dimensions[name].value;
export const spacing = (name:'spacing/xs'|'spacing/sm'|'spacing/md'|'spacing/lg'|'spacing/xl') => dimensions[name].value;

/**
 * Resolves a Timing into normalized progress `p` and local time `t` in frames.
 * When only `progress` is given, `t` is reconstructed as progress × duration so seeded flicker still steps per frame.
 */
/**
 * The frame kit timing is measured on. Inside a stage clock (RemotionClock / ManualClock) this is stage time in frames,
 * so startFrame can be a cue time (ms × fps / 1000); outside one it is the Remotion frame.
 */
/** Remotion frame, or null outside a composition (the browser review player drives a stage clock with no Remotion timeline). */
function useRemotionFrame():number|null {
  const inRemotion = useContext(Internals.CanUseRemotionHooks);
  // useCurrentFrame only reads contexts, and inRemotion is fixed for a mounted tree, so the conditional call is stable.
  return inRemotion ? useCurrentFrame() : null;
}
export function useKitFrame() {
  const clock = useContext(ClockContext), frame = useRemotionFrame();
  return clock === NO_CLOCK ? (frame ?? 0) : Math.round(clock.ms*clock.fps/1000);
}
/** Kit frame minus Remotion frame: add to a Remotion-local Sequence offset to land on kit time. */
export function useKitFrameOffset() {
  const kit = useKitFrame(), frame = useRemotionFrame();
  return frame === null ? 0 : kit - frame;
}
export function useTiming(timing:Timing, defaultDuration:number) {
  const frame = useKitFrame();
  return resolveTiming(timing, defaultDuration, frame);
}
export function resolveTiming({progress, startFrame = 0, durationFrames}:Timing, defaultDuration:number, frame:number) {
  const d = Math.max(1, durationFrames ?? defaultDuration);
  if (progress !== undefined) {
    const p = clamp01(progress);
    return {p, t:p*d, d};
  }
  const t = Math.max(0, Math.min(d, frame - startFrame));
  return {p:t/d, t, d};
}

/** Integer hash of (seed, a, b) mapped to [0, 1). Same inputs, same output, on every render. */
export function hash(seed:number, a = 0, b = 0) {
  let h = (Math.imul(seed | 0, 0x9e3779b1) ^ Math.imul(a | 0, 0x85ebca6b) ^ Math.imul(b | 0, 0xc2b2ae35)) >>> 0;
  h ^= h >>> 16; h = Math.imul(h, 0x7feb352d) >>> 0;
  h ^= h >>> 15; h = Math.imul(h, 0x846ca68b) >>> 0;
  h ^= h >>> 16;
  return (h >>> 0)/4294967296;
}

export type Pulse = {start:number; length:number; level:number};

/**
 * The seeded power-on schedule: two to four uneven pulses inside the first 58% of the run, each brighter than the last,
 * then ignition. Times are integer frames relative to the start, so sound cues can be read straight off it.
 */
export function flickerSchedule(seed:number, duration:number, intensity = 1) {
  const d = Math.max(4, Math.round(duration));
  const ignite = Math.max(2, Math.round(d*.58));
  const count = intensity <= 0 ? 0 : 2 + Math.floor(hash(seed, 1)*(1 + 2*clamp01(intensity)) * .999);
  const pulses:Pulse[] = [];
  // Slots split [0, ignite) into `count` uneven windows; each pulse sits at a seeded spot in its window.
  let cursor = 0;
  for (let i = 0; i < count; i++) {
    const room = (ignite - cursor)/(count - i);
    const jitter = hash(seed, 10 + i);
    const start = Math.floor(cursor + room*(.15 + .5*jitter));
    const length = Math.max(1, Math.min(Math.floor(room*.6), 1 + Math.floor(hash(seed, 20 + i)*2.4)));
    const level = Math.min(.95, .32 + .55*(i + 1)/count + .12*(hash(seed, 30 + i) - .5));
    if (start + length < ignite) pulses.push({start, length, level});
    cursor = Math.max(cursor + room, start + length + 1);
  }
  // One single-frame sag shortly after ignition: the tube catching, not a glitch.
  const sag = ignite + 1 + Math.floor(hash(seed, 40)*Math.max(1, (d - ignite)*.4));
  return {pulses, ignite, sag: sag < d - 1 && intensity > .25 ? sag : -1, duration:d};
}

/**
 * Power-on light level at local frame `t` (fractional values are floored so flicker steps per frame).
 * 0 is dark, 1 is steady, values above 1 are the ignition overshoot.
 */
export function flickerLevel(seed:number, t:number, duration:number, intensity = 1) {
  const k = clamp01(intensity);
  const {pulses, ignite, sag, duration:d} = flickerSchedule(seed, duration, k);
  const f = Math.floor(t);
  if (t >= d) return 1;
  const fade = cubicOut(clamp01(t/d));
  if (k === 0) return fade;
  let level = 0;
  if (f < ignite) {
    for (const pulse of pulses) if (f >= pulse.start && f < pulse.start + pulse.length) level = pulse.level;
    // A faint afterglow between pulses keeps the off-states from reading as a hole in the frame.
    if (level === 0) level = .05*k*clamp01(f/ignite);
  } else {
    const since = (t - ignite)/Math.max(1, d - ignite);
    level = 1 + .5*k*Math.exp(-5*since)*(1 - since);
    if (f === sag) level *= .62;
  }
  return mix(fade, level, k);
}

/** Maps a light level to CSS: opacity carries 0..1, brightness and a gold glow carry the overshoot. */
export function lightStyle(level:number, glowToken:string) {
  const over = Math.max(0, level - 1);
  return {
    opacity:clamp01(level),
    filter:over > .005 ? `brightness(${(1 + 1.4*over).toFixed(3)}) drop-shadow(0 0 ${(3 + 22*over).toFixed(2)}px ${glowToken})` : undefined,
  };
}

/** Power-down schedule: a flare, two uneven dips, then the collapse. Returned as frame offsets. */
export function powerDownSchedule(seed:number, duration:number) {
  const d = Math.max(6, Math.round(duration));
  const flare = 0;
  const dipA = Math.max(1, Math.round(d*(.14 + .06*hash(seed, 50))));
  const dipB = Math.max(dipA + 2, Math.round(d*(.32 + .08*hash(seed, 51))));
  const collapse = Math.round(d*.5);
  return {flare, dipA, dipB, collapse, duration:d};
}

/** Power-down light level and CRT collapse scales at local frame `t`. */
export function powerDownState(seed:number, t:number, duration:number, intensity = 1) {
  const k = clamp01(intensity);
  const {dipA, dipB, collapse, duration:d} = powerDownSchedule(seed, duration);
  const f = Math.floor(t);
  if (t <= 0) return {level:1, sx:1, sy:1};
  if (t >= d) return {level:0, sx:0, sy:0};
  let level = 1;
  if (f < dipA) level = 1 + .35*k*(1 - f/dipA);
  else if (f < dipA + 1) level = 1 - .7*k;
  else if (f < dipB) level = 1;
  else if (f < dipB + 2) level = f === dipB ? 1 - .45*k : 1 - .85*k;
  else if (f < collapse) level = 1 + .15*k;
  if (f < collapse) return {level, sx:1, sy:1};
  // CRT collapse: squeeze to a bright scan line, then pinch the line to a point.
  const c = (t - collapse)/(d - collapse);
  const sy = Math.max(.012, 1 - cubicInOut(clamp01(c/.55)));
  const sx = 1 - cubicInOut(clamp01((c - .5)/.5));
  return {level:1 + .9*k*clamp01(c/.55) - 1.9*clamp01((c - .78)/.22), sx, sy};
}

/** Cumulative polyline geometry: total length and the points up to `length`. */
export function polylineLength(points:readonly Point[]) {
  let total = 0;
  for (let i = 1; i < points.length; i++) total += Math.hypot(points[i][0] - points[i-1][0], points[i][1] - points[i-1][1]);
  return total;
}
export function pointAt(points:readonly Point[], length:number):{p:Point; angle:number; index:number} {
  let run = 0;
  for (let i = 1; i < points.length; i++) {
    const [x0, y0] = points[i-1], [x1, y1] = points[i];
    const seg = Math.hypot(x1 - x0, y1 - y0);
    if (run + seg >= length || i === points.length - 1) {
      const u = seg === 0 ? 0 : clamp01((length - run)/seg);
      return {p:[x0 + (x1 - x0)*u, y0 + (y1 - y0)*u], angle:Math.atan2(y1 - y0, x1 - x0), index:i - 1};
    }
    run += seg;
  }
  return {p:points[0], angle:0, index:0};
}
/** The sub-polyline between lengths a and b. */
export function subPolyline(points:readonly Point[], a:number, b:number):Point[] {
  if (b <= a || points.length < 2) return [];
  const out:Point[] = [pointAt(points, a).p];
  let run = 0;
  for (let i = 1; i < points.length; i++) {
    run += Math.hypot(points[i][0] - points[i-1][0], points[i][1] - points[i-1][1]);
    if (run > a && run < b) out.push(points[i]);
  }
  out.push(pointAt(points, b).p);
  return out;
}
export const toPath = (points:readonly Point[]) => points.map(([x, y], i) => `${i ? 'L' : 'M'}${x.toFixed(2)} ${y.toFixed(2)}`).join('');
