import {VARIANTS, type Level, type ReviewItem, type VariantName} from './useKitReview';

const LEVELS = ['P0', 'P1', 'P2', 'P3'] as const;
const levelSelect = (value: Level, onPick: (level: Level) => void) => <select value={value ?? ''} onChange={(event) => onPick((event.target.value || null) as Level)}><option value="">None</option>{LEVELS.map((level) => <option key={level}>{level}</option>)}</select>;

export function ReviewPanel({source, item, motion, variant, onPickVariant, onChange, saveState}: {source: string; item: ReviewItem; motion?: boolean; variant: VariantName; onPickVariant: (v: VariantName) => void; onChange: (patch: Partial<Omit<ReviewItem, 'updated'>>) => void; saveState: string}) {
  const setLevel = (name: VariantName, level: Level) => onChange({variants: {...item.variants, [name]: level}});
  return <aside className="review-panel"><p className="source">src/stage/kit/{source}</p>
    <label>Notes <textarea value={item.notes} onChange={(event) => onChange({notes: event.target.value})} placeholder="Markdown notes…" /></label>
    <label>Priority {levelSelect(item.priority, (priority) => onChange({priority}))}</label>
    {motion && <fieldset className="variant-levels"><legend>Variant levels (P0 = dominant)</legend>{VARIANTS.map((name) => <div key={name} className={`variant-level${name === variant ? ' viewing' : ''}`}><button type="button" onClick={() => onPickVariant(name)} title="Show this variant">{name === variant ? '▶ ' : ''}{name}</button>{levelSelect(item.variants?.[name] ?? null, (level) => setLevel(name, level))}</div>)}</fieldset>}
    <label>Status <select value={item.status} onChange={(event) => onChange({status: event.target.value as ReviewItem['status']})}>{['unreviewed', 'keep', 'change', 'cut'].map((value) => <option key={value} value={value}>{value[0].toUpperCase() + value.slice(1)}</option>)}</select></label>
    <label>Tags <input value={item.tags.join(', ')} onChange={(event) => onChange({tags: event.target.value.split(',').map((tag) => tag.trim()).filter(Boolean)})} placeholder="layout, motion" /></label>
    <p className={`save ${saveState}`}>{saveState === 'saved' ? '✓ Saved' : saveState}</p></aside>;
}
