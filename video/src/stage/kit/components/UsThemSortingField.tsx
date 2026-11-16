import React from 'react';
import {color} from '../theme/tokens';
import {PersonMark} from '../primitives/PersonMark';
import type {PersonGroup, SortingLabels, SortingMark} from './types';

export type SortingMode = 'neutral' | 'partition' | 'allocation' | 'compressed';
export type UsThemSortingFieldProps = {
  marks: readonly SortingMark[];
  partitionProgress: number;
  ingroupProgress: number;
  outgroupProgress: number;
  labels: SortingLabels;
  mode: SortingMode;
};

export const SORTING_FIELD = {width:760,height:430,boundaryX:380} as const;
const clamp=(value:number)=>Math.max(0,Math.min(1,value));
const coordinate=(value:number,span:number)=>Math.abs(value)<=1?value*span:value;

export function UsThemSortingField({marks, partitionProgress:rawPartition, ingroupProgress:rawIngroup, outgroupProgress:rawOutgroup, labels, mode}: UsThemSortingFieldProps) {
  const partition=clamp(rawPartition),ingroup=clamp(rawIngroup),outgroup=clamp(rawOutgroup);
  return <div data-shared="UsThemSortingField" data-mode={mode} style={{position:'relative', width:SORTING_FIELD.width, height:SORTING_FIELD.height, overflow:'hidden', background:color('color/fill/panel-render'), border:`1px solid ${color('color/line/hairline')}`}}>
    <svg width={SORTING_FIELD.width} height={SORTING_FIELD.height} viewBox={`0 0 ${SORTING_FIELD.width} ${SORTING_FIELD.height}`} style={{position:'absolute',inset:0}} aria-hidden="true">
      <rect x="0" y="0" width={SORTING_FIELD.boundaryX} height={SORTING_FIELD.height} fill={color('color/fill/gold-wash')} opacity={.55*ingroup}/>
      <rect x={SORTING_FIELD.boundaryX} y="0" width={SORTING_FIELD.boundaryX} height={SORTING_FIELD.height} fill={color('color/fill/red-faint')} opacity={.7*outgroup}/>
      <line data-partition="true" x1={SORTING_FIELD.boundaryX} y1={26+180*(1-partition)} x2={SORTING_FIELD.boundaryX} y2={SORTING_FIELD.height-26-180*(1-partition)} stroke={color('color/line/hairline-strong')} strokeWidth="3" opacity={partition}/>
      {mode==='allocation' && <>
        <path d="M90 365H320" stroke={color('color/fill/gold')} strokeWidth="8" opacity={ingroup}/>
        <path d="M440 365H650" stroke={color('color/marker/race')} strokeWidth="4" opacity={outgroup}/>
      </>}
    </svg>
    {marks.map((mark,index)=>{
      const baseX=coordinate(mark.x,SORTING_FIELD.width), baseY=coordinate(mark.y,SORTING_FIELD.height);
      const resolvedGroup:PersonGroup=mark.group??(baseX<SORTING_FIELD.boundaryX?'ingroup':'outgroup');
      const direction=resolvedGroup==='ingroup'?-1:resolvedGroup==='outgroup'?1:0;
      const groupProgress=resolvedGroup==='ingroup'?ingroup:resolvedGroup==='outgroup'?outgroup:0;
      const compression=mode==='compressed'?.7:1;
      return <PersonMark key={mark.id??index} x={SORTING_FIELD.boundaryX+(baseX-SORTING_FIELD.boundaryX)*compression+direction*32*partition} y={baseY} group={partition>.08?resolvedGroup:'neutral'} emphasis={clamp(mark.emphasis??(.45+.55*groupProgress))} scale={mark.scale??1} tone={partition > .08 ? mark.tone : undefined}/>;
    })}
    {ingroup>0 && <div style={{position:'absolute',left:34,top:24,font:'500 20px "IBM Plex Mono"',letterSpacing:1.2,color:color('color/fill/gold'),opacity:ingroup}}>{labels.ingroup}</div>}
    {outgroup>0 && <div style={{position:'absolute',right:34,top:24,font:'500 20px "IBM Plex Mono"',letterSpacing:1.2,color:color('color/marker/race'),opacity:outgroup}}>{labels.outgroup}</div>}
    {labels.boundary && partition>0 && <div style={{position:'absolute',left:SORTING_FIELD.boundaryX,top:24,transform:'translateX(-50%)',padding:'0 10px',font:'500 20px "IBM Plex Mono"',lineHeight:'normal',letterSpacing:1.2,color:color('color/text/muted'),background:color('color/bg/field'),opacity:partition}}>{labels.boundary}</div>}
  </div>;
}
