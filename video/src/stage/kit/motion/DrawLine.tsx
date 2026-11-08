import React from 'react';
import {DrawLine as OpusDrawLine, type DrawLineProps as OpusDrawLineProps} from './DrawLine.opus';
import {DrawLine as AstraDrawLine} from './variants/astra/primitives';
import {DrawLine as KimiDrawLine} from './variants/kimi/DrawLine';
import type {MotionVariant} from './DrawRect';
import {useKitSounds} from './KitSound';

export * from './DrawLine.opus';
export type DrawLineProps = OpusDrawLineProps & {variant?:MotionVariant};

export function DrawLine({variant='opus', ...props}:DrawLineProps) {
  const audio=useKitSounds('DrawLine',variant,props.progress === undefined ? props.startFrame : undefined);
  const visual=variant === 'astra' ? <AstraDrawLine points={props.points.map(([x,y])=>({x,y}))} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames} stroke={props.strokeToken as `color/${string}`|undefined} strokeWidth={props.strokeWidth}/>
    : variant === 'kimi' ? <KimiDrawLine points={props.points} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames} stroke={props.strokeToken} headStroke={props.glowToken} strokeWidth={props.strokeWidth} headLength={props.tail}/>
    : <OpusDrawLine {...props}/>;
  return <>{visual}{audio}</>;
}
