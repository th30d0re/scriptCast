import React from 'react';
import {color} from '../theme/tokens';
import {DrawRect} from '../motion/DrawRect';

// Layout-audit helpers for the minutes 7–12 shared components. Every audited element declares its
// absolute 1920 × 1080 box with data attributes (see tools/layout_audit.ts), and each helper here
// derives the declaration from the same numbers that position the element.

export type Box = {x:number; y:number; w:number; h:number};
export type Point = readonly [number, number];

export const clamp01 = (value:number) => Number.isFinite(value) ? Math.max(0, Math.min(1, value)) : 0;
const round = (value:number) => Math.round(value*10)/10;
export const boxData = (box:Box) => `${round(box.x)} ${round(box.y)} ${round(box.w)} ${round(box.h)}`;

export const textAttrs = (id:string, box:Box, fit?:{size:number; ls?:number; lh?:number; sans?:boolean}) => ({
  'data-text':id, 'data-box':boxData(box),
  ...(fit ? {'data-fit':`${fit.sans ? 'sans' : 'mono'} ${fit.size} ${fit.ls ?? 0} ${fit.lh ?? 1.25}`} : {}),
});
export const shapeAttrs = (id:string, box:Box, role:'frame'|'mark' = 'mark', layer?:string) => ({
  'data-shape':id, 'data-role':role, 'data-box':boxData(box), ...(layer ? {'data-layer':layer} : {}),
});
export const lineAttrs = (id:string, points:readonly Point[], pad = 2) => {
  const xs = points.map(([x]) => x), ys = points.map(([, y]) => y);
  const x = Math.min(...xs), y = Math.min(...ys), w = Math.max(...xs) - x, h = Math.max(...ys) - y;
  return {
    'data-shape':id, 'data-role':'line', 'data-box':boxData({x:x-pad, y:y-pad, w:w+2*pad, h:h+2*pad}),
    'data-points':points.map(([px, py]) => `${round(px)},${round(py)}`).join(' '),
  };
};
export const pathFrom = (points:readonly Point[]) => points.map(([x, y], index) => `${index ? 'L' : 'M'}${round(x)} ${round(y)}`).join(' ');

/** IBM Plex Mono advances every glyph by 0.6 em; the audit estimates IBM Plex Sans at 0.56 em. */
export const MONO_ADVANCE = .6;
export const SANS_ADVANCE = .56;
/** Width of a single mono line, letter spacing included. */
export const monoWidth = (text:string, size:number, letterSpacing = 0) => [...text].length*(MONO_ADVANCE*size + letterSpacing);

/** Greedy word wrap that mirrors the layout audit's fit estimate. `advance` is the glyph advance in em. */
export function wrapLineCount(text:string, width:number, size:number, advance:number, letterSpacing = 0) {
  const step = advance*size + letterSpacing, maxChars = Math.max(1, Math.floor((width + 1)/step));
  let lines = 1, current = 0, overflow = false;
  for (const word of text.split(' ').filter(Boolean)) {
    const length = [...word].length;
    if (length > maxChars) overflow = true;
    if (current === 0) current = length;
    else if (current + 1 + length <= maxChars) current += 1 + length;
    else { lines += 1; current = length; }
  }
  return {lines, overflow};
}

/** The first `p` of a polyline's length, so a stroke can draw itself and its audit points follow. */
export function partialPolyline(points:readonly Point[], p:number):Point[] {
  const t = clamp01(p);
  if (points.length < 2) return [...points];
  const lengths:number[] = [];
  let total = 0;
  for (let index = 1; index < points.length; index++) {
    const length = Math.hypot(points[index][0] - points[index-1][0], points[index][1] - points[index-1][1]);
    lengths.push(length); total += length;
  }
  let remaining = total*t;
  const out:Point[] = [points[0]];
  for (let index = 1; index < points.length; index++) {
    const length = lengths[index-1];
    if (remaining >= length - 1e-9) { out.push(points[index]); remaining -= length; continue; }
    const f = length > 0 ? remaining/length : 0;
    out.push([points[index-1][0] + (points[index][0] - points[index-1][0])*f, points[index-1][1] + (points[index][1] - points[index-1][1])*f]);
    break;
  }
  return out;
}

/** Arrowhead barbs for a stroke ending at `end` and travelling along the unit vector (ux, uy). */
export function arrowBarbs(end:Point, ux:number, uy:number, length = 15, spread = 9):[Point, Point, Point] {
  const nx = -uy, ny = ux;
  return [[end[0] - ux*length + nx*spread, end[1] - uy*length + ny*spread], end, [end[0] - ux*length - nx*spread, end[1] - uy*length - ny*spread]];
}

export function Label({id, box, children, size = 20, weight = 500, ls = 0, lh = 1.25, sans = false, align = 'left', valign = 'top', ink, opacity = 1, style}:{
  id:string; box:Box; children:React.ReactNode; size?:number; weight?:number; ls?:number; lh?:number; sans?:boolean;
  align?:'left'|'center'|'right'; valign?:'top'|'middle'; ink?:string; opacity?:number; style?:React.CSSProperties;
}) {
  return <div {...textAttrs(id, box, {size, ls, lh, sans})} style={{position:'absolute', left:box.x, top:box.y, width:box.w, height:box.h, boxSizing:'border-box', font:`${weight} ${size}px "${sans ? 'IBM Plex Sans' : 'IBM Plex Mono'}"`, letterSpacing:ls, lineHeight:lh, textAlign:align, color:ink ?? color('color/text/primary'), opacity, ...style}}>
    {valign === 'middle' ? <div style={{position:'absolute', left:0, right:0, top:'50%', transform:'translateY(-50%)'}}>{children}</div> : children}
  </div>;
}

export function Panel({id, box, opacity = 1, accent = 'color/line/hairline', fillToken = 'color/fill/panel-render', startMs, durationMs = 420, style, children}:{
  id:string; box:Box; opacity?:number; accent?:string; fillToken?:string; startMs?:number; durationMs?:number; style?:React.CSSProperties; children?:React.ReactNode;
}) {
  const {background, backgroundColor, border, borderBottom, borderBottomColor, borderBottomStyle, borderBottomWidth, borderColor, borderLeft, borderLeftColor, borderLeftStyle, borderLeftWidth, borderRight, borderRightColor, borderRightStyle, borderRightWidth, borderStyle, borderTop, borderTopColor, borderTopStyle, borderTopWidth, borderWidth, ...contentStyle} = style ?? {};
  const timing = startMs === undefined
    ? {progress:1 as const}
    : {startFrame:Math.round(startMs*30/1000), durationFrames:Math.round(durationMs*30/1000)};
  return <div {...shapeAttrs(id, box, 'frame')} style={{position:'absolute', left:box.x, top:box.y, width:box.w, height:box.h, opacity}}>
    <DrawRect box={box} strokeToken={accent} fillToken={fillToken} {...timing}/>
    <div style={{position:'absolute', inset:0, boxSizing:'border-box', overflow:'hidden', ...contentStyle}}>{children}</div>
  </div>;
}
