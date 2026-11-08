import React from 'react';
import {color} from '../theme/tokens';
import {Label, Panel, arrowBarbs, clamp01, lineAttrs, partialPolyline, pathFrom, shapeAttrs, type Box, type Point} from './audit';

export type LaneDirection = 'right' | 'left' | 'down' | 'up';

export type LaneToken = {
  id:string;
  /** Position along the rail, 0 at the start and 1 at the arrowhead. */
  at:number;
  /** Brightness 0–1. Defaults to 1. */
  emphasis?:number;
  /** Dashboard colour token overriding the lane accent for this token. */
  accent?:string;
  /** `dot` is an 18 px disc. `card` is a 34 × 22 chip. Defaults to `dot`. */
  shape?:'dot'|'card';
};

export type SignalLaneProps = {
  /** Absolute frame box of the whole lane panel. Horizontal lanes need h ≥ 88 with a label, vertical lanes w ≥ 60. */
  box:Box;
  /** Lane heading, set in a 28 px band at the top of the box. An empty string removes the band. */
  label:string;
  /** 0–1. The panel and label fade in over the first 35%; the rail draws over progress 0.2–1. Nothing renders at 0. */
  progress:number;
  /** Dashboard colour token for the rail, arrowhead and tokens. */
  accent:string;
  direction:LaneDirection;
  tokens:readonly LaneToken[];
  dashed?:boolean;
  /** Draws no panel frame, leaving only label, rail and tokens. */
  bare?:boolean;
  /** Audit id prefix. Give each lane on screen its own id. */
  id?:string;
};

const LABEL_BAND = 48, EDGE = 24;

/** Rail endpoints of a lane. Packages place overlays, gates and token paths from the same numbers. */
export function laneGeometry(box:Box, direction:LaneDirection, hasLabel = true):{start:Point; end:Point} {
  const top = box.y + (hasLabel ? LABEL_BAND : 16), bottom = box.y + box.h - 20, left = box.x + EDGE, right = box.x + box.w - EDGE;
  const midY = (top + bottom)/2, midX = box.x + box.w/2;
  if (direction === 'right') return {start:[left, midY], end:[right, midY]};
  if (direction === 'left') return {start:[right, midY], end:[left, midY]};
  const vTop = top + 8, vBottom = bottom - 4;
  return direction === 'down' ? {start:[midX, vTop], end:[midX, vBottom]} : {start:[midX, vBottom], end:[midX, vTop]};
}

/** Point at fraction `t` of the way along a lane's rail. */
export const lanePoint = (box:Box, direction:LaneDirection, t:number, hasLabel = true):Point => {
  const {start, end} = laneGeometry(box, direction, hasLabel), f = clamp01(t);
  return [start[0] + (end[0] - start[0])*f, start[1] + (end[1] - start[1])*f];
};

export function SignalLane({box, label, progress, accent, direction, tokens, dashed = false, bare = false, id = 'signal-lane'}:SignalLaneProps) {
  const p = clamp01(progress);
  if (p <= 0) return null;
  const hasLabel = label !== '', panelP = clamp01(p/.35), railP = clamp01((p - .2)/.8);
  const {start, end} = laneGeometry(box, direction, hasLabel);
  const dx = end[0] - start[0], dy = end[1] - start[1], length = Math.hypot(dx, dy), ux = dx/length, uy = dy/length;
  const drawn = partialPolyline([start, end], railP);
  const ink = color(accent);
  return <div data-shared="SignalLane" data-lane-direction={direction} data-lane-progress={p.toFixed(3)} data-lane-dashed={dashed ? 'true' : 'false'}>
    <div style={{opacity:panelP}}>
      {!bare && <Panel id={`${id}-frame`} box={box}/>}
      {hasLabel && <Label id={`${id}-label`} box={{x:box.x + 16, y:box.y + 10, w:box.w - 32, h:28}} size={20} weight={500} ls={1.2} lh={1.25} ink={color('color/text/soft')}>{label}</Label>}
    </div>
    <svg width="1920" height="1080" viewBox="0 0 1920 1080" style={{position:'absolute', inset:0}} aria-hidden="true">
      {drawn.length >= 2 && railP > 0 && <path {...lineAttrs(`${id}-rail`, drawn)} d={pathFrom(drawn)} fill="none" stroke={ink} strokeWidth="4" strokeLinecap="round" strokeDasharray={dashed ? '12 9' : undefined} opacity={panelP}/>}
      {railP >= .9 && (() => {
        const barbs = arrowBarbs(end, ux, uy);
        return <path {...lineAttrs(`${id}-arrow`, barbs)} d={pathFrom(barbs)} fill="none" stroke={ink} strokeWidth="4" strokeLinecap="round" strokeLinejoin="round" opacity={clamp01((railP - .9)/.1)}/>;
      })()}
    </svg>
    {tokens.map(token => {
      const w = token.shape === 'card' ? 34 : 18, h = token.shape === 'card' ? 22 : 18;
      const [cx, cy] = [start[0] + dx*clamp01(token.at), start[1] + dy*clamp01(token.at)];
      const tokenBox:Box = {x:cx - w/2, y:cy - h/2, w, h}, e = clamp01(token.emphasis ?? 1);
      return <div key={token.id} {...shapeAttrs(`${id}-token-${token.id}`, tokenBox, 'mark', `${id}-token-${token.id}`)} data-lane-token={token.id} style={{position:'absolute', left:tokenBox.x, top:tokenBox.y, width:w, height:h, boxSizing:'border-box', borderRadius:token.shape === 'card' ? 4 : h/2, background:color(token.accent ?? accent), border:`2px solid ${color('color/bg/field')}`, opacity:(.35 + .65*e)*panelP}}/>;
    })}
  </div>;
}
