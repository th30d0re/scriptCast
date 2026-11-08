import React from 'react';
import {color} from '../theme/tokens';
import {DrawRect, type NotchSide, type RectCorners} from './DrawRect.opus';
import {Flicker} from './Flicker.opus';
import {hash, spacing, span, useTiming, type Box, type Timing} from './core';
import {cssFont, useFontsReady, wrapText, type FontSpec} from './measure';

type SpacingToken = 'spacing/xs'|'spacing/sm'|'spacing/md'|'spacing/lg'|'spacing/xl';

export type FitBoxProps = Timing & {
  text:string;
  /** Anchor point in scene coordinates; see `anchor`. */
  x:number;
  y:number;
  anchor?:'top-left'|'top-center'|'center';
  /** Text wraps at this width (text area, not box). Omit for a single line. */
  maxWidth?:number;
  size?:number;
  family?:'sans'|'mono';
  weight?:number;
  lineHeight?:number;
  letterSpacing?:number;
  padX?:SpacingToken;
  padY?:SpacingToken;
  textToken?:string;
  strokeToken?:string;
  fillToken?:string;
  /** Cursor and newest-character colour. */
  accentToken?:string;
  /** A short mono tag set into the top-left edge, e.g. "SYS·01". */
  tag?:string;
  corners?:RectCorners;
  notch?:NotchSide|false;
  seed?:number;
  intensity?:number;
};

export const FIT_BOX_PHASES = {rect:[0, .62], tag:[.5, .7], text:[.52, .92], cursorOut:[.92, 1]} as const;
export const FIT_BOX_DEFAULT_FRAMES = 48;

/** Measured layout for a FitBox. Valid only once the kit fonts have loaded (FitBox waits for them). */
export function fitBoxLayout(props:Pick<FitBoxProps, 'text'|'x'|'y'|'anchor'|'maxWidth'|'size'|'family'|'weight'|'lineHeight'|'letterSpacing'|'padX'|'padY'>) {
  const {text, x, y, anchor = 'top-left', maxWidth = Infinity, size = 32, family = 'sans', weight = 500, lineHeight = 1.25, letterSpacing = 0, padX = 'spacing/md', padY = 'spacing/sm'} = props;
  const font:FontSpec = {family, size, weight};
  const {lines, width} = wrapText(text, font, maxWidth, letterSpacing);
  const px = spacing(padX), py = spacing(padY), lh = size*lineHeight;
  const w = Math.ceil(width) + 2*px, h = Math.ceil(lines.length*lh) + 2*py;
  const bx = anchor === 'top-left' ? x : x - w/2, by = anchor === 'center' ? y - h/2 : y;
  return {box:{x:Math.round(bx), y:Math.round(by), w, h} as Box, lines, font, px, py, lh};
}

/** A bordered text box that sizes itself to its measured text, draws its frame, flickers its fill on, then types its text. */
export function FitBox(props:FitBoxProps) {
  const {text, family = 'sans', size = 32, weight = 500, letterSpacing = 0, textToken = 'color/text/primary', strokeToken = 'color/line/hairline-strong', fillToken = 'color/surface/1', accentToken = 'color/fill/gold', tag, corners = 'tl-br', notch = false, seed = 11, intensity = 1, progress, startFrame, durationFrames} = props;
  const fontsReady = useFontsReady([{family, size, weight}, {family:'mono', size:15, weight:500}]);
  const {p, t, d} = useTiming({progress, startFrame, durationFrames}, FIT_BOX_DEFAULT_FRAMES);
  if (!fontsReady || p <= 0) return null;
  const {box, lines, font, px, py, lh} = fitBoxLayout(props);
  const P = FIT_BOX_PHASES;
  const typed = span(p, P.text[0], P.text[1]);
  const cursorOut = span(p, P.cursorOut[0], P.cursorOut[1]);
  const chars = lines.reduce((n, l) => n + [...l].length, 0);
  const shown = Math.floor(typed*chars + 1e-6);
  const frame = Math.floor(t);
  const accent = color(accentToken), ink = color(textToken);
  // Block cursor: solid while typing, then two seeded blinks and out.
  const cursorOn = typed > 0 && cursorOut < 1 && (typed < 1 || hash(seed, 77, Math.floor(cursorOut*6)) > .35);
  let index = 0;
  let cursorDrawn = false;
  return <div data-bakeoff="opus/FitBox" style={{position:'absolute', inset:0, pointerEvents:'none'}}>
    <DrawRect box={box} corners={corners} notch={notch} seed={seed} intensity={intensity} strokeToken={strokeToken} fillToken={fillToken} glowToken={accentToken} progress={span(p, P.rect[0], P.rect[1])} durationFrames={(P.rect[1] - P.rect[0])*d}/>
    {tag && <Flicker seed={seed + 3} intensity={intensity} progress={span(p, P.tag[0], P.tag[1])} durationFrames={(P.tag[1] - P.tag[0])*d}>
      <div style={{position:'absolute', left:box.x + 14, top:box.y - 10, height:20, padding:'0 6px', background:color('color/bg/field'), font:cssFont({family:'mono', size:15, weight:500}), lineHeight:'20px', letterSpacing:1.2, color:accent}}>{tag}</div>
    </Flicker>}
    {typed > 0 && lines.map((line, row) => {
      const glyphs = [...line];
      return <div key={row} style={{position:'absolute', left:box.x + px, top:box.y + py + row*lh, height:lh, whiteSpace:'pre', font:cssFont(font), lineHeight:`${lh}px`, letterSpacing, color:ink}}>
        {glyphs.map((g, i) => {
          const k = index++;
          const age = shown - 1 - k; // 0 = newest character
          const visible = k < shown;
          const cursorHere = !cursorDrawn && cursorOn && (k === shown || (typed >= 1 && k === chars - 1));
          if (cursorHere) cursorDrawn = true;
          // The newest two characters burn in accent and catch on one seeded frame; older ones are steady text.
          const style:React.CSSProperties = !visible ? {color:'transparent'} : age <= 1 && typed < 1 ? {color:accent, opacity:hash(seed, k, frame) > .2 ? 1 : .35} : {};
          return <span key={i} style={{position:'relative', ...style}}>{g}{cursorHere && <Cursor after={k < shown} size={size} lh={lh} accent={accent}/>}</span>;
        })}
      </div>;
    })}
  </div>;
}

function Cursor({after, size, lh, accent}:{after:boolean; size:number; lh:number; accent:string}) {
  const w = Math.max(3, Math.round(size*.5)), h = Math.round(size*1.02);
  return <span style={{position:'absolute', left:after ? '100%' : 0, marginLeft:after ? 3 : 0, top:(lh - h)/2, width:w, height:h, background:accent, opacity:.9}}/>;
}
