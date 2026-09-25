import {Img, staticFile} from 'remotion';
import {Frame, type BaseProps} from './Frame';
import {palette as p} from '../theme';
import {Reveal} from './motion';

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
      {props.parts.map((part, i) => <Reveal key={i} delay={8 + i * 8}><div data-part style={{display: 'flex', alignItems: 'center', gap: Math.round(24 * scale)}}>
        <div style={{width: Math.round(380 * scale), height: Math.round(180 * scale), flexShrink: 0}}>
          <Img src={staticFile(part.image)} alt="" style={{display: 'block', width: '100%', height: '100%', objectFit: 'contain', filter: 'brightness(2) contrast(1.1) drop-shadow(0 0 5px rgba(244,234,210,0.55))'}} />
        </div>
        <div style={{flex: 1}}>
          <div style={{fontSize: Math.round(38 * scale), color: p.gold, fontWeight: 800, lineHeight: 1.1}}>{part.title}</div>
          <div style={{fontSize: Math.round(30 * scale), color: p.cream, marginTop: Math.round(4 * scale), lineHeight: 1.2}}>{part.detail}</div>
        </div>
      </div></Reveal>)}
    </div>
  }</Frame>;
}
