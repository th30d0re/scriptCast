import React, {createContext, useContext} from 'react';
import {Audio, Sequence, staticFile} from 'remotion';
import {motionSounds, type SoundVariant} from './sounds';

export type KitSoundProps = React.PropsWithChildren<{
  enabled?:boolean;
  gainDb?:number;
  /** Additional attenuation to apply while this provider is inside a narration region. */
  duckDb?:number;
}>;

type KitSoundState = {enabled:boolean; gainDb:number; duckDb:number};
const KitSoundContext = createContext<KitSoundState>({enabled:true, gainDb:0, duckDb:0});

export const dbToGain = (db:number) => 10 ** (db/20);
export const soundFileName = (id:string) => `${id.replaceAll('/', '__')}.wav`;

export function KitSound({enabled=true, gainDb=0, duckDb=0, children}:KitSoundProps) {
  return <KitSoundContext.Provider value={{enabled,gainDb,duckDb}}>{children}</KitSoundContext.Provider>;
}

const cueComponent = (component:string, variant:SoundVariant, mode?:string) => {
  if (component === 'PowerDown' && variant === 'astra') return 'Flicker.off';
  if (component !== 'BootIn') return component;
  if (variant === 'kimi') return `BootIn/${mode ?? 'scan'}`;
  return `BootIn.${mode ?? 'scan'}`;
};

/** Audio scheduled for an absolute-timeline animation. Progress-driven previews are intentionally silent. */
export function useKitSounds(component:string, variant:SoundVariant, startFrame?:number, mode?:string) {
  const sound = useContext(KitSoundContext);
  if (!sound.enabled || startFrame === undefined) return null;
  const name = cueComponent(component, variant, mode);
  return motionSounds.filter((cue) => cue.variant === variant && cue.component === name).map((cue) =>
    <Sequence key={`${cue.event}-${cue.frameOffset}`} from={startFrame + cue.frameOffset} layout="none">
      <Audio src={staticFile(`sfx/${soundFileName(cue.sound)}`)} volume={dbToGain(cue.gain_db + sound.gainDb + sound.duckDb)}/>
    </Sequence>
  );
}
