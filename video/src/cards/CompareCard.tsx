import {Frame, type BaseProps} from './Frame';
import {palette as p} from '../theme';

export type CompareProps = BaseProps & {items: {title: string; points: string[]}[]; verdict?: string};

export function CompareCard(props: CompareProps) {
  return <Frame {...props}>{scale => 
    <div style={{display: 'flex', flexDirection: 'column', gap: Math.round(20 * scale), flex: 1, minHeight: 0}}>
      <div style={{display: 'flex', gap: Math.round(20 * scale), flex: 1, minHeight: 0}}>
        {props.items.map((item, i) => <div key={i} style={{flex: 1, minWidth: 0, background: p.navy2, padding: Math.round(24 * scale), borderRadius: 12, display: 'flex', flexDirection: 'column', gap: Math.round(16 * scale)}}>
          <h2 style={{fontSize: Math.round(40 * scale), color: p.gold, margin: 0, fontWeight: 800, textAlign: 'center'}}>{item.title}</h2>
          <div style={{width: 60, height: 3, background: p.teal, alignSelf: 'center'}} />
          <ul style={{margin: 0, padding: 0, listStyle: 'none', display: 'flex', flexDirection: 'column', gap: Math.round(16 * scale)}}>
            {item.points.map((point, j) => <li key={j} style={{fontSize: Math.round(30 * scale), lineHeight: 1.2, color: p.cream, display: 'flex', gap: 12}}>
              <span style={{color: p.mute}}>•</span>
              <span>{point}</span>
            </li>)}
          </ul>
        </div>)}
      </div>
      {props.verdict && <div style={{background: p.navy2, padding: Math.round(20 * scale), borderRadius: 12, textAlign: 'center', fontSize: Math.round(32 * scale), fontWeight: 800, color: p.gold, border: `2px solid ${p.red}`}}>{props.verdict}</div>}
    </div>
  }</Frame>;
}
