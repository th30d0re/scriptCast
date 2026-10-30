import {useEffect, useRef, useState, type ReactNode} from 'react';
import {CardFormatProvider, type CardFormat} from '@scriptcast/cards';

// Draws the card at its format's full size and scales it to fit its box.
export function FitCard({format, children}: {format: CardFormat; children: ReactNode}) {
  const ref = useRef<HTMLDivElement>(null);
  const [scale, setScale] = useState(0);
  const {width, height} = format;
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const fit = () => setScale(Math.min(el.clientWidth / width, el.clientHeight / height));
    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(el);
    return () => observer.disconnect();
  }, [width, height]);
  return <div ref={ref} className="fit">
    <div style={{width: width * scale, height: height * scale, overflow: 'hidden', position: 'relative'}}>
      <div style={{width, height, transform: `scale(${scale})`, transformOrigin: '0 0', position: 'absolute'}}>
        <CardFormatProvider value={format}>{children}</CardFormatProvider>
      </div>
    </div>
  </div>;
}
