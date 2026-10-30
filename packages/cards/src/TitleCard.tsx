import {Frame, type BaseProps} from './Frame';
import {palette as p} from './theme';
import {Reveal, staggerBeats} from './motion';

export type TitleProps = BaseProps & {items: {title: string; emphasis?: string; detail?: string; color?: 'red' | 'green'}[]};

export const titleBeats = (props: TitleProps) => staggerBeats(props.items.length, 5, 8);

export function TitleCard(props: TitleProps) {
  return <Frame {...props} beats={titleBeats(props)}>{scale => 
    <div style={{display: 'flex', flexDirection: 'column', gap: Math.round(24 * scale), justifyContent: 'center', flex: 1}}>
      {props.items.map((item, i) => <Reveal key={i} delay={i * 5} dur={8}><div style={{textAlign: 'center', marginBottom: Math.round(16 * scale)}}>
        <div style={{fontSize: Math.round(44 * scale), color: item.color ? p[item.color] : p.gold, fontWeight: 800, lineHeight: 1.1}}>{item.title}</div>
        {item.emphasis && <div style={{fontSize: Math.round(72 * scale), color: p.cream, fontWeight: 900, margin: `${Math.round(8 * scale)}px 0`, lineHeight: 1.05}}>{item.emphasis}</div>}
        {item.detail && <div style={{fontSize: Math.round(32 * scale), color: p.mute, marginTop: Math.round(8 * scale), lineHeight: 1.2}}>{item.detail}</div>}
      </div></Reveal>)}
    </div>
  }</Frame>;
}
