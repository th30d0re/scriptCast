import React from 'react';
import {clamp01, hash01} from './utils';
import {useCurrentFrame} from 'remotion';

export type FlickerProps = React.PropsWithChildren<{
  seed: number;
  startFrame?: number;
  durationFrames?: number;
  intensity?: number;
  progress?: number;
}>;

type PowerSample = {opacity: number; brightness: number};

function powerOnSample(seed: number, frame: number, duration: number, intensity: number): PowerSample {
  if (frame <= 0) return {opacity: 0, brightness: 1};
  if (frame >= duration) return {opacity: 1, brightness: 1};
  const p = clamp01(frame / duration);
  const noise = 1 - intensity * 0.1 + hash01(seed, frame) * 0.1 * intensity;
  let gate = 1;
  if (p < 0.1) gate = 0;
  else if (p < 0.23) gate = 0.78;
  else if (p < 0.31) gate = 0.1;
  else if (p < 0.47) gate = 0.96;
  else if (p < 0.56) gate = 0.3;
  else if (p < 0.74) gate = 1;
  const opacity = p >= 0.74 ? 1 : (1 - (1 - gate) * intensity) * noise;
  const overshoot = p >= 0.58 && p < 0.9 ? Math.sin((p - 0.58) / 0.32 * Math.PI) * 0.62 * intensity : 0;
  return {opacity: clamp01(opacity), brightness: 1 + overshoot};
}

function FlickerFrame({children, seed, startFrame = 0, durationFrames = 18, intensity = 0.82, progress, direction}: FlickerProps & {direction: 'in' | 'out'}) {
  const currentFrame = useCurrentFrame();
  const duration = Math.max(1, durationFrames);
  const elapsed = progress === undefined ? currentFrame - startFrame : Math.round(progress * duration);
  const sample = direction === 'in'
    ? powerOnSample(seed, elapsed, duration, clamp01(intensity))
    : powerOnSample(seed, duration - elapsed, duration, clamp01(intensity));
  return <div data-bakeoff={direction === 'in' ? 'Flicker' : 'PowerDown'} style={{width: '100%', height: '100%', opacity: sample.opacity, filter: `brightness(${sample.brightness.toFixed(3)})`}}>{children}</div>;
}

export function Flicker(props: FlickerProps) {
  return <FlickerFrame {...props} direction="in"/>;
}

export function PowerDown(props: FlickerProps) {
  return <FlickerFrame {...props} direction="out"/>;
}
