import React from 'react';
import {Flicker as OpusFlicker, PowerDown as OpusPowerDown, type FlickerProps as OpusFlickerProps, type PowerDownProps as OpusPowerDownProps} from './Flicker.opus';
import {Flicker as AstraFlicker} from './variants/astra/primitives';
import {Flicker as KimiFlicker, PowerDown as KimiPowerDown} from './variants/kimi/Flicker';
import type {MotionVariant} from './DrawRect';
import {useKitSounds} from './KitSound';

export {FLICKER_DEFAULT_FRAMES, POWER_DOWN_DEFAULT_FRAMES} from './Flicker.opus';
export type FlickerProps = OpusFlickerProps & {variant?:MotionVariant};
export type PowerDownProps = OpusPowerDownProps & {variant?:MotionVariant};

export function Flicker({variant='opus', ...props}:FlickerProps) {
  const audio=useKitSounds('Flicker',variant,props.progress === undefined ? props.startFrame : undefined);
  const visual=variant === 'astra' ? <AstraFlicker {...props} children={props.children}/> : variant === 'kimi' ? <KimiFlicker {...props} seed={props.seed ?? 1}/> : <OpusFlicker {...props}/>;
  return <>{visual}{audio}</>;
}
export function PowerDown({variant='opus', ...props}:PowerDownProps) {
  const audio=useKitSounds('PowerDown',variant,props.progress === undefined ? props.startFrame : undefined);
  const visual=variant === 'astra' ? <AstraFlicker {...props} children={props.children} direction="off"/> : variant === 'kimi' ? <KimiPowerDown {...props} seed={props.seed ?? 1}/> : <OpusPowerDown {...props}/>;
  return <>{visual}{audio}</>;
}
