import React from 'react';
import {CardBorder} from './CardBorder';
// The card border for a scene. Episode code supplies animated notch progress when needed.
export function SceneBorder({notch=1}:{notch?:number}) {
 return <CardBorder notch={notch}/>;
}
