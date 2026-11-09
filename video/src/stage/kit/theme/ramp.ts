/**
 * Skin tone ramp for population figures (0..1):
 * - 0 = white/pale (cream/near-white token rgba(232,201,155,1) / #E8C99B)
 * - ~0.35 = light tan (#D0A87E)
 * - ~0.60 = brown (#7A4E30)
 * - 1 = near-black (#0B0B0B)
 */
export function skinToneRamp(tone: number): string {
  const t = Math.max(0, Math.min(1, Number.isFinite(tone) ? tone : 0));
  let r: number, g: number, b: number;
  if (t <= 0.35) {
    const u = t / 0.35;
    r = 232 + (208 - 232) * u;
    g = 201 + (168 - 201) * u;
    b = 155 + (126 - 155) * u;
  } else if (t <= 0.60) {
    const u = (t - 0.35) / 0.25;
    r = 208 + (122 - 208) * u;
    g = 168 + (78 - 168) * u;
    b = 126 + (48 - 126) * u;
  } else {
    const u = (t - 0.60) / 0.40;
    r = 122 + (11 - 122) * u;
    g = 78 + (11 - 78) * u;
    b = 48 + (11 - 48) * u;
  }
  const toHex = (n: number) => Math.round(n).toString(16).padStart(2, '0');
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
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
