import {clamp, cubicOut} from '../../../../motion';

/**
 * Pure deterministic 32-bit hash of (seed, step).
 * Uses SplitMix32 / Murmur3-style avalanche mixing — zero state, no Math.random, no Date.
 */
export function seedToInt(seed: number | string): number {
  if (typeof seed === 'number') {
    return (Math.imul(Math.floor(seed * 2654435761), 0x9e3779b1) ^ 0x85ebca6b) >>> 0;
  }
  let h = 0x811c9dc5;
  for (let i = 0; i < seed.length; i++) {
    h ^= seed.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

export function hash32(seed: number | string, step: number): number {
  let h = seedToInt(seed) ^ Math.imul(step | 0, 0x9e3779b9);
  h ^= h >>> 16;
  h = Math.imul(h, 0x85ebca6b);
  h ^= h >>> 13;
  h = Math.imul(h, 0xc2b2ae35);
  h ^= h >>> 16;
  return h >>> 0;
}

/** Deterministic float in [0, 1) from (seed, step). */
export function hashFloat(seed: number | string, step: number): number {
  return hash32(seed, step) / 4294967296;
}

/** Deterministic float in [min, max) from (seed, step). */
export function hashRange(seed: number | string, step: number, min: number, max: number): number {
  return min + (max - min) * hashFloat(seed, step);
}

export type FlickerEnvelope = {
  /** Normalized progress 0..1 within the transition window. */
  progress: number;
  /** Resulting element opacity 0..1. */
  opacity: number;
  /** Brightness multiplier (1 = normal, >1 = phosphor overshoot, <1 = dim). */
  brightness: number;
  /** Glow bloom factor 0..1 during overshoot peaks. */
  overshoot: number;
  /** Kept at 0 so text is never displaced or duplicated during pulses. */
  scanOffsetPx: number;
  /** Kept at 1 so text is never vertically squashed. */
  scaleY: number;
  /** Which pulse phase is active ('off' | 'pulse-1' | 'dip-1' | 'pulse-2' | 'dip-2' | 'overshoot' | 'steady'). */
  phase: 'off' | 'pulse-1' | 'dip-1' | 'pulse-2' | 'dip-2' | 'overshoot' | 'steady';
};

/**
 * Intentional, seeded CRT/HUD power-up envelope.
 * Produces 2-3 uneven on/off pulses, a brightness overshoot peak, then rock-steady settle.
 */
export function computePowerOnEnvelope(
  rawProgress: number,
  seed: number | string = 1,
  rawIntensity = 0.85,
): FlickerEnvelope {
  const p = clamp(rawProgress, 0, 1);
  const intensity = clamp(rawIntensity, 0, 1.5);

  if (p <= 0) {
    return {
      progress: 0,
      opacity: 0,
      brightness: 0,
      overshoot: 0,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'off',
    };
  }
  if (p >= 1) {
    return {
      progress: 1,
      opacity: 1,
      brightness: 1,
      overshoot: 0,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'steady',
    };
  }

  // Seed-derived phase boundaries so adjacent components feel organically staggered
  // while preserving a deliberate 3-stage power-up rhythm.
  const t1Start = hashRange(seed, 1, 0.02, 0.06);
  const t1End = t1Start + hashRange(seed, 2, 0.09, 0.13);
  const d1End = t1End + hashRange(seed, 3, 0.09, 0.13);
  const t2End = d1End + hashRange(seed, 4, 0.15, 0.21);
  const d2End = t2End + hashRange(seed, 5, 0.07, 0.11);
  const overshootEnd = Math.min(0.88, d2End + hashRange(seed, 6, 0.20, 0.26));

  // Deterministic frame-quantized micro-ripple for texture during active pulses
  const quantStep = Math.floor(p * 24);
  const ripple = (hashFloat(seed, 100 + quantStep) - 0.5) * 0.12 * intensity;

  if (p < t1Start) {
    return {
      progress: p,
      opacity: 0,
      brightness: 0,
      overshoot: 0,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'off',
    };
  }

  // Pulse 1: Initial relay strike
  if (p < t1End) {
    const local = (p - t1Start) / (t1End - t1Start);
    const arch = Math.sin(local * Math.PI);
    const amp = hashRange(seed, 7, 0.58, 0.78);
    const opacity = clamp(amp * (0.65 + 0.35 * arch) + ripple, 0.15, 0.9);
    const brightness = 1 + 0.38 * intensity * arch;
    return {
      progress: p,
      opacity,
      brightness,
      overshoot: 0.35 * arch * intensity,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'pulse-1',
    };
  }

  // Dip 1: Primary capacitor recharge dropout
  if (p < d1End) {
    const floor = clamp(0.14 * (1 - 0.85 * Math.min(1, intensity)), 0.02, 0.25);
    return {
      progress: p,
      opacity: floor,
      brightness: 0.7,
      overshoot: 0,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'dip-1',
    };
  }

  // Pulse 2: Secondary catch pulse
  if (p < t2End) {
    const local = (p - d1End) / (t2End - d1End);
    const arch = Math.sin(local * Math.PI * 0.85 + 0.15);
    const amp = hashRange(seed, 8, 0.82, 0.95);
    const opacity = clamp(amp * (0.75 + 0.25 * arch) + ripple * 0.7, 0.4, 1);
    const brightness = 1 + 0.28 * intensity * arch;
    return {
      progress: p,
      opacity,
      brightness,
      overshoot: 0.25 * arch * intensity,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'pulse-2',
    };
  }

  // Dip 2: Brief sync notch before rail lock
  if (p < d2End) {
    const dipDepth = clamp(0.28 * (1 - 0.65 * Math.min(1, intensity)), 0.08, 0.45);
    return {
      progress: p,
      opacity: dipDepth,
      brightness: 0.88,
      overshoot: 0,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'dip-2',
    };
  }

  // Pulse 3: Rail lock with brightness overshoot
  if (p < overshootEnd) {
    const local = (p - d2End) / (overshootEnd - d2End);
    const env = local < 0.28
      ? cubicOut(local / 0.28)
      : 1 - cubicOut((local - 0.28) / 0.72) * 0.82;
    const microNotch = local > 0.48 && local < 0.58 ? -0.08 * intensity : 0;
    const overshoot = clamp(env * intensity, 0, 1.2);
    return {
      progress: p,
      opacity: clamp(1 + microNotch, 0.82, 1),
      brightness: 1 + 0.52 * overshoot,
      overshoot,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'overshoot',
    };
  }

  // Final settle tail
  const tail = (p - overshootEnd) / Math.max(0.001, 1 - overshootEnd);
  const residual = (1 - cubicOut(tail)) * 0.1 * intensity;
  return {
    progress: p,
    opacity: 1,
    brightness: 1 + residual,
    overshoot: residual,
    scanOffsetPx: 0,
    scaleY: 1,
    phase: 'steady',
  };
}

/**
 * Matching CRT/HUD power-down envelope:
 * Rail unlatch -> inductive flyback flash -> dropout -> last ember pulse -> dark.
 */
export function computePowerDownEnvelope(
  rawProgress: number,
  seed: number | string = 1,
  rawIntensity = 0.85,
): FlickerEnvelope {
  const p = clamp(rawProgress, 0, 1);
  const intensity = clamp(rawIntensity, 0, 1.5);

  if (p <= 0) {
    return {
      progress: 0,
      opacity: 1,
      brightness: 1,
      overshoot: 0,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'steady',
    };
  }
  if (p >= 1) {
    return {
      progress: 1,
      opacity: 0,
      brightness: 0,
      overshoot: 0,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'off',
    };
  }

  const sagEnd = hashRange(seed, 21, 0.18, 0.24);
  const flashEnd = sagEnd + hashRange(seed, 22, 0.14, 0.18);
  const dropEnd = flashEnd + hashRange(seed, 23, 0.14, 0.18);
  const emberEnd = dropEnd + hashRange(seed, 24, 0.16, 0.22);

  // Stage 1: Initial voltage sag
  if (p < sagEnd) {
    const local = p / sagEnd;
    return {
      progress: p,
      opacity: 1 - 0.68 * local,
      brightness: 1 - 0.22 * local,
      overshoot: 0,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'dip-1',
    };
  }

  // Stage 2: Inductive flyback flash (overshoot before collapse)
  if (p < flashEnd) {
    const local = (p - sagEnd) / (flashEnd - sagEnd);
    const arch = Math.sin(local * Math.PI);
    const overshoot = arch * 0.85 * intensity;
    return {
      progress: p,
      opacity: clamp(0.35 + 0.62 * arch, 0.2, 0.96),
      brightness: 1 + 0.48 * overshoot,
      overshoot,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'overshoot',
    };
  }

  // Stage 3: Deep dropout
  if (p < dropEnd) {
    return {
      progress: p,
      opacity: 0.06,
      brightness: 0.65,
      overshoot: 0,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'dip-2',
    };
  }

  // Stage 4: Final weak ember pulse
  if (p < emberEnd) {
    const local = (p - dropEnd) / (emberEnd - dropEnd);
    const arch = Math.sin(local * Math.PI);
    return {
      progress: p,
      opacity: clamp(0.06 + 0.34 * arch * Math.min(1, intensity + 0.2), 0.04, 0.45),
      brightness: 0.85 + 0.2 * arch,
      overshoot: 0.12 * arch,
      scanOffsetPx: 0,
      scaleY: 1,
      phase: 'pulse-1',
    };
  }

  // Stage 5: Final collapse to zero
  const tail = (p - emberEnd) / Math.max(0.001, 1 - emberEnd);
  const fade = 1 - cubicOut(tail);
  return {
    progress: p,
    opacity: 0.08 * fade,
    brightness: 0.6 * fade,
    overshoot: 0,
    scanOffsetPx: 0,
    scaleY: 1,
    phase: 'off',
  };
}
