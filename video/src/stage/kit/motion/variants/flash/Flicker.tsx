import {useKitFrame} from '../../core';
import React from 'react';
import {hashFloat} from './hash';

export interface FlickerProps {
  children: React.ReactNode;
  seed?: number;
  startFrame?: number;
  durationFrames?: number;
  intensity?: number; // 0..1, defaults to 0.8
  mode?: 'power-on' | 'power-off' | 'steady';
  style?: React.CSSProperties;
}

export const Flicker: React.FC<FlickerProps> = ({
  children,
  seed = 42,
  startFrame = 0,
  durationFrames = 18,
  intensity = 0.8,
  mode = 'power-on',
  style,
}) => {
  const currentFrame = useKitFrame();

  if (mode === 'steady') {
    return <div style={{width: '100%', height: '100%', ...style}}>{children}</div>;
  }

  const relFrame = currentFrame - startFrame;

  // Before sequence starts
  if (relFrame < 0) {
    if (mode === 'power-on') {
      return null;
    } else {
      return <div style={{width: '100%', height: '100%', ...style}}>{children}</div>;
    }
  }

  // After sequence settles
  if (relFrame >= durationFrames) {
    if (mode === 'power-on') {
      return <div style={{width: '100%', height: '100%', ...style}}>{children}</div>;
    } else {
      return null;
    }
  }

  // Active flicker phase: 0 <= relFrame < durationFrames
  // Normalized transition progress: 0 to 1
  const t = relFrame / durationFrames;

  // Pattern of pulses:
  // Phase 1 (0 -> 0.3): Staccato dark micro-flashes (CRT warmup)
  // Phase 2 (0.3 -> 0.65): Tube ignition pulse, high overshoot brightness
  // Phase 3 (0.65 -> 0.9): Minor hum jitter settling into 100%
  // Phase 4 (0.9 -> 1.0): Fully stable
  const frameNoise = hashFloat(seed, relFrame);
  const microJitter = (hashFloat(seed + 101, relFrame) - 0.5) * 0.15 * intensity;

  let baseOpacity = 1;
  let brightness = 1;

  if (mode === 'power-on') {
    if (t < 0.25) {
      // Uneven ignition flashes
      const flash = frameNoise > 0.45 ? 0.75 : 0.05;
      baseOpacity = flash;
      brightness = flash > 0.5 ? 1.6 : 0.8;
    } else if (t < 0.45) {
      // Brief blackout before full surge
      const flash = frameNoise > 0.65 ? 0.9 : 0.15;
      baseOpacity = flash;
      brightness = flash > 0.5 ? 1.4 : 0.9;
    } else if (t < 0.75) {
      // Brightness overshoot (CRT phosphor flare)
      const overshoot = 1 + (0.75 - t) * 1.5 * intensity;
      baseOpacity = 0.85 + microJitter;
      brightness = overshoot;
    } else {
      // Settling hum
      baseOpacity = 0.95 + microJitter * (1 - t);
      brightness = 1 + (1 - t) * 0.2;
    }
  } else {
    // power-off
    const invT = 1 - t;
    if (t < 0.2) {
      // Initial flicker stutter
      baseOpacity = frameNoise > 0.3 ? 0.8 : 0.2;
      brightness = 1.3;
    } else if (t < 0.6) {
      // Collapse into a narrow bright streak / rapid fade
      baseOpacity = (1 - t) * (frameNoise > 0.5 ? 0.9 : 0.2);
      brightness = 1.8;
    } else {
      // Dying glow
      baseOpacity = Math.max(0, (1 - t) * 0.3);
      brightness = 0.5;
    }
  }

  // Clamp values
  const finalOpacity = Math.max(0, Math.min(1, baseOpacity));
  const finalBrightness = Math.max(0.2, brightness);

  return (
    <div
      data-kit="Flicker"
      style={{
        width: '100%',
        height: '100%',
        opacity: finalOpacity,
        filter: finalBrightness !== 1 ? `brightness(${finalBrightness.toFixed(2)}) contrast(${Math.min(1.4, 0.9 + finalBrightness * 0.1).toFixed(2)})` : undefined,
        willChange: 'opacity, filter',
        ...style,
      }}
    >
      {children}
    </div>
  );
};
