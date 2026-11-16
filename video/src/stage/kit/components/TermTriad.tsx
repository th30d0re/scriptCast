import React from 'react';
import {color} from '../theme/tokens';
import {FitBox} from '../motion/FitBox';
import {PersonMark} from '../primitives/PersonMark';
import type {TermState} from './types';
import {Label, MONO_ADVANCE, Panel, clamp01, shapeAttrs, wrapLineCount, type Box} from './audit';

export type TriadKind = 'bias' | 'stereotype' | 'prejudice' | 'discrimination';

/** Established per-term accent and glyph: Bias gold with the person glyph, Stereotype cream with the belief grid, Prejudice red with the heart, Discrimination orange with the action arrow. */
export const TRIAD_KINDS:Record<TriadKind, {accent:string}> = {
  bias:{accent:'color/fill/gold'},
  stereotype:{accent:'color/text/primary'},
  prejudice:{accent:'color/marker/race'},
  discrimination:{accent:'color/text/accent'},
};

export type TriadLine = {text:string; progress:number; tone?:'accent'|'soft'};
export type TriadTerm = {kind:TriadKind; title:string; lines?:readonly TriadLine[]};
export type TriadSource = {text:string; progress:number} | null | undefined;

export type TermTriadProps = {
  /** Absolute frame boxes, one per term, in reading order. */
  boxes:readonly Box[];
  terms:readonly TriadTerm[];
  /** Per block: `hidden`, `dim`, `active`, `selected`, or an entrance progress 0–1. A hidden block renders nothing. */
  states:readonly TermState[];
  /** Source tag per block, set in a 20 px mono line at the block's bottom edge. */
  sources?:readonly TriadSource[];
  /** Entrance 0–1 for all blocks. */
  progress:number;
  /** Glow 0–1 per block, for a spoken-word pulse. */
  pulses?:readonly number[];
  /** Dashed empty sockets at each block box, fading as the block enters. 0–1, default 0. */
  socketProgress?:number;
  /** `row` puts the glyph beside the title, `stack` puts it above. `auto` picks `row` from 420 px wide. */
  layout?:'auto'|'row'|'stack';
  /** Audit id prefix. */
  id?:string;
};

const TITLE_SIZE = 30, TITLE_LH = 36, LINE_SIZE = 24, LINE_PITCH = 36, GLYPH = 52, CAPS_ADVANCE = .68;
const stateAmount = (state:TermState) => typeof state === 'number' ? clamp01(state) : state === 'hidden' ? 0 : state === 'dim' ? .42 : state === 'active' ? .82 : 1;

/** Interior layout of one block: glyph, title, detail lines and source box, in absolute frame coordinates. */
export function triadBlockLayout(box:Box, term:TriadTerm, layout:'auto'|'row'|'stack' = 'auto', sourceText?:string) {
  const mode = layout === 'auto' ? (box.w >= 420 ? 'row' : 'stack') : layout;
  const glyph:Box = {x:box.x + (mode === 'row' ? 20 : 22), y:box.y + (mode === 'row' ? 16 : 0), w:GLYPH, h:GLYPH};
  const titleX = mode === 'row' ? box.x + 86 : box.x + 22;
  const titleW = mode === 'row' ? box.w - 86 - 18 : box.w - 44;
  const titleLines = wrapLineCount(term.title, titleW, TITLE_SIZE, CAPS_ADVANCE).lines;
  const titleY = mode === 'row' ? box.y + 18 : glyph.y + GLYPH + 24; // room for the FitBox tag above the title
  const title:Box = {x:titleX, y:titleY, w:titleW, h:titleLines*TITLE_LH};
  const linesTop = Math.max(title.y + title.h, mode === 'row' ? glyph.y + GLYPH : 0) + 26; // clears the FitBox padding around the title
  const lines:Box[] = (term.lines ?? []).map((_, index) => ({x:box.x + 22, y:linesTop + index*LINE_PITCH, w:box.w - 44, h:30}));
  // The source tag wraps at the block's inner width and grows upward from 16 px above the bottom edge.
  const sourceLines = sourceText ? wrapLineCount(sourceText, box.w - 44, 20, MONO_ADVANCE, .4).lines : 1;
  const sourceH = sourceLines*25 + 1;
  const source:Box = {x:box.x + 22, y:box.y + box.h - 16 - sourceH, w:box.w - 44, h:sourceH};
  return {mode, glyph, title, lines, source};
}

function Glyph({kind, box, ink}:{kind:TriadKind; box:Box; ink:string}) {
  if (kind === 'bias') return <>
    <PersonMark x={box.x + box.w/2} y={box.y + box.h/2} group="ingroup" emphasis={1} scale={.85}/>
    <i {...shapeAttrs('glyph-bias', {x:box.x + box.w/2 - 18.7, y:box.y + box.h/2 - 24.31, w:37.4, h:48.62}, 'mark', 'glyph')}/>
  </>;
  return <div {...shapeAttrs(`glyph-${kind}`, box, 'mark', 'glyph')} data-triad-glyph={kind} style={{position:'absolute', left:box.x, top:box.y, width:box.w, height:box.h, color:ink}}>
    <svg width={box.w} height={box.h} viewBox="0 0 48 48" aria-hidden="true">
      {kind === 'stereotype' && [0, 1, 2].flatMap(row => [0, 1, 2].map(col => <rect key={`${row}${col}`} x={7 + col*13} y={7 + row*13} width="9" height="9" rx="1.5" fill="none" stroke="currentColor" strokeWidth="2.5"/>))}
      {kind === 'prejudice' && <path d="M24 40 C18 34 7 27 7 17 C7 7 19 5 24 13 C29 5 41 7 41 17 C41 27 30 34 24 40 Z" fill="none" stroke="currentColor" strokeWidth="2.5"/>}
      {kind === 'discrimination' && <path d="M7 24 H38 M28 13 L39 24 L28 35" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"/>}
    </svg>
  </div>;
}

export function TermTriad({boxes, terms, states, sources, progress, pulses, socketProgress = 0, layout = 'auto', id = 'term-triad'}:TermTriadProps) {
  const count = Math.min(boxes.length, terms.length, states.length), revealP = clamp01(progress), socketP = clamp01(socketProgress);
  return <div data-shared="TermTriad" data-triad-count={count}>
    {Array.from({length:count}, (_, index) => {
      const box = boxes[index], term = terms[index], state = states[index], amount = stateAmount(state), selected = state === 'selected';
      // State is visual emphasis only. It must never leave FitBox's typed title at a partial word.
      const reveal = revealP, opacity = amount*reveal;
      const accent = TRIAD_KINDS[term.kind].accent, ink = color(accent), geometry = triadBlockLayout(box, term, layout, sources?.[index]?.text);
      const source = sources?.[index], pulse = clamp01(pulses?.[index] ?? 0);
      return <div key={`${id}-${index}`} data-triad-term={term.kind} data-triad-state={String(state)}>
        {socketP > 0 && amount < 1 && <Panel id={`${id}-socket-${index + 1}`} box={box} opacity={socketP*(1 - amount)} accent="color/line/hairline-strong" style={{borderStyle:'dashed', background:'transparent'}}/>}
        {opacity > 0 && <div style={{opacity}}>
          <FitBox text={term.title} x={geometry.title.x - 16} y={geometry.title.y - 8} maxWidth={geometry.title.w} size={TITLE_SIZE} weight={600} lineHeight={1.15} padX="spacing/md" padY="spacing/sm" textToken="color/text/primary" strokeToken={selected ? accent : 'color/line/hairline'} fillToken="color/fill/panel-render" accentToken={accent} tag={term.kind.toUpperCase()} corners={selected ? 'tr-bl' : 'tl-br'} seed={101 + index} intensity={Math.max(.35, pulse)} progress={reveal}/>
          <Glyph kind={term.kind} box={geometry.glyph} ink={ink}/>
          {(term.lines ?? []).map((line, lineIndex) => line.progress > 0 && <Label key={lineIndex} id={`${id}-line-${index + 1}-${lineIndex + 1}`} box={geometry.lines[lineIndex]} size={LINE_SIZE} weight={500} ls={1.4} lh={1.25} ink={line.tone === 'accent' ? ink : color('color/text/soft')} opacity={clamp01(line.progress)}>{line.text}</Label>)}
          {source && source.progress > 0 && <Label id={`${id}-source-${index + 1}`} box={geometry.source} size={20} weight={500} ls={.4} lh={1.25} ink={color('color/text/muted')} opacity={clamp01(source.progress)}>{source.text}</Label>}
        </div>}
      </div>;
    })}
  </div>;
}
