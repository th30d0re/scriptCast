import {useKitFrame} from '../../core';
import React from 'react';
import {color, dimensions, skinToneColor, needsFigureRim} from '../../../theme';
import type {PersonGroup} from '../../../components/types';
import {clamp01} from '../../../components/audit';
import {cubicInOut, cubicOut, mix} from '../../../../motion';

export type StickFigureProps = {
  /** Center X position in scene/field coordinates (matching PersonMark). */
  x: number;
  /** Center Y position in scene/field coordinates (matching PersonMark). */
  y: number;
  /** Group classification ('neutral' | 'ingroup' | 'outgroup'), matching PersonMark. */
  group: PersonGroup;
  /** Visual weight & brightness 0..1 (default 1, matching PersonMark). */
  emphasis?: number;
  /** Uniform scale multiplier (1 = 40px tall, matching PersonMark). */
  scale?: number;
  /** Optional normalized draw-on progress 0..1. Defaults to 1 when not animating. */
  progress?: number;
  /** Optional frame when draw-on animation begins. */
  startFrame?: number;
  /** Duration in frames of the draw-on animation (default 22). */
  durationFrames?: number;
  /** If true, animates draw-on from startFrame even when progress is omitted. */
  animate?: boolean;
  /** Optional skin tone 0..1 (or legacy kit colour token overriding the group colour). */
  tone?: number | string;
  /** Optional DOM/audit identifier. */
  id?: string;
};

export function stickFigureColor(group: PersonGroup, emphasis: number, tone?: number | string): string {
  if (typeof tone === 'number') return skinToneColor(tone);
  if (typeof tone === 'string') return color(tone);
  if (group === 'ingroup') return color('color/line/wire');
  if (group === 'outgroup') return '#0B0B0B';
  return emphasis > 0.78 ? color('color/line/wire') : color('color/text/muted');
}

export function StickFigure({
  x,
  y,
  group,
  emphasis: rawEmphasis = 1,
  scale: rawScale = 1,
  progress: rawProgress,
  startFrame,
  durationFrames = 22,
  animate = false,
  tone,
  id,
}: StickFigureProps) {
  const frame = useKitFrame();
  const emphasis = clamp01(rawEmphasis);
  const scale = Math.max(0.25, rawScale);

  // If caller supplies progress, startFrame, or animate=true, compute draw-on progress; otherwise 1 (settled).
  const isAnimated = typeof rawProgress === 'number' || typeof startFrame === 'number' || animate;
  const p = typeof rawProgress === 'number'
    ? clamp01(rawProgress)
    : isAnimated
      ? clamp01((frame - (startFrame ?? 0)) / Math.max(1, durationFrames))
      : 1;

  if (p <= 0) return null;

  // Base dimensions: 28 x 40 px at scale = 1 (readable at 40px tall as required)
  const w = 28 * scale;
  const h = 40 * scale;

  const ink = stickFigureColor(group, emphasis, tone);
  const sparkInk = color('color/fill/gold');
  const wireInk = color('color/line/wire');

  // Stroke weight in kit token range: stroke/thin (2.2) -> stroke/wire (3.2)
  const strokeW = mix(dimensions['stroke/thin'].value, dimensions['stroke/wire'].value, emphasis);
  const numTone = typeof tone === 'number' ? tone : undefined;
  const needsRim = needsFigureRim(numTone, group);
  const rimColor = color('color/line/hairline-strong');
  const rimW = strokeW + 2.8;

  // Staggered limb draw-on phases:
  // 1. Head (0.00 -> 0.36)
  // 2. Torso (0.24 -> 0.58)
  // 3. Two arms (0.44 -> 0.80)
  // 4. Two legs (0.60 -> 1.00)
  const headP = cubicOut(clamp01(p / 0.36));
  const torsoP = cubicInOut(clamp01((p - 0.24) / 0.34));
  const armsP = cubicOut(clamp01((p - 0.44) / 0.36));
  const legsP = cubicOut(clamp01((p - 0.60) / 0.40));

  // Geometry in 28x40 viewBox:
  const headCx = 14;
  const headCy = 6.2;
  const headR = 4.2;
  const headCircumference = 2 * Math.PI * headR;

  const neckY = 12.2;
  const hipY = 24.4;
  const torsoEndY = mix(neckY, hipY, torsoP);

  const shoulderY = 13.4;
  const leftHandX = 4.6;
  const rightHandX = 23.4;
  const handY = 22.0;

  const leftArmEndX = mix(13.2, leftHandX, armsP);
  const leftArmEndY = mix(shoulderY, handY, armsP);
  const rightArmEndX = mix(14.8, rightHandX, armsP);
  const rightArmEndY = mix(shoulderY, handY, armsP);

  const leftFootX = 7.6;
  const rightFootX = 20.4;
  const footY = 37.4;

  const leftLegEndX = mix(14, leftFootX, legsP);
  const leftLegEndY = mix(hipY, footY, legsP);
  const rightLegEndX = mix(14, rightFootX, legsP);
  const rightLegEndY = mix(hipY, footY, legsP);

  const headFillOpacity = emphasis > 0.65 ? ((emphasis - 0.65) / 0.35) * 0.28 * headP : 0;

  return (
    <div
      data-shared="PersonMark"
      data-bakeoff="StickFigure"
      data-group={group}
      data-emphasis={emphasis.toFixed(2)}
      data-stick-id={id}
      style={{
        position: 'absolute',
        left: x,
        top: y,
        width: w,
        height: h,
        transform: 'translate(-50%, -50%)',
        opacity: 0.42 + 0.58 * emphasis,
        color: ink,
        pointerEvents: 'none',
      }}
    >
      <svg
        width="100%"
        height="100%"
        viewBox="0 0 28 40"
        fill="none"
        style={{overflow: 'visible'}}
        aria-hidden="true"
      >
        {/* Subtle HUD ground bracket for high-emphasis figures */}
        {emphasis > 0.78 && legsP > 0.85 && (
          <path
            d="M 5.5 39.4 H 22.5"
            stroke="currentColor"
            strokeWidth={1.3}
            strokeLinecap="round"
            opacity={0.42 * ((legsP - 0.85) / 0.15)}
          />
        )}

        {/* Rim for dark figures on dark backgrounds */}
        {needsRim && (
          <g stroke={rimColor} fill="none" opacity={0.88}>
            {headP > 0 && (
              <circle
                cx={headCx}
                cy={headCy}
                r={headR}
                strokeWidth={rimW}
                strokeLinecap="round"
                strokeDasharray={headCircumference}
                strokeDashoffset={headCircumference * (1 - headP)}
                transform={`rotate(-90 ${headCx} ${headCy})`}
              />
            )}
            {torsoP > 0 && (
              <line
                x1={14}
                y1={neckY}
                x2={14}
                y2={torsoEndY}
                strokeWidth={rimW}
                strokeLinecap="round"
              />
            )}
            {armsP > 0 && (
              <>
                <line
                  x1={13.2}
                  y1={shoulderY}
                  x2={leftArmEndX}
                  y2={leftArmEndY}
                  strokeWidth={rimW * 0.94}
                  strokeLinecap="round"
                />
                <line
                  x1={14.8}
                  y1={shoulderY}
                  x2={rightArmEndX}
                  y2={rightArmEndY}
                  strokeWidth={rimW * 0.94}
                  strokeLinecap="round"
                />
              </>
            )}
            {legsP > 0 && (
              <>
                <line
                  x1={14}
                  y1={hipY}
                  x2={leftLegEndX}
                  y2={leftLegEndY}
                  strokeWidth={rimW * 0.96}
                  strokeLinecap="round"
                />
                <line
                  x1={14}
                  y1={hipY}
                  x2={rightLegEndX}
                  y2={rightLegEndY}
                  strokeWidth={rimW * 0.96}
                  strokeLinecap="round"
                />
              </>
            )}
          </g>
        )}

        {/* 1. Head */}
        {headP > 0 && (
          <circle
            cx={headCx}
            cy={headCy}
            r={headR}
            fill="currentColor"
            fillOpacity={headFillOpacity}
            stroke="currentColor"
            strokeWidth={strokeW}
            strokeLinecap="round"
            strokeDasharray={headCircumference}
            strokeDashoffset={headCircumference * (1 - headP)}
            transform={`rotate(-90 ${headCx} ${headCy})`}
          />
        )}

        {/* 2. Torso */}
        {torsoP > 0 && (
          <line
            x1={14}
            y1={neckY}
            x2={14}
            y2={torsoEndY}
            stroke="currentColor"
            strokeWidth={strokeW}
            strokeLinecap="round"
          />
        )}

        {/* 3. Two Arms */}
        {armsP > 0 && (
          <>
            <line
              x1={13.2}
              y1={shoulderY}
              x2={leftArmEndX}
              y2={leftArmEndY}
              stroke="currentColor"
              strokeWidth={strokeW * 0.94}
              strokeLinecap="round"
            />
            <line
              x1={14.8}
              y1={shoulderY}
              x2={rightArmEndX}
              y2={rightArmEndY}
              stroke="currentColor"
              strokeWidth={strokeW * 0.94}
              strokeLinecap="round"
            />
          </>
        )}

        {/* 4. Two Legs */}
        {legsP > 0 && (
          <>
            <line
              x1={14}
              y1={hipY}
              x2={leftLegEndX}
              y2={leftLegEndY}
              stroke="currentColor"
              strokeWidth={strokeW * 0.96}
              strokeLinecap="round"
            />
            <line
              x1={14}
              y1={hipY}
              x2={rightLegEndX}
              y2={rightLegEndY}
              stroke="currentColor"
              strokeWidth={strokeW * 0.96}
              strokeLinecap="round"
            />
          </>
        )}

        {/* Leading draw-on sparks while p < 1 */}
        {p > 0.01 && p < 0.98 && (
          <g>
            {torsoP > 0.02 && torsoP < 0.98 && (
              <circle cx={14} cy={torsoEndY} r={2} fill={sparkInk} />
            )}
            {armsP > 0.02 && armsP < 0.98 && (
              <>
                <circle cx={leftArmEndX} cy={leftArmEndY} r={1.8} fill={sparkInk} />
                <circle cx={rightArmEndX} cy={rightArmEndY} r={1.8} fill={sparkInk} />
              </>
            )}
            {legsP > 0.02 && legsP < 0.98 && (
              <>
                <circle cx={leftLegEndX} cy={leftLegEndY} r={1.9} fill={sparkInk} />
                <circle cx={rightLegEndX} cy={rightLegEndY} r={1.9} fill={wireInk} />
              </>
            )}
          </g>
        )}
      </svg>
    </div>
  );
}
