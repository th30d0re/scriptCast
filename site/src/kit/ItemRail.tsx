import {KitSheetIndex} from '../../../video/src/stage/kit/KitSheet';
import type {KitReview} from './useKitReview';

type Item = (typeof KitSheetIndex)[number];
export function ItemRail({items, review, selected, onSelect}: {items: readonly Item[]; review: KitReview; selected: string; onSelect: (id: string) => void}) {
  const groups = [['Components', items.filter((item) => item.source.startsWith('components/'))], ['Primitives', items.filter((item) => item.source.startsWith('primitives/'))], ['Motion', items.filter((item) => item.motion)]] as const;
  const orphaned = Object.keys(review.items).filter((id) => !items.some((item) => item.id === id));
  return <aside className="kit-rail"><h2>Kit review</h2>{groups.map(([name, group]) => <section key={name}><h3>{name} <small>{group.length}</small></h3>{group.map((item) => {
    const value = review.items[item.id]; return <button key={item.id} className={selected === item.id ? 'selected' : ''} onClick={() => onSelect(item.id)}><span className={`status ${value?.status ?? 'unreviewed'}`} /><span>{item.id}</span>{value?.priority && <b>{value.priority}</b>}</button>;
  })}</section>)}{orphaned.length > 0 && <section><h3>Orphaned</h3>{orphaned.map((id) => <div className="orphan" key={id}>{id}</div>)}</section>}</aside>;
}
