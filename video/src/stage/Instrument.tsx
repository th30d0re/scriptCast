import React, {useContext, useState} from 'react';
import {ClockContext} from './clock';
import {cubicInOut, mix, progress} from './motion';
import {useTokens} from './tokens';
export type InstrumentMode = 'compact' | 'expanded' | 'settings';
export type ModeKey = {at: number; mode: InstrumentMode};
export type Rect = {x: number; y: number; width: number; height: number};
export type InstrumentGrow = 'right-down' | 'left-down' | 'right-up' | 'left-up' | 'centre';
const modes: InstrumentMode[] = ['compact', 'expanded', 'settings'];
export function growRect(compact: Rect, target: Rect, grow: InstrumentGrow): Rect {
  const x = grow === 'right-down' || grow === 'right-up' ? compact.x
    : grow === 'left-down' || grow === 'left-up' ? compact.x+compact.width-target.width
    : compact.x+(compact.width-target.width)/2;
  const y = grow === 'right-down' || grow === 'left-down' ? compact.y
    : grow === 'right-up' || grow === 'left-up' ? compact.y+compact.height-target.height
    : compact.y+(compact.height-target.height)/2;
  return {x, y, width: target.width, height: target.height};
}
export function resolveMode(ms: number, timeline: ModeKey[], durationMs = 400) {
  let weights = {compact: 1, expanded: 0, settings: 0}, source = {...weights}, target: InstrumentMode = 'compact', at = -Infinity;
  const sample = (time: number) => {
    const t = cubicInOut(progress(time, at, at+durationMs));
    return Object.fromEntries(modes.map(mode => [mode, mix(source[mode], Number(mode === target), t)])) as typeof weights;
  };
  for (const key of [...timeline].sort((a,b) => a.at-b.at)) {
    if (key.at > ms) break;
    weights = Number.isFinite(at) ? sample(key.at) : weights;
    source = weights; target = key.mode; at = key.at;
  }
  return {mode: target, weights: Number.isFinite(at) ? sample(ms) : weights};
}
export type InstrumentProps = {
  mode?: InstrumentMode; modeTimeline?: ModeKey[]; transitionMs?: number;
  transition?: {from: InstrumentMode; to: InstrumentMode; progress: number};
  grow?: InstrumentGrow;
  unstyled?: boolean;
  onInteractiveModeChange?: (mode: InstrumentMode | null) => void;
  style?: React.CSSProperties;
  anchor: {compact: Rect; expanded: Rect; settings?: Rect};
  compact: () => React.ReactNode; expanded: () => React.ReactNode; settings: () => React.ReactNode;
};
export function Instrument(props: InstrumentProps) {
  const {interactive} = useContext(ClockContext);
  return interactive ? <InteractiveInstrument {...props}/> : <InstrumentFrame {...props}/>;
}
function InteractiveInstrument(props: InstrumentProps) {
  const [override, setOverride] = useState<InstrumentMode | null>(null);
  const changeOverride = (mode: InstrumentMode | null) => {setOverride(mode); props.onInteractiveModeChange?.(mode);};
  return <InstrumentFrame {...props} override={override} changeOverride={changeOverride}/>;
}
function InstrumentFrame(props: InstrumentProps & {override?: InstrumentMode | null; changeOverride?: (mode: InstrumentMode | null) => void}) {
  const {ms, interactive} = useContext(ClockContext), tokens = useTokens();
  const {override, changeOverride} = props;
  const resolved = resolveMode(ms, props.modeTimeline ?? [], props.transitionMs);
  let weights = resolved.weights;
  if (props.mode) weights = {compact: Number(props.mode === 'compact'), expanded: Number(props.mode === 'expanded'), settings: Number(props.mode === 'settings')};
  if (props.transition) {const t = Math.max(0, Math.min(1, props.transition.progress)); weights = Object.fromEntries(modes.map(m => [m, Number(m === props.transition!.from)*(1-t)+Number(m === props.transition!.to)*t])) as typeof weights;}
  if (interactive && override) weights = {compact: Number(override === 'compact'), expanded: Number(override === 'expanded'), settings: Number(override === 'settings')};
  const anchored = {compact: props.anchor.compact, expanded: growRect(props.anchor.compact, props.anchor.expanded, props.grow ?? 'right-down'), settings: growRect(props.anchor.compact, props.anchor.settings ?? props.anchor.expanded, props.grow ?? 'right-down')};
  const rect = {x: 0, y: 0, width: 0, height: 0};
  for (const m of modes) {const r = anchored[m]; for (const k of Object.keys(rect) as (keyof Rect)[]) rect[k] += weights[m]*r[k];}
  const active = modes.reduce((a,b) => weights[a] >= weights[b] ? a : b);
  return <section onMouseEnter={() => interactive && changeOverride?.(active === 'settings' ? 'settings' : 'expanded')} onMouseLeave={() => interactive && changeOverride?.(override === 'settings' ? 'settings' : null)} style={{position: 'absolute', left: rect.x, top: rect.y, width: rect.width, height: rect.height, background: tokens.colours.surface, borderRadius: tokens.radii.panel, border: `${tokens.strokes.border}px solid ${tokens.colours.border}`, overflow: 'hidden', boxSizing: 'border-box', ...(props.unstyled ? {background: 'transparent', border: 'none', borderRadius: 0} : {}), ...props.style, zIndex: override && override !== 'compact' ? 50 : props.style?.zIndex}}>
    {modes.filter(m => weights[m] > 0).map(m => <div key={m} aria-hidden={active !== m} style={{position: 'absolute', inset: props.unstyled ? 0 : 20, opacity: weights[m], pointerEvents: active === m ? 'auto' : 'none', visibility: weights[m] === 0 ? 'hidden' : 'visible'}}>{props[m]()}</div>)}
    {interactive && <button aria-label={active === 'settings' ? 'Close settings' : 'Open settings'} onClick={() => changeOverride?.(active === 'settings' ? 'compact' : 'settings')} style={{position: 'absolute', top: 17.6, right: 17.6, width: 28, height: 28, padding: 0, background: tokens.colours.glass, color: tokens.colours.text, border: `1.6px solid ${tokens.colours.border}`, borderRadius: 6}}>{active === 'settings' ? '×' : '⚙'}</button>}
  </section>;
}
