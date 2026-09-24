import {Img, staticFile, continueRender, delayRender} from 'remotion';
import {ReactNode, useLayoutEffect, useRef, useState} from 'react';
import {USABLE_BOX, WIDTH, HEIGHT} from '../safeZone';
import {palette as p, fontFamily} from '../theme';
import {QrCode} from './QrCode';

export type ArtProp = {src: string; size?: 'strip' | 'hero'; caption?: string};
export type BaseProps = {headline: string; sources: string; svgAsset?: string; art?: ArtProp; qr?: boolean; qrUrl?: string};

// Type sizes are fixed at their base values; Frame shrinks the content block
// until it fits (measured in the browser). Kept for callers that pass it through.
export function computeDensity(itemCount: number, textLength: number) {
  void itemCount; void textLength;
  return 1.0;
  // eslint-disable-next-line no-unreachable
  let scale = 1.0;
  if (textLength > 200 || itemCount > 2) scale = 0.9;
  if (textLength > 300 || itemCount > 3) scale = 0.8;
  if (textLength > 450 || itemCount >= 5) scale = 0.72;
  if (textLength > 600 || itemCount >= 6) scale = 0.65;
  return scale;
}

export function getTextLength(obj: any): number {
  if (typeof obj === 'string') return obj.length;
  if (typeof obj === 'number') return String(obj).length;
  if (Array.isArray(obj)) return obj.reduce((sum: number, v) => sum + getTextLength(v), 0);
  if (obj && typeof obj === 'object') {
    return Object.values(obj).reduce((sum: number, v) => sum + getTextLength(v), 0);
  }
  return 0;
}

// Fitted scale per card, kept for the life of the page so later frames of the
// same card start at the answer instead of re-running the shrink loop.
const fittedScale = new Map<string, number>();

export function Frame({headline, sources, svgAsset, art, qr = true, qrUrl, children}: BaseProps & {children: (scale: number) => ReactNode}) {
  const effectiveArt = art || (svgAsset ? {src: svgAsset, size: 'strip' as const} : undefined);
  
  if (effectiveArt?.src) {
    const src = effectiveArt.src;
    if (/^(\/|[a-z]+:)/i.test(src) || src.split('/').includes('..') || /[\\?#%]/.test(src) || !src.endsWith('.svg')) {
      throw new Error('art.src must be a relative path under public/ and end with .svg');
    }
  }

  const [handle] = useState(() => delayRender());
  const cacheKey = `${headline}\u0000${sources}`;
  const [scale, setScale] = useState(() => fittedScale.get(cacheKey) ?? 1);
  const contentRef = useRef<HTMLElement>(null);

  // Shrink-to-fit: lower the type scale in 5% steps until every descendant of the
  // content block sits inside it (measured in the browser). Fails loudly below the
  // floor instead of clipping.
  useLayoutEffect(() => {
    if (typeof window === 'undefined') return;
    const el = contentRef.current;
    const boxBottom = el ? el.getBoundingClientRect().bottom : 0;
    const bottom = el ? Math.max(0, ...Array.from(el.querySelectorAll('*')).map(n => n.getBoundingClientRect().bottom)) : 0;
    if (el && bottom > boxBottom + 2) {
      if (scale > 0.5) { setScale(sc => Number((sc - 0.05).toFixed(2))); return; }
      throw new Error(`Overflow detected in card: ${headline} (content bottom ${Math.round(bottom)}px, box bottom ${Math.round(boxBottom)}px, scale ${scale})`);
    }
    fittedScale.set(cacheKey, scale);
    continueRender(handle);
  }, [handle, headline, scale, cacheKey]);

  const hero = effectiveArt?.size === 'hero';
  
  return <div style={{width: WIDTH, height: HEIGHT, background: p.navy, color: p.cream, fontFamily}}>
    <main style={{position: 'absolute', left: USABLE_BOX.x, top: USABLE_BOX.y, width: USABLE_BOX.width, height: USABLE_BOX.height, boxSizing: 'border-box', padding: 12, overflow: 'hidden', display: 'flex', flexDirection: 'column', gap: Math.round(16 * scale)}}>
      <div style={{flexShrink: 0, display: 'flex', flexDirection: 'column', gap: Math.round(12 * scale)}}>
        <h1 style={{margin: 0, fontSize: Math.max(32, Math.round(56 * scale)), lineHeight: 1.08, fontWeight: 800, textAlign: 'center'}}>{headline}</h1>
        <div style={{width: 160, height: 4, background: p.gold, alignSelf: 'center'}} />
      </div>

      {effectiveArt?.src && <div data-svg-slot style={{
        width: '100%',
        height: hero ? '34%' : 64,
        flexShrink: 0, 
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: 8
      }}>
        {effectiveArt.src && <Img src={staticFile(effectiveArt.src)} alt="" style={{display: 'block', width: '100%', flex: 1, minHeight: 0, objectFit: 'contain'}} />}
        {effectiveArt.caption && hero && <div style={{fontSize: Math.round(26 * scale), color: p.mute, textAlign: 'center', flexShrink: 0}}>{effectiveArt.caption}</div>}
      </div>}

      <section ref={contentRef} style={{flex: 1, minHeight: 0, overflow: 'hidden', display: 'flex', flexDirection: 'column', justifyContent: 'space-evenly'}}>
        {children(scale)}
      </section>

      <footer style={{textAlign: 'center', flexShrink: 0}}>
        <div style={{fontSize: Math.max(16, Math.round(26 * scale)), lineHeight: 1.2, color: p.mute}}>{sources}</div>
        {qr && (qrUrl ? <div style={{margin: `${Math.round(12 * scale)}px auto 0`, width: 150}}><QrCode url={qrUrl} size={150} /></div> : <div style={{boxSizing: 'border-box', width: 120, height: 120, border: `3px solid ${p.mute}`, borderRadius: 10, margin: `${Math.round(12 * scale)}px auto 0`, display: 'flex', alignItems: 'center', justifyContent: 'center', color: p.mute, fontSize: 30, fontWeight: 700}}>QR</div>)}
      </footer>
    </main>
  </div>;
}
