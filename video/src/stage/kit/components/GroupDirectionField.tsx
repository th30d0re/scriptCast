import React from 'react';
import {color} from '../theme/tokens';
import {PersonMark} from '../primitives/PersonMark';
import type {PersonGroup} from './types';
import {Label, Panel, arrowBarbs, clamp01, lineAttrs, partialPolyline, pathFrom, shapeAttrs, type Box, type Point} from './audit';

export type GroupMarkSpec = {
  id:string;
  side:'left'|'right';
  /** Normalized position inside the group boundary's mark region, 0–1. */
  x:number;
  y:number;
  /** Colour group of the person mark. Defaults to `neutral`. */
  accent?:PersonGroup;
  emphasis?:number;
  scale?:number;
  /** Entrance 0–1. The mark renders nothing at 0. Defaults to 1. */
  progress?:number;
};

export type GroupDirectionFieldProps = {
  /** Absolute frame box of the field. Both group boundaries and both arrows stay inside it. */
  box:Box;
  marks:readonly GroupMarkSpec[];
  leftLabel:string;
  rightLabel:string;
  /** Group A → Group B arrow draw 0–1. */
  forwardProgress:number;
  /** Group B → Group A arrow draw 0–1, timed independently of the forward arrow. */
  reverseProgress:number;
  /** Captions at the bottom of each boundary, with their own entrance progress (default 1). */
  insideLabels?:{left?:string; right?:string; progress?:number};
  /** Group boundaries and group labels entrance 0–1. Defaults to 1. Nothing renders when it is 0 and both arrows are 0. */
  boundaryProgress?:number;
  /** Width of the arrow gutter between the boundaries. Defaults to clamp(box.w × 0.2, 120, 220). */
  gutter?:number;
  /** Audit id prefix. */
  id?:string;
};

const ARROW_WEIGHT = 5;

/** Boundaries and arrow endpoints of a field. Packages place overlays from the same numbers. */
export function groupFieldGeometry(box:Box, gutter?:number) {
  const g = gutter ?? Math.max(120, Math.min(220, box.w*.2)), bw = (box.w - g)/2;
  const left:Box = {x:box.x, y:box.y, w:bw, h:box.h}, right:Box = {x:box.x + box.w - bw, y:box.y, w:bw, h:box.h};
  const midY = box.y + box.h/2;
  const forward:[Point, Point] = [[left.x + left.w + 8, midY - 28], [right.x - 8, midY - 28]];
  const reverse:[Point, Point] = [[right.x - 8, midY + 28], [left.x + left.w + 8, midY + 28]];
  return {left, right, forward, reverse};
}

/** Centre of a mark, and the box its glyph occupies, inside the field geometry. */
export function groupMarkPlacement(boundary:Box, mark:GroupMarkSpec, hasInsideLabel:boolean) {
  const scale = Math.max(.2, mark.scale ?? 1), half = 28.6*scale, halfW = 22*scale;
  const loY = boundary.y + 54 + half, hiY = boundary.y + boundary.h - (hasInsideLabel ? 56 : 16) - half;
  const loX = boundary.x + 14 + halfW, hiX = boundary.x + boundary.w - 14 - halfW;
  const cx = loX + clamp01(mark.x)*(hiX - loX), cy = loY + clamp01(mark.y)*Math.max(0, hiY - loY);
  return {cx, cy, box:{x:cx - halfW, y:cy - half, w:halfW*2, h:half*2} as Box};
}

function Arrow({id, points, progress}:{id:string; points:readonly [Point, Point]; progress:number}) {
  const p = clamp01(progress);
  if (p <= 0) return null;
  const drawn = partialPolyline(points, p), [a, b] = points, length = Math.hypot(b[0] - a[0], b[1] - a[1]);
  const barbs = arrowBarbs(b, (b[0] - a[0])/length, (b[1] - a[1])/length, 18, 11);
  return <>
    {drawn.length >= 2 && <path {...lineAttrs(id, drawn)} d={pathFrom(drawn)} fill="none" stroke={color('color/fill/gold')} strokeWidth={ARROW_WEIGHT} strokeLinecap="round"/>}
    {p >= .92 && <path {...lineAttrs(`${id}-head`, barbs)} d={pathFrom(barbs)} fill="none" stroke={color('color/fill/gold')} strokeWidth={ARROW_WEIGHT} strokeLinecap="round" strokeLinejoin="round" opacity={clamp01((p - .92)/.08)}/>}
  </>;
}

export function GroupDirectionField({box, marks, leftLabel, rightLabel, forwardProgress, reverseProgress, insideLabels, boundaryProgress = 1, gutter, id = 'group-direction'}:GroupDirectionFieldProps) {
  const boundaryP = clamp01(boundaryProgress), forwardP = clamp01(forwardProgress), reverseP = clamp01(reverseProgress);
  if (boundaryP <= 0 && forwardP <= 0 && reverseP <= 0) return null;
  const {left, right, forward, reverse} = groupFieldGeometry(box, gutter);
  const insideP = clamp01(insideLabels?.progress ?? 1);
  const boundaries = [
    {side:'left' as const, box:left, label:leftLabel, caption:insideLabels?.left, wash:'color/fill/gold-wash', ink:'color/fill/gold'},
    {side:'right' as const, box:right, label:rightLabel, caption:insideLabels?.right, wash:'color/fill/panel-wash', ink:'color/text/primary'},
  ];
  return <div data-shared="GroupDirectionField" data-forward-progress={forwardP.toFixed(3)} data-reverse-progress={reverseP.toFixed(3)}>
    {boundaryP > 0 && boundaries.map(boundary => <div key={boundary.side} data-group-boundary={boundary.side} style={{opacity:boundaryP}}>
      <Panel id={`${id}-boundary-${boundary.side}`} box={boundary.box} accent="color/line/hairline-strong" style={{borderRadius:20, borderWidth:2, background:color(boundary.wash)}}/>
      <Label id={`${id}-label-${boundary.side}`} box={{x:boundary.box.x + 18, y:boundary.box.y + 12, w:boundary.box.w - 36, h:30}} size={22} weight={500} ls={1.4} lh={1.25} ink={color(boundary.ink)}>{boundary.label}</Label>
      {boundary.caption && insideP > 0 && <Label id={`${id}-caption-${boundary.side}`} box={{x:boundary.box.x + 18, y:boundary.box.y + boundary.box.h - 44, w:boundary.box.w - 36, h:28}} size={20} weight={500} ls={.6} lh={1.25} ink={color('color/text/soft')} opacity={insideP}>{boundary.caption}</Label>}
    </div>)}
    {boundaryP > 0 && marks.map(mark => {
      const entering = clamp01(mark.progress ?? 1);
      if (entering <= 0) return null;
      const boundary = mark.side === 'left' ? left : right, hasCaption = Boolean(mark.side === 'left' ? insideLabels?.left : insideLabels?.right);
      const place = groupMarkPlacement(boundary, mark, hasCaption);
      return <div key={mark.id} data-group-mark={mark.id} style={{opacity:entering*boundaryP}}>
        <PersonMark x={place.cx} y={place.cy} group={mark.accent ?? 'neutral'} emphasis={mark.emphasis ?? .8} scale={mark.scale ?? 1}/>
        <i {...shapeAttrs(`${id}-mark-${mark.id}`, place.box, 'mark', `${id}-mark-${mark.id}`)}/>
      </div>;
    })}
    <svg width="1920" height="1080" viewBox="0 0 1920 1080" style={{position:'absolute', inset:0}} aria-hidden="true">
      <Arrow id={`${id}-forward`} points={forward} progress={forwardP}/>
      <Arrow id={`${id}-reverse`} points={reverse} progress={reverseP}/>
    </svg>
  </div>;
}
