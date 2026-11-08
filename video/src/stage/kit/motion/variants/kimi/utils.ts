import {useKitFrame} from '../../core';

export type Box = {x: number; y: number; w: number; h: number};
export type Point = readonly [number, number];
export type TimingProps = {
  progress?: number;
  startFrame?: number;
  durationFrames?: number;
};

export const clamp01 = (value: number) => Math.max(0, Math.min(1, value));
export const mix = (from: number, to: number, amount: number) => from + (to - from) * amount;
export const cubicOut = (value: number) => 1 - (1 - clamp01(value)) ** 3;
export const cubicInOut = (value: number) => {
  const t = clamp01(value);
  return t < 0.5 ? 4 * t ** 3 : 1 - (-2 * t + 2) ** 3 / 2;
};

export function hash01(seed: number, frame: number) {
  let value = (Math.imul(seed | 0, 0x45d9f3b) ^ Math.imul(frame | 0, 0x27d4eb2d)) >>> 0;
  value = Math.imul(value ^ (value >>> 16), 0x45d9f3b) >>> 0;
  value = Math.imul(value ^ (value >>> 16), 0x45d9f3b) >>> 0;
  value = (value ^ (value >>> 16)) >>> 0;
  return value / 0xffffffff;
}

export function useAnimationState({progress, startFrame = 0, durationFrames = 30}: TimingProps) {
  const currentFrame = useKitFrame();
  const duration = Math.max(1, durationFrames);
  const frame = progress === undefined ? currentFrame - startFrame : Math.round(progress * duration);
  return {frame, progress: clamp01(frame / duration), duration};
}

export function segmentLength(a: Point, b: Point) {
  return Math.hypot(b[0] - a[0], b[1] - a[1]);
}

export function pointAtDistance(points: readonly Point[], distance: number): Point {
  if (points.length === 0) return [0, 0];
  if (points.length === 1) return points[0];
  let remaining = Math.max(0, distance);
  for (let index = 1; index < points.length; index++) {
    const from = points[index - 1];
    const to = points[index];
    const length = segmentLength(from, to);
    if (remaining <= length) {
      const amount = length === 0 ? 0 : remaining / length;
      return [mix(from[0], to[0], amount), mix(from[1], to[1], amount)];
    }
    remaining -= length;
  }
  return points[points.length - 1];
}

export function polylineLength(points: readonly Point[]) {
  let length = 0;
  for (let index = 1; index < points.length; index++) length += segmentLength(points[index - 1], points[index]);
  return length;
}

export function polylineSlice(points: readonly Point[], fromDistance: number, toDistance: number) {
  if (points.length < 2 || toDistance <= fromDistance) return '';
  const total = polylineLength(points);
  const from = Math.max(0, Math.min(total, fromDistance));
  const to = Math.max(from, Math.min(total, toDistance));
  const slice: Point[] = [pointAtDistance(points, from)];
  let traversed = 0;
  for (let index = 1; index < points.length; index++) {
    traversed += segmentLength(points[index - 1], points[index]);
    if (traversed > from && traversed < to) slice.push(points[index]);
  }
  slice.push(pointAtDistance(points, to));
  return slice.map((point, index) => `${index === 0 ? 'M' : 'L'}${point[0].toFixed(2)} ${point[1].toFixed(2)}`).join(' ');
}
