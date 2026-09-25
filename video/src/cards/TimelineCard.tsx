import {Frame, type BaseProps} from './Frame';
import {palette as p} from '../theme';
import {Reveal} from './motion';

export type TimelineProps = BaseProps & {items: {date: string; title: string; detail: string; highlight?: boolean; badge?: string}[]; note?: string};

export function TimelineCard(props: TimelineProps) {

  return <Frame {...props}>{scale => <div style={{display: 'flex', flexDirection: 'column', gap: Math.round(14 * scale)}}>
    {props.items.map((item, i) => <Reveal key={i} delay={8 + i * 8}><div style={{display: 'grid', gridTemplateColumns: '110px 1fr', gap: Math.round(18 * scale)}}>
      <div style={{fontSize: Math.round(32 * scale), color: p.mute, whiteSpace: 'pre-line', textAlign: 'right', fontWeight: 600}}>{item.date}</div>
      <div style={{borderLeft: `5px solid ${item.highlight ? p.red : p.teal}`, paddingLeft: Math.round(18 * scale)}}>
        {item.badge && <div style={{color: p.gold, fontSize: Math.round(30 * scale), fontWeight: 900}}>{item.badge}</div>}
        <div style={{fontSize: Math.round(42 * scale), lineHeight: 1.1, fontWeight: 800, color: item.highlight ? p.gold : p.cream}}>{item.title}</div>
        <div style={{fontSize: Math.round(32 * scale), lineHeight: 1.2, marginTop: 4, color: item.highlight ? p.cream : p.mute}}>{item.detail}</div>
      </div>
    </div></Reveal>)}
    {props.note && <div style={{border: `3px solid ${p.red}`, borderRadius: 12, padding: Math.round(14 * scale), fontSize: Math.round(32 * scale), textAlign: 'center'}}>{props.note}</div>}
  </div>}</Frame>;
}
