import {Frame, type BaseProps} from './Frame';
import {palette as p} from '../theme';
export type CompareProps = BaseProps & {items: {title: string; points: string[]}[]};
export function CompareCard(props: CompareProps) {
  return <Frame {...props}><div style={{display: 'flex', gap: 18}}>{props.items.map((item, i) => <div key={i} style={{flex: 1, minWidth: 0, background: p.navy2, padding: 18}}><h2 style={{fontSize: 32, color: p.gold}}>{item.title}</h2>{item.points.map((point, j) => <p key={j} style={{fontSize: 26}}>{point}</p>)}</div>)}</div></Frame>;
}
