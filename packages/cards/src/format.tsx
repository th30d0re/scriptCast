import {createContext, useContext} from 'react';
import {WIDTH, HEIGHT, USABLE_BOX} from './safeZone';

// The layout a card is drawn in: its size in CSS px and the box the content must stay
// inside. A host scales the drawn card to its output size, so 4K is the landscape layout
// drawn at 2x rather than a layout of its own.
export type CardFormat = {
  name: 'portrait' | 'landscape';
  width: number;
  height: number;
  box: {x: number; y: number; width: number; height: number};
};

// Shorts / Reels: the platform overlays set the box (see safeZone.ts).
export const PORTRAIT: CardFormat = {name: 'portrait', width: WIDTH, height: HEIGHT, box: USABLE_BOX};

// YouTube 16:9: no overlays over the picture, so the box is the 5% title-safe margin.
const LANDSCAPE_W = 1920;
const LANDSCAPE_H = 1080;
const MARGIN = 0.05;
export const LANDSCAPE: CardFormat = {
  name: 'landscape', width: LANDSCAPE_W, height: LANDSCAPE_H,
  box: {x: LANDSCAPE_W * MARGIN, y: LANDSCAPE_H * MARGIN, width: LANDSCAPE_W * (1 - 2 * MARGIN), height: LANDSCAPE_H * (1 - 2 * MARGIN)},
};

// Wider than tall is landscape; anything else is portrait.
export const formatFor = (width: number, height: number): CardFormat => width > height ? LANDSCAPE : PORTRAIT;

const FormatContext = createContext<CardFormat>(PORTRAIT);
export const CardFormatProvider = FormatContext.Provider;
export const useCardFormat = () => useContext(FormatContext);
