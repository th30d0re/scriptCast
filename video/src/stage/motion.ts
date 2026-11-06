import {interpolateZoom} from 'd3-interpolate';
import {geoInterpolate} from 'd3-geo';

export const clamp = (n: number, low = 0, high = 1) => Math.max(low, Math.min(high, n));
export const mix = (a: number, b: number, t: number) => a + (b - a) * t;
export const linear = (t: number) => t;
export const cubicIn = (t: number) => t ** 3;
export const cubicOut = (t: number) => 1 - (1 - t) ** 3;
export const cubicInOut = (t: number) => t < 0.5 ? 4 * t ** 3 : 1 - (-2 * t + 2) ** 3 / 2;
export const expoOut = (t: number) => t === 1 ? 1 : 1 - 2 ** (-10 * t);
export const backOut = (t: number) => 1 + 2.70158 * (t - 1) ** 3 + 1.70158 * (t - 1) ** 2;
export const easing = {linear, cubicIn, cubicOut, cubicInOut, expoOut, backOut};
export type Ease = (t: number) => number;
export const progress = (ms: number, from: number, to: number) => to <= from ? Number(ms >= to) : clamp((ms - from) / (to - from));
export function tween<T extends number | number[]>(ms: number, keys: {at: number; value: T}[], ease: Ease = linear): T {
  if (!keys.length) throw new Error('tween requires keys');
  if (keys.some((k, i) => i > 0 && k.at <= keys[i-1].at)) throw new Error('tween keys must increase');
  if (ms <= keys[0].at) return keys[0].value;
  for (let i = 1; i < keys.length; i++) {
    if (ms <= keys[i].at) {
      const a = keys[i-1], b = keys[i], t = ease(progress(ms, a.at, b.at));
      if (Array.isArray(a.value) && Array.isArray(b.value)) {
        const values = b.value;
        if (a.value.length !== values.length) throw new Error('tween vector sizes differ');
        return a.value.map((v, j) => mix(v, values[j], t)) as T;
      }
      if (typeof a.value !== 'number' || typeof b.value !== 'number') throw new Error('tween key types differ');
      return mix(a.value, b.value, t) as T;
    }
  }
  return keys[keys.length-1].value;
}

export type FlyCamera = {center: [number, number]; zoom: number};
const WORLD_SPAN_DEG = 360;
export type FlyKey = {at: number; camera: FlyCamera};

/**
 * A deterministic van Wijk camera flight. `interpolateZoom` treats inverse zoom as
 * viewport width; its planar centre progress is mapped onto the geographic
 * great-circle so long moves never sweep across an unrelated part of the map.
 * Key times, rather than d3's suggested duration, remain authoritative.
 */
export function flyTo(ms: number, keys: FlyKey[], ease: Ease = linear): FlyCamera {
  if (!keys.length) throw new Error('flyTo requires keys');
  if (keys.some((key, i) => key.camera.zoom <= 0 || (i > 0 && key.at <= keys[i-1].at))) {
    throw new Error('flyTo keys must increase and zooms must be positive');
  }
  if (ms <= keys[0].at) return keys[0].camera;
  for (let i = 1; i < keys.length; i++) {
    if (ms <= keys[i].at) {
      const from = keys[i-1], to = keys[i];
      const t = ease(progress(ms, from.at, to.at));
      const [x0, y0] = from.camera.center;
      const rawLonDelta = to.camera.center[0]-x0;
      const lonDelta = ((rawLonDelta+540)%360)-180;
      const x1 = x0+lonDelta, y1 = to.camera.center[1];
      // van Wijk needs viewport width in the centre's units: degrees visible, 360 at zoom 1.
      const interpolate = interpolateZoom([x0, y0, WORLD_SPAN_DEG/from.camera.zoom], [x1, y1, WORLD_SPAN_DEG/to.camera.zoom]);
      const [x, y, span] = interpolate(t);
      const inverseZoom = span/WORLD_SPAN_DEG;
      const distanceSquared = (x1-x0)**2+(y1-y0)**2;
      const along = distanceSquared === 0 ? t : clamp(((x-x0)*(x1-x0)+(y-y0)*(y1-y0))/distanceSquared);
      const center = geoInterpolate(from.camera.center, to.camera.center)(along) as [number, number];
      return {center, zoom: 1/inverseZoom};
    }
  }
  return keys[keys.length-1].camera;
}
/** Unit-mass spring response, with zero initial position and velocity; time in ms. */
export function settle(sinceMs: number, {stiffness = 100, damping = 20}: {stiffness?: number; damping?: number} = {}) {
  if (stiffness <= 0 || damping < 0) throw new Error('Invalid spring parameters');
  const t = Math.max(0, sinceMs) / 1000, w = Math.sqrt(stiffness), a = damping / 2;
  if (Math.abs(a - w) < 1e-8) return 1 - Math.exp(-w*t)*(1+w*t);
  if (a < w) {
    const b = Math.sqrt(stiffness-a*a);
    return 1-Math.exp(-a*t)*(Math.cos(b*t)+a/b*Math.sin(b*t));
  }
  const b = Math.sqrt(a*a-stiffness), r1 = -a+b, r2 = -a-b;
  return 1+(r2*Math.exp(r1*t)-r1*Math.exp(r2*t))/(r1-r2);
}
