import React from 'react';
import {Flicker as OpusFlicker, PowerDown as OpusPowerDown, type FlickerProps as OpusFlickerProps, type PowerDownProps as OpusPowerDownProps} from './Flicker.opus';
import {Flicker as AstraFlicker} from './variants/astra/primitives';
import {Flicker as KimiFlicker, PowerDown as KimiPowerDown} from './variants/kimi/Flicker';
import {Flicker as ArgonFlicker} from './variants/argon/Flicker';
import type {MotionVariant} from './DrawRect';
import {useKitSounds, type KitSoundCueProps} from './KitSound';

export {FLICKER_DEFAULT_FRAMES, POWER_DOWN_DEFAULT_FRAMES} from './Flicker.opus';
export type FlickerProps = OpusFlickerProps & KitSoundCueProps & {variant?:MotionVariant};
export type PowerDownProps = OpusPowerDownProps & KitSoundCueProps & {variant?:MotionVariant};

export function Flicker({variant='opus', beat, beatCue, ...props}:FlickerProps) {
  const audio=useKitSounds('Flicker',variant,props.progress === undefined ? props.startFrame : undefined,undefined,beat,beatCue);
  const visual=variant === 'argon' ? <ArgonFlicker {...props}/> : variant === 'astra' ? <AstraFlicker {...props} children={props.children}/> : variant === 'kimi' ? <KimiFlicker {...props} seed={props.seed ?? 1}/> : <OpusFlicker {...props}/>;
  return <>{visual}{audio}</>;
}
export function PowerDown({variant='opus', beat, beatCue, ...props}:PowerDownProps) {
  const audio=useKitSounds('PowerDown',variant,props.progress === undefined ? props.startFrame : undefined,undefined,beat,beatCue);
  const visual=variant === 'argon' ? <ArgonFlicker {...props} mode="off"/> : variant === 'astra' ? <AstraFlicker {...props} children={props.children} direction="off"/> : variant === 'kimi' ? <KimiPowerDown {...props} seed={props.seed ?? 1}/> : <OpusPowerDown {...props}/>;
  return <>{visual}{audio}</>;
}
