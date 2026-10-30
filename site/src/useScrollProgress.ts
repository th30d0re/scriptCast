import {useEffect, useRef, useState} from 'react';

// Progress 0 to 1 while a tall section scrolls past its sticky child: 0 when the
// section top meets the viewport top, 1 when its bottom meets the viewport bottom.
// The IntersectionObserver keeps the scroll listener off while the section is out of view.
export function useScrollProgress<T extends HTMLElement>() {
  const ref = useRef<T>(null);
  const [progress, setProgress] = useState(0);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    let frame = 0;
    const measure = () => {
      frame = 0;
      const rect = el.getBoundingClientRect();
      const travel = rect.height - innerHeight;
      const value = travel > 0 ? -rect.top / travel : rect.top <= 0 ? 1 : 0;
      setProgress(Math.min(1, Math.max(0, value)));
    };
    const onScroll = () => { if (!frame) frame = requestAnimationFrame(measure); };
    const observer = new IntersectionObserver(([entry]) => {
      if (entry.isIntersecting) {
        addEventListener('scroll', onScroll, {passive: true});
        addEventListener('resize', onScroll);
      } else {
        removeEventListener('scroll', onScroll);
        removeEventListener('resize', onScroll);
      }
      measure();
    });
    observer.observe(el);
    return () => {
      observer.disconnect();
      removeEventListener('scroll', onScroll);
      removeEventListener('resize', onScroll);
      cancelAnimationFrame(frame);
    };
  }, []);
  return {ref, progress};
}
