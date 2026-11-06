import {createContext, useContext} from 'react';
import {useStageMs} from './clock';
import {clamp, linear, progress, type Ease} from './motion';
import type {StagePlan} from './types';
export const PlanContext = createContext<StagePlan | null>(null);
export function usePlan() {
  const plan = useContext(PlanContext);
  if (!plan) throw new Error('Stage requires PlanContext');
  return plan;
}
export function lookupCue(cues: Record<string, number>, id: string) {
  if (!Object.prototype.hasOwnProperty.call(cues, id)) throw new Error(`Unknown cue "${id}". Known ids: ${Object.keys(cues).join(', ')}`);
  return cues[id];
}
export function useCue(id: string) {
  const at = lookupCue(usePlan().cues, id), ms = useStageMs();
  return {at, since: ms-at, on: ms >= at};
}
export function useCueProgress(id: string, durationMs: number, ease: Ease = linear) {
  const {since} = useCue(id);
  return clamp(ease(progress(since, 0, durationMs)));
}
export const useWords = (turnId: string) => usePlan().words?.[turnId] ?? [];
