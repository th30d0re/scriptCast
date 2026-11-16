import React, {createContext, useCallback, useContext, useEffect, useLayoutEffect, useMemo, useRef, useState} from 'react';
import {nearestCue, sceneAt} from './cue';
import {nearestMark, parseBox, type Box, type Mark} from './hit';
import {loadNotes, saveNotes} from './storage';
import type {ReviewNote} from './schema';
import type {StagePlan} from '../types';

type Draft = {scene: string; element: string; ms: number; cue: string; note: string; author: string; box: Box};

type ReviewApi = {
  enabled: boolean;
  plan: StagePlan;
  ms: number;
  seek: (ms: number) => void;
  notes: ReviewNote[];
  error: string;
  draft: Draft | null;
  hover: Mark | null;
  highlight: string | null;
  selected: number | null;
  setHover: (mark: Mark | null) => void;
  selectMark: (mark: Mark) => void;
  selectNote: (index: number) => void;
  setDraftNote: (note: string) => void;
  setDraftAuthor: (author: string) => void;
  saveDraft: () => void;
  setStageEl: (el: HTMLElement | null) => void;
  stageRef: React.RefObject<HTMLElement | null>;
};

const ReviewContext = createContext<ReviewApi | null>(null);
const useReview = () => {
  const api = useContext(ReviewContext);
  if (!api) throw new Error('Review UI requires ReviewSession');
  return api;
};

const AUTHOR_KEY = 'stage-review-author';

function shown(el: Element, stage: HTMLElement) {
  let node: Element | null = el;
  while (node) {
    const style = getComputedStyle(node);
    if (style.display === 'none' || style.visibility === 'hidden') return false;
    const opacity = Number(style.opacity);
    if (Number.isFinite(opacity) && opacity < 0.05) return false;
    if (node === stage) return true;
    node = node.parentElement;
  }
  return true;
}

function boxFor(el: Element, stage: HTMLElement, width: number, height: number): Box | null {
  const declared = parseBox(el.getAttribute('data-box'));
  if (declared) return declared;
  const rect = el.getBoundingClientRect();
  const stageRect = stage.getBoundingClientRect();
  if (rect.width <= 0 || rect.height <= 0 || stageRect.width <= 0 || stageRect.height <= 0) return null;
  return {
    x: (rect.left - stageRect.left) * (width / stageRect.width),
    y: (rect.top - stageRect.top) * (height / stageRect.height),
    w: rect.width * (width / stageRect.width),
    h: rect.height * (height / stageRect.height),
  };
}

export function collectMarks(stage: HTMLElement, width: number, height: number): Mark[] {
  const marks: Mark[] = [];
  const nodes = stage.querySelectorAll<Element>('[data-text], [data-shape]');
  for (let index = 0; index < nodes.length; index++) {
    const el = nodes[index];
    if (el.closest('[data-review-ui]')) continue;
    if (!shown(el, stage)) continue;
    const text = el.getAttribute('data-text');
    const shape = el.getAttribute('data-shape');
    const id = text ?? shape;
    if (!id) continue;
    const box = boxFor(el, stage, width, height);
    if (!box) continue;
    marks.push({id, kind: text !== null ? 'text' : 'shape', box});
  }
  return marks;
}

function pointIn(event: {clientX: number; clientY: number}, stage: HTMLElement, width: number, height: number) {
  const rect = stage.getBoundingClientRect();
  return {
    x: (event.clientX - rect.left) * (width / rect.width),
    y: (event.clientY - rect.top) * (height / rect.height),
  };
}

export function ReviewSession({plan, ms, seek, enabled, onEnabled, children}: React.PropsWithChildren<{
  plan: StagePlan; ms: number; seek: (ms: number) => void; enabled: boolean; onEnabled: (on: boolean) => void;
}>) {
  const [notes, setNotes] = useState<ReviewNote[]>([]);
  const [error, setError] = useState('');
  const [draft, setDraft] = useState<Draft | null>(null);
  const [hover, setHoverState] = useState<Mark | null>(null);
  const setHover = useCallback((mark: Mark | null) => {
    setHoverState(current => current?.id === mark?.id && current?.kind === mark?.kind ? current : mark);
  }, []);
  const [highlight, setHighlight] = useState<string | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const [author, setAuthor] = useState('author');
  const stageRef = useRef<HTMLElement | null>(null);
  const setStageEl = useCallback((el: HTMLElement | null) => {stageRef.current = el;}, []);

  useEffect(() => {
    try {const stored = localStorage.getItem(AUTHOR_KEY); if (stored) setAuthor(stored);} catch { /* private mode */ }
  }, []);
  useEffect(() => {
    let active = true;
    loadNotes().then(loaded => {if (active) setNotes(loaded);}, reason => {if (active) setError(String(reason));});
    return () => {active = false;};
  }, []);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const tag = target?.tagName ?? '';
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || target?.isContentEditable) return;
      if (event.metaKey || event.ctrlKey || event.altKey) return;
      if (event.key !== 'r' && event.key !== 'R') return;
      event.preventDefault();
      onEnabled(!enabled);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [enabled, onEnabled]);
  useEffect(() => {if (!enabled) setHover(null);}, [enabled]);

  const selectMark = useCallback((mark: Mark) => {
    setHighlight(mark.id);
    setSelected(null);
    setDraft({
      scene: sceneAt(plan, ms),
      element: mark.id,
      ms: Math.round(ms),
      cue: nearestCue(plan.cues, ms),
      note: '',
      author,
      box: mark.box,
    });
    setError('');
  }, [author, ms, plan]);

  const selectNote = useCallback((index: number) => {
    const note = notes[index];
    if (!note) return;
    setSelected(index);
    setHighlight(note.element);
    setDraft(null);
    seek(note.ms);
  }, [notes, seek]);

  const saveDraft = useCallback(() => {
    if (!draft || !draft.note.trim()) return;
    const note: ReviewNote = {
      scene: draft.scene,
      element: draft.element,
      ms: draft.ms,
      cue: draft.cue,
      note: draft.note.trim(),
      author: draft.author.trim() || 'author',
      status: 'open',
    };
    const next = [...notes, note];
    saveNotes(next).then(() => {
      setNotes(next);
      setAuthor(note.author);
      setDraft(null);
      setError('');
      try {localStorage.setItem(AUTHOR_KEY, note.author);} catch { /* private mode */ }
    }, reason => setError(String(reason)));
  }, [draft, notes]);

  const api = useMemo<ReviewApi>(() => ({
    enabled, plan, ms, seek, notes, error, draft, hover, highlight, selected,
    setHover, selectMark, selectNote, saveDraft, setStageEl, stageRef,
    setDraftNote: note => setDraft(current => current ? {...current, note} : current),
    setDraftAuthor: next => setDraft(current => current ? {...current, author: next} : current),
  }), [draft, enabled, error, highlight, hover, ms, notes, plan, saveDraft, seek, selectMark, selectNote, selected, setStageEl]);

  return <ReviewContext.Provider value={api}>{children}</ReviewContext.Provider>;
}

function Outline({box, selected}: {box: Box; selected: boolean}) {
  return <div data-review-ui style={{
    position: 'absolute', left: box.x, top: box.y, width: box.w, height: box.h, boxSizing: 'border-box',
    outline: selected ? '10px solid #d9a441' : '8px dashed #d9a441', outlineOffset: 0, pointerEvents: 'none',
    background: selected ? 'rgba(217,164,65,0.16)' : 'rgba(217,164,65,0.08)', zIndex: selected ? 6 : 5,
  }} />;
}

export function ReviewHitSurface() {
  const api = useReview();
  const ref = useRef<HTMLDivElement>(null);
  const [frameMarks, setFrameMarks] = useState<Mark[]>([]);
  useLayoutEffect(() => {
    const stage = ref.current?.parentElement ?? null;
    api.setStageEl(stage);
    if (stage) setFrameMarks(collectMarks(stage, api.plan.width, api.plan.height));
    return () => api.setStageEl(null);
  }, [api.enabled, api.ms, api.highlight, api.plan.height, api.plan.width, api.setStageEl]);
  if (!api.enabled) return null;
  const read = (event: {clientX: number; clientY: number}) => {
    const stage = ref.current?.parentElement;
    if (!stage || stage.getBoundingClientRect().width <= 0) return null;
    const point = pointIn(event, stage, api.plan.width, api.plan.height);
    return nearestMark(collectMarks(stage, api.plan.width, api.plan.height), point.x, point.y);
  };
  const selected = frameMarks.filter(mark => mark.id === api.highlight);
  const hover = api.hover && api.hover.id !== api.highlight ? api.hover : null;
  return <div
    ref={ref}
    data-review-ui
    data-review-hit
    style={{position: 'absolute', inset: 0, zIndex: 4, cursor: 'crosshair'}}
    onMouseMove={event => api.setHover(read(event))}
    onMouseLeave={() => api.setHover(null)}
    onClick={event => {const mark = read(event); if (mark) api.selectMark(mark);}}
  >
    {hover && <Outline box={hover.box} selected={false} />}
    {selected.map(mark => <Outline key={`${mark.kind}:${mark.id}:${mark.box.x}:${mark.box.y}`} box={mark.box} selected />)}
  </div>;
}

function screenBox(box: Box, stage: HTMLElement, width: number, height: number) {
  const rect = stage.getBoundingClientRect();
  const sx = rect.width / width;
  const sy = rect.height / height;
  return {left: rect.left + box.x * sx, top: rect.top + box.y * sy, width: box.w * sx, height: box.h * sy};
}

const panel: React.CSSProperties = {
  background: '#101725', color: '#f4ead2', border: '1px solid #d9a441', borderRadius: 8,
  boxShadow: '0 12px 40px rgba(0,0,0,0.35)', font: '14px/1.4 "IBM Plex Sans", system-ui, sans-serif',
};

export function ReviewChrome() {
  const api = useReview();
  if (!api.enabled) return null;
  const stage = api.stageRef.current;
  const hoverLabel = api.hover && stage ? screenBox(api.hover.box, stage, api.plan.width, api.plan.height) : null;
  const draftBox = api.draft && stage ? screenBox(api.draft.box, stage, api.plan.width, api.plan.height) : null;
  const noteStyle: React.CSSProperties = {position: 'fixed', width: 300, zIndex: 50, padding: 12, ...panel};
  if (draftBox) {
    let left = draftBox.left + draftBox.width + 10;
    let top = Math.max(68, draftBox.top);
    if (left + 300 > window.innerWidth - 8) left = Math.max(8, draftBox.left - 310);
    if (top + 280 > window.innerHeight - 8) top = Math.max(68, window.innerHeight - 288);
    noteStyle.left = left;
    noteStyle.top = top;
  }
  const open = api.notes.filter(note => note.status === 'open');
  return <>
    {hoverLabel && <div data-review-ui data-review-hover style={{
      position: 'fixed', left: Math.min(hoverLabel.left, window.innerWidth - 220), top: Math.max(8, hoverLabel.top - 28),
      zIndex: 45, pointerEvents: 'none', padding: '2px 8px', background: '#101725', color: '#d9a441',
      border: '1px solid #d9a441', borderRadius: 4, font: '13px/1.4 "IBM Plex Mono", ui-monospace, monospace',
    }}>{api.hover?.id}</div>}
    <aside data-review-ui data-review-list aria-label="Open notes" style={{
      position: 'fixed', left: 12, top: 72, width: 280, maxHeight: 'calc(100vh - 150px)', overflow: 'auto',
      zIndex: 40, padding: 12, ...panel,
    }}>
      <strong style={{display: 'block', marginBottom: 8}}>Open notes</strong>
      {api.error && <p role="alert" style={{color: '#e2573f', margin: '0 0 8px'}}>{api.error}</p>}
      {open.length === 0 && <p style={{margin: 0, color: '#9fb0cf'}}>No open notes</p>}
      {api.notes.map((note, index) => note.status === 'open' ? <button key={`${note.scene}:${note.element}:${note.ms}:${index}`} type="button" onClick={() => api.selectNote(index)} aria-current={api.selected === index ? 'true' : undefined} style={{
        display: 'block', width: '100%', textAlign: 'left', marginTop: 8, padding: 8, cursor: 'pointer',
        background: api.selected === index ? '#685020' : '#24324c', color: 'inherit', border: '1px solid #687992', borderRadius: 5,
      }}>
        <span style={{display: 'block', font: '12px/1.3 "IBM Plex Mono", ui-monospace, monospace', color: '#d9a441'}}>{note.element}</span>
        <span style={{display: 'block', fontSize: 12, color: '#9fb0cf'}}>{note.scene}</span>
        <span style={{display: 'block', fontSize: 12, color: '#9fb0cf'}}>{note.cue} · {note.ms} ms</span>
        <span style={{display: 'block', marginTop: 4}}>{note.note}</span>
      </button> : null)}
    </aside>
    {api.draft && <form data-review-ui data-review-note style={noteStyle} onSubmit={event => {event.preventDefault(); api.saveDraft();}}>
      <div style={{font: '12px/1.4 "IBM Plex Mono", ui-monospace, monospace', color: '#d9a441', marginBottom: 8}}>{api.draft.element}</div>
      <div style={{marginBottom: 4}}>Scene {api.draft.scene}</div>
      <div style={{marginBottom: 4}}>{api.draft.ms} ms</div>
      <div style={{marginBottom: 8}}>Cue {api.draft.cue || 'none'}</div>
      <label style={{display: 'block', marginBottom: 8}}>Author
        <input aria-label="Author" value={api.draft.author} onChange={event => api.setDraftAuthor(event.target.value)} style={{display: 'block', width: '100%', marginTop: 4, font: 'inherit', color: 'inherit', background: '#0f1b33', border: '1px solid #687992', borderRadius: 4, padding: '4px 6px'}} />
      </label>
      <label style={{display: 'block', marginBottom: 8}}>Note
        <textarea aria-label="Note" required rows={4} value={api.draft.note} onChange={event => api.setDraftNote(event.target.value)} style={{display: 'block', width: '100%', marginTop: 4, resize: 'vertical', font: 'inherit', color: 'inherit', background: '#0f1b33', border: '1px solid #687992', borderRadius: 4, padding: '4px 6px'}} />
      </label>
      <button type="submit" disabled={!api.draft.note.trim()}>Save note</button>
    </form>}
  </>;
}
