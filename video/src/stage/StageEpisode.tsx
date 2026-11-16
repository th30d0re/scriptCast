import React, {useEffect, useState} from 'react';
import {AbsoluteFill, Audio, Sequence, cancelRender, continueRender, delayRender, staticFile} from 'remotion';
import '@fontsource/ibm-plex-sans/400.css';
import '@fontsource/ibm-plex-sans/500.css';
import '@fontsource/ibm-plex-sans/600.css';
import '@fontsource/ibm-plex-mono/400.css';
import '@fontsource/ibm-plex-mono/500.css';
import {scenes} from '@episode';
import {RemotionClock, useStageMs} from './clock';
import {PlanContext} from './plan';
import type {SceneRegistry} from './registry';
import type {Segment, StagePlan} from './types';
import {useTokens} from './tokens';
const registry: SceneRegistry = scenes;
function Scene({segment, interactive}: {segment: Segment; interactive: boolean}) {
  const Component = registry[segment.scene];
  if (!Component) {
    if (!interactive) throw new Error(`Unknown Stage scene: ${segment.scene}`);
    return <div style={{background: '#a00000', color: 'white', padding: 40}}>Unknown Stage scene: {segment.scene}</div>;
  }
  return <Component segment={segment} shots={segment.shots} shotAt={ms => segment.shots.find(s => ms >= s.start_ms && ms < s.end_ms)} />;
}
export function sceneWarnings(plan: StagePlan) {
  return [...plan.warnings, ...plan.segments.filter(s => !registry[s.scene]).map(s => `Unknown Stage scene: ${s.scene}`)];
}
function RemotionFontGate() {
  const [handle] = useState(() => delayRender('Loading IBM Plex fonts'));
  useEffect(() => {
    let active = true, settled = false;
    const ready = typeof document === 'undefined' || !document.fonts ? Promise.resolve() : document.fonts.ready;
    ready.then(() => {if (active) {settled = true; continueRender(handle);}}, error => {if (active) {settled = true; cancelRender(error);}});
    return () => {active = false; if (!settled) continueRender(handle);};
  }, [handle]);
  return null;
}
/** Browser host; its clock is supplied by ManualClock. */
export function StageContent({plan}: {plan: StagePlan}) {
  const ms = useStageMs(), tokens = useTokens();
  return <PlanContext.Provider value={plan}><div style={{position: 'relative', width: plan.width, height: plan.height, overflow: 'hidden', background: tokens.colours.background, color: tokens.colours.text, fontFamily: tokens.fonts.body}}>
    {plan.segments.map((seg, i) => ms >= seg.start_ms && ms < seg.end_ms ? <Scene key={`${i}:${seg.instance}`} segment={seg} interactive /> : null)}
  </div></PlanContext.Provider>;
}
export function StageEpisode(plan: StagePlan) {
  const tokens = useTokens(), fps = plan.fps;
  return <PlanContext.Provider value={plan}><AbsoluteFill style={{background: tokens.colours.background, color: tokens.colours.text, fontFamily: tokens.fonts.body}}><RemotionFontGate />
    {plan.audio && <Audio src={staticFile(plan.audio)} startFrom={Math.round(plan.range.start_ms*fps/1000)} endAt={Math.ceil(plan.range.end_ms*fps/1000)} />}
    {plan.segments.map((seg, i) => {
      const from = Math.ceil((seg.start_ms-plan.range.start_ms)*fps/1000);
      const end = Math.ceil((seg.end_ms-plan.range.start_ms)*fps/1000);
      return <Sequence key={`${i}:${seg.instance}`} from={from} durationInFrames={Math.max(1, end-from)}>
        <RemotionClock startMs={plan.range.start_ms+from*1000/fps}><Scene segment={seg} interactive={false} /></RemotionClock>
      </Sequence>;
    })}
  </AbsoluteFill></PlanContext.Provider>;
}
