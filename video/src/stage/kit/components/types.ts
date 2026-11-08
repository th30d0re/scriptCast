import type React from 'react';

export type Accent = string;
export type TermState = 'hidden' | 'dim' | 'active' | 'selected' | number;
export type PersonGroup = 'neutral' | 'ingroup' | 'outgroup';

export type SortingMark = {
  id?: string | number;
  x: number;
  y: number;
  group?: PersonGroup;
  emphasis?: number;
  scale?: number;
};

export type SortingLabels = {
  ingroup: string;
  outgroup: string;
  boundary?: string;
};

export type TermBlockGlyph = React.ReactNode;

