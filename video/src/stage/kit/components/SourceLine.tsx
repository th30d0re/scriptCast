import React from 'react';
import {color} from '../theme/tokens';

export type SourceLineProps = {
  text: string;
  x?: number;
  y?: number;
  tone?: 'muted' | 'soft' | 'accent' | string;
};

const toneColor = (tone: SourceLineProps['tone']) => {
  if (tone === 'soft') return color('color/text/soft');
  if (tone === 'accent') return color('color/text/accent');
  if (!tone || tone === 'muted') return color('color/text/muted');
  return color(tone);
};

export function SourceLine({text, x = 116, y = 994, tone = 'muted'}: SourceLineProps) {
  return <div data-shared="SourceLine" style={{position:'absolute', left:x, top:y, right:116, font:'400 20px "IBM Plex Mono"', lineHeight:1.35, letterSpacing:.35, color:toneColor(tone)}}>{text}</div>;
}

