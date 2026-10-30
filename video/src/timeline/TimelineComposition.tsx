import {AbsoluteFill, Composition, Sequence} from 'remotion';
import type {CalculateMetadataFunction} from 'remotion';
import {cardRegistry} from './registry';
import type {Timeline} from './types';
import example from '../../examples/timeline.example.json';

export const secToFrame = (sec: number, fps: number) => Math.round(sec * fps);

export const timelineMetadata: CalculateMetadataFunction<Timeline> = ({props}) => ({
  durationInFrames: Math.max(1, secToFrame(props.durationSec, props.fps)),
  fps: props.fps, width: props.width, height: props.height,
});

export function TimelineView({fps, items}: Timeline) {
  return <AbsoluteFill style={{backgroundColor: 'black'}}>
    {items.map((item, index) => {
      const Card = cardRegistry[item.card];
      if (!Card) throw new Error(`Timeline item ${index}: unknown card "${item.card}"`);
      const from = secToFrame(item.startSec, fps);
      const durationInFrames = secToFrame(item.endSec, fps) - from;
      return durationInFrames > 0 && <Sequence key={index} from={from} durationInFrames={durationInFrames}>
        <Card {...item.props} />
      </Sequence>;
    })}
  </AbsoluteFill>;
}

// The size comes from the timeline JSON, so any of these renders any format; the extra
// ids preview the example at YouTube 1080p and 4K in the Studio.
const portrait = example as Timeline;
const sizes = [
  {id: 'Timeline', width: portrait.width, height: portrait.height},
  {id: 'Timeline-YouTube', width: 1920, height: 1080},
  {id: 'Timeline-YouTube-4K', width: 3840, height: 2160},
];
export const TimelineComposition = () => <>{sizes.map(({id, width, height}) => {
  const defaults = {...portrait, width, height};
  return <Composition key={id} id={id} component={TimelineView}
    defaultProps={defaults} calculateMetadata={timelineMetadata}
    fps={defaults.fps} width={width} height={height} durationInFrames={1} />;
})}</>;
