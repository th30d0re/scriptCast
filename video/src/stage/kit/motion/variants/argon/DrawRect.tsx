import {useKitFrame} from '../../core';
import React from 'react';
import {color, dimensions} from '../../../theme';
import {notchPath} from '../../../primitives/notch';
import {clamp01, shapeAttrs, type Box} from '../../../components/audit';
import {cubicInOut, cubicOut} from '../../../../motion';
import {computePowerOnEnvelope} from './hash';

export type DrawRectCorner = 'tl-br' | 'tr-bl' | 'top-left' | 'top-right';
export type DrawRectNotch = boolean | number | 'top' | 'bottom' | 'both';

export type DrawRectProps = React.PropsWithChildren<{
  /** Absolute 1920x1080 bounding box (or local box within a positioned parent). */
  box: Box;
  /** Direct normalized progress 0..1. Overrides startFrame/durationFrames when provided. */
  progress?: number;
  /** Frame when the rectangle starts drawing (used when progress is omitted). */
  startFrame?: number;
  /** Total duration in frames for edge draw + fill flicker (default 28). */
  durationFrames?: number;
  /** Kit colour token for the border stroke (default 'color/line/wire'). */
  stroke?: string;
  /** Kit colour token for the interior fill (default 'color/fill/panel-render'). */
  fill?: string;
  /** Kit colour token for active leading heads and corner ticks (default 'color/fill/gold'). */
  accent?: string;
  /** Stroke width in px (defaults to kit stroke/thin = 2.2 or 1.6). */
  strokeWidth?: number;
  /** Which pair of opposite corners initiate the draw ('tl-br' or 'tr-bl'). */
  corner?: DrawRectCorner;
  /** Alias for corner. */
  corners?: DrawRectCorner;
  /** Optional kit 5-tier notch on top/bottom edges (boolean, 0..1 progress, or 'top'|'bottom'|'both'). */
  notch?: DrawRectNotch;
  /** Deterministic seed for the fill power-on flicker. */
  seed?: number | string;
  /** Optional audit/DOM identifier. */
  id?: string;
  /** Optional extra inline styles on the outer container. */
  style?: React.CSSProperties;
}>;

const NOTCH_CX = 960;
const NOTCH_KINK = 62.4;
const NOTCH_DEPTH = 48;

export function resolveDrawRectProgress(
  frame: number,
  progress?: number,
  startFrame = 0,
  durationFrames = 28,
): number {
  if (typeof progress === 'number') return clamp01(progress);
  return clamp01((frame - startFrame) / Math.max(1, durationFrames));
}

export function DrawRect({
  box,
  progress: rawProgress,
  startFrame = 0,
  durationFrames = 28,
  stroke = 'color/line/wire',
  fill = 'color/fill/panel-render',
  accent = 'color/fill/gold',
  strokeWidth = dimensions['stroke/chart'].value,
  corner,
  corners,
  notch = false,
  seed = 101,
  id = 'draw-rect',
  style,
  children,
}: DrawRectProps) {
  const frame = useKitFrame();
  const p = resolveDrawRectProgress(frame, rawProgress, startFrame, durationFrames);
  if (p <= 0) return null;

  const chosenCorner = corner ?? corners ?? 'tl-br';
  const isTlBr = chosenCorner === 'tl-br' || chosenCorner === 'top-left';

  // Phase 1 (0.00 -> 0.64): Opposite corners draw along their two orthogonal edges until meeting.
  const rawEdgeP = clamp01(p / 0.64);
  const edgeP = cubicInOut(rawEdgeP);

  // Phase 2 (0.50 -> 1.00): Interior fill flickers in with seeded CRT pulses + overshoot.
  const rawFillP = clamp01((p - 0.50) / 0.50);
  const fillEnv = computePowerOnEnvelope(rawFillP, seed, 0.85);

  // Brief corner closure flash right as edgeP reaches 1 (around p = 0.64..0.82)
  const closureFlash = p >= 0.60 && p < 0.86
    ? Math.sin(((p - 0.60) / 0.26) * Math.PI)
    : 0;

  // Notch configuration reusing kit's notchPath(y0, dir, p)
  const hasTopNotch = notch === true || notch === 'top' || notch === 'both' || (typeof notch === 'number' && notch > 0);
  const hasBottomNotch = notch === 'bottom' || notch === 'both';
  const targetNotchAmount = typeof notch === 'number' ? clamp01(notch) : notch ? 1 : 0;
  // Notch deploys once the horizontal lines cross the midpoint (p > 0.32)
  const notchAnimP = cubicOut(clamp01((p - 0.32) / 0.52)) * targetNotchAmount;

  const {w, h} = box;
  const notchScale = Math.min(0.62, (w * 0.36) / (NOTCH_KINK * 2), (h * 0.26) / NOTCH_DEPTH);
  const notchHalfSpan = NOTCH_KINK * notchScale;
  const midX = w / 2;
  const notchLeftX = midX - notchHalfSpan;
  const notchRightX = midX + notchHalfSpan;

  const strokeInk = color(stroke);
  const accentInk = color(accent);
  const fillInk = color(fill);

  // Compute the 4 growing edge segments in local [0..w, 0..h] coordinates.
  // Top edge (y = 0), Bottom edge (y = h), Left edge (x = 0), Right edge (x = w)
  const topStartX = isTlBr ? 0 : w;
  const topEndX = isTlBr ? w * edgeP : w * (1 - edgeP);

  const bottomStartX = isTlBr ? w : 0;
  const bottomEndX = isTlBr ? w * (1 - edgeP) : w * edgeP;

  const leftStartY = isTlBr ? 0 : h;
  const leftEndY = isTlBr ? h * edgeP : h * (1 - edgeP);

  const rightStartY = isTlBr ? h : 0;
  const rightEndY = isTlBr ? h * (1 - edgeP) : h * edgeP;

  // Helper to render a horizontal edge that leaves room for the kit notch when active
  const renderHorizontalEdge = (
    y: number,
    xStart: number,
    xEnd: number,
    withNotch: boolean,
    dir: 1 | -1,
    edgeKey: string,
  ) => {
    const xMin = Math.min(xStart, xEnd);
    const xMax = Math.max(xStart, xEnd);
    if (xMax - xMin <= 0.1) return null;

    if (!withNotch || notchAnimP <= 0.01) {
      return (
        <line
          key={edgeKey}
          x1={xStart}
          y1={y}
          x2={xEnd}
          y2={y}
          stroke={strokeInk}
          strokeWidth={strokeWidth}
          strokeLinecap="square"
        />
      );
    }

    // Split segment around [notchLeftX, notchRightX] so notchPath renders the center V + tiered bars
    const segs: [number, number][] = [];
    if (xMin < notchLeftX) {
      segs.push([xMin, Math.min(xMax, notchLeftX)]);
    }
    if (xMax > notchRightX) {
      segs.push([Math.max(xMin, notchRightX), xMax]);
    }

    const clipId = `${id}-notch-clip-${edgeKey}`;
    // Clip the kit notchPath to the exact kink interval [960 - 62.4, 960 + 62.4] and current drawn x range
    const localClipMin = NOTCH_CX + (Math.max(xMin, notchLeftX) - midX) / notchScale;
    const localClipMax = NOTCH_CX + (Math.min(xMax, notchRightX) - midX) / notchScale;
    const showNotchBody = localClipMax > localClipMin;

    return (
      <g key={edgeKey}>
        {segs.map(([s0, s1], idx) => (
          <line
            key={idx}
            x1={s0}
            y1={y}
            x2={s1}
            y2={y}
            stroke={strokeInk}
            strokeWidth={strokeWidth}
            strokeLinecap="square"
          />
        ))}
        {showNotchBody && (
          <>
            <defs>
              <clipPath id={clipId}>
                <rect
                  x={localClipMin - 1}
                  y={dir > 0 ? -4 : -NOTCH_DEPTH - 8}
                  width={Math.max(0, localClipMax - localClipMin + 2)}
                  height={NOTCH_DEPTH + 12}
                />
              </clipPath>
            </defs>
            <g
              clipPath={`url(#${clipId})`}
              transform={`translate(${midX - NOTCH_CX * notchScale}, ${y}) scale(${notchScale})`}
            >
              <path
                d={notchPath(0, dir, notchAnimP)}
                stroke={strokeInk}
                strokeWidth={strokeWidth / notchScale}
                strokeLinecap="round"
                strokeLinejoin="round"
                fill="none"
              />
            </g>
          </>
        )}
      </g>
    );
  };

  const tickLen = Math.min(12, Math.min(w, h) * 0.12);
  const showHeads = rawEdgeP > 0 && rawEdgeP < 0.995;

  return (
    <div
      {...shapeAttrs(id, box, 'frame')}
      data-bakeoff="DrawRect"
      data-corner={isTlBr ? 'tl-br' : 'tr-bl'}
      data-progress={p.toFixed(3)}
      style={{
        position: 'absolute',
        left: box.x,
        top: box.y,
        width: w,
        height: h,
        boxSizing: 'border-box',
        pointerEvents: 'none',
        ...style,
      }}
    >
      {/* Flickering interior fill */}
      {fill !== 'none' && fillEnv.opacity > 0.001 && (
        <div
          aria-hidden="true"
          style={{
            position: 'absolute',
            inset: 0,
            background: fillInk,
            opacity: fillEnv.opacity,
            filter: Math.abs(fillEnv.brightness - 1) > 0.01
              ? `brightness(${fillEnv.brightness.toFixed(3)})`
              : undefined,
          }}
        />
      )}

      {/* Overshoot phosphor wash during fill power-on */}
      {fillEnv.overshoot > 0.04 && (
        <div
          aria-hidden="true"
          style={{
            position: 'absolute',
            inset: 0,
            background: color('color/fill/gold-wash'),
            opacity: clamp01(fillEnv.overshoot * 0.45),
          }}
        />
      )}

      {/* Optional children inside the rect */}
      {children && (
        <div
          style={{
            position: 'absolute',
            inset: 0,
            opacity: fillEnv.opacity,
          }}
        >
          {children}
        </div>
      )}

      {/* SVG perimeter draw + notch + corner ticks + active laser heads */}
      <svg
        width={w}
        height={h}
        viewBox={`0 0 ${w} ${h}`}
        style={{position: 'absolute', inset: 0, overflow: 'visible'}}
        aria-hidden="true"
      >
        {/* Four orthogonal growing edges */}
        {renderHorizontalEdge(0, topStartX, topEndX, hasTopNotch, 1, 'top')}
        {renderHorizontalEdge(h, bottomStartX, bottomEndX, hasBottomNotch, -1, 'bottom')}
        <line
          x1={0}
          y1={leftStartY}
          x2={0}
          y2={leftEndY}
          stroke={strokeInk}
          strokeWidth={strokeWidth}
          strokeLinecap="square"
        />
        <line
          x1={w}
          y1={rightStartY}
          x2={w}
          y2={rightEndY}
          stroke={strokeInk}
          strokeWidth={strokeWidth}
          strokeLinecap="square"
        />

        {/* Origin corner brackets (always visible once drawing starts) */}
        {isTlBr ? (
          <g stroke={accentInk} strokeWidth={strokeWidth + 0.8} fill="none">
            <path d={`M 0 ${tickLen} V 0 H ${tickLen}`} />
            <path d={`M ${w - tickLen} ${h} H ${w} V ${h - tickLen}`} />
          </g>
        ) : (
          <g stroke={accentInk} strokeWidth={strokeWidth + 0.8} fill="none">
            <path d={`M ${w - tickLen} 0 H ${w} V ${tickLen}`} />
            <path d={`M 0 ${h - tickLen} V ${h} H ${tickLen}`} />
          </g>
        )}

        {/* Meeting corner lock brackets (appear when edges meet at edgeP >= 0.96) */}
        {edgeP >= 0.96 && (
          <g
            stroke={closureFlash > 0.1 ? accentInk : strokeInk}
            strokeWidth={strokeWidth + 0.8 + closureFlash * 1.4}
            opacity={clamp01((edgeP - 0.96) / 0.04)}
            fill="none"
          >
            {isTlBr ? (
              <>
                <path d={`M ${w - tickLen} 0 H ${w} V ${tickLen}`} />
                <path d={`M 0 ${h - tickLen} V ${h} H ${tickLen}`} />
              </>
            ) : (
              <>
                <path d={`M 0 ${tickLen} V 0 H ${tickLen}`} />
                <path d={`M ${w - tickLen} ${h} H ${w} V ${h - tickLen}`} />
              </>
            )}
          </g>
        )}

        {/* Meeting-point closure flare */}
        {closureFlash > 0.02 && (
          <g opacity={closureFlash}>
            {(isTlBr ? [[w, 0], [0, h]] : [[0, 0], [w, h]]).map(([cx, cy], idx) => (
              <g key={idx}>
                <circle cx={cx} cy={cy} r={8 * closureFlash + 2} fill={color('color/fill/gold-wash')} />
                <circle cx={cx} cy={cy} r={3.2} fill={accentInk} />
              </g>
            ))}
          </g>
        )}

        {/* Bright leading heads while the 4 edges are actively growing */}
        {showHeads && (
          <g>
            {[
              [topEndX, 0],
              [bottomEndX, h],
              [0, leftEndY],
              [w, rightEndY],
            ].map(([hx, hy], idx) => (
              <g key={idx}>
                <circle cx={hx} cy={hy} r={6} fill={color('color/fill/gold-wash')} />
                <circle cx={hx} cy={hy} r={2.6} fill={accentInk} />
              </g>
            ))}
          </g>
        )}
      </svg>
    </div>
  );
}
