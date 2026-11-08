import {Player, type PlayerRef} from '@remotion/player';
import {useRef, useState} from 'react';
import {KitSheet} from '../../../video/src/stage/kit/KitSheet';
import {VARIANTS, type VariantName} from './useKitReview';

export function ItemPlayer({item, motion, variant, setVariant}: {item: string; motion?: boolean; variant: VariantName; setVariant: (v: VariantName) => void}) {
  const ref = useRef<PlayerRef>(null); const [speed, setSpeed] = useState(1);
  const step = (amount: number) => ref.current?.seekTo(Math.max(0, ref.current.getCurrentFrame() + amount));
  return <section className="kit-player"><Player ref={ref} component={KitSheet} inputProps={{item, live: true, variant: motion && variant !== 'default' ? variant : undefined}} durationInFrames={150} fps={30} compositionWidth={1920} compositionHeight={1080} loop controls playbackRate={speed} style={{width: '100%', aspectRatio: '16 / 9'}} />
    <div className="kit-controls"><button onClick={() => ref.current?.pause()}>Pause</button><button onClick={() => ref.current?.play()}>Play</button><button onClick={() => step(-1)}>← frame</button><button onClick={() => step(1)}>frame →</button><label>Speed <select value={speed} onChange={(event) => setSpeed(Number(event.target.value))}>{[.25, .5, 1].map((value) => <option key={value} value={value}>{value}×</option>)}</select></label>{motion && <label>Variant <select value={variant} onChange={(event) => setVariant(event.target.value as VariantName)}>{VARIANTS.map((value) => <option key={value} value={value}>{value}</option>)}</select></label>}</div>
  </section>;
}
