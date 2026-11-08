import React from 'react';
import {FitBox as OpusFitBox, type FitBoxProps as OpusFitBoxProps} from './FitBox.opus';
import {FitBox as AstraFitBox} from './variants/astra/primitives';
import {FitBox as KimiFitBox} from './variants/kimi/FitBox';
import {FitBox as FlashFitBox} from './variants/flash/FitBox';
import {useKitSounds} from './KitSound';

export * from './FitBox.opus';
export type FitBoxVariant = 'opus'|'astra'|'kimi'|'flash';
export type FitBoxProps = OpusFitBoxProps & {variant?:FitBoxVariant};

export function FitBox({variant='opus', ...props}:FitBoxProps) {
  const audio=useKitSounds('FitBox',variant,props.progress === undefined ? props.startFrame : undefined);
  const x=props.x, y=props.y, fontSize=props.size, family=props.family;
  const visual=variant === 'astra' ? <AstraFitBox text={props.text} x={x} y={y} fontSize={fontSize} padding={props.padX === 'spacing/xl' ? 'spacing/lg' : props.padX} seed={props.seed} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames}/>
    : variant === 'kimi' ? <KimiFitBox text={props.text} x={x} y={y} fontSize={fontSize} fontWeight={props.weight as 400|500|600|700|undefined} family={family} paddingX={props.padX} paddingY={props.padY} stroke={props.strokeToken} fill={props.fillToken} textColor={props.textToken} notch={Boolean(props.notch)} seed={props.seed} progress={props.progress} startFrame={props.startFrame} durationFrames={props.durationFrames}/>
    : variant === 'flash' ? <FlashFitBox text={props.text} x={x} y={y} fontSize={fontSize} fontFamily={family} paddingX={undefined} paddingY={undefined} strokeToken={props.strokeToken} fillToken={props.fillToken} textToken={props.textToken} badge={props.tag} startFrame={props.startFrame} drawDuration={props.durationFrames ? Math.round(props.durationFrames*.56) : undefined} flickerDuration={props.durationFrames ? Math.round(props.durationFrames*.44) : undefined}/>
    : <OpusFitBox {...props}/>;
  return <>{visual}{audio}</>;
}
