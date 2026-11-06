export type ReviewNote = {
  scene: string;
  element: string;
  ms: number;
  cue: string;
  note: string;
  author: string;
  status: 'open';
};

export function parseNotes(value: unknown): ReviewNote[] | null {
  if (!Array.isArray(value)) return null;
  const notes: ReviewNote[] = [];
  for (const item of value) {
    if (!item || typeof item !== 'object') return null;
    const record = item as Record<string, unknown>;
    const {scene, element, ms, cue, note, author, status} = record;
    if (typeof scene !== 'string' || typeof element !== 'string' || typeof ms !== 'number' || !Number.isFinite(ms)) return null;
    if (typeof cue !== 'string' || typeof note !== 'string' || typeof author !== 'string' || status !== 'open') return null;
    notes.push({scene, element, ms, cue, note, author, status: 'open'});
  }
  return notes;
}
