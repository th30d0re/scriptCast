import React from 'react';
import {color} from '../theme/tokens';
import {Label, Panel, clamp01, monoWidth, shapeAttrs, type Box} from './audit';

export type EvidenceKind = 'fact' | 'documented' | 'experience' | 'contrary' | 'surroundings';

/** Ink, border treatment and side bar per evidence kind. Colours resolve through dashboard tokens. */
export const EVIDENCE_KINDS:Record<EvidenceKind, {ink:string; edge:string; bar?:string; dashed:boolean}> = {
  fact:{ink:'color/text/primary', edge:'color/line/hairline', dashed:false},
  documented:{ink:'color/text/primary', edge:'color/fill/gold', bar:'color/fill/gold', dashed:false},
  experience:{ink:'color/text/soft', edge:'color/text/muted', dashed:true},
  contrary:{ink:'color/text/primary', edge:'color/line/hairline-strong', bar:'color/text/accent', dashed:false},
  surroundings:{ink:'color/text/soft', edge:'color/line/chart-faint', dashed:false},
};

export type EvidenceSocket = {lock:number; side?:'left'|'right'};

export type EvidenceTokenProps = {
  /** Text of the token. An empty string renders a bare chip with no text element. */
  label:string;
  kind:EvidenceKind;
  /** Entrance 0–1. The token renders nothing at 0 and fades in with progress. */
  progress:number;
  /** Brightness and glow 0–1. */
  emphasis:number;
  /** Absolute frame box. Movement along a path comes from the caller interpolating this box. */
  box:Box;
  /** Source socket on the token's edge. `lock` 0 is an open ring and 1 a filled, locked socket. */
  sourceSocket?:EvidenceSocket;
  /** Audit id prefix. Give each token on screen its own id. */
  id?:string;
  /** Label size in px, at least 20. */
  size?:number;
};

const LABEL_LS = .6, PAD = 14, BAR = 4, SOCKET = 28, END_PAD = 12;

/** Smallest single-line token for a label: the width and height that keep the label on one line. */
export function evidenceTokenSize(label:string, opts:{size?:number; socket?:boolean; kind?:EvidenceKind} = {}) {
  const size = opts.size ?? 20, bar = opts.kind && EVIDENCE_KINDS[opts.kind].bar ? BAR : 0;
  const textW = label ? Math.ceil(monoWidth(label, size, LABEL_LS)) : 0;
  return {w:PAD + bar + (opts.socket ? SOCKET : 0) + textW + END_PAD, h:Math.ceil(size*1.2 + 22)};
}

export function EvidenceToken({label, kind, progress, emphasis, box, sourceSocket, id = 'evidence-token', size = 20}:EvidenceTokenProps) {
  const p = clamp01(progress), e = clamp01(emphasis);
  if (p <= 0) return null;
  const style = EVIDENCE_KINDS[kind], bar = style.bar ? BAR : 0;
  const side = sourceSocket?.side ?? 'left';
  const socketLeft = sourceSocket && side === 'left', socketRight = sourceSocket && side === 'right';
  const textX = box.x + PAD + bar + (socketLeft ? SOCKET : 0);
  const textW = box.w - (textX - box.x) - END_PAD - (socketRight ? SOCKET : 0);
  const lock = clamp01(sourceSocket?.lock ?? 0);
  const socketBox:Box|undefined = sourceSocket ? {x:side === 'left' ? box.x + PAD + bar : box.x + box.w - END_PAD - 22, y:box.y + box.h/2 - 11, w:22, h:22} : undefined;
  const glow = e > 0 ? `0 0 ${(18*e).toFixed(1)}px ${color(style.bar ?? 'color/fill/gold')}` : undefined;
  return <div data-shared="EvidenceToken" data-evidence-kind={kind} data-emphasis={e.toFixed(3)} style={{opacity:p*(.62 + .38*e)}}>
    <Panel id={`${id}-frame`} box={box} accent={style.edge} style={{borderStyle:style.dashed ? 'dashed' : 'solid', borderLeft:style.bar ? `${BAR}px solid ${color(style.bar)}` : undefined, boxShadow:glow}}/>
    {socketBox && <div {...shapeAttrs(`${id}-socket`, socketBox, 'mark', `${id}-socket`)} data-socket-lock={lock.toFixed(3)} style={{position:'absolute', left:socketBox.x, top:socketBox.y, width:22, height:22}}>
      <svg width="22" height="22" viewBox="0 0 22 22" aria-hidden="true">
        <circle cx="11" cy="11" r="8" fill={color('color/fill/gold')} fillOpacity={lock} stroke={color('color/fill/gold')} strokeWidth="2.5"/>
        {lock > .6 && <path d="M7 11.5L10 14.5L15.5 8" fill="none" stroke={color('color/bg/field')} strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" opacity={clamp01((lock - .6)/.4)}/>}
      </svg>
    </div>}
    {label !== '' && <Label id={`${id}-label`} box={{x:textX, y:box.y + 5, w:textW, h:box.h - 10}} size={size} weight={500} ls={LABEL_LS} lh={1.2} valign="middle" ink={color(style.ink)}>{label}</Label>}
  </div>;
}
