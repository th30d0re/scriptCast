export const WIDTH = 1080;
export const HEIGHT = 1920;
export const TOP = 0.14;
export const BOTTOM = 0.35;
export const SIDES = 0.06;
export const BOTTOM_RIGHT = {bottom: 0.40, width: 0.21} as const;
export const USABLE_BOX = {
  x: WIDTH * SIDES,
  y: HEIGHT * TOP,
  width: WIDTH * (1 - 2 * SIDES),
  // Conservatively exclude the deeper right overlay across the whole rectangle.
  height: HEIGHT * (1 - TOP - BOTTOM_RIGHT.bottom),
} as const;

export type Layout = 'portrait' | 'landscape';
export const layoutFromDimensions = (width: number, height: number): Layout => width > height ? 'landscape' : 'portrait';
export const LANDSCAPE = {
  width: 1920, height: 1080,
  box: {x: 96, y: 64.8, width: 1728, height: 864},
  captions: {top: 928.8, height: 151.2, sides: 96},
} as const;
export const safeBox = (layout: Layout) => layout === 'landscape' ? LANDSCAPE.box : USABLE_BOX;
