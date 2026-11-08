import {useKitFrame} from '../../core';
import React from 'react';
import {color} from '../../../theme';
import {clamp} from '../../../../motion';
import {
  computePowerDownEnvelope,
  computePowerOnEnvelope,
  type FlickerEnvelope,
} from './hash';

export type FlickerMode = 'on' | 'off' | 'both';

export type FlickerProps = React.PropsWithChildren<{
  /** Deterministic seed controlling pulse spacing and micro-ripple. */
  seed?: number | string;
  /** Frame when power-on (or power-down if mode='off') begins. */
  startFrame?: number;
  /** Duration in frames of the flicker transition. */
  durationFrames?: number;
  /** Intensity of dropout depth and brightness overshoot (0..1.5, default 0.85). */
  intensity?: number;
  /** 'on' powers up to steady; 'off' powers down from steady to dark; 'both' runs on then off. */
  mode?: FlickerMode;
  /** Optional frame at which to begin power-down after powering on. */
  powerDownStartFrame?: number;
  /** Optional duration in frames for the power-down phase (defaults to durationFrames). */
  powerDownDurationFrames?: number;
  /** Optional normalized progress override (0..1). When supplied, overrides frame timing. */
  progress?: number;
  /** Optional inline container style. */
  style?: React.CSSProperties;
  /** Optional audit / debug identifier. */
  id?: string;
}>;

export function resolveFlickerEnvelope({
  frame,
  seed = 1,
  startFrame = 0,
  durationFrames = 18,
  intensity = 0.85,
  mode = 'on',
  powerDownStartFrame,
  powerDownDurationFrames,
  progress,
}: Omit<FlickerProps, 'children' | 'style' | 'id'> & {frame: number}): FlickerEnvelope {
  const dur = Math.max(1, durationFrames);

  if (typeof progress === 'number') {
    const p = clamp(progress, 0, 1);
    return mode === 'off'
      ? computePowerDownEnvelope(p, seed, intensity)
      : computePowerOnEnvelope(p, seed, intensity);
  }

  if (mode === 'off') {
    const p = clamp((frame - startFrame) / dur, 0, 1);
    return computePowerDownEnvelope(p, seed, intensity);
  }

  const hasPowerDown = mode === 'both' || typeof powerDownStartFrame === 'number';
  const downStart = powerDownStartFrame ?? startFrame + dur + 24;
  const downDur = Math.max(1, powerDownDurationFrames ?? dur);

  if (hasPowerDown && frame >= downStart) {
    const pDown = clamp((frame - downStart) / downDur, 0, 1);
    return computePowerDownEnvelope(pDown, seed, intensity);
  }

  const pOn = clamp((frame - startFrame) / dur, 0, 1);
  return computePowerOnEnvelope(pOn, seed, intensity);
}

export function Flicker({
  seed = 1,
  startFrame = 0,
  durationFrames = 18,
  intensity = 0.85,
  mode = 'on',
  powerDownStartFrame,
  powerDownDurationFrames,
  progress,
  style,
  id = 'flicker',
  children,
}: FlickerProps) {
  const frame = useKitFrame();
  const env = resolveFlickerEnvelope({
    frame,
    seed,
    startFrame,
    durationFrames,
    intensity,
    mode,
    powerDownStartFrame,
    powerDownDurationFrames,
    progress,
  });

  if (env.opacity <= 0.001) {
    return null;
  }

  const filter = Math.abs(env.brightness - 1) > 0.01
    ? `brightness(${env.brightness.toFixed(3)})`
    : undefined;

  return (
    <div
      data-bakeoff="Flicker"
      data-flicker-id={id}
      data-flicker-mode={mode}
      data-flicker-phase={env.phase}
      data-flicker-opacity={env.opacity.toFixed(3)}
      style={{
        position: 'relative',
        opacity: env.opacity,
        filter,
        ...style,
      }}
    >
      {children}
      {env.overshoot > 0.04 && (
        <div
          aria-hidden="true"
          style={{
            position: 'absolute',
            inset: 0,
            pointerEvents: 'none',
            background: color('color/fill/gold-wash'),
            opacity: clamp(env.overshoot * 0.42, 0, 0.45),
          }}
        />
      )}
    </div>
  );
}

/** Convenience wrapper for power-down mode. */
export function FlickerPowerDown(props: Omit<FlickerProps, 'mode'>) {
  return <Flicker {...props} mode="off" />;
}
