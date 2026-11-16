import React from 'react';
import {color, dimensions} from '../../../theme/tokens';
import {clamp01, cubicInOut, pointAtDistance, polylineLength, polylineSlice, useAnimationState, type Point, type TimingProps} from './utils';

export type DrawLineProps = TimingProps & {
  points: readonly Point[];
  stroke?: string;
  headStroke?: string;
  strokeWidth?: number;
  headLength?: number;
};

export function DrawLine({
  points,
  progress,
  startFrame,
  durationFrames = 26,
  stroke = 'color/line/chart',
  headStroke = 'color/line/wire',
  strokeWidth = dimensions['stroke/wire'].value,
  headLength = 96,
}: DrawLineProps) {
  const state = useAnimationState({progress, startFrame, durationFrames});
  const total = polylineLength(points);
  const drawProgress = cubicInOut(state.progress);
  const distance = total * drawProgress;
  const trailStart = Math.max(0, distance - headLength);
  const trailLength = distance - trailStart;
  const trailSteps = 14;
  const head = pointAtDistance(points, distance);
  const resting = clamp01((state.progress - 0.88) / 0.12);

  return <svg data-bakeoff="DrawLine" style={{position: 'absolute', inset: 0, width: '100%', height: '100%', overflow: 'visible', pointerEvents: 'none'}} fill="none" aria-hidden="true">
    {distance > 0 && <path d={polylineSlice(points, 0, distance)} stroke={color(stroke)} strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round" opacity={0.72}/>}
    {trailLength > 0 && Array.from({length: trailSteps}, (_, index) => {
      const from = trailStart + trailLength * index / trailSteps;
      const to = trailStart + trailLength * (index + 1) / trailSteps;
      const strength = (index + 1) / trailSteps;
      return <path key={index} d={polylineSlice(points, from, to)} stroke={color(headStroke)} strokeWidth={strokeWidth + strength * 0.35} strokeLinecap="round" strokeLinejoin="round" opacity={0.12 + strength * 0.82}/>;
    })}
    {total > 0 && state.progress > 0 && <g opacity={1 - resting * 0.45}>
      <circle cx={head[0]} cy={head[1]} r={strokeWidth * (2.3 - resting)} stroke={color(headStroke)} strokeWidth={strokeWidth * 0.55} opacity={0.28 * (1 - resting)}/>
      <circle cx={head[0]} cy={head[1]} r={strokeWidth * (0.9 - resting * 0.25)} fill={color(headStroke)}/>
    </g>}
  </svg>;
}
