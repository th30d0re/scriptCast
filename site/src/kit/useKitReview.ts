import {useEffect, useRef, useState} from 'react';

export type Level = 'P0'|'P1'|'P2'|'P3'|null;
export const VARIANTS = ['default', 'opus', 'argon', 'astra', 'kimi'] as const;
export type VariantName = typeof VARIANTS[number];
export type ReviewItem = {priority: Level; variants?: Partial<Record<VariantName, Level>>; status: 'unreviewed'|'keep'|'change'|'cut'; notes: string; tags: string[]; updated: string};
export type KitReview = {version: 1; items: Record<string, ReviewItem>};
export const emptyItem = (): ReviewItem => ({priority: null, status: 'unreviewed', notes: '', tags: [], updated: new Date().toISOString()});
const storageKey = 'scriptcast-kit-review-unsaved';

export function useKitReview() {
  const [review, setReview] = useState<KitReview>({version: 1, items: {}});
  const [state, setState] = useState<'loading'|'saved'|'saving'|'unsaved'|'error'>('loading');
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => {
    fetch('/api/kit-review').then(async (response) => {
      if (!response.ok) throw new Error(await response.text());
      return response.json() as Promise<KitReview>;
    }).then((server) => {
      const cached = localStorage.getItem(storageKey);
      setReview(cached ? {...server, items: {...server.items, ...JSON.parse(cached).items}} : server);
      setState(cached ? 'unsaved' : 'saved');
    }).catch(() => setState('error'));
    return () => window.clearTimeout(timer.current);
  }, []);

  const updateItem = (id: string, update: Partial<Omit<ReviewItem, 'updated'>>) => {
    setReview((previous) => {
      const item = {...(previous.items[id] ?? emptyItem()), ...update, updated: new Date().toISOString()};
      const next = {...previous, items: {...previous.items, [id]: item}};
      localStorage.setItem(storageKey, JSON.stringify(next));
      window.clearTimeout(timer.current);
      timer.current = window.setTimeout(() => {
        setState('saving');
        fetch('/api/kit-review', {method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({id, item})})
          .then((response) => { if (!response.ok) throw new Error('Save failed'); localStorage.removeItem(storageKey); setState('saved'); })
          .catch(() => setState('unsaved'));
      }, 500);
      return next;
    });
  };
  return {review, updateItem, state};
}
