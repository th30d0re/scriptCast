import React from 'react';
import {DrawLine as OpusDrawLine, type DrawLineProps as OpusDrawLineProps} from './DrawLine.opus';
import {DrawLine as AstraDrawLine} from './variants/astra/primitives';
import {DrawLine as KimiDrawLine} from './variants/kimi/DrawLine';
import {DrawLine as ArgonDrawLine} from './variants/argon/DrawLine';
import type {MotionVariant} from './DrawRect';
import {useKitSounds, type KitSoundCueProps} from './KitSound';

export * from './DrawLine.opus';
export type DrawLineProps = OpusDrawLineProps & KitSoundCueProps & {variant?:MotionVariant};

export function DrawLine({variant='argon', beat, beatCue, ...props}:DrawLineProps) {
  const audio=useKitSounds('DrawLine',variant,props.progress === undefined ? props.startFrame : undefined,undefined,beat,beatCue);
  const visual=variant === 'argon' ? <ArgonDrawLine points={props.points} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames} stroke={props.strokeToken} headColor={props.glowToken} strokeWidth={props.strokeWidth} tailLength={props.tail === undefined ? undefined : Math.min(1,props.tail/200)} arrow={props.end === 'arrow'} nodes={props.nodes}/>
    : variant === 'astra' ? <AstraDrawLine points={props.points.map(([x,y])=>({x,y}))} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames} stroke={props.strokeToken as `color/${string}`|undefined} strokeWidth={props.strokeWidth}/>
    : variant === 'kimi' ? <KimiDrawLine points={props.points} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames} stroke={props.strokeToken} headStroke={props.glowToken} strokeWidth={props.strokeWidth} headLength={props.tail}/>
    : <OpusDrawLine {...props}/>;
  return <>{visual}{audio}</>;
}
