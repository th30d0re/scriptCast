import React from 'react';
import {BootIn as OpusBootIn, type BootInProps as OpusBootInProps} from './BootIn.opus';
import {BootIn as AstraBootIn} from './variants/astra/primitives';
import {BootIn as KimiBootIn} from './variants/kimi/BootIn';
import type {MotionVariant} from './DrawRect';

export * from './BootIn.opus';
export type BootInProps = Omit<OpusBootInProps, 'variant'> & {variant?:MotionVariant; mode?:import('./BootIn.opus').BootVariant};

export function BootIn({variant='opus', mode='scan', ...props}:BootInProps) {
  if (variant === 'astra') return <AstraBootIn box={props.box} variant={mode} seed={props.seed} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames}>{props.children}</AstraBootIn>;
  if (variant === 'kimi') return <KimiBootIn box={props.box} variant={mode} seed={props.seed} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames} stroke={props.strokeToken} fill={props.fillToken} notch={Boolean(props.notch)}>{props.children}</KimiBootIn>;
  return <OpusBootIn {...props} variant={mode}/>;
}
