import {Player, type PlayerRef} from '@remotion/player';
import {useEffect, useRef, useState} from 'react';
import {KitSheet} from '../../../video/src/stage/kit/KitSheet';
import {VARIANTS, type VariantName} from './useKitReview';

export function ItemPlayer({item, motion, variant, setVariant}: {item: string; motion?: boolean; variant: VariantName; setVariant: (v: VariantName) => void}) {
  const liveRef = useRef<PlayerRef>(null); const videoRef = useRef<HTMLVideoElement>(null);
  const [speed, setSpeed] = useState(1); const [mode, setMode] = useState<'rendered' | 'live'>('rendered'); const [noRender, setNoRender] = useState(false);
  const renderedVariant = motion && variant !== 'default' ? `.${variant}` : '';
  const videoSrc = `/kit-review/${item}${renderedVariant}.mp4`;
  useEffect(() => { setMode('rendered'); setNoRender(false); }, [item, renderedVariant]);
  useEffect(() => { if (videoRef.current) videoRef.current.playbackRate = speed; }, [speed, mode]);
  const step = (amount: number) => {
    if (mode === 'live') liveRef.current?.seekTo(Math.max(0, liveRef.current.getCurrentFrame() + amount));
    else if (videoRef.current) videoRef.current.currentTime = Math.max(0, videoRef.current.currentTime + amount / 30);
  };
  const play = () => mode === 'live' ? liveRef.current?.play() : videoRef.current?.play();
  const pause = () => mode === 'live' ? liveRef.current?.pause() : videoRef.current?.pause();
  return <section className="kit-player">
    <div className="kit-mode" role="group" aria-label="Playback source"><button className={mode === 'rendered' ? 'selected' : ''} onClick={() => setMode('rendered')} disabled={noRender}>Rendered</button><button className={mode === 'live' ? 'selected' : ''} onClick={() => setMode('live')}>Live</button>{noRender && <small>no render yet</small>}</div>
    {mode === 'rendered' ? <video ref={videoRef} src={videoSrc} loop autoPlay muted playsInline controls onError={() => { setNoRender(true); setMode('live'); }} style={{width: '100%', aspectRatio: '16 / 9'}} /> : <Player ref={liveRef} component={KitSheet} inputProps={{item, live: true, variant: motion && variant !== 'default' ? variant : undefined}} durationInFrames={150} fps={30} compositionWidth={1920} compositionHeight={1080} loop controls playbackRate={speed} style={{width: '100%', aspectRatio: '16 / 9'}} />}
    <div className="kit-controls"><button onClick={pause}>Pause</button><button onClick={play}>Play</button><button onClick={() => step(-1)}>← frame</button><button onClick={() => step(1)}>frame →</button><label>Speed <select value={speed} onChange={(event) => setSpeed(Number(event.target.value))}>{[.25, .5, 1].map((value) => <option key={value} value={value}>{value}×</option>)}</select></label>{motion && <label>Variant <select value={variant} onChange={(event) => setVariant(event.target.value as VariantName)}>{VARIANTS.map((value) => <option key={value} value={value}>{value}</option>)}</select></label>}</div>
  </section>;
}
