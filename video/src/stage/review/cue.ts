import type {StagePlan} from '../types';

export function sceneAt(plan: Pick<StagePlan, 'segments'>, ms: number) {
  return plan.segments.find(segment => ms >= segment.start_ms && ms < segment.end_ms)?.scene ?? '';
}

/** Closest cue to ms. A bare name wins a tie with its shot-qualified twin. */
export function nearestCue(cues: Record<string, number>, ms: number) {
  let name = '';
  let best = Infinity;
  for (const [id, at] of Object.entries(cues)) {
    const distance = Math.abs(at - ms);
    const bare = !id.includes('.');
    const bestBare = name !== '' && !name.includes('.');
    if (distance < best || (distance === best && bare && !bestBare)) {
      name = id;
      best = distance;
    }
  }
  return name;
}
