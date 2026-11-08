import React from 'react';
import {Flicker as OpusFlicker, PowerDown as OpusPowerDown, type FlickerProps as OpusFlickerProps, type PowerDownProps as OpusPowerDownProps} from './Flicker.opus';
import {Flicker as AstraFlicker} from './variants/astra/primitives';
import {Flicker as KimiFlicker, PowerDown as KimiPowerDown} from './variants/kimi/Flicker';
import type {MotionVariant} from './DrawRect';

export {FLICKER_DEFAULT_FRAMES, POWER_DOWN_DEFAULT_FRAMES} from './Flicker.opus';
export type FlickerProps = OpusFlickerProps & {variant?:MotionVariant};
export type PowerDownProps = OpusPowerDownProps & {variant?:MotionVariant};

export function Flicker({variant='opus', ...props}:FlickerProps) {
  if (variant === 'astra') return <AstraFlicker {...props} children={props.children}/>;
  if (variant === 'kimi') return <KimiFlicker {...props} seed={props.seed ?? 1}/>;
  return <OpusFlicker {...props}/>;
}
export function PowerDown({variant='opus', ...props}:PowerDownProps) {
  if (variant === 'astra') return <AstraFlicker {...props} children={props.children} direction="off"/>;
  if (variant === 'kimi') return <KimiPowerDown {...props} seed={props.seed ?? 1}/>;
  return <OpusPowerDown {...props}/>;
}
