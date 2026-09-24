import {Frame, type BaseProps, computeDensity, getTextLength} from './Frame';
import {palette as p} from '../theme';

export type StatBarsProps = BaseProps & {lead?: string; items: {label: string; sublabel?: string; period: string; values: number[]; delta?: string; color: string}[]; quote?: string; attribution?: string};

export function barWidths(values: number[]): number[] {
  if (values.some(v => !Number.isFinite(v) || v < 0)) throw new Error('Bar values must be finite and nonnegative');
  const max = Math.max(0, ...values);
  return values.map(v => max === 0 ? 0 : v / max * 100);
}

export function StatBarsCard(props: StatBarsProps) {
  const textLength = getTextLength(props.items) + getTextLength(props.lead || '') + getTextLength(props.quote || '');
  const scale = computeDensity(props.items.length * 2.5, textLength); // count items more heavily since they take vertical space

  return <Frame {...props} scale={scale}>
    <div style={{display: 'flex', flexDirection: 'column', gap: Math.round(12 * scale), minHeight: 0}}>
      {props.lead && <div style={{fontSize: Math.round(32 * scale), lineHeight: 1.2, textAlign: 'center', marginBottom: Math.round(8 * scale)}}>{props.lead}</div>}
      
      {props.items.map((group, i) => <div key={i} style={{marginBottom: Math.round(12 * scale)}}>
        <div style={{display: 'flex', justifyContent: 'space-between', fontSize: Math.round(32 * scale), fontWeight: 800}}>
          <span>{group.label}{group.sublabel && <span style={{color: p.mute, marginLeft: 12, fontSize: Math.round(26 * scale)}}>{group.sublabel}</span>}</span>
          <span style={{color: p.mute, fontSize: Math.round(28 * scale)}}>{group.period}</span>
        </div>
        {barWidths(group.values).map((width, j) => {
          const bg = (group.values.length === 1 || j > 0) ? (p[group.color as keyof typeof p] || group.color) : p.mute;
          return <div key={j} style={{display: 'flex', alignItems: 'center', gap: 12, marginTop: Math.round(8 * scale)}}>
            <div style={{flex: 1, background: p.navy2}}><div data-bar-width={width} style={{width: `${width}%`, height: Math.round(28 * scale), background: bg, borderRadius: 5}} /></div>
            <span style={{fontSize: Math.round(32 * scale), width: Math.round(130 * scale), textAlign: 'right', fontWeight: 800}}>{group.values[j].toLocaleString('en-US')}</span>
          </div>;
        })}
        {group.delta && <div style={{textAlign: 'right', color: p[group.color as keyof typeof p] || group.color, fontSize: Math.round(36 * scale), fontWeight: 900, marginTop: 4}}>{group.delta}</div>}
      </div>)}
      
      {props.quote && <div style={{borderLeft: `5px solid ${p.gold}`, paddingLeft: Math.round(18 * scale), marginTop: Math.round(8 * scale)}}>
        <q style={{fontSize: Math.round(38 * scale), fontWeight: 800, fontStyle: 'italic'}}>{props.quote}</q>
        {props.attribution && <div style={{fontSize: Math.round(28 * scale), color: p.mute, marginTop: 6}}>{props.attribution}</div>}
      </div>}
    </div>
  </Frame>;
}
