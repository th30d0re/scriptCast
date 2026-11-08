import React from 'react';
import type {Segment} from '../../types';
import {color} from '../theme/tokens';
import {SceneBorder} from '../primitives/SceneBorder';
import {SourceLine} from './SourceLine';

export type StageSceneFrameProps = React.PropsWithChildren<{
  segment: Segment;
  title?: string;
  source?: string;
  notch?: number;
}>;

export function StageSceneFrame({segment, title, source, notch=1, children}: StageSceneFrameProps) {
  return <div data-shared="StageSceneFrame" data-scene={segment.scene} style={{position:'absolute', inset:0, width:1920, height:1080, overflow:'hidden', background:color('color/bg/field'), color:color('color/text/primary'), fontFamily:'IBM Plex Sans'}}>
    <SceneBorder notch={notch}/>
    {title && <div data-frame-title="true" style={{position:'absolute', left:116, top:82, right:116, fontSize:54, lineHeight:1.08, fontWeight:500, color:color('color/text/primary')}}>{title}</div>}
    {children}
    {source && <SourceLine text={source}/>} 
  </div>;
}
