import React from 'react';
import type {PersonGroup} from '../components/types';
import {StickFigure} from '../motion/StickFigure';

export type PersonMarkProps = {
  x: number;
  y: number;
  group: PersonGroup;
  emphasis: number;
  scale: number;
  tone?: number;
};

/** Static person glyph. Renders the kit's default (Opus) StickFigure fully drawn; use StickFigure directly to animate it. */
export function PersonMark(props: PersonMarkProps) {
  return <StickFigure {...props} animate={false}/>;
}
