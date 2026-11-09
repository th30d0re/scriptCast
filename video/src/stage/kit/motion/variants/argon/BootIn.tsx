import {useKitFrame} from '../../core';
import React from 'react';
import {color} from '../../../theme';
import {ScanLines} from '../../../primitives/Icons';
import {clamp01, type Box} from '../../../components/audit';
import {cubicInOut, cubicOut, settle} from '../../../../motion';
import {DrawRect, type DrawRectCorner, type DrawRectNotch} from './DrawRect';
import {Flicker} from './Flicker';
import {hashFloat} from './hash';

export type BootInVariant = 'snap' | 'scan' | 'glitch';

export type BootInSchedule = {
  variant: BootInVariant;
  progress: number;
  rectProgress: number;
  flickerProgress: number;
  revealProgress: number;
  scanBeamY: number;
  glitchOffsetPx: number;
  snapScale: number;
};

/**
 * Pure preset function returning the staged progress values for
 * DrawRect -> Flicker -> content reveal across the three named variations.
 */
export function bootInPreset(
  variant: BootInVariant,
  rawProgress: number,
  seed: number | string = 303,
): BootInSchedule {
  const p = clamp01(rawProgress);

  if (variant === 'snap') {
    const rectProgress = clamp01(p / 0.48);
    const flickerProgress = clamp01((p - 0.30) / 0.50);
    const revealProgress = clamp01((p - 0.40) / 0.60);
    // Deterministic unit-mass spring settle from stage/motion.ts
    const springVal = revealProgress <= 0
      ? 0
      : revealProgress >= 1
        ? 1
        : settle(revealProgress * 550, {stiffness: 185, damping: 19});
    const snapScale = 0.94 + 0.06 * springVal;
    return {
      variant,
      progress: p,
      rectProgress,
      flickerProgress,
      revealProgress,
      scanBeamY: 1,
      glitchOffsetPx: 0,
      snapScale,
    };
  }

  if (variant === 'scan') {
    const rectProgress = clamp01(p / 0.54);
    const flickerProgress = clamp01((p - 0.28) / 0.54);
    const revealProgress = cubicInOut(clamp01((p - 0.34) / 0.58));
    return {
      variant,
      progress: p,
      rectProgress,
      flickerProgress,
      revealProgress,
      scanBeamY: revealProgress,
      glitchOffsetPx: 0,
      snapScale: 1,
    };
  }

  // 'glitch' variant
  const rectProgress = clamp01(p / 0.50);
  const flickerProgress = clamp01((p - 0.26) / 0.60);
  const revealProgress = cubicOut(clamp01((p - 0.32) / 0.68));
  const step = Math.floor(p * 30);
  const isGlitching = p > 0.32 && p < 0.85;
  const decay = isGlitching ? 1 - (p - 0.32) / 0.53 : 0;
  const signedRand = (hashFloat(seed, 400 + step) - 0.5) * 2;
  const glitchOffsetPx = isGlitching ? signedRand * 10 * decay : 0;

  return {
    variant,
    progress: p,
    rectProgress,
    flickerProgress,
    revealProgress,
    scanBeamY: 1,
    glitchOffsetPx,
    snapScale: 1,
  };
}

export type BootInProps = React.PropsWithChildren<{
  /** Target bounding box in scene/parent coordinates. */
  box: Box;
  /** Named boot preset variation ('snap' | 'scan' | 'glitch'). Default 'snap'. */
  variant?: BootInVariant;
  /** Direct progress 0..1 (overrides startFrame/durationFrames). */
  progress?: number;
  /** Start frame of the boot sequence (default 0). */
  startFrame?: number;
  /** Total duration in frames of the boot sequence (default 36). */
  durationFrames?: number;
  /** Deterministic seed for flicker & glitch slices. */
  seed?: number | string;
  /** Kit colour token for border stroke. */
  stroke?: string;
  /** Kit colour token for panel fill. */
  fill?: string;
  /** Kit colour token for accent highlights. */
  accent?: string;
  /** Optional override for DrawRect corner choice. */
  corner?: DrawRectCorner;
  /** Optional override for DrawRect notch. */
  notch?: DrawRectNotch;
  /** Optional small preset badge in top-right corner of the box (default true). */
  showBadge?: boolean;
  /** Audit / debug identifier. */
  id?: string;
}>;

export function BootIn({
  box,
  variant = 'snap',
  progress: rawProgress,
  startFrame = 0,
  durationFrames = 36,
  seed = 303,
  stroke = 'color/line/wire',
  fill = 'color/fill/panel-render',
  accent,
  corner,
  notch,
  showBadge = true,
  id = 'boot-in',
  children,
}: BootInProps) {
  const frame = useKitFrame();
  const p = typeof rawProgress === 'number'
    ? clamp01(rawProgress)
    : clamp01((frame - startFrame) / Math.max(1, durationFrames));

  if (p <= 0) return null;

  const resolvedAccent = accent ?? (
    variant === 'glitch'
      ? 'color/line/accent'
      : variant === 'scan'
        ? 'color/fill/gold'
        : 'color/fill/gold'
  );
  const resolvedCorner: DrawRectCorner = corner ?? (variant === 'glitch' ? 'tr-bl' : 'tl-br');
  const resolvedNotch: DrawRectNotch = notch ?? (variant === 'scan' ? 'top' : false);

  const sched = bootInPreset(variant, p, seed);
  const {w, h} = box;
  // Scan lines are a boot treatment, not a persistent panel texture. Keep them
  // through the reveal, then clear them as the boot settles.
  const scanLinesOpacity = 0.32 * sched.revealProgress * (1 - cubicInOut(clamp01((p - 0.82) / 0.18)));

  return (
    <div
      data-bakeoff="BootIn"
      data-bootin-variant={variant}
      data-bootin-id={id}
      data-progress={p.toFixed(3)}
      style={{
        position: 'absolute',
        left: 0,
        top: 0,
        width: 0,
        height: 0,
      }}
    >
      {/* Stage 1: DrawRect perimeter + fill power-on */}
      <DrawRect
        id={`${id}-rect`}
        box={box}
        progress={sched.rectProgress}
        stroke={stroke}
        fill={fill}
        accent={resolvedAccent}
        corner={resolvedCorner}
        notch={resolvedNotch}
        seed={seed}
      />

      {/* Stage 2 & 3: Flicker + Variant-specific Content Reveal */}
      {sched.flickerProgress > 0 && (
        <Flicker
          id={`${id}-flicker`}
          seed={seed}
          progress={sched.flickerProgress}
          intensity={variant === 'glitch' ? 1.15 : variant === 'scan' ? 0.8 : 0.9}
          style={{
            position: 'absolute',
            left: box.x,
            top: box.y,
            width: w,
            height: h,
            overflow: 'hidden',
            boxSizing: 'border-box',
          }}
        >
          {/* Variant background details */}
          {variant === 'scan' && (
            <svg
              width={w}
              height={h}
              viewBox={`0 0 ${w} ${h}`}
              style={{position: 'absolute', inset: 0, pointerEvents: 'none'}}
              aria-hidden="true"
            >
              {/* Reuse kit ScanLines primitive */}
              <g opacity={scanLinesOpacity}>
                <ScanLines
                  x={12}
                  y={14}
                  w={w - 24}
                  h={h - 28}
                  progress={sched.revealProgress}
                />
              </g>
              {/* Active raster scan bar sweeping down during reveal */}
              {sched.revealProgress > 0.01 && sched.revealProgress < 0.99 && (
                <g>
                  <rect
                    x={2}
                    y={Math.max(0, sched.scanBeamY * h - 14)}
                    width={w - 4}
                    height={14}
                    fill={color('color/fill/gold-wash')}
                  />
                  <line
                    x1={0}
                    y1={sched.scanBeamY * h}
                    x2={w}
                    y2={sched.scanBeamY * h}
                    stroke={color('color/fill/gold')}
                    strokeWidth={2.2}
                  />
                </g>
              )}
            </svg>
          )}

          {variant === 'snap' && (
            <svg
              width={w}
              height={h}
              viewBox={`0 0 ${w} ${h}`}
              style={{position: 'absolute', inset: 0, pointerEvents: 'none'}}
              aria-hidden="true"
            >
              {/* Inner precision reticle ticks that snap inward */}
              <g
                stroke={color(resolvedAccent)}
                strokeWidth={1.4}
                opacity={0.55 * sched.revealProgress}
              >
                <path d="M 10 18 V 10 H 18" fill="none" />
                <path d={`M ${w - 18} 10 H ${w - 10} V 18`} fill="none" />
                <path d={`M 10 ${h - 18} V ${h - 10} H 18`} fill="none" />
                <path d={`M ${w - 18} ${h - 10} H ${w - 10} V ${h - 18}`} fill="none" />
              </g>
            </svg>
          )}

          {variant === 'glitch' && Math.abs(sched.glitchOffsetPx) > 0.2 && (
            <div
              aria-hidden="true"
              style={{
                position: 'absolute',
                left: -sched.glitchOffsetPx * 1.1,
                top: 0,
                width: w,
                height: h,
                borderTop: `2px solid ${color('color/line/accent-soft')}`,
                borderBottom: `2px solid ${color('color/line/accent-soft')}`,
                background: color('color/fill/accent-wash'),
                opacity: 0.45,
                pointerEvents: 'none',
              }}
            />
          )}

          {/* Child content wrapper with variant reveal transform/clip */}
          <div
            style={{
              position: 'absolute',
              inset: 0,
              opacity: clamp01(sched.revealProgress * 1.15),
              transform:
                variant === 'snap'
                  ? `scale(${sched.snapScale.toFixed(4)})`
                  : variant === 'glitch' && Math.abs(sched.glitchOffsetPx) > 0.1
                    ? `translateX(${sched.glitchOffsetPx.toFixed(2)}px)`
                    : undefined,
              transformOrigin: 'center center',
              clipPath:
                variant === 'scan' && sched.revealProgress < 0.995
                  ? `inset(0px 0px ${((1 - sched.scanBeamY) * 100).toFixed(2)}% 0px)`
                  : undefined,
            }}
          >
            {children}
          </div>

          {/* Preset telemetry mode badge */}
          {showBadge && (
            <div
              style={{
                position: 'absolute',
                right: 12,
                bottom: 8,
                padding: '2px 7px',
                font: '500 11px "IBM Plex Mono"',
                letterSpacing: 1.1,
                color: color(resolvedAccent),
                background: color('color/surface/well'),
                border: `1px solid ${color('color/line/graticule')}`,
                opacity: clamp01((sched.revealProgress - 0.2) / 0.5),
              }}
            >
              BOOT::{variant.toUpperCase()}
            </div>
          )}
        </Flicker>
      )}
    </div>
  );
}
