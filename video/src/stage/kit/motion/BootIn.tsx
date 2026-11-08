import React from 'react';
import {BootIn as OpusBootIn, type BootInProps as OpusBootInProps} from './BootIn.opus';
import {BootIn as AstraBootIn} from './variants/astra/primitives';
import {BootIn as KimiBootIn} from './variants/kimi/BootIn';
import {BootIn as ArgonBootIn} from './variants/argon/BootIn';
import type {MotionVariant} from './DrawRect';
import {useTiming} from './core';
import {useKitSounds, type KitSoundCueProps} from './KitSound';

export * from './BootIn.opus';
export type BootInProps = Omit<OpusBootInProps, 'variant'> & KitSoundCueProps & {variant?:MotionVariant; mode?:import('./BootIn.opus').BootVariant};

export function BootIn({variant='argon', mode='scan', beat, beatCue, ...props}:BootInProps) {
  const audio=useKitSounds('BootIn',variant,props.progress === undefined ? props.startFrame : undefined,mode,beat,beatCue);
  const visual=variant === 'argon' ? <ArgonBootIn box={props.box} variant={mode} seed={props.seed} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames} stroke={props.strokeToken} fill={props.fillToken} accent={props.glowToken} corner={props.corners} notch={props.notch}>{props.children}</ArgonBootIn>
    : variant === 'astra' ? <AstraBootIn box={props.box} variant={mode} seed={props.seed} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames}>{props.children}</AstraBootIn>
    : variant === 'kimi' ? <KimiBootIn box={props.box} variant={mode} seed={props.seed} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames} stroke={props.strokeToken} fill={props.fillToken} notch={Boolean(props.notch)}>{props.children}</KimiBootIn>
    : <OpusBootIn {...props} variant={mode}/>;
  // A childless BootIn is a boot flash over a panel drawn elsewhere: it leaves nothing behind once done.
  const {p}=useTiming(props, props.durationFrames ?? 18);
  if (props.children == null && p >= 1) return <>{audio}</>;
  return <>{visual}{audio}</>;
}
