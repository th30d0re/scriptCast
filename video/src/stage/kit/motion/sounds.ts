import {sounds as opusSounds} from './sounds.opus';
import {sounds as astraSounds} from './variants/astra/sounds';
import {sounds as kimiSounds} from './variants/kimi/sounds';
import {sounds as flashSounds} from './variants/flash/sounds';

export * from './sounds.opus';
export type SoundVariant = 'opus'|'astra'|'kimi'|'flash';
export type MotionSoundCue = {component:string; variant:SoundVariant; event:string; frameOffset:number; sound:string; gain_db:number; note?:string};
const tagged=(variant:SoundVariant, cues:readonly {component:string;event:string;frameOffset:number;sound:string;gain_db:number;note?:string}[]):MotionSoundCue[]=>cues.map(c=>({...c,variant}));

/** All selected bake-off cues, keyed by component + variant + event. Opus is the default variant. */
export const motionSounds:readonly MotionSoundCue[]=[...tagged('opus',opusSounds),...tagged('astra',astraSounds),...tagged('kimi',kimiSounds),...tagged('flash',flashSounds.filter(c=>c.component==='FitBox'))];
export const soundCueTable=Object.fromEntries(motionSounds.map(c=>[`${c.component}.${c.variant}.${c.event}`,c])) as Record<string,MotionSoundCue>;
export const defaultSounds=motionSounds.filter(c=>c.variant==='opus');
