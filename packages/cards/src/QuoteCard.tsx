import {Frame} from './Frame';
import {FRAME_BEATS} from './motion';
import {palette as p} from './theme';
export type QuoteProps = {progress?: number; headline?: string; quote: string; speaker: string; work: string; year?: string | number; sources: string; qrUrl?: string};
export const quoteBeats = (_props: QuoteProps) => FRAME_BEATS;
export function QuoteCard(props: QuoteProps) {
  return <Frame {...props} beats={quoteBeats(props)} headline={props.headline ?? ''} qr={Boolean(props.qrUrl)}>{scale => <figure style={{margin: 0, display: 'flex', flexDirection: 'column', gap: 36 * scale}}>
    <blockquote style={{margin: 0, borderLeft: `5px solid ${p.gold}`, paddingLeft: 28 * scale, fontSize: 48 * scale, lineHeight: 1.3}}>“{props.quote}”</blockquote>
    <figcaption style={{fontSize: 30 * scale, lineHeight: 1.3, color: p.mute}}><strong style={{color: p.gold}}>{props.speaker}</strong><br /><cite>{props.work}</cite>{props.year !== undefined && ` (${props.year})`}</figcaption>
  </figure>}</Frame>;
}
