/**
 * Skin tone ramp for population figures (0..1):
 * - 0 = white (#F5F2EC), 0.2 = pale
 * - ~0.35 = light tan (#D0A87E)
 * - ~0.60 = brown (#7A4E30)
 * - 1 = near-black (#0B0B0B)
 */
export const FIGURE_WHITE = '#F5F2EC';
const STOPS: [number, [number, number, number]][] = [
  [0, [245, 242, 236]],   // white
  [0.2, [236, 214, 190]], // pale
  [0.35, [214, 174, 133]],// light tan
  [0.6, [122, 78, 48]],   // brown
  [1, [11, 11, 11]],      // near-black
];
export function skinToneRamp(tone: number): string {
  const t = Math.max(0, Math.min(1, Number.isFinite(tone) ? tone : 0));
  let i = 0;
  while (i < STOPS.length - 2 && t > STOPS[i + 1][0]) i++;
  const [t0, a] = STOPS[i], [t1, b] = STOPS[i + 1], u = (t - t0) / (t1 - t0);
  const hex = (k: number) => Math.round(a[k] + (b[k] - a[k]) * u).toString(16).padStart(2, '0');
  return `#${hex(0)}${hex(1)}${hex(2)}`;
}

export const skinToneColor = skinToneRamp;

/**
 * Returns true if the figure needs a light rim stroke to remain legible against the navy field.
 * Black/near-black figures (outgroup or tone >= 0.65) receive a 1.5-2px cream rim stroke.
 */
export function needsFigureRim(tone?: number, group?: string): boolean {
  if (typeof tone === 'number') {
    return tone >= 0.65;
  }
  return group === 'outgroup';
}
