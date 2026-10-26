import {Img, staticFile} from 'remotion';
import {Frame, type BaseProps} from './Frame';
import {palette as p, fontFamily} from '../theme';
import {USABLE_BOX, WIDTH, HEIGHT} from '../safeZone';
import {QrCode} from './QrCode';
import {Reveal} from './motion';

export type Callout = {shape: 'rect' | 'ellipse'; x: number; y: number; w: number; h: number; label?: string};
// A still frame from real footage with outlined callouts. Callout boxes are fractions
// (0 to 1) of the image, so they stay attached to the picture at any rendered size.
// fullBleed lays the picture across the whole frame at full width, as a background, with the
// headline, caption and QR in a fading navy scrim over its top. imageTop is the picture's top edge in px.
export type FreezeProps = BaseProps & {image: string; aspect: number; callouts: Callout[]; caption?: string; fullBleed?: boolean; imageTop?: number};

function BleedLayout(props: FreezeProps) {
  const top = props.imageTop ?? 380;
  const imgH = Math.round(WIDTH / props.aspect);
  const scrim = (dir: string) => `linear-gradient(${dir}, rgba(15,27,51,0.96) 0%, rgba(15,27,51,0.9) 62%, rgba(15,27,51,0) 100%)`;
  return <div style={{width: WIDTH, height: HEIGHT, background: p.navy, color: p.cream, fontFamily, position: 'relative', overflow: 'hidden'}}>
    <div style={{position: 'absolute', left: 0, top, width: WIDTH, height: imgH}}>
      <Img src={staticFile(props.image)} alt="" style={{display: 'block', width: '100%', height: '100%'}} />
      {props.callouts.map((c, i) => <div key={i} data-callout style={{
        position: 'absolute', left: `${c.x * 100}%`, top: `${c.y * 100}%`, width: `${c.w * 100}%`, height: `${c.h * 100}%`,
        boxSizing: 'border-box', border: `6px solid ${p.gold}`, borderRadius: c.shape === 'ellipse' ? '50%' : 8, boxShadow: '0 0 0 2px rgba(15,27,51,0.6)',
      }} />)}
    </div>
    <div style={{position: 'absolute', left: 0, top: 0, width: WIDTH, height: top + 90, background: scrim('to bottom')}} />
    <div style={{position: 'absolute', left: USABLE_BOX.x, top: USABLE_BOX.y, width: USABLE_BOX.width}}>
      <div style={{textAlign: 'center'}}>
        <h1 style={{margin: 0, fontSize: 56, lineHeight: 1.08, fontWeight: 800}}>{props.headline}</h1>
        <div style={{width: 160, height: 4, background: p.gold, margin: '12px auto 0'}} />
      </div>
      <div style={{display: 'flex', alignItems: 'center', gap: 20, marginTop: 14}}>
        <div style={{flex: 1, textAlign: 'center'}}>
          {props.caption && <div style={{fontSize: 30, lineHeight: 1.18}}>{props.caption}</div>}
          <div style={{fontSize: 22, lineHeight: 1.2, color: p.mute, marginTop: 8}}>{props.sources}</div>
        </div>
        {props.qr && props.qrUrl && <div style={{width: 110, flexShrink: 0}}><QrCode url={props.qrUrl} size={110} /></div>}
      </div>
    </div>
  </div>;
}

export function FreezeCallout(props: FreezeProps) {
  if (/^(\/|[a-z]+:)/i.test(props.image) || props.image.split('/').includes('..')) {
    throw new Error('image must be a relative path under public/');
  }
  if (props.fullBleed) return <BleedLayout {...props} />;
  return <Frame {...props} qr={props.qr ?? false}>{scale => <>
    <Reveal delay={0} dur={8} style={{flex: 1, minHeight: 0, display: 'flex', justifyContent: 'center'}}>
      <div style={{height: '100%', aspectRatio: String(props.aspect), position: 'relative'}}>
        <Img src={staticFile(props.image)} alt="" style={{display: 'block', width: '100%', height: '100%'}} />
        {props.callouts.map((c, i) => <div key={i} data-callout style={{
          position: 'absolute', left: `${c.x * 100}%`, top: `${c.y * 100}%`, width: `${c.w * 100}%`, height: `${c.h * 100}%`,
          boxSizing: 'border-box', border: `${Math.round(6 * scale)}px solid ${p.gold}`,
          borderRadius: c.shape === 'ellipse' ? '50%' : 8, boxShadow: '0 0 0 2px rgba(15,27,51,0.6)',
        }} />)}
      </div>
    </Reveal>
    {props.caption && <div style={{fontSize: Math.round(30 * scale), textAlign: 'center', color: p.cream, marginTop: Math.round(12 * scale)}}>{props.caption}</div>}
  </>}</Frame>;
}
