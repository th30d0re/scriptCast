import React from 'react';
import {color} from '../theme/tokens';
import {Label, SANS_ADVANCE, clamp01, shapeAttrs, wrapLineCount, type Box} from './audit';

export type QuestionHeaderProps = {
  /** Absolute frame box of the header. Height 110–140 holds a two-line question at 44–48 px. */
  box:Box;
  /** The spoken question, verbatim. */
  text:string;
  /** Entrance 0–1. The header renders nothing at 0 and fades in with progress. */
  progress:number;
  /** Substring of `text` set in `accent`. Matched case-insensitively on its first occurrence. */
  accentTerm?:string;
  /** Dashboard colour token for the accent term. Defaults to the race-marker red. */
  accent?:string;
  /** Audit id prefix. */
  id?:string;
};

const SIZES = [48, 44, 40, 36, 32, 28] as const, LH = 1.22, BAR_GAP = 26;

/** Largest size in {48, 44, 40, 36, 32, 28} whose wrapped text fits the box, by the layout audit's own estimate. */
export function questionHeaderFit(text:string, box:Box) {
  const textBox:Box = {x:box.x + BAR_GAP, y:box.y, w:box.w - BAR_GAP, h:box.h};
  for (const size of SIZES) {
    const {lines, overflow} = wrapLineCount(text, textBox.w, size, SANS_ADVANCE);
    if (!overflow && lines*size*LH <= textBox.h) return {size, lines, textBox};
  }
  const size = SIZES[SIZES.length - 1];
  return {size, lines:wrapLineCount(text, textBox.w, size, SANS_ADVANCE).lines, textBox};
}

export function QuestionHeader({box, text, progress, accentTerm, accent = 'color/marker/race', id = 'question-header'}:QuestionHeaderProps) {
  const p = clamp01(progress);
  if (p <= 0) return null;
  const {size, textBox} = questionHeaderFit(text, box);
  const at = accentTerm ? text.toLowerCase().indexOf(accentTerm.toLowerCase()) : -1;
  const barBox:Box = {x:box.x, y:box.y + 6, w:5, h:box.h - 12};
  return <div data-shared="QuestionHeader" data-question-size={size} style={{opacity:p}}>
    <div {...shapeAttrs(`${id}-bar`, barBox, 'mark', `${id}-bar`)} style={{position:'absolute', left:barBox.x, top:barBox.y, width:barBox.w, height:barBox.h, background:color('color/fill/gold')}}/>
    <Label id={`${id}-text`} box={textBox} size={size} weight={500} lh={LH} sans valign="middle" ink={color('color/text/primary')}>
      {at < 0 ? text : <>{text.slice(0, at)}<span data-accent-term="true" style={{color:color(accent), fontWeight:650}}>{text.slice(at, at + (accentTerm as string).length)}</span>{text.slice(at + (accentTerm as string).length)}</>}
    </Label>
  </div>;
}
