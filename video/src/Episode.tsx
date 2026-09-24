import {AbsoluteFill, Audio, Freeze, OffthreadVideo, Sequence, staticFile, useCurrentFrame} from 'remotion';
import {TimelineCard, type TimelineProps} from './cards/TimelineCard';
import {StatBarsCard, type StatBarsProps} from './cards/StatBarsCard';
import {TitleCard, type TitleProps} from './cards/TitleCard';
import {CompareCard, type CompareProps} from './cards/CompareCard';
import {USABLE_BOX} from './safeZone';
import {palette} from './theme';

export type Card = ({component: 'TimelineCard'} & TimelineProps)
  | ({component: 'StatBarsCard'} & StatBarsProps)
  | ({component: 'TitleCard'} & TitleProps)
  | ({component: 'CompareCard'} & CompareProps);
type Window = {start_ms: number; end_ms: number};
type Clip = Window & {turn_index: number; clip_id: string; src: string; in_ms: number; out_ms: number};
export type EpisodePlan = {fps: number; width: number; height: number; duration_ms: number;
  audio: string; clips: Clip[]; cards: (Window & {shot_id: string; card: Card})[]; warnings: string[]};
export const frameAt = (ms: number, fps: number) => Math.round(ms * fps / 1000);
export const frameWindow = (window: Window, fps: number) => {
  const from = frameAt(window.start_ms, fps);
  return {from, durationInFrames: Math.max(0, frameAt(window.end_ms, fps) - from)};
};
export const episodeMetadata = ({props}: {props: EpisodePlan}) => ({
  durationInFrames: Math.max(1, frameAt(props.duration_ms, props.fps)),
  fps: props.fps, width: props.width, height: props.height,
});
function CardView({card}: {card: Card}) {
  switch (card.component) {
    case 'TimelineCard': return <TimelineCard {...card} />;
    case 'StatBarsCard': return <StatBarsCard {...card} />;
    case 'TitleCard': return <TitleCard {...card} />;
    case 'CompareCard': return <CompareCard {...card} />;
  }
}
export const clipSourceFrame = (clip: Clip, fps: number, frame: number) => {
  const first = frameAt(clip.in_ms, fps);
  const last = Math.max(first, frameAt(clip.out_ms, fps) - 1);
  return Math.min(first + frame, last);
};
function ClipView({clip, fps}: {clip: Clip; fps: number}) {
  const frame = useCurrentFrame();
  // Freeze evaluates a source frame directly: clamp to the excerpt's final frame
  // for long turns, without replaying or depending on the media's EOF behavior.
  return <Freeze frame={clipSourceFrame(clip, fps, frame)}>
    <OffthreadVideo src={staticFile(clip.src)} muted style={{
      position: 'absolute', left: USABLE_BOX.x, top: '50%',
      transform: 'translateY(-50%)', width: USABLE_BOX.width,
      height: '100%', objectFit: 'contain',
    }} />
  </Freeze>;
}
export function Episode(plan: EpisodePlan) {
  return <AbsoluteFill style={{backgroundColor: palette.navy}}>
    <Audio src={staticFile(plan.audio)} />
    {plan.clips.map(clip => {
      const window = frameWindow(clip, plan.fps);
      return window.durationInFrames > 0 && <Sequence key={clip.turn_index} {...window}>
        <ClipView clip={clip} fps={plan.fps} />
      </Sequence>;
    })}
    {plan.cards.map(card => {
      const window = frameWindow(card, plan.fps);
      return window.durationInFrames > 0 && <Sequence key={card.shot_id} {...window}>
        <CardView card={card.card} />
      </Sequence>;
    })}
  </AbsoluteFill>;
}
