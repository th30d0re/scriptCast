import {Frame, type BaseProps} from './Frame';
import {palette as p} from '../theme';

export type TitleProps = BaseProps & {items: {title: string; emphasis?: string; detail?: string}[]};

export function TitleCard(props: TitleProps) {
  return <Frame {...props}>{scale => 
    <div style={{display: 'flex', flexDirection: 'column', gap: Math.round(24 * scale), justifyContent: 'center', flex: 1}}>
      {props.items.map((item, i) => <div key={i} style={{textAlign: 'center', marginBottom: Math.round(16 * scale)}}>
        <div style={{fontSize: Math.round(44 * scale), color: p.gold, fontWeight: 800, lineHeight: 1.1}}>{item.title}</div>
        {item.emphasis && <div style={{fontSize: Math.round(72 * scale), color: p.cream, fontWeight: 900, margin: `${Math.round(8 * scale)}px 0`, lineHeight: 1.05}}>{item.emphasis}</div>}
        {item.detail && <div style={{fontSize: Math.round(32 * scale), color: p.mute, marginTop: Math.round(8 * scale), lineHeight: 1.2}}>{item.detail}</div>}
      </div>)}
    </div>
  }</Frame>;
}
