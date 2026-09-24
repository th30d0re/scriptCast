import {Frame, type BaseProps} from './Frame';
import {palette as p} from '../theme';
export type StatBarsProps = BaseProps & {lead?: string; items: {label: string; period: string; values: number[]; delta: string; color: 'red' | 'gold' | 'teal'}[]; quote?: string; attribution?: string};
export function barWidths(values: number[]): number[] {
  if (values.some(v => !Number.isFinite(v) || v < 0)) throw new Error('Bar values must be finite and nonnegative');
  const max = Math.max(0, ...values);
  return values.map(v => max === 0 ? 0 : v / max * 100);
}
export function StatBarsCard(props: StatBarsProps) {
  return <Frame {...props}>
    {props.lead && <div style={{fontSize: 24, lineHeight: 1.2, textAlign: 'center', marginBottom: 16}}>{props.lead}</div>}
    {props.items.map((group, i) => <div key={i} style={{marginBottom: 14}}>
      <div style={{display: 'flex', justifyContent: 'space-between', fontSize: 25, fontWeight: 800}}><span>{group.label}</span><span style={{color: p.mute, fontSize: 21}}>{group.period}</span></div>
      {barWidths(group.values).map((width, j) => <div key={j} style={{display: 'flex', alignItems: 'center', gap: 12, marginTop: 5}}>
        <div style={{flex: 1, background: p.navy2}}><div data-bar-width={width} style={{width: `${width}%`, height: 28, background: j === 0 ? p.mute : p[group.color], borderRadius: 5}} /></div>
        <span style={{fontSize: 25, width: 115, textAlign: 'right', fontWeight: 800}}>{group.values[j].toLocaleString('en-US')}</span>
      </div>)}
      <div style={{textAlign: 'right', color: p[group.color], fontSize: 30, fontWeight: 900}}>{group.delta}</div>
    </div>)}
    {props.quote && <div style={{borderLeft: `5px solid ${p.gold}`, paddingLeft: 16}}><q style={{fontSize: 31, fontWeight: 800}}>{props.quote}</q><div style={{fontSize: 21, color: p.mute, marginTop: 6}}>{props.attribution}</div></div>}
  </Frame>;
}
