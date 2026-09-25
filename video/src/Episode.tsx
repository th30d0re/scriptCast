import {AbsoluteFill, Audio, Freeze, OffthreadVideo, Sequence, staticFile, useCurrentFrame} from 'remotion';
import {TimelineCard, type TimelineProps} from './cards/TimelineCard';
import {StatBarsCard, type StatBarsProps} from './cards/StatBarsCard';
import {TitleCard, type TitleProps} from './cards/TitleCard';
import {CompareCard, type CompareProps} from './cards/CompareCard';
import {FreezeCallout, type FreezeProps} from './cards/FreezeCallout';
import {USABLE_BOX} from './safeZone';
import {palette} from './theme';

export type Card = ({component: 'TimelineCard'} & TimelineProps)
  | ({component: 'StatBarsCard'} & StatBarsProps)
  | ({component: 'TitleCard'} & TitleProps)
  | ({component: 'CompareCard'} & CompareProps)
  | ({component: 'FreezeCallout'} & FreezeProps);
type Window = {start_ms: number; end_ms: number};
type Clip = Window & {turn_index: number; clip_id: string; src: string; in_ms: number; out_ms: number};
export type EpisodePlan = {fps: number; width: number; height: number; duration_ms: number;
  audio: string; clips: Clip[]; cards: (Window & {shot_id: string; card: Card})[]; captions?: (Window & {text: string})[]; warnings: string[]};
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
    case 'FreezeCallout': return <FreezeCallout {...card} />;
  }
}
export const clipSourceFrame = (clip: Clip, fps: number, frame: number) => {
  const first = frameAt(clip.in_ms, fps);
  const last = Math.max(first, frameAt(clip.out_ms, fps) - 1);
  return Math.min(first + frame, last);
};
function ClipView({clip, fps}: {clip: Clip; fps: number}) {
  const first = frameAt(clip.in_ms, fps);
  const last = clipSourceFrame(clip, fps, Number.MAX_SAFE_INTEGER);
  const style = {
    position: 'absolute', left: USABLE_BOX.x, top: '50%',
    transform: 'translateY(-50%)', width: USABLE_BOX.width,
    height: '100%', objectFit: 'contain',
  } as const;
  // Play the excerpt from its own in-point (startFrom seeks the source; Freeze
  // with an absolute frame does not), then hold the final frame for any remainder.
  return <>
    <Sequence durationInFrames={last - first + 1}>
      <OffthreadVideo src={staticFile(clip.src)} muted startFrom={first} endAt={last + 1} style={style} />
    </Sequence>
    <Sequence from={last - first + 1}>
      <Freeze frame={0}>
        <OffthreadVideo src={staticFile(clip.src)} muted startFrom={last} endAt={last + 1} style={style} />
      </Freeze>
    </Sequence>
  </>;
}
// Captions sit in the strip between the card area (ends at 60% of the height) and the
// platform's bottom overlay (starts at 65%). Sides keep the 6% margin.
export const CAPTION_TOP = 1166;
export function CaptionView({text}: {text: string}) {
  return <div style={{position: 'absolute', left: 65, right: 65, top: CAPTION_TOP, height: 84,
    display: 'flex', justifyContent: 'center', alignItems: 'center'}}>
    <span data-caption style={{fontFamily: 'Helvetica Neue, Helvetica, Arial, sans-serif', fontWeight: 800,
      fontSize: 50, lineHeight: 1.1, textAlign: 'center', color: palette.cream, background: 'rgba(15,27,51,0.78)',
      borderRadius: 14, padding: '6px 22px', textShadow: '0 2px 6px rgba(0,0,0,0.6)'}}>{text}</span>
  </div>;
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
    {(plan.captions ?? []).map((caption, index) => {
      const window = frameWindow(caption, plan.fps);
      return window.durationInFrames > 0 && <Sequence key={`caption-${index}`} {...window}>
        <CaptionView text={caption.text} />
      </Sequence>;
    })}
  </AbsoluteFill>;
}
