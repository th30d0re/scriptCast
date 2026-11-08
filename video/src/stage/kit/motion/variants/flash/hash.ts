// Deterministic pseudo-random hashing functions for pure frame-based animation.
// Frame and seed are hashed using a 32-bit integer mixer (MurmurHash3 / Mulberry32 style).

export function hashInt(seed: number, frame: number): number {
  let a = (seed ^ Math.imul(frame, 0x9e3779b9)) >>> 0;
  a = Math.imul(a ^ (a >>> 16), 0x21f0aaad);
  a = Math.imul(a ^ (a >>> 15), 0x735a2d97);
  return (a ^ (a >>> 15)) >>> 0;
}

export function hashFloat(seed: number, frame: number): number {
  return hashInt(seed, frame) / 4294967296;
}

export function stringToSeed(str: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}
