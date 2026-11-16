export type Box = {x: number; y: number; w: number; h: number};
export type Mark = {id: string; kind: 'text' | 'shape'; box: Box};

export function parseBox(value: string | null | undefined): Box | null {
  if (!value) return null;
  const parts = value.trim().split(/[\s,]+/).map(Number);
  if (parts.length !== 4 || parts.some(part => !Number.isFinite(part))) return null;
  const [x, y, w, h] = parts;
  if (w <= 0 || h <= 0) return null;
  return {x, y, w, h};
}

const area = (box: Box) => box.w * box.h;

export function contains(box: Box, x: number, y: number) {
  return x >= box.x && y >= box.y && x <= box.x + box.w && y <= box.y + box.h;
}

export function distanceToBox(box: Box, x: number, y: number) {
  const dx = x < box.x ? box.x - x : x > box.x + box.w ? x - (box.x + box.w) : 0;
  const dy = y < box.y ? box.y - y : y > box.y + box.h ? y - (box.y + box.h) : 0;
  return Math.hypot(dx, dy);
}

/** Smallest box under the point, otherwise the box closest to the point. */
export function nearestMark(marks: readonly Mark[], x: number, y: number): Mark | null {
  let best: Mark | null = null;
  let bestInside = false;
  let bestScore = Infinity;
  for (const mark of marks) {
    const inside = contains(mark.box, x, y);
    const score = inside ? area(mark.box) : distanceToBox(mark.box, x, y);
    const better = inside ? !bestInside || score < bestScore : !bestInside && score < bestScore;
    if (!best || better) {
      best = mark;
      bestInside = inside;
      bestScore = score;
    }
  }
  return best;
}
