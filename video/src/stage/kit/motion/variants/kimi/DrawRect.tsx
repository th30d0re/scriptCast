import React from 'react';
import {color, dimensions} from '../../../theme/tokens';
import {notchPath} from '../../../primitives/notch';
import {clamp01, cubicInOut, cubicOut, hash01, useAnimationState, type Box, type Point, type TimingProps} from './utils';

export type DrawRectCorner = 'nw-se' | 'ne-sw';
export type DrawRectProps = TimingProps & {
  box: Box;
  stroke?: string;
  fill?: string;
  strokeWidth?: number;
  corners?: DrawRectCorner;
  notch?: boolean | number;
  seed?: number;
};

type Edge = {id: string; from: Point; to: Point};

export function DrawRect({
  box,
  progress,
  startFrame,
  durationFrames = 32,
  stroke = 'color/line/wire',
  fill = 'color/fill/panel-wash',
  strokeWidth = dimensions['stroke/thin'].value,
  corners = 'nw-se',
  notch = false,
  seed = 17,
}: DrawRectProps) {
  const state = useAnimationState({progress, startFrame, durationFrames});
  const {x, y, w, h} = box;
  const nw: Point = [x, y];
  const ne: Point = [x + w, y];
  const se: Point = [x + w, y + h];
  const sw: Point = [x, y + h];
  const edges: Edge[] = corners === 'nw-se'
    ? [{id: 'top', from: nw, to: ne}, {id: 'left', from: nw, to: sw}, {id: 'right', from: se, to: ne}, {id: 'bottom', from: se, to: sw}]
    : [{id: 'top', from: ne, to: nw}, {id: 'right', from: ne, to: se}, {id: 'left', from: sw, to: nw}, {id: 'bottom', from: sw, to: se}];
  const lineProgress = cubicOut(state.progress / 0.68);
  const fillProgress = clamp01((state.progress - 0.7) / 0.3);
  const fillGate = fillProgress >= 1 ? 1 : fillProgress < 0.18 ? 0.08 : fillProgress < 0.4 ? 0.72 : fillProgress < 0.58 ? 0.26 : fillProgress < 0.78 ? 0.9 : 1;
  const fillOpacity = fill === 'none' ? 0 : fillGate * (fillProgress >= 1 ? 1 : 0.9 + hash01(seed, state.frame) * 0.1);
  const notchAmount = (typeof notch === 'boolean' ? Number(notch) : clamp01(notch)) * clamp01((state.progress - 0.68) / 0.22);
  const notchScale = Math.min(0.82, w / 440, h / 130);
  const notchCenter = x + w / 2;
  const hasNotch = notchAmount > 0 && lineProgress >= 1 && notchScale > 0.05;
  const starts = corners === 'nw-se' ? [nw, se] : [ne, sw];

  return <svg data-bakeoff="DrawRect" style={{position: 'absolute', inset: 0, width: '100%', height: '100%', overflow: 'visible', pointerEvents: 'none'}} fill="none" aria-hidden="true">
    {fillOpacity > 0 && <rect x={x} y={y} width={w} height={h} fill={color(fill)} opacity={fillOpacity}/>}
    {edges.map(edge => {
      if (edge.id === 'top' && hasNotch) return null;
      const end: Point = [edge.from[0] + (edge.to[0] - edge.from[0]) * lineProgress, edge.from[1] + (edge.to[1] - edge.from[1]) * lineProgress];
      return <line key={edge.id} x1={edge.from[0]} y1={edge.from[1]} x2={end[0]} y2={end[1]} stroke={color(stroke)} strokeWidth={strokeWidth} strokeLinecap="square"/>;
    })}
    {hasNotch && <>
      <path d={`M${x} ${y}H${notchCenter - 174 * notchScale} M${notchCenter + 174 * notchScale} ${y}H${x + w}`} stroke={color(stroke)} strokeWidth={strokeWidth}/>
      <g transform={`translate(${notchCenter - 960 * notchScale} ${y}) scale(${notchScale})`}>
        <path d={notchPath(0, 1, cubicInOut(notchAmount))} stroke={color(stroke)} strokeWidth={strokeWidth / notchScale} strokeLinecap="round" strokeLinejoin="round"/>
      </g>
    </>}
    {state.progress > 0 && lineProgress < 1 && starts.map((point, index) => <rect key={index} x={point[0] - strokeWidth * 1.5} y={point[1] - strokeWidth * 1.5} width={strokeWidth * 3} height={strokeWidth * 3} fill={color(stroke)} opacity={0.45 + 0.55 * (1 - lineProgress)}/>)}
  </svg>;
}
