import {Frame, type BaseProps} from './Frame';
import {palette as p} from '../theme';
export type TimelineProps = BaseProps & {items: {date: string; title: string; detail: string; highlight?: boolean; badge?: string}[]; note?: string};
export function TimelineCard(props: TimelineProps) {
  return <Frame {...props}><div style={{display: 'flex', flexDirection: 'column', gap: 12}}>
    {props.items.map((item, i) => <div key={i} style={{display: 'grid', gridTemplateColumns: '90px 1fr', gap: 14}}>
      <div style={{fontSize: 20, color: p.mute, whiteSpace: 'pre-line', textAlign: 'right'}}>{item.date}</div>
      <div style={{borderLeft: `4px solid ${item.highlight ? p.red : p.teal}`, paddingLeft: 14}}>
        {item.badge && <div style={{color: p.gold, fontSize: 21, fontWeight: 900}}>{item.badge}</div>}
        <div style={{fontSize: 27, lineHeight: 1.1, fontWeight: 800, color: item.highlight ? p.gold : p.cream}}>{item.title}</div>
        <div style={{fontSize: 20, lineHeight: 1.2, marginTop: 3, color: item.highlight ? p.cream : p.mute}}>{item.detail}</div>
      </div>
    </div>)}
    {props.note && <div style={{border: `2px solid ${p.red}`, borderRadius: 12, padding: 10, fontSize: 21, textAlign: 'center'}}>{props.note}</div>}
  </div></Frame>;
}
