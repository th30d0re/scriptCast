import {clipSourceFrame, episodeMetadata, frameAt, frameWindow, type EpisodePlan} from '../src/Episode';
import sample from '../examples/plan.sample.json';

const plan: EpisodePlan = {...sample, cards: sample.cards.map(c => ({...c, card: {...c.card, component: 'TitleCard' as const}}))};
function check(condition: boolean, message: string) {
  if (!condition) throw new Error(message);
}
check(episodeMetadata({props: plan}).durationInFrames === 90, 'sample duration');
const windows = [{start_ms: 0, end_ms: 1033}, {start_ms: 1033, end_ms: 2033}, {start_ms: 2033, end_ms: 4033}].map(w => frameWindow(w, 30));
check(windows.every(w => w.durationInFrames > 0), 'positive durations');
check(windows.slice(1).every((w, i) => w.from === windows[i].from + windows[i].durationInFrames), 'no boundary gaps or overlaps');
check(frameAt(50, 30) === 2, 'half-frame rounding');
const clip = {...plan.clips[1], in_ms: 2000, out_ms: 3000};
check(clipSourceFrame(clip, 30, 0) === 60, 'source in point');
check(clipSourceFrame(clip, 30, 28) === 88, 'source progression');
check(clipSourceFrame(clip, 30, 29) === 89, 'last source frame');
check(clipSourceFrame(clip, 30, 59) === 89, 'hold instead of looping');
check(frameWindow({start_ms: 1033, end_ms: 1034}, 30).durationInFrames === 0, 'subframe omitted');
console.log('PASS: sample metadata, shared frame boundaries, excerpt progression and last-frame hold.');
