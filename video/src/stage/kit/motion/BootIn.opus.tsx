import React from 'react';
import {color} from '../theme/tokens';
import {DrawRect, type NotchSide, type RectCorners} from './DrawRect.opus';
import {Flicker} from './Flicker.opus';
import {cubicInOut, hash, lightStyle, span, useTiming, type Box, type Timing} from './core';

export type BootVariant = 'snap'|'scan'|'glitch';

export type BootInProps = Timing & React.PropsWithChildren<{
  box:Box;
  /**
   * snap — fast frame, one crisp pulse, content on. For small labels and repeated items.
   * scan — frame, then a bright raster line wipes the content down the box. For panels and reveals.
   * glitch — frame, then the content tears in horizontal bands and locks. For a hard cut or a contradiction.
   */
  variant?:BootVariant;
  seed?:number;
  intensity?:number;
  corners?:RectCorners;
  notch?:NotchSide|false;
  strokeToken?:string;
  fillToken?:string;
  glowToken?:string;
}>;

/** Phase windows per variant, as fractions of the run; sounds.ts reads the same table. */
export const BOOT_IN = {
  snap:{frames:22, rect:[0, .55], content:[.45, .85]},
  scan:{frames:42, rect:[0, .42], content:[.4, .88], texture:[.82, 1]},
  glitch:{frames:38, rect:[0, .42], content:[.4, .82]},
} as const;

const BANDS = 7;

/** Runs DrawRect → Flicker → content reveal for any box. Children are positioned relative to the box. */
export function BootIn({box, variant = 'scan', seed = 21, intensity = 1, corners = 'tl-br', notch = false, strokeToken = 'color/line/hairline-strong', fillToken = 'color/surface/1', glowToken = 'color/fill/gold', children, ...timing}:BootInProps) {
  const spec = BOOT_IN[variant];
  const {p, t, d} = useTiming(timing, spec.frames);
  if (p <= 0) return null;
  const rectP = span(p, spec.rect[0], spec.rect[1]);
  const contentP = span(p, spec.content[0], spec.content[1]);
  const contentFrames = (spec.content[1] - spec.content[0])*d;
  const glow = color(glowToken), ink = color(strokeToken);
  const frameBox:React.CSSProperties = {position:'absolute', left:box.x, top:box.y, width:box.w, height:box.h, overflow:'hidden'};
  let content:React.ReactNode = null;
  if (contentP > 0) {
    if (variant === 'snap') {
      content = <Flicker seed={seed} intensity={.4*intensity} progress={contentP} durationFrames={contentFrames}><div style={frameBox}>{children}</div></Flicker>;
    } else if (variant === 'scan') {
      const s = cubicInOut(contentP), edge = box.h*s;
      const texture = 1 - span(p, BOOT_IN.scan.texture[0], BOOT_IN.scan.texture[1]);
      content = <>
        <div style={{...frameBox, clipPath:`inset(0 0 ${(100*(1 - s)).toFixed(3)}% 0)`}}>{children}</div>
        {/* Raster texture over the revealed area, fading once the wipe lands. */}
        {texture > 0 && <div style={{...frameBox, height:edge, opacity:.5*texture, background:`repeating-linear-gradient(180deg, ${color('color/line/graticule')} 0 1px, transparent 1px 4px)`}}/>}
        {s < 1 && <>
          <div style={{position:'absolute', left:box.x, top:box.y + edge - 28, width:box.w, height:28, background:`linear-gradient(180deg, transparent, ${color('color/fill/gold-wash')})`}}/>
          <div style={{position:'absolute', left:box.x, top:box.y + edge - 1, width:box.w, height:2, background:ink, ...lightStyle(1.6, glow)}}/>
        </>}
      </>;
    } else {
      // Bands hold each offset for two frames so the tear reads as deliberate steps, and calm frames sit between tears.
      const step = Math.floor(t/2), amp = 46*intensity*(1 - cubicInOut(contentP));
      const torn = contentP < 1 && hash(seed, 900, step) > .25;
      const level = Math.min(1, contentP*3) * (contentP < 1 && hash(seed, 901, Math.floor(t)) < .18 ? .45 : 1);
      content = !torn
        ? <div style={{...frameBox, opacity:level}}>{children}</div>
        : Array.from({length:BANDS}, (_, i) => {
            const top = 100*i/BANDS, bottom = 100*(1 - (i + 1)/BANDS);
            const dx = hash(seed, i, step) > .55 ? (hash(seed, i + 40, step) - .5)*2*amp : 0;
            return <div key={i} style={{...frameBox, clipPath:`inset(${top.toFixed(2)}% 0 ${bottom.toFixed(2)}% 0)`, transform:`translateX(${dx.toFixed(1)}px)`, opacity:level, overflow:'visible'}}>{children}</div>;
          });
    }
  }
  return <div data-bakeoff="opus/BootIn" data-variant={variant} style={{position:'absolute', inset:0, pointerEvents:'none'}}>
    <DrawRect box={box} corners={corners} notch={notch} seed={seed} intensity={intensity} strokeToken={strokeToken} fillToken={fillToken} glowToken={glowToken} progress={rectP} durationFrames={(spec.rect[1] - spec.rect[0])*d}/>
    {content}
  </div>;
}
