import {Img, staticFile} from 'remotion';
import {Frame, type BaseProps} from './Frame';
import {palette as p} from '../theme';

export type Callout = {shape: 'rect' | 'ellipse'; x: number; y: number; w: number; h: number; label?: string};
// A still frame from real footage with outlined callouts. Callout boxes are fractions
// (0 to 1) of the image, so they stay attached to the picture at any rendered size.
export type FreezeProps = BaseProps & {image: string; aspect: number; callouts: Callout[]; caption?: string};

export function FreezeCallout(props: FreezeProps) {
  if (/^(\/|[a-z]+:)/i.test(props.image) || props.image.split('/').includes('..')) {
    throw new Error('image must be a relative path under public/');
  }
  return <Frame {...props} qr={props.qr ?? false}>{scale => <>
    <div style={{flex: 1, minHeight: 0, display: 'flex', justifyContent: 'center'}}>
      <div style={{height: '100%', aspectRatio: String(props.aspect), position: 'relative'}}>
        <Img src={staticFile(props.image)} alt="" style={{display: 'block', width: '100%', height: '100%'}} />
        {props.callouts.map((c, i) => <div key={i} data-callout style={{
          position: 'absolute', left: `${c.x * 100}%`, top: `${c.y * 100}%`, width: `${c.w * 100}%`, height: `${c.h * 100}%`,
          boxSizing: 'border-box', border: `${Math.round(6 * scale)}px solid ${p.gold}`,
          borderRadius: c.shape === 'ellipse' ? '50%' : 8, boxShadow: '0 0 0 2px rgba(15,27,51,0.6)',
        }} />)}
      </div>
    </div>
    {props.caption && <div style={{fontSize: Math.round(30 * scale), textAlign: 'center', color: p.cream, marginTop: Math.round(12 * scale)}}>{props.caption}</div>}
  </>}</Frame>;
}
