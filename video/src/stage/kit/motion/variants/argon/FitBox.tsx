import React from 'react';
import {useCurrentFrame} from 'remotion';
import {color, dimensions} from '../../../theme';
import {
  MONO_ADVANCE,
  SANS_ADVANCE,
  clamp01,
  monoWidth,
  textAttrs,
  type Box,
} from '../../../components/audit';
import {DrawRect, type DrawRectCorner, type DrawRectNotch} from './DrawRect';
import {Flicker} from './Flicker';

export type SpacingToken =
  | 'spacing/xs'
  | 'spacing/sm'
  | 'spacing/md'
  | 'spacing/lg'
  | 'spacing/xl'
  | 'spacing/border'
  | 'xs'
  | 'sm'
  | 'md'
  | 'lg'
  | 'xl';

export type PaddingSpec =
  | SpacingToken
  | number
  | readonly [SpacingToken | number, SpacingToken | number];

export function resolveSpacingToken(token: SpacingToken | number): number {
  if (typeof token === 'number') return token;
  const key = token.startsWith('spacing/') ? token : (`spacing/${token}` as keyof typeof dimensions);
  const entry = dimensions[key as keyof typeof dimensions];
  return entry ? entry.value : 16;
}

export function resolvePadding(
  padding?: PaddingSpec,
  paddingX?: SpacingToken | number,
  paddingY?: SpacingToken | number,
): {padX: number; padY: number} {
  let padX = resolveSpacingToken('spacing/md');
  let padY = resolveSpacingToken('spacing/sm');
  if (Array.isArray(padding)) {
    padX = resolveSpacingToken(padding[0]);
    padY = resolveSpacingToken(padding[1]);
  } else if (padding !== undefined) {
    const v = resolveSpacingToken(padding as SpacingToken | number);
    padX = v;
    padY = v;
  }
  if (paddingX !== undefined) padX = resolveSpacingToken(paddingX);
  if (paddingY !== undefined) padY = resolveSpacingToken(paddingY);
  return {padX, padY};
}

/** Measures a single line in IBM Plex Mono or IBM Plex Sans using kit font metrics. */
export function measureKitLineWidth(
  line: string,
  fontSize: number,
  font: 'mono' | 'sans',
  letterSpacing = 0,
): number {
  if (font === 'mono') {
    return monoWidth(line, fontSize, letterSpacing);
  }
  // IBM Plex Sans per-glyph class estimate anchored to kit SANS_ADVANCE (0.56em)
  let totalEm = 0;
  for (const ch of line) {
    if ('ilI1|.,:;!\'"` '.includes(ch)) totalEm += 0.34;
    else if ('mwMW@%—'.includes(ch)) totalEm += 0.82;
    else if (ch >= 'A' && ch <= 'Z') totalEm += 0.64;
    else totalEm += SANS_ADVANCE;
  }
  // Guarantee we are never smaller than kit audit's average SANS_ADVANCE * 0.96
  const fallback = [...line].length * SANS_ADVANCE * 0.96;
  return Math.max(totalEm * fontSize, fallback) + [...line].length * letterSpacing;
}

export type FitBoxMeasureOptions = {
  font?: 'mono' | 'sans';
  size?: number;
  letterSpacing?: number;
  lineHeight?: number;
  padding?: PaddingSpec;
  paddingX?: SpacingToken | number;
  paddingY?: SpacingToken | number;
  maxWidth?: number;
  tag?: string;
  accentBar?: boolean;
  hasTopNotch?: boolean;
};

export type FitBoxMeasurement = {
  w: number;
  h: number;
  textW: number;
  textH: number;
  padX: number;
  padY: number;
  topExtra: number;
  lines: string[];
  tagH: number;
  barOffset: number;
};

/**
 * Deterministically measures `text` with the kit fonts (IBM Plex Mono / IBM Plex Sans)
 * and spacing tokens to compute the exact bounding box dimensions.
 */
export function measureFitBox(text: string, opts: FitBoxMeasureOptions = {}): FitBoxMeasurement {
  const font = opts.font ?? 'mono';
  const size = opts.size ?? 22;
  const ls = opts.letterSpacing ?? (font === 'mono' ? 0.6 : 0.2);
  const lh = opts.lineHeight ?? 1.32;
  const {padX, padY} = resolvePadding(opts.padding, opts.paddingX, opts.paddingY);
  const barOffset = opts.accentBar ? 10 : 0;
  const topExtra = opts.hasTopNotch ? 14 : 0;

  const maxTextW = opts.maxWidth
    ? Math.max(60, opts.maxWidth - padX * 2 - barOffset)
    : Infinity;

  const paragraphs = text.split('\n');
  const lines: string[] = [];

  for (const para of paragraphs) {
    const words = para.split(' ').filter(Boolean);
    if (words.length === 0) {
      lines.push('');
      continue;
    }
    let current = words[0];
    for (let i = 1; i < words.length; i++) {
      const candidate = `${current} ${words[i]}`;
      if (measureKitLineWidth(candidate, size, font, ls) <= maxTextW) {
        current = candidate;
      } else {
        lines.push(current);
        current = words[i];
      }
    }
    lines.push(current);
  }

  const measuredLines = lines.map(l => measureKitLineWidth(l, size, font, ls));
  const rawTextW = measuredLines.length ? Math.max(...measuredLines) : 0;
  const tagW = opts.tag ? monoWidth(opts.tag, 13, 1.1) + 12 : 0;
  // Add 10px subpixel safety margin so browser glyph kerning never forces an unwanted wrap
  const textW = Math.ceil(Math.max(rawTextW, tagW) + 10);
  const linePx = Math.ceil(size * lh);
  const textH = Math.max(linePx, lines.length * linePx);
  const tagH = opts.tag ? 22 : 0;

  const w = Math.ceil(padX * 2 + barOffset + textW);
  const h = Math.ceil(padY * 2 + topExtra + tagH + textH);

  return {w, h, textW, textH, padX, padY, topExtra, lines, tagH, barOffset};
}

export type FitBoxProps = {
  /** Text string to measure and display inside the box. */
  text: string;
  /** X coordinate in scene/parent space (top-left by default, or center if anchor='center'). */
  x?: number;
  /** Y coordinate in scene/parent space. */
  y?: number;
  /** Anchor mode for (x, y) placement (default 'top-left'). */
  anchor?: 'top-left' | 'center';
  /** Kit font family: 'mono' (IBM Plex Mono) or 'sans' (IBM Plex Sans). Default 'mono'. */
  font?: 'mono' | 'sans';
  /** Font size in px (default 22). */
  size?: number;
  /** Font weight (default 500). */
  weight?: number;
  /** Letter spacing in px. */
  letterSpacing?: number;
  /** Line height multiplier (default 1.32). */
  lineHeight?: number;
  /** Kit spacing token(s) for padding, e.g. 'spacing/sm', 'spacing/md', or ['spacing/md', 'spacing/sm']. */
  padding?: PaddingSpec;
  /** Horizontal padding token override. */
  paddingX?: SpacingToken | number;
  /** Vertical padding token override. */
  paddingY?: SpacingToken | number;
  /** Optional maximum box width in px before wrapping text onto multiple lines. */
  maxWidth?: number;
  /** Optional small mono telemetry tag above the text inside the box. */
  tag?: string;
  /** Kit colour token for text (default 'color/text/primary'). */
  ink?: string;
  /** Kit colour token for border stroke (default 'color/line/wire'). */
  stroke?: string;
  /** Kit colour token for interior fill (default 'color/fill/panel-render'). */
  fill?: string;
  /** Kit colour token for corner/bar accents (default 'color/fill/gold'). */
  accent?: string;
  /** Draw a vertical accent bar along the left inner edge (default true). */
  accentBar?: boolean;
  /** Corner pair for DrawRect ('tl-br' or 'tr-bl'). */
  corner?: DrawRectCorner;
  /** Optional kit notch on DrawRect. */
  notch?: DrawRectNotch;
  /** Direct progress 0..1 (overrides startFrame/durationFrames). */
  progress?: number;
  /** Entrance start frame (default 0). */
  startFrame?: number;
  /** Entrance duration in frames (default 30). */
  durationFrames?: number;
  /** Deterministic seed for Flicker. */
  seed?: number | string;
  /** Audit / debug ID. */
  id?: string;
};

export function FitBox({
  text,
  x = 0,
  y = 0,
  anchor = 'top-left',
  font = 'mono',
  size = 22,
  weight = 500,
  letterSpacing,
  lineHeight = 1.32,
  padding,
  paddingX,
  paddingY,
  maxWidth,
  tag,
  ink = 'color/text/primary',
  stroke = 'color/line/wire',
  fill = 'color/fill/panel-render',
  accent = 'color/fill/gold',
  accentBar = true,
  corner = 'tl-br',
  notch = false,
  progress: rawProgress,
  startFrame = 0,
  durationFrames = 30,
  seed = 202,
  id = 'fit-box',
}: FitBoxProps) {
  const frame = useCurrentFrame();
  const ls = letterSpacing ?? (font === 'mono' ? 0.6 : 0.2);
  const hasTopNotch =
    notch === true || notch === 'top' || notch === 'both' || (typeof notch === 'number' && notch > 0);

  const measured = measureFitBox(text, {
    font,
    size,
    letterSpacing: ls,
    lineHeight,
    padding,
    paddingX,
    paddingY,
    maxWidth,
    tag,
    accentBar,
    hasTopNotch,
  });

  const boxX = anchor === 'center' ? Math.round(x - measured.w / 2) : Math.round(x);
  const boxY = anchor === 'center' ? Math.round(y - measured.h / 2) : Math.round(y);
  const box: Box = {x: boxX, y: boxY, w: measured.w, h: measured.h};

  const p = typeof rawProgress === 'number'
    ? clamp01(rawProgress)
    : clamp01((frame - startFrame) / Math.max(1, durationFrames));

  if (p <= 0) return null;

  // DrawRect runs over 0..0.78 of FitBox entrance; Flicker powers the text in over 0.34..1.00
  const rectP = clamp01(p / 0.78);
  const flickerP = clamp01((p - 0.34) / 0.66);

  const topPad = measured.padY + measured.topExtra;
  const textBox: Box = {
    x: boxX + measured.padX + measured.barOffset,
    y: boxY + topPad + measured.tagH,
    w: measured.textW,
    h: measured.textH,
  };

  return (
    <div
      data-bakeoff="FitBox"
      data-fitbox-id={id}
      data-fitbox-size={`${measured.w}x${measured.h}`}
      style={{position: 'absolute', left: 0, top: 0, width: 0, height: 0}}
    >
      <DrawRect
        id={`${id}-rect`}
        box={box}
        progress={rectP}
        stroke={stroke}
        fill={fill}
        accent={accent}
        corner={corner}
        notch={notch}
        seed={seed}
      />

      {flickerP > 0 && (
        <Flicker
          id={`${id}-flicker`}
          seed={seed}
          progress={flickerP}
          intensity={0.9}
          style={{
            position: 'absolute',
            left: box.x,
            top: box.y,
            width: box.w,
            height: box.h,
            pointerEvents: 'none',
          }}
        >
          {accentBar && (
            <div
              aria-hidden="true"
              style={{
                position: 'absolute',
                left: Math.max(4, Math.floor(measured.padX * 0.45)),
                top: Math.max(6, Math.floor(topPad * 0.65)),
                bottom: Math.max(6, Math.floor(measured.padY * 0.65)),
                width: 3,
                background: color(accent),
              }}
            />
          )}

          {tag && (
            <div
              style={{
                position: 'absolute',
                left: measured.padX + measured.barOffset,
                top: topPad - 2,
                font: '500 13px "IBM Plex Mono"',
                letterSpacing: 1.1,
                color: color(accent),
                whiteSpace: 'nowrap',
              }}
            >
              {tag}
            </div>
          )}

          <div
            {...textAttrs(`${id}-text`, textBox, {
              size,
              ls,
              lh: lineHeight,
              sans: font === 'sans',
            })}
            style={{
              position: 'absolute',
              left: measured.padX + measured.barOffset,
              top: topPad + measured.tagH,
              width: measured.textW,
              height: measured.textH,
              font: `${weight} ${size}px "${font === 'sans' ? 'IBM Plex Sans' : 'IBM Plex Mono'}"`,
              letterSpacing: ls,
              lineHeight,
              color: color(ink),
              whiteSpace: 'pre',
            }}
          >
            {measured.lines.map((line, idx) => (
              <div key={idx} style={{height: size * lineHeight}}>
                {line}
              </div>
            ))}
          </div>
        </Flicker>
      )}
    </div>
  );
}
