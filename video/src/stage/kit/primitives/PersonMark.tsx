import React from 'react';
import {color} from '../theme/tokens';
import type {PersonGroup} from '../components/types';

export type PersonMarkProps = {
  x: number;
  y: number;
  group: PersonGroup;
  emphasis: number;
  scale: number;
};

const groupColor = (group: PersonGroup) => group === 'ingroup'
  ? color('color/fill/gold')
  : group === 'outgroup' ? color('color/marker/race') : color('color/text/muted');

export function PersonMark({x, y, group, emphasis:rawEmphasis, scale}: PersonMarkProps) {
  const emphasis=Math.max(0,Math.min(1,rawEmphasis)), size=44*Math.max(.2,scale);
  return <div data-shared="PersonMark" data-group={group} style={{position:'absolute', left:x, top:y, width:size, height:size*1.3, transform:'translate(-50%,-50%)', opacity:.42+.58*emphasis, color:groupColor(group)}}>
    <svg width="100%" height="100%" viewBox="0 0 44 58" fill="none" aria-hidden="true">
      {/* Head and shoulders, matching PersonIcon. The old arm bar read as a line struck through the figure. */}
      <circle cx="22" cy="14" r="8.5" fill="currentColor"/>
      <path d="M6 50Q9 29 22 29Q35 29 38 50" stroke="currentColor" strokeWidth={3+2*emphasis} strokeLinecap="round"/>
    </svg>
  </div>;
}
