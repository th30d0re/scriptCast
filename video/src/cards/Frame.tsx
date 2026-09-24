import type {ReactNode} from 'react';
import {USABLE_BOX, WIDTH, HEIGHT} from '../safeZone';
import {palette as p, fontFamily} from '../theme';
export type BaseProps = {headline: string; sources: string; svgAsset?: string};
// Reserved for Phase 4. Accept only public-relative paths; no external resources.
export function Frame({headline, sources, svgAsset, children}: BaseProps & {children: ReactNode}) {
  if (svgAsset && (/^(\/|[a-z]+:)/i.test(svgAsset) || svgAsset.split('/').includes('..'))) {
    throw new Error('svgAsset must be a relative path under public/');
  }
  return <div style={{width: WIDTH, height: HEIGHT, background: p.navy, color: p.cream, fontFamily}}>
    <main style={{position: 'absolute', left: USABLE_BOX.x, top: USABLE_BOX.y, width: USABLE_BOX.width, height: USABLE_BOX.height, boxSizing: 'border-box', padding: 12, overflow: 'hidden', display: 'flex', flexDirection: 'column', gap: 12}}>
      <h1 style={{margin: 0, fontSize: 44, lineHeight: 1.08, fontWeight: 800, textAlign: 'center'}}>{headline}</h1>
      <div style={{width: 160, height: 3, background: p.gold, alignSelf: 'center', flexShrink: 0}} />
      <section style={{flex: 1, minHeight: 0, overflow: 'hidden'}}>{children}</section>
      <footer style={{textAlign: 'center', flexShrink: 0}}>
        <div style={{fontSize: 18, lineHeight: 1.2, color: p.mute}}>{sources}</div>
        <div style={{boxSizing: 'border-box', width: 120, height: 120, border: `3px solid ${p.mute}`, borderRadius: 10, margin: '8px auto 0', display: 'flex', alignItems: 'center', justifyContent: 'center', color: p.mute, fontSize: 30, fontWeight: 700}}>QR</div>
      </footer>
    </main>
  </div>;
}
