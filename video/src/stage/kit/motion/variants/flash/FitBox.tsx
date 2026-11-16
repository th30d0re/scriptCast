import React, {useMemo} from 'react';
import {color} from '../../../theme/tokens';
import {DrawRect} from './DrawRect';
import {Flicker} from './Flicker';
import {stringToSeed} from './hash';

export interface FitBoxProps {
  text: string;
  x: number;
  y: number;
  startFrame?: number;
  drawDuration?: number; // duration of border draw
  flickerDuration?: number; // duration of content flicker
  fontSize?: number; // defaults to 18px
  fontFamily?: 'mono' | 'sans';
  paddingX?: number; // default 16px (spacing/sm)
  paddingY?: number; // default 8px (spacing/xs)
  strokeToken?: string;
  fillToken?: string;
  textToken?: string;
  badge?: string; // e.g. "SYS.01" or "STATUS"
  style?: React.CSSProperties;
}

// Approximate text measurement for IBM Plex fonts
// IBM Plex Mono has strict fixed width: ~0.6 * fontSize
// IBM Plex Sans has proportional average width: ~0.55 * fontSize
export function measureTextWidth(text: string, fontSize: number, fontFamily: 'mono' | 'sans'): number {
  const lines = text.split('\n');
  let maxLen = 0;
  for (const line of lines) {
    if (line.length > maxLen) maxLen = line.length;
  }
  const charWidthRatio = fontFamily === 'mono' ? 0.60 : 0.54;
  return Math.ceil(maxLen * fontSize * charWidthRatio);
}

export const FitBox: React.FC<FitBoxProps> = ({
  text,
  x,
  y,
  startFrame = 0,
  drawDuration = 18,
  flickerDuration = 14,
  fontSize = 18,
  fontFamily = 'sans',
  paddingX = 18,
  paddingY = 12,
  strokeToken = 'color/line/wire',
  fillToken = 'color/surface/1',
  textToken = 'color/text/primary',
  badge,
  style,
}) => {
  const textLines = text.split('\n');
  const lineHeight = fontSize * 1.35;

  const textWidth = useMemo(
    () => measureTextWidth(text, fontSize, fontFamily),
    [text, fontSize, fontFamily]
  );
  const badgeWidth = useMemo(
    () => (badge ? measureTextWidth(badge, fontSize * 0.7, 'mono') + 16 : 0),
    [badge, fontSize]
  );

  const contentWidth = Math.max(textWidth, badgeWidth);
  const contentHeight = textLines.length * lineHeight;

  const totalWidth = contentWidth + paddingX * 2;
  const totalHeight = contentHeight + paddingY * 2 + (badge ? fontSize * 0.9 : 0);

  const box = {
    x,
    y,
    w: totalWidth,
    h: totalHeight,
  };

  const seed = useMemo(() => stringToSeed(text + (badge ?? '')), [text, badge]);
  const fontFace = fontFamily === 'mono' ? '"IBM Plex Mono", monospace' : '"IBM Plex Sans", sans-serif';
  const textCol = color(textToken);
  const badgeCol = color('color/text/accent');

  const flickerStart = startFrame + Math.floor(drawDuration * 0.7);

  return (
    <div data-kit="FitBox" style={{position: 'absolute', left: 0, top: 0, ...style}}>
      {/* Self-drawing bordered chassis */}
      <DrawRect
        box={box}
        startFrame={startFrame}
        durationFrames={drawDuration}
        strokeToken={strokeToken}
        fillToken={fillToken}
        strokeWidth={1.8}
        cornerChoice="tl-br"
      />

      {/* Content powered up with intentional flicker */}
      <div
        style={{
          position: 'absolute',
          left: x + paddingX,
          top: y + paddingY,
          width: contentWidth,
          height: contentHeight + (badge ? fontSize * 0.9 : 0),
          pointerEvents: 'none',
        }}
      >
        <Flicker
          seed={seed}
          startFrame={flickerStart}
          durationFrames={flickerDuration}
          intensity={0.85}
        >
          {badge && (
            <div
              style={{
                fontFamily: '"IBM Plex Mono", monospace',
                fontSize: fontSize * 0.68,
                letterSpacing: '0.12em',
                textTransform: 'uppercase',
                color: badgeCol,
                marginBottom: 4,
                opacity: 0.88,
              }}
            >
              [{badge}]
            </div>
          )}
          <div
            style={{
              fontFamily: fontFace,
              fontSize,
              lineHeight: `${lineHeight}px`,
              color: textCol,
              whiteSpace: 'pre-wrap',
            }}
          >
            {text}
          </div>
        </Flicker>
      </div>
    </div>
  );
};
