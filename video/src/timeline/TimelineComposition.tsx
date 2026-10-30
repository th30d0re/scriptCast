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

const defaults = example as Timeline;
export const TimelineComposition = () => <Composition id="Timeline" component={TimelineView}
  defaultProps={defaults} calculateMetadata={timelineMetadata}
  fps={defaults.fps} width={defaults.width} height={defaults.height} durationInFrames={1} />;
