import {parseNotes, type ReviewNote} from './schema';

export async function loadNotes(): Promise<ReviewNote[]> {
  const response = await fetch('/review/notes.json', {headers: {Accept: 'application/json'}});
  if (!response.ok) throw new Error(`Notes HTTP ${response.status}`);
  const notes = parseNotes(await response.json());
  if (!notes) throw new Error('review/notes.json is not a list of open notes');
  return notes;
}

export async function saveNotes(notes: readonly ReviewNote[]): Promise<void> {
  const response = await fetch('/review/notes.json', {
    method: 'PUT',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(notes, null, 2) + '\n',
  });
  if (!response.ok) throw new Error(`Could not save notes (HTTP ${response.status})`);
}
