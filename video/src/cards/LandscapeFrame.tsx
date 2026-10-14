import {useLayoutEffect, useRef, useState, type ReactNode} from 'react';
import {continueRender, delayRender, Img, staticFile} from 'remotion';
import {LANDSCAPE} from '../safeZone';
import {palette as p, fontFamily} from '../theme';
import {AnimatedSvg, type BaseProps} from './Frame';
import {QrCode} from './QrCode';
import {Reveal, useEnter} from './motion';

export function LandscapeFrame({headline, highlight, sources, art, svgAsset, qr = true, qrUrl, qrs, children}: BaseProps & {children: (scale: number) => ReactNode}) {
  const effectiveArt = art ?? (svgAsset ? {src: svgAsset, size: 'strip' as const} : undefined);
  if (effectiveArt && (/^(\/|[a-z]+:)/i.test(effectiveArt.src) || effectiveArt.src.split('/').includes('..') || /[\\?#%]/.test(effectiveArt.src) || !effectiveArt.src.endsWith('.svg'))) {
    throw new Error('art.src must be a relative path under public/ and end with .svg');
  }
  const [handle] = useState(() => delayRender('Fit landscape card'));
  const [scale, setScale] = useState(1);
  const [fontsReady, setFontsReady] = useState(false);
  const main = useRef<HTMLElement>(null);
  const body = useRef<HTMLElement>(null);
  useLayoutEffect(() => {
    let active = true;
    document.fonts.ready.then(() => {if (active) setFontsReady(true);});
    return () => {active = false;};
  }, []);
  useLayoutEffect(() => {
    if (!fontsReady || !main.current || !body.current) return;
    const fits = (el: HTMLElement) => {
      const box = el.getBoundingClientRect();
      return el.scrollWidth <= el.clientWidth + 2 && el.scrollHeight <= el.clientHeight + 2 &&
        Array.from(el.querySelectorAll('*')).every(n => {
          // Accessibility math and SVG definitions have no visible layout box.
          if (n.closest('.katex-mathml') || (n instanceof SVGElement && n.tagName.toLowerCase() !== 'svg')) return true;
          const r = n.getBoundingClientRect();
          if (r.width === 0 && r.height === 0) return true;
          return r.right <= box.right + 2 && r.bottom <= box.bottom + 2 && r.left >= box.left - 2 && r.top >= box.top - 2;
        });
    };
    if (!fits(main.current) || !fits(body.current)) {
      if (scale > 0.5) {setScale(s => Number((s - 0.05).toFixed(2))); return;}
      throw new Error(`Landscape overflow: ${headline} at scale ${scale}`);
    }
    continueRender(handle);
  }, [fontsReady, scale, handle, headline]);
  const rule = useEnter(0, 8);
  const at = highlight ? headline.indexOf(highlight) : -1;
  const codes = qrs ?? (qr ? [{url: qrUrl, label: ''}] : []);
  return <div style={{width: LANDSCAPE.width, height: LANDSCAPE.height, background: p.navy, color: p.cream, fontFamily}}>
    <main ref={main} data-layout="landscape" style={{position: 'absolute', left: LANDSCAPE.box.x, top: LANDSCAPE.box.y, width: LANDSCAPE.box.width, height: LANDSCAPE.box.height, boxSizing: 'border-box', padding: 12, display: 'flex', flexDirection: 'column', gap: 20 * scale, overflow: 'hidden', overflowWrap: 'anywhere'}}>
      <header style={{flexShrink: 0, textAlign: 'center'}}>
        <h1 style={{margin: 0, fontSize: 56 * scale, lineHeight: 1.08, fontWeight: 800}}>{at >= 0 && highlight ? <>{headline.slice(0, at)}<span data-highlight style={{borderBottom: `4px solid ${p.gold}`}}>{highlight}</span>{headline.slice(at + highlight.length)}</> : headline}</h1>
        {!highlight && <div style={{width: 160 * rule, height: 4, background: p.gold, margin: '12px auto 0'}} />}
      </header>
      <div style={{display: 'flex', flex: 1, minHeight: 0, gap: 36}}>
        <section ref={body} style={{flex: effectiveArt ? 1.4 : 1, minWidth: 0, minHeight: 0, display: 'flex', flexDirection: 'column', justifyContent: 'space-evenly', overflow: 'hidden'}}>{children(scale)}</section>
        {effectiveArt && <aside data-svg-slot style={{flex: 1, minWidth: 0, minHeight: 0, display: 'flex', flexDirection: 'column', gap: 12}}>
          {effectiveArt.animate ? <AnimatedSvg src={effectiveArt.src} atSeconds={effectiveArt.atSeconds} /> : <Img src={staticFile(effectiveArt.src)} alt="" style={{width: '100%', flex: 1, minHeight: 0, objectFit: 'contain'}} />}
          {effectiveArt.caption && <div style={{fontSize: 26 * scale, color: p.mute, textAlign: 'center'}}>{effectiveArt.caption}</div>}
        </aside>}
      </div>
      <Reveal delay={6} dur={8} style={{flexShrink: 0}}><footer style={{display: 'flex', alignItems: 'flex-end', gap: 24}}>
        <div style={{flex: 1, minWidth: 0, fontSize: 26 * scale, lineHeight: 1.2, color: p.mute}}>{sources}</div>
        {codes.map((code, i) => <div key={i} style={{textAlign: 'center', flexShrink: 0}}>{code.url ? <QrCode url={code.url} size={120} /> : <div style={{width: 120, height: 120, border: `3px solid ${p.mute}`, boxSizing: 'border-box', display: 'grid', placeItems: 'center', fontSize: 30}}>QR</div>}{code.label && <div style={{fontSize: 22 * scale, marginTop: 4}}>{code.label}</div>}</div>)}
      </footer></Reveal>
    </main>
  </div>;
}
