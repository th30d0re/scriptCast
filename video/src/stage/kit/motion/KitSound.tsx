import React, {createContext, useContext} from 'react';
import {Audio, Internals, Sequence, staticFile} from 'remotion';
import {useKitFrameOffset} from './core';
import {motionSounds, type SoundVariant} from './sounds';

export type KitSoundProps = React.PropsWithChildren<{
  enabled?:boolean;
  gainDb?:number;
  /** Additional attenuation to apply while this provider is inside a narration region. */
  duckDb?:number;
  /** `beats` plays marked beats at full level and every other motion sound NON_BEAT_DB quieter; `all` plays everything at full level. */
  mode?:'all'|'beats';
}>;

export type KitSoundCueProps = {
  /** Marks this motion as a narration beat when its nearest KitSound uses `mode="beats"`. */
  beat?:boolean;
  /** Overrides the sound-table component key for this beat, for example `Flicker:powerDown`. */
  beatCue?:string;
};

type KitSoundState = {enabled:boolean; gainDb:number; duckDb:number; mode:'all'|'beats'};
const KitSoundContext = createContext<KitSoundState>({enabled:true, gainDb:0, duckDb:0, mode:'all'});

export const dbToGain = (db:number) => 10 ** (db/20);
export const soundFileName = (id:string) => `${id.replaceAll('/', '__')}.wav`;

export function KitSound({enabled=true, gainDb=0, duckDb=0, mode='all', children}:KitSoundProps) {
  return <KitSoundContext.Provider value={{enabled,gainDb,duckDb,mode}}>{children}</KitSoundContext.Provider>;
}

const cueComponent = (component:string, variant:SoundVariant, mode?:string) => {
  if (component === 'PowerDown' && variant === 'argon') return 'Flicker:powerDown';
  if (component === 'PowerDown' && variant === 'astra') return 'Flicker.off';
  if (component !== 'BootIn') return component;
  if (variant === 'argon') return `BootIn:${mode ?? 'scan'}`;
  if (variant === 'kimi') return `BootIn/${mode ?? 'scan'}`;
  return `BootIn.${mode ?? 'scan'}`;
};

/** Level of non-beat motion sounds under `mode="beats"`, relative to beats. */
export const NON_BEAT_DB = -9;

/** Audio scheduled for an absolute-timeline animation. Progress-driven previews are intentionally silent. */
export function useKitSounds(component:string, variant:SoundVariant, startFrame?:number, cueMode?:string, beat?:boolean, beatCue?:string) {
  const sound = useContext(KitSoundContext);
  const offset = useKitFrameOffset(), inRemotion = useContext(Internals.CanUseRemotionHooks);
  if (!inRemotion || !sound.enabled || startFrame === undefined) return null;
  // In beats mode every motion sounds; marked beats stay at full level and the rest sit underneath them.
  const beatDuck = sound.mode === 'beats' && !beat ? NON_BEAT_DB : 0;
  const name = beatCue ?? cueComponent(component, variant, cueMode);
  return motionSounds.filter((cue) => cue.variant === variant && cue.component === name).map((cue) =>
    <Sequence key={`${cue.event}-${cue.frameOffset}`} from={startFrame + cue.frameOffset - offset} layout="none">
      <Audio src={staticFile(`sfx/${soundFileName(cue.sound)}`)} volume={dbToGain(cue.gain_db + sound.gainDb + sound.duckDb + beatDuck)}/>
    </Sequence>
  );
}
