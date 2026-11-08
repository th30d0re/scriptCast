// Text measurement with the kit's own faces. Measuring before the fonts load would size boxes to a fallback face,
// so the hook holds the render (delayRender) until every requested face is ready, then measures with canvas.
import {useEffect, useState} from 'react';
import {continueRender, delayRender} from 'remotion';
import '../theme/fonts';

export type FontSpec = {family:'sans'|'mono'; size:number; weight?:number; italic?:boolean};

const FAMILY = {sans:'IBM Plex Sans', mono:'IBM Plex Mono'} as const;
export const cssFont = ({family, size, weight = 500, italic}:FontSpec) => `${italic ? 'italic ' : ''}${weight} ${size}px "${FAMILY[family]}"`;

const ready = new Set<string>();
let ctx:CanvasRenderingContext2D | null = null;

export function measureWidth(text:string, font:FontSpec, letterSpacing = 0) {
  ctx ??= document.createElement('canvas').getContext('2d');
  if (!ctx) throw new Error('measureWidth: no 2D canvas context');
  ctx.font = cssFont(font);
  return ctx.measureText(text).width + letterSpacing*Math.max(0, [...text].length - 1);
}

/** Greedy word wrap against measured widths. Words longer than the line are left whole rather than split mid-word. */
export function wrapText(text:string, font:FontSpec, maxWidth:number, letterSpacing = 0) {
  const lines:string[] = [];
  for (const paragraph of text.split('\n')) {
    let line = '';
    for (const word of paragraph.split(/\s+/).filter(Boolean)) {
      const next = line ? `${line} ${word}` : word;
      if (line && measureWidth(next, font, letterSpacing) > maxWidth) { lines.push(line); line = word; }
      else line = next;
    }
    lines.push(line);
  }
  const widths = lines.map(l => measureWidth(l, font, letterSpacing));
  return {lines, widths, width:Math.max(0, ...widths)};
}

/** True once the given faces are loaded. Blocks the Remotion frame capture until then. */
export function useFontsReady(fonts:FontSpec[]) {
  const key = fonts.map(cssFont).join('|');
  const [ok, setOk] = useState(() => fonts.every(f => ready.has(cssFont(f))));
  useEffect(() => {
    if (ok) return;
    const handle = delayRender(`opus FitBox fonts: ${key}`);
    let live = true;
    Promise.all(fonts.map(f => document.fonts.load(cssFont(f), 'Ag'))).then(() => {
      fonts.forEach(f => ready.add(cssFont(f)));
      if (live) setOk(true);
      continueRender(handle);
    }, error => { continueRender(handle); throw error; });
    return () => { live = false; };
    // fonts is described completely by key.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, ok]);
  return ok;
}
