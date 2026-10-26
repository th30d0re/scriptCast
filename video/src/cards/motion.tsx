import {Easing, interpolate, useCurrentFrame, useVideoConfig} from 'remotion';
import {ReactNode} from 'react';

// Entrance motion for cards. Progress runs 0 to 1 over `dur` frames after `delay`, counted from
// the card's own first frame (each card sits in its own Sequence). A still (one-frame
// composition) always reads 1, so previews and copy checks see the finished card.
export function useEnter(delay = 0, dur = 12): number {
  let frame = 0;
  let durationInFrames = 1;
  try { // outside a Remotion tree (the copy checks render static markup) there is no timeline
    frame = useCurrentFrame();
    durationInFrames = useVideoConfig().durationInFrames;
  } catch { return 1; }
  if (durationInFrames <= 1) return 1;
  return interpolate(frame - delay, [0, dur], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp', easing: Easing.out(Easing.cubic)});
}

// Opacity only: a transform would move the boxes Frame measures when it fits type to the card.
export function Reveal({delay = 0, dur = 12, children, style}: {delay?: number; dur?: number; children: ReactNode; style?: React.CSSProperties}) {
  const t = useEnter(delay, dur);
  return <div style={{opacity: t, ...style}}>{children}</div>;
}
