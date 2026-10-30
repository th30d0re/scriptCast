import {Player} from '@remotion/player';
import {TimelineView, secToFrame} from '../../video/src/timeline/TimelineComposition';
import type {Timeline} from '../../video/src/timeline/types';
import example from '../../video/examples/timeline.example.json';

import type {CardFormat} from '@scriptcast/cards';

// The example timeline at the chosen format's size; the cards lay out from that size.
export default function PlayerPage({format}: {format: CardFormat}) {
  const timeline = {...(example as Timeline), width: format.width, height: format.height};
  return <main className="player" style={{'--aspect': `${format.width} / ${format.height}`} as React.CSSProperties}>
    <Player component={TimelineView} inputProps={timeline}
      durationInFrames={Math.max(1, secToFrame(timeline.durationSec, timeline.fps))}
      fps={timeline.fps} compositionWidth={timeline.width} compositionHeight={timeline.height}
      controls loop key={format.name} style={{width: '100%', maxHeight: 'calc(100svh - 64px)', aspectRatio: `${timeline.width} / ${timeline.height}`}} />
  </main>;
}
