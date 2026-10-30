import {AbsoluteFill, Audio, Freeze, OffthreadVideo, Sequence, staticFile, useCurrentFrame, useVideoConfig} from 'remotion';
import type {ReactNode} from 'react';
import {PORTRAIT, formatFor, palette, type TimelineProps, type StatBarsProps, type TitleProps, type CompareProps, type FreezeProps, type PartsProps} from '@scriptcast/cards';
import {TimelineCard, StatBarsCard, TitleCard, CompareCard, FreezeCallout, PartsCard} from './cards';

export type Card = ({component: 'TimelineCard'} & TimelineProps)
  | ({component: 'StatBarsCard'} & StatBarsProps)
  | ({component: 'TitleCard'} & TitleProps)
  | ({component: 'CompareCard'} & CompareProps)
  | ({component: 'FreezeCallout'} & FreezeProps)
  | ({component: 'PartsCard'} & PartsProps);
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
    case 'PartsCard': return <PartsCard {...card} />;
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
  const {width, height} = useVideoConfig();
  const format = formatFor(width, height);
  // Portrait keeps footage inside the safe zone's side margins; landscape footage fills the frame.
  const side = format.name === 'portrait' ? format.box.x / format.width : 0;
  const style = {
    position: 'absolute', left: `${side * 100}%`, top: '50%',
    transform: 'translateY(-50%)', width: `${(1 - 2 * side) * 100}%`,
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
// Draws children at the layout's own size and scales them to the output (2x for 4K),
// as the card wrappers in cards.tsx do.
function Stage({children}: {children: ReactNode}) {
  const {width, height} = useVideoConfig();
  const format = formatFor(width, height);
  const scale = width / format.width;
  return <div style={{position: 'absolute', left: 0, top: 0, width: format.width, height: format.height,
    transform: scale === 1 ? undefined : `scale(${scale})`, transformOrigin: '0 0'}}>{children}</div>;
}
// Captions sit in the format's caption strip, below the card area. Portrait sides keep the
// 6% margin; landscape sides follow the title-safe box.
export const CAPTION_TOP = PORTRAIT.captionTop;
const CAPTION_HEIGHT = 84;
export function CaptionView({text}: {text: string}) {
  const {width, height} = useVideoConfig();
  const format = formatFor(width, height);
  const side = format.name === 'portrait' ? 65 : format.box.x;
  const top = format.captionTop;
  return <Stage><div style={{position: 'absolute', left: side, right: side, top, height: CAPTION_HEIGHT,
    display: 'flex', justifyContent: 'center', alignItems: 'center'}}>
    <span data-caption style={{fontFamily: 'Helvetica Neue, Helvetica, Arial, sans-serif', fontWeight: 800,
      fontSize: 50, lineHeight: 1.1, textAlign: 'center', color: palette.cream, background: 'rgba(15,27,51,0.78)',
      borderRadius: 14, padding: '6px 22px', textShadow: '0 2px 6px rgba(0,0,0,0.6)'}}>{text}</span>
  </div></Stage>;
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
