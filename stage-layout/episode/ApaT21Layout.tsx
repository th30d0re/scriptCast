import React from 'react';
import type {SceneProps} from '@stage';
import layout from './layout.json';

const SANS = '"IBM Plex Sans", "Helvetica Neue", Helvetica, Arial, sans-serif';
const MONO = '"IBM Plex Mono", ui-monospace, SFMono-Regular, Menlo, monospace';
const GOLD = 'rgba(224,163,60,1)';
const TEXT = 'rgba(232,201,155,1)';
const MUTED = 'rgba(232,201,155,0.72)';
const PANEL = 'rgba(13,27,42,0.88)';
const HAIRLINE = 'rgba(232,201,155,0.55)';
const FIELD = '#0d1b2a';

type Line = {text: string; x: number; y: number; w: number};
type Box = {x: number; y: number; w: number; h: number; lines?: Line[]};
const boxes = layout as Record<string, Box>;

const textStyle: Record<string, React.CSSProperties> = {
  'definition-term': {fontFamily: SANS, fontWeight: 650, fontSize: 38, lineHeight: 1.25, color: GOLD},
  'definition-body': {fontFamily: SANS, fontWeight: 400, fontSize: 28, lineHeight: 1.34, color: TEXT},
  'definition-source': {fontFamily: MONO, fontWeight: 400, fontSize: 20, lineHeight: 1.3, color: MUTED},
  'definition-paraphrased': {fontFamily: MONO, fontWeight: 400, fontSize: 20, lineHeight: 1.3, color: MUTED},
  'tray-heading': {fontFamily: MONO, fontWeight: 500, fontSize: 20, lineHeight: 1.25, letterSpacing: 1.1, color: GOLD},
  'allocation-label-1': {fontFamily: MONO, fontWeight: 500, fontSize: 19, lineHeight: 1.05, color: TEXT},
  'allocation-label-2': {fontFamily: MONO, fontWeight: 500, fontSize: 19, lineHeight: 1.05, color: TEXT},
  'allocation-label-3': {fontFamily: MONO, fontWeight: 500, fontSize: 19, lineHeight: 1.05, color: TEXT},
  'resource-eight': {fontFamily: MONO, fontWeight: 500, fontSize: 20, lineHeight: 1.25, color: GOLD},
  'resource-four': {fontFamily: MONO, fontWeight: 500, fontSize: 20, lineHeight: 1.25, color: MUTED},
  'conduct-warm-mark': {fontFamily: MONO, fontWeight: 500, fontSize: 28, lineHeight: 1.25, color: TEXT},
  'conduct-cool-mark': {fontFamily: MONO, fontWeight: 500, fontSize: 28, lineHeight: 1.25, color: TEXT},
  'behavior-favorable': {fontFamily: MONO, fontWeight: 400, fontSize: 20, lineHeight: 1.25, color: GOLD},
  'behavior-situational': {fontFamily: MONO, fontWeight: 400, fontSize: 20, lineHeight: 1.25, color: MUTED},
};

const panels = ['definition-card', 'apa-field', 'allocation-tray', 'allocation-panel-1', 'allocation-panel-2', 'allocation-panel-3'];

function frameStyle(id: string, box: Box): React.CSSProperties {
  const base: React.CSSProperties = {position: 'absolute', left: box.x, top: box.y, width: box.w, height: box.h, boxSizing: 'border-box'};
  if (id === 'definition-accent') return {...base, background: GOLD};
  if (id === 'group-boundary') return {...base, borderRadius: '50%', border: `3px solid ${GOLD}`, background: 'rgba(224,163,60,0.2)'};
  if (id === 'conduct-warm') return {...base, border: `3px solid ${GOLD}`};
  if (id === 'conduct-cool') return {...base, border: `1px solid ${MUTED}`};
  return {...base, background: PANEL, border: `1px solid ${HAIRLINE}`};
}

export function ApaT21Layout(_props: SceneProps) {
  const drawn = [...panels, 'definition-accent', 'group-boundary', 'conduct-warm', 'conduct-cool'];
  return <div style={{position: 'absolute', inset: 0, background: FIELD}}>
    {drawn.map(id => boxes[id] ? <div key={id} style={frameStyle(id, boxes[id])} /> : null)}
    {Object.entries(boxes).flatMap(([id, box]) => (box.lines ?? []).map((line, index) => <span key={`${id}-${index}`} style={{position: 'absolute', left: line.x, top: line.y, whiteSpace: 'pre', ...(textStyle[id] ?? {fontFamily: SANS, fontSize: 20, color: TEXT})}}>{line.text}</span>))}
  </div>;
}
