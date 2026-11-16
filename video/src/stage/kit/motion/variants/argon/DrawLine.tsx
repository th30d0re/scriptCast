import {useKitFrame} from '../../core';
import React from 'react';
import {color, dimensions} from '../../../theme';
import {
  arrowBarbs,
  clamp01,
  lineAttrs,
  partialPolyline,
  pathFrom,
  type Point,
} from '../../../components/audit';
import {cubicInOut, cubicOut} from '../../../../motion';

export type DrawLineProps = {
  /** Polyline vertices [[x, y], ...]. Can be 2 points (straight line) or many (polyline). */
  points?: readonly Point[];
  /** Shorthand start X when drawing a 2-point straight line. */
  x1?: number;
  /** Shorthand start Y when drawing a 2-point straight line. */
  y1?: number;
  /** Shorthand end X when drawing a 2-point straight line. */
  x2?: number;
  /** Shorthand end Y when drawing a 2-point straight line. */
  y2?: number;
  /** Normalized progress 0..1. Overrides startFrame/durationFrames when provided. */
  progress?: number;
  /** Frame when the line begins drawing (used when progress is omitted). */
  startFrame?: number;
  /** Duration in frames (default 24). */
  durationFrames?: number;
  /** Kit token for the dimmed settled trace behind the wake (default 'color/line/chart'). */
  stroke?: string;
  /** Kit token for the bright leading head and hot wake (default 'color/fill/gold'). */
  headColor?: string;
  /** Base stroke width in px (default kit stroke/thin = 2.2). */
  strokeWidth?: number;
  /** Fraction of total path length occupied by the dimming wake behind the head (default 0.34). */
  tailLength?: number;
  /** Draw an arrowhead at the leading tip / terminal end. */
  arrow?: boolean;
  /** Render transient corner glints as the head passes intermediate bends while drawing (default false). */
  nodes?: boolean;
  /** Dashed settled trace. */
  dashed?: boolean;
  /** If true, renders only a <g> element for embedding inside an outer <svg>. */
  bareSvg?: boolean;
  /** Audit / debug identifier. */
  id?: string;
};

/**
 * Extracts the sub-polyline between normalized arc-length fractions [t0, t1]
 * by composing the kit's `partialPolyline` helper twice.
 */
export function slicePolyline(points: readonly Point[], t0: number, t1: number): Point[] {
  const start = clamp01(t0);
  const end = clamp01(t1);
  if (end <= start + 1e-5 || points.length < 2) return [];
  const prefix = partialPolyline(points, end);
  if (start <= 1e-5 || prefix.length < 2) return prefix;
  const keepRatio = (end - start) / end;
  const reversed = [...prefix].reverse();
  return partialPolyline(reversed, keepRatio).reverse();
}

export function DrawLine({
  points: rawPoints,
  x1 = 0,
  y1 = 0,
  x2 = 100,
  y2 = 0,
  progress: rawProgress,
  startFrame = 0,
  durationFrames = 24,
  stroke = 'color/line/chart',
  headColor = 'color/fill/gold',
  strokeWidth = dimensions['stroke/thin'].value,
  tailLength = 0.34,
  arrow = false,
  nodes = false,
  dashed = false,
  bareSvg = false,
  id = 'draw-line',
}: DrawLineProps) {
  const frame = useKitFrame();
  const points: readonly Point[] = rawPoints && rawPoints.length >= 2
    ? rawPoints
    : [[x1, y1], [x2, y2]];

  const p = typeof rawProgress === 'number'
    ? clamp01(rawProgress)
    : clamp01((frame - startFrame) / Math.max(1, durationFrames));

  if (p <= 0 || points.length < 2) return null;

  // The leading tip reaches the end of the path at p = 0.80; from 0.80 -> 1.00 the bright head
  // and wake fade out completely to 0 so the finished line at rest ends completely clean.
  const travelP = cubicInOut(clamp01(p / 0.80));
  const settleP = clamp01((p - 0.80) / 0.20);
  const headFade = p >= 1 ? 0 : clamp01(p / 0.06) * (1 - cubicOut(settleP));
  const effectiveTail = tailLength * headFade;

  const drawn = partialPolyline(points, travelP);
  if (drawn.length < 2) return null;

  const tip = drawn[drawn.length - 1];
  const prev = drawn[drawn.length - 2];
  const segDx = tip[0] - prev[0];
  const segDy = tip[1] - prev[1];
  const segLen = Math.hypot(segDx, segDy) || 1;
  const ux = segDx / segLen;
  const uy = segDy / segLen;

  // Build 6 graduated wake segments from [travelP - effectiveTail .. travelP] while drawing
  const WAKE_STEPS = 6;
  const wakeStart = Math.max(0, travelP - effectiveTail);
  const wakeSpan = travelP - wakeStart;

  const wakeSlices = headFade > 0.005 && wakeSpan > 0.005
    ? Array.from({length: WAKE_STEPS}, (_, i) => {
        const f0 = wakeStart + (wakeSpan * i) / WAKE_STEPS;
        const f1 = wakeStart + (wakeSpan * (i + 1.15)) / WAKE_STEPS;
        const heat = (i + 1) / WAKE_STEPS;
        return {
          pts: slicePolyline(points, f0, Math.min(travelP, f1)),
          heat,
        };
      }).filter(s => s.pts.length >= 2)
    : [];

  const settledInk = color(stroke);
  const wireInk = color('color/line/wire');
  const headInk = color(headColor);
  const washInk = color('color/fill/gold-wash');

  const content = (
    <g
      data-bakeoff="DrawLine"
      data-line-id={id}
      data-progress={p.toFixed(3)}
    >
      {/* 1. Clean settled trace */}
      <path
        {...lineAttrs(id, drawn)}
        d={pathFrom(drawn)}
        fill="none"
        stroke={settledInk}
        strokeWidth={strokeWidth}
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeDasharray={dashed ? '10 7' : undefined}
        opacity={0.88}
      />

      {/* 2. Graduated dimming wake behind the leading head (fades out completely by p=1) */}
      {wakeSlices.map((slice, idx) => {
        const {pts, heat} = slice;
        const isHot = heat > 0.65;
        return (
          <React.Fragment key={idx}>
            {isHot && (
              <path
                d={pathFrom(pts)}
                fill="none"
                stroke={washInk}
                strokeWidth={strokeWidth + 6 * heat}
                strokeLinecap="round"
                strokeLinejoin="round"
                opacity={0.55 * heat * headFade}
              />
            )}
            <path
              d={pathFrom(pts)}
              fill="none"
              stroke={isHot ? headInk : wireInk}
              strokeWidth={strokeWidth * (0.95 + 0.55 * heat)}
              strokeLinecap="round"
              strokeLinejoin="round"
              opacity={(0.22 + 0.78 * heat) * headFade}
            />
          </React.Fragment>
        );
      })}

      {/* 3. Optional arrowhead at the terminal end */}
      {arrow && travelP > 0.15 && (
        <path
          d={pathFrom(arrowBarbs(tip, ux, uy, 13, 7.5))}
          fill="none"
          stroke={headFade > 0.35 ? headInk : settledInk}
          strokeWidth={strokeWidth}
          strokeLinecap="round"
          strokeLinejoin="round"
          opacity={0.88}
        />
      )}

      {/* 4. Bright leading head (only visible while drawing; fades out completely to 0 by p=1) */}
      {headFade > 0.005 && (
        <g transform={`translate(${tip[0]}, ${tip[1]})`} opacity={headFade}>
          <circle r={10 * (0.5 + 0.5 * headFade)} fill={washInk} opacity={0.85} />
          <circle
            r={5.8 * (0.6 + 0.4 * headFade)}
            fill="none"
            stroke={headInk}
            strokeWidth={1.4}
            opacity={0.9}
          />
          <circle r={3.0} fill={headInk} />
          <circle r={1.4} fill={wireInk} />
        </g>
      )}
    </g>
  );

  if (bareSvg) return content;

  return (
    <svg
      width="1920"
      height="1080"
      viewBox="0 0 1920 1080"
      style={{position: 'absolute', inset: 0, pointerEvents: 'none', overflow: 'visible'}}
      aria-hidden="true"
    >
      {content}
    </svg>
  );
}
