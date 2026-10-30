import {useEffect, useRef, useState, type ReactNode} from 'react';
import {WIDTH, HEIGHT} from '@scriptcast/cards';

// Draws the 1080x1920 portrait card at full size and scales it to fit its box.
export function FitCard({children}: {children: ReactNode}) {
  const ref = useRef<HTMLDivElement>(null);
  const [scale, setScale] = useState(0);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const observer = new ResizeObserver(() => setScale(Math.min(el.clientWidth / WIDTH, el.clientHeight / HEIGHT)));
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  return <div ref={ref} className="fit">
    <div style={{width: WIDTH * scale, height: HEIGHT * scale, overflow: 'hidden', position: 'relative'}}>
      <div style={{width: WIDTH, height: HEIGHT, transform: `scale(${scale})`, transformOrigin: '0 0', position: 'absolute'}}>
        {children}
      </div>
    </div>
  </div>;
}
