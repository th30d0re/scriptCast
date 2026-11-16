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
  tone?: number;
};

export type SortingLabels = {
  ingroup: string;
  outgroup: string;
  boundary?: string;
};

export type TermBlockGlyph = React.ReactNode;

export type ArchLayout = {
  viewBox: [number, number];
  nodes: {id: string; label: string; x: number; y: number; w: number; h: number; kind?: string}[];
  edges: {id: string; from: string; to: string; points: [number, number][]; label?: string; labelBox?: {x: number; y: number; w: number; h: number}; labelSize?: number; labelAt?: [number, number]}[];
  groups: {label: string; x: number; y: number; w: number; h: number}[];
};
