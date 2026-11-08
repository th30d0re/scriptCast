import React from 'react';
import {BootIn as OpusBootIn, type BootInProps as OpusBootInProps} from './BootIn.opus';
import {BootIn as AstraBootIn} from './variants/astra/primitives';
import {BootIn as KimiBootIn} from './variants/kimi/BootIn';
import type {MotionVariant} from './DrawRect';
import {useKitSounds} from './KitSound';

export * from './BootIn.opus';
export type BootInProps = Omit<OpusBootInProps, 'variant'> & {variant?:MotionVariant; mode?:import('./BootIn.opus').BootVariant};

export function BootIn({variant='opus', mode='scan', ...props}:BootInProps) {
  const audio=useKitSounds('BootIn',variant,props.progress === undefined ? props.startFrame : undefined,mode);
  const visual=variant === 'astra' ? <AstraBootIn box={props.box} variant={mode} seed={props.seed} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames}>{props.children}</AstraBootIn>
    : variant === 'kimi' ? <KimiBootIn box={props.box} variant={mode} seed={props.seed} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames} stroke={props.strokeToken} fill={props.fillToken} notch={Boolean(props.notch)}>{props.children}</KimiBootIn>
    : <OpusBootIn {...props} variant={mode}/>;
  return <>{visual}{audio}</>;
}
