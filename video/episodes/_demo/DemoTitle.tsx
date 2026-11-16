import React from 'react';
import {Reveal, useTokens, type SceneProps} from '@stage';
export function DemoTitle({shots}: SceneProps) {
  const tokens = useTokens();
  return <div style={{padding: 120}}><Reveal cue="hello"><h1 style={{fontSize: tokens.sizes.title}}>{String(shots[0].props.title ?? 'Stage')}</h1></Reveal><Reveal cue="world"><p style={{fontSize: 36}}>One scene. Two clocks. A world of stories.</p></Reveal></div>;
}
