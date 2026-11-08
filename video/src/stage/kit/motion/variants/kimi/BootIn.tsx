import React from 'react';
import {color} from '../../../theme/tokens';
import {ScanLines} from '../../../primitives/Icons';
import {DrawRect} from './DrawRect';
import {Flicker} from './Flicker';
import {clamp01, cubicOut, hash01, useAnimationState, type Box, type TimingProps} from './utils';

export type BootInVariant = 'snap' | 'scan' | 'glitch';
export type BootInProps = React.PropsWithChildren<TimingProps & {
  box: Box;
  variant?: BootInVariant;
  seed?: number;
  stroke?: string;
  fill?: string;
  notch?: boolean | number;
}>;

const variants: Record<BootInVariant, {duration: number; drawEnd: number; flickerStart: number; revealStart: number}> = {
  snap: {duration: 32, drawEnd: 0.36, flickerStart: 0.3, revealStart: 0.52},
  scan: {duration: 44, drawEnd: 0.46, flickerStart: 0.38, revealStart: 0.54},
  glitch: {duration: 40, drawEnd: 0.4, flickerStart: 0.32, revealStart: 0.5},
};

export function BootIn({
  box,
  variant = 'snap',
  seed = 47,
  stroke = 'color/line/wire',
  fill = 'color/fill/panel-wash',
  notch = false,
  progress,
  startFrame,
  durationFrames,
  children,
}: BootInProps) {
  const preset = variants[variant];
  const state = useAnimationState({progress, startFrame, durationFrames: durationFrames ?? preset.duration});
  const drawProgress = clamp01(state.progress / preset.drawEnd);
  const flickerProgress = clamp01((state.progress - preset.flickerStart) / (0.84 - preset.flickerStart));
  const reveal = cubicOut((state.progress - preset.revealStart) / (1 - preset.revealStart));
  const glitch = variant === 'glitch' ? (hash01(seed, state.frame) - 0.5) * 14 * (1 - reveal) : 0;
  const clipPath = variant === 'scan'
    ? `inset(0 0 ${100 * (1 - reveal)}% 0)`
    : `inset(0 ${100 * (1 - reveal) * (variant === 'glitch' ? 0.4 : 1)}% 0 0)`;
  const translate = variant === 'snap' ? -10 * (1 - reveal) : glitch;

  return <div data-bakeoff="BootIn" data-variant={variant} style={{position: 'absolute', inset: 0, pointerEvents: 'none'}}>
    <DrawRect box={box} progress={drawProgress} stroke="none" fill={fill} seed={seed}/>
    <div style={{position: 'absolute', left: box.x, top: box.y, width: box.w, height: box.h, overflow: 'hidden'}}>
      <Flicker seed={seed + 2} progress={flickerProgress} durationFrames={14} intensity={variant === 'glitch' ? 0.92 : 0.7}>
        <div style={{width: '100%', height: '100%', opacity: reveal, clipPath, transform: `translateX(${translate.toFixed(2)}px)`}}>{children}</div>
      </Flicker>
    </div>
    <DrawRect box={box} progress={drawProgress} stroke={stroke} fill="none" notch={notch} seed={seed}/>
    {variant === 'scan' && reveal > 0 && reveal < 1 && <svg style={{position: 'absolute', left: box.x, top: box.y, width: box.w, height: box.h, overflow: 'hidden', pointerEvents: 'none'}}>
      <g transform={`translate(0 ${(box.h - 72) * reveal})`} opacity={(1 - reveal) * 0.85}>
        <ScanLines x={0} y={0} w={box.w} h={72} progress={clamp01(reveal * 2)}/>
        <line x1={0} y1={72} x2={box.w} y2={72} stroke={color('color/fill/gold')} strokeWidth={1.6} opacity={0.72}/>
      </g>
    </svg>}
    {variant === 'glitch' && reveal > 0 && reveal < 1 && [0.28, 0.67].map((at, index) => {
      const jitter = hash01(seed + index + 90, state.frame);
      return <div key={at} style={{position: 'absolute', left: box.x + jitter * 18, top: box.y + box.h * at, width: box.w * (0.35 + jitter * 0.45), height: 2, background: color('color/line/accent-soft'), opacity: (1 - reveal) * 0.65, transform: `translateX(${(jitter - 0.5) * 24}px)`}}/>;
    })}
  </div>;
}
