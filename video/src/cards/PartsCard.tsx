import {Img, staticFile} from 'remotion';
import {Frame, type BaseProps} from './Frame';
import {palette as p} from '../theme';

// Rows of a picture (a transparent PNG cut-out under public/) beside a name and what it does.
export type PartsProps = BaseProps & {parts: {image: string; title: string; detail: string}[]};

export function PartsCard(props: PartsProps) {
  for (const part of props.parts) {
    if (/^(\/|[a-z]+:)/i.test(part.image) || part.image.split('/').includes('..') || !/\.(png|svg|webp)$/.test(part.image)) {
      throw new Error('part image must be a relative .png, .svg or .webp path under public/');
    }
  }
  return <Frame {...props}>{scale =>
    <div style={{display: 'flex', flexDirection: 'column', gap: Math.round(14 * scale), flex: 1, justifyContent: 'space-evenly'}}>
      {props.parts.map((part, i) => <div key={i} data-part style={{display: 'flex', alignItems: 'center', gap: Math.round(24 * scale)}}>
        <div style={{width: Math.round(300 * scale), height: Math.round(150 * scale), flexShrink: 0, boxSizing: 'border-box', padding: Math.round(10 * scale), background: p.cream, borderRadius: 12}}>
          <Img src={staticFile(part.image)} alt="" style={{display: 'block', width: '100%', height: '100%', objectFit: 'contain'}} />
        </div>
        <div style={{flex: 1}}>
          <div style={{fontSize: Math.round(38 * scale), color: p.gold, fontWeight: 800, lineHeight: 1.1}}>{part.title}</div>
          <div style={{fontSize: Math.round(30 * scale), color: p.cream, marginTop: Math.round(4 * scale), lineHeight: 1.2}}>{part.detail}</div>
        </div>
      </div>)}
    </div>
  }</Frame>;
}
