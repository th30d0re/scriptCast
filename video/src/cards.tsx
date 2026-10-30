import {Img, continueRender, delayRender, interpolate, staticFile, useCurrentFrame, useVideoConfig} from 'remotion';
import * as cards from '@scriptcast/cards';
import type {BaseProps, CardHost} from '@scriptcast/cards';

// Remotion side of the shared cards: the frame counter becomes `progress`, and the
// host hooks become staticFile, Img and delayRender.
const host: CardHost = {
  asset: staticFile,
  Image: Img,
  hold: label => {
    const handle = delayRender(label);
    return () => continueRender(handle);
  },
};

// Progress 0 to 1 over the card's entrance (beats are frames at ENTRANCE_FPS), counted
// from the card's own first frame (each card sits in its own Sequence). A still
// (one-frame composition) always reads 1, so previews show the finished card.
function useCardClock(beats: number) {
  const frame = useCurrentFrame();
  const {fps, durationInFrames} = useVideoConfig();
  if (durationInFrames <= 1) return {progress: 1, seconds: 0};
  const progress = interpolate(frame, [0, beats * fps / cards.ENTRANCE_FPS], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  return {progress, seconds: frame / fps};
}

function clocked<P extends BaseProps>(Card: (props: P) => React.ReactNode, beats: (props: P) => number) {
  return function ClockedCard(props: P) {
    const {progress, seconds} = useCardClock(beats(props));
    // An animated SVG plays on the card's own clock unless the plan pins a moment.
    const art = props.art?.animate ? {...props.art, atSeconds: props.art.atSeconds ?? seconds} : props.art;
    return <cards.CardHostProvider value={host}><Card {...props} art={art} progress={progress} /></cards.CardHostProvider>;
  };
}

export const TimelineCard = clocked(cards.TimelineCard, cards.timelineBeats);
export const StatBarsCard = clocked(cards.StatBarsCard, cards.statBarsBeats);
export const TitleCard = clocked(cards.TitleCard, cards.titleBeats);
export const CompareCard = clocked(cards.CompareCard, cards.compareBeats);
export const FreezeCallout = clocked(cards.FreezeCallout, cards.freezeBeats);
export const PartsCard = clocked(cards.PartsCard, cards.partsBeats);
