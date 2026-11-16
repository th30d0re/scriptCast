import React from 'react';
import {color, dimensions, Tokens as KitTokens} from '../../../theme/tokens';
import {DrawRect} from './DrawRect';
import {Flicker} from './Flicker';
import {clamp01, cubicOut, useAnimationState, type Box, type TimingProps} from './utils';

export type SpacingToken = 'spacing/xs' | 'spacing/sm' | 'spacing/md' | 'spacing/lg' | 'spacing/xl';
export type FitBoxProps = TimingProps & {
  text: string;
  x: number;
  y: number;
  fontSize?: number;
  fontWeight?: 400 | 500 | 600 | 700;
  family?: 'sans' | 'mono';
  paddingX?: SpacingToken;
  paddingY?: SpacingToken;
  stroke?: string;
  fill?: string;
  textColor?: string;
  notch?: boolean | number;
  seed?: number;
};

type TextMetrics = {textWidth: number; fontFamily: string; lineHeight: number; paddingX: number; paddingY: number};
let canvasContext: CanvasRenderingContext2D | null | undefined;

function getCanvasContext() {
  if (canvasContext !== undefined) return canvasContext;
  if (typeof document === 'undefined') {
    canvasContext = null;
  } else {
    canvasContext = document.createElement('canvas').getContext('2d');
  }
  return canvasContext;
}

export function measureFitBox(text: string, {
  fontSize = 28,
  fontWeight = 500,
  family = 'mono',
  paddingX = 'spacing/md',
  paddingY = 'spacing/xs',
}: Pick<FitBoxProps, 'fontSize' | 'fontWeight' | 'family' | 'paddingX' | 'paddingY'> = {}): Box & TextMetrics {
  const fontFamily = KitTokens.fonts[family];
  const font = `${fontWeight} ${fontSize}px ${fontFamily}`;
  const context = getCanvasContext();
  let textWidth: number;
  if (context) {
    context.font = font;
    textWidth = context.measureText(text).width;
  } else {
    const factor = family === 'mono' ? 0.6 : 0.52;
    textWidth = Array.from(text).reduce((width, character) => width + (character === ' ' ? factor * 0.62 : character === 'i' || character === 'l' ? factor * 0.58 : factor), 0) * fontSize;
  }
  const px = dimensions[paddingX].value;
  const py = dimensions[paddingY].value;
  const lineHeight = Math.ceil(fontSize * 1.35);
  return {x: 0, y: 0, w: Math.ceil(textWidth + px * 2), h: lineHeight + py * 2, textWidth, fontFamily, lineHeight, paddingX: px, paddingY: py};
}

export function FitBox({
  text,
  x,
  y,
  fontSize = 28,
  fontWeight = 500,
  family = 'mono',
  paddingX = 'spacing/md',
  paddingY = 'spacing/xs',
  stroke = 'color/line/border',
  fill = 'color/fill/panel-wash',
  textColor = 'color/text/primary',
  notch = false,
  seed = 31,
  progress,
  startFrame,
  durationFrames = 42,
}: FitBoxProps) {
  const metrics = measureFitBox(text, {fontSize, fontWeight, family, paddingX, paddingY});
  const box = {x, y, w: metrics.w, h: metrics.h};
  const state = useAnimationState({progress, startFrame, durationFrames});
  const drawProgress = clamp01(state.progress / 0.58);
  const flickerProgress = clamp01((state.progress - 0.48) / 0.3);
  const textProgress = cubicOut((state.progress - 0.66) / 0.34);

  return <div data-bakeoff="FitBox" data-fitbox-width={box.w} style={{position: 'absolute', inset: 0, pointerEvents: 'none'}}>
    <DrawRect box={box} progress={drawProgress} stroke={stroke} fill={fill} notch={notch} seed={seed}/>
    <div style={{position: 'absolute', left: box.x, top: box.y, width: box.w, height: box.h}}>
      <Flicker seed={seed + 1} progress={flickerProgress} durationFrames={12} intensity={0.68}>
        <div style={{position: 'relative', width: '100%', height: '100%', overflow: 'hidden'}}>
          <div style={{position: 'absolute', left: metrics.paddingX, top: metrics.paddingY, width: metrics.textWidth, height: metrics.lineHeight, clipPath: `inset(0 ${100 * (1 - textProgress)}% 0 0)`, color: color(textColor), fontFamily: metrics.fontFamily, fontSize, fontWeight, lineHeight: `${metrics.lineHeight}px`, whiteSpace: 'pre'}}>{text}</div>
          <div style={{position: 'absolute', left: metrics.paddingX + metrics.textWidth * textProgress, top: metrics.paddingY + 3, width: 2, height: metrics.lineHeight - 6, background: color('color/fill/gold'), opacity: textProgress > 0 && textProgress < 1 ? 0.85 : 0}}/>
        </div>
      </Flicker>
    </div>
  </div>;
}
