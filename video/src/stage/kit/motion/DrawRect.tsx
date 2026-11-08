import React from 'react';
import {DrawRect as OpusDrawRect, type DrawRectProps as OpusDrawRectProps} from './DrawRect.opus';
import {DrawRect as AstraDrawRect} from './variants/astra/primitives';
import {DrawRect as KimiDrawRect} from './variants/kimi/DrawRect';
import {useKitSounds} from './KitSound';

export * from './DrawRect.opus';
export type MotionVariant = 'opus'|'astra'|'kimi';
export type DrawRectProps = OpusDrawRectProps & {variant?:MotionVariant};

export function DrawRect({variant='opus', ...props}:DrawRectProps) {
  const audio=useKitSounds('DrawRect',variant,props.progress === undefined ? props.startFrame : undefined);
  const visual=variant === 'astra' ? <AstraDrawRect box={props.box} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames} stroke={props.strokeToken as `color/${string}`|undefined} fill={props.fillToken as `color/${string}`|undefined} corner={props.corners === 'tr-bl' ? 'top-right' : 'top-left'} notch={Boolean(props.notch)} seed={props.seed}/>
    : variant === 'kimi' ? <KimiDrawRect box={props.box} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames} stroke={props.strokeToken} fill={props.fillToken} strokeWidth={props.strokeWidth} corners={props.corners === 'tr-bl' ? 'ne-sw' : 'nw-se'} notch={Boolean(props.notch)} seed={props.seed}/>
    : <OpusDrawRect {...props}/>;
  return <>{visual}{audio}</>;
}
