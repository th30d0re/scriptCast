import React from 'react';
import {Composition} from 'remotion';
import {StageEpisode} from './StageEpisode';
import {StageMarks} from './StageMarks';
import type {StagePlan} from './types';
const defaults: StagePlan = {episode: 'Stage', fps: 30, width: 1920, height: 1080, duration_ms: 1000, range: {start_ms: 0, end_ms: 1000}, audio: '', cues: {}, segments: [], warnings: []};
const metadata = ({props}: {props: StagePlan}) => ({fps: props.fps, width: props.width, height: props.height, durationInFrames: Math.max(1, Math.ceil((props.range.end_ms-props.range.start_ms)*props.fps/1000))});
export const Root = () => <>{['StageEpisode', 'StageStill'].map(id => <Composition key={id} id={id} component={StageEpisode} defaultProps={defaults} calculateMetadata={metadata} fps={30} width={1920} height={1080} durationInFrames={30}/>)}
  <Composition id="StageMarks" component={StageMarks} defaultProps={defaults} calculateMetadata={metadata} fps={30} width={1920} height={1080} durationInFrames={30}/>
</>;
