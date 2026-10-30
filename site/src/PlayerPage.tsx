import {Player} from '@remotion/player';
import {TimelineView, secToFrame} from '../../video/src/timeline/TimelineComposition';
import type {Timeline} from '../../video/src/timeline/types';
import example from '../../video/examples/timeline.example.json';

const timeline = example as Timeline;

export default function PlayerPage() {
  return <main className="player">
    <Player component={TimelineView} inputProps={timeline}
      durationInFrames={Math.max(1, secToFrame(timeline.durationSec, timeline.fps))}
      fps={timeline.fps} compositionWidth={timeline.width} compositionHeight={timeline.height}
      controls loop style={{width: '100%', maxHeight: 'calc(100svh - 64px)', aspectRatio: `${timeline.width} / ${timeline.height}`}} />
  </main>;
}
