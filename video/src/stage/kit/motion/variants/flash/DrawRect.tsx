import React from 'react';
import {useCurrentFrame} from 'remotion';
import {color} from '../../../theme/tokens';
import {notchPath} from '../../../primitives/notch';

export type CornerPair = 'tl-br' | 'tr-bl';

export interface DrawRectProps {
  box: {x: number; y: number; w: number; h: number};
  progress?: number;
  startFrame?: number;
  durationFrames?: number;
  strokeToken?: string;
  fillToken?: string;
  strokeWidth?: number;
  cornerChoice?: CornerPair;
  notch?: number; // 0..1 progress of the kit notch on top & bottom edges
  children?: React.ReactNode;
  style?: React.CSSProperties;
}

export const DrawRect: React.FC<DrawRectProps> = ({
  box,
  progress: directProgress,
  startFrame,
  durationFrames = 24,
  strokeToken = 'color/line/wire',
  fillToken = 'color/fill/panel-wash',
  strokeWidth = 1.6,
  cornerChoice = 'tl-br',
  notch,
  children,
  style,
}) => {
  const currentFrame = useCurrentFrame();

  let p: number;
  if (directProgress !== undefined) {
    p = Math.max(0, Math.min(1, directProgress));
  } else if (startFrame !== undefined) {
    p = Math.max(0, Math.min(1, (currentFrame - startFrame) / Math.max(1, durationFrames)));
  } else {
    p = 1;
  }

  const {x, y, w, h} = box;
  const strokeCol = color(strokeToken);
  const fillCol = color(fillToken);

  // Outline drawing phase: 0 -> 0.75
  // Fill fade/flicker phase: 0.75 -> 1.0
  const lineProgress = Math.min(1, p / 0.75);
  const fillProgress = Math.max(0, (p - 0.75) / 0.25);

  const halfPerim = w + h;
  const drawnDistance = lineProgress * halfPerim;

  // We trace two paths originating from opposite corners.
  // When cornerChoice is 'tl-br':
  // Arm 1: from (x, y) [Top-Left] -> heads Right to (x+w, y) and Down to (x, y+h)
  // Arm 2: from (x+w, y+h) [Bottom-Right] -> heads Left to (x, y+h) and Up to (x+w, y)
  // Each arm has two branches that grow simultaneously:
  // Branch Horizontal: grows up to w
  // Branch Vertical: grows up to h
  // As lineProgress goes 0 -> 1, horizontal branch grows by lineProgress * w,
  // vertical branch grows by lineProgress * h.
  // They meet at the other two corners (tr and bl) simultaneously at lineProgress = 1!
  const hDist = lineProgress * w;
  const vDist = lineProgress * h;

  let pathData = '';

  if (cornerChoice === 'tl-br') {
    // Top-Left corner:
    // Arm 1A (horizontal toward TR): (x, y) to (x + hDist, y)
    // Arm 1B (vertical toward BL): (x, y) to (x, y + vDist)
    // Bottom-Right corner:
    // Arm 2A (horizontal toward BL): (x + w, y + h) to (x + w - hDist, y + h)
    // Arm 2B (vertical toward TR): (x + w, y + h) to (x + w, y + h - vDist)
    pathData = `
      M ${x} ${y} H ${x + hDist}
      M ${x} ${y} V ${y + vDist}
      M ${x + w} ${y + h} H ${x + w - hDist}
      M ${x + w} ${y + h} V ${y + h - vDist}
    `;
  } else {
    // Top-Right corner:
    // Arm 1A (horizontal toward TL): (x + w, y) to (x + w - hDist, y)
    // Arm 1B (vertical toward BR): (x + w, y) to (x + w, y + vDist)
    // Bottom-Left corner:
    // Arm 2A (horizontal toward BR): (x, y + h) to (x + hDist, y + h)
    // Arm 2B (vertical toward TL): (x, y + h) to (x, y + h - vDist)
    pathData = `
      M ${x + w} ${y} H ${x + w - hDist}
      M ${x + w} ${y} V ${y + vDist}
      M ${x} ${y + h} H ${x + hDist}
      M ${x} ${y + h} V ${y + h - vDist}
    `;
  }

  // Corner accent pips (small cyber brackets at starting corners)
  const pipLength = Math.min(10, w / 4, h / 4);

  return (
    <div
      data-kit="DrawRect"
      style={{
        position: 'absolute',
        left: x,
        top: y,
        width: w,
        height: h,
        ...style,
      }}
    >
      <svg
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '100%',
          height: '100%',
          overflow: 'visible',
          pointerEvents: 'none',
        }}
        viewBox={`0 0 ${w} ${h}`}
      >
        {/* Fill wash */}
        {fillProgress > 0 && (
          <rect
            x={0}
            y={0}
            width={w}
            height={h}
            fill={fillCol}
            opacity={fillProgress}
          />
        )}

        {/* Dynamic growing outline */}
        <g transform={`translate(${-x}, ${-y})`}>
          <path
            d={pathData}
            stroke={strokeCol}
            strokeWidth={strokeWidth}
            strokeLinecap="round"
            strokeLinejoin="round"
            fill="none"
          />

          {/* Leading heads/pulses at the meeting points during draw */}
          {lineProgress > 0 && lineProgress < 1 && (
            <>
              {cornerChoice === 'tl-br' ? (
                <>
                  <circle cx={x + hDist} cy={y} r={strokeWidth * 1.5} fill={strokeCol} />
                  <circle cx={x} cy={y + vDist} r={strokeWidth * 1.5} fill={strokeCol} />
                  <circle cx={x + w - hDist} cy={y + h} r={strokeWidth * 1.5} fill={strokeCol} />
                  <circle cx={x + w} cy={y + h - vDist} r={strokeWidth * 1.5} fill={strokeCol} />
                </>
              ) : (
                <>
                  <circle cx={x + w - hDist} cy={y} r={strokeWidth * 1.5} fill={strokeCol} />
                  <circle cx={x + w} cy={y + vDist} r={strokeWidth * 1.5} fill={strokeCol} />
                  <circle cx={x + hDist} cy={y + h} r={strokeWidth * 1.5} fill={strokeCol} />
                  <circle cx={x} cy={y + h - vDist} r={strokeWidth * 1.5} fill={strokeCol} />
                </>
              )}
            </>
          )}

          {/* Corner tick notches */}
          {lineProgress > 0.1 && (
            <path
              d={
                cornerChoice === 'tl-br'
                  ? `M ${x + 2} ${y + pipLength} L ${x + 2} ${y + 2} L ${x + pipLength} ${y + 2}
                     M ${x + w - 2} ${y + h - pipLength} L ${x + w - 2} ${y + h - 2} L ${x + w - pipLength} ${y + h - 2}`
                  : `M ${x + w - 2} ${y + pipLength} L ${x + w - 2} ${y + 2} L ${x + w - pipLength} ${y + 2}
                     M ${x + 2} ${y + h - pipLength} L ${x + 2} ${y + h - 2} L ${x + pipLength} ${y + h - 2}`
              }
              stroke={strokeCol}
              strokeWidth={strokeWidth * 1.2}
              fill="none"
              opacity={Math.min(1, lineProgress * 1.5)}
            />
          )}

          {/* Optional Kit notch along the top edge if requested */}
          {notch !== undefined && notch > 0 && w >= 360 && (
            <g transform={`translate(${x + w / 2 - 960}, ${y})`}>
              <path
                d={notchPath(0, 1, notch)}
                stroke={strokeCol}
                strokeWidth={strokeWidth}
                fill="none"
              />
            </g>
          )}
        </g>
      </svg>

      {/* Content wrapper with matching reveal */}
      {children && (
        <div
          style={{
            position: 'relative',
            width: '100%',
            height: '100%',
            opacity: fillProgress,
          }}
        >
          {children}
        </div>
      )}
    </div>
  );
};
