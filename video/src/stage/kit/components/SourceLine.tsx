import React from 'react';
import {useTiming, type Timing} from '../motion/core';
import {color} from '../theme/tokens';
import {QRCode} from '../primitives/QRCode';

export type SourceLineProps = Timing & {
  text: string;
  x?: number;
  y?: number;
  tone?: 'muted' | 'soft' | 'accent' | string;
  /** Optional URL rendered as a QR code beside the citation. */
  qr?: string;
  /** Limits wrapped copy when this line occupies a reserved footer row. */
  maxLines?: number;
};

const toneColor = (tone: SourceLineProps['tone']) => {
  if (tone === 'soft') return color('color/text/soft');
  if (tone === 'accent') return color('color/text/accent');
  if (!tone || tone === 'muted') return color('color/text/muted');
  return color(tone);
};

export function SourceLine({text, x = 116, y = 994, tone = 'muted', qr, maxLines, progress, startFrame, durationFrames}: SourceLineProps) {
  // Untimed means fully shown (kit convention).
  if (progress === undefined && startFrame === undefined) progress = 1;
  const hasTiming = progress !== undefined || startFrame !== undefined || durationFrames !== undefined;
  const {p} = useTiming({progress, startFrame, durationFrames}, 18);
  const visibleText = hasTiming ? text.slice(0, Math.ceil(text.length*p)) : text;
  const textStyle: React.CSSProperties = {overflow:maxLines ? 'hidden' : undefined, display:maxLines ? '-webkit-box' : undefined, WebkitBoxOrient:maxLines ? 'vertical' : undefined, WebkitLineClamp:maxLines, font:'400 20px "IBM Plex Mono"', lineHeight:1.35, letterSpacing:.35, color:toneColor(tone)};
  if (!qr) return <div data-shared="SourceLine" style={{position:'absolute', left:x, top:y, right:116, ...textStyle}}>{visibleText}</div>;
  return <div data-shared="SourceLine" style={{position:'absolute', left:x, top:y, right:116, display:'flex', alignItems:'center', gap:16}}>
    <QRCode url={qr} size={96} progress={p}/>
    <div style={{...textStyle, flex:1}}>{visibleText}</div>
  </div>;
}
