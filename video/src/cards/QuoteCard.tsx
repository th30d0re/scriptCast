import {Frame} from './Frame';
import {palette as p} from '../theme';
export type QuoteProps = {headline?: string; quote: string; speaker: string; work: string; year?: string | number; sources: string; qrUrl?: string};
export function QuoteCard(props: QuoteProps) {
  return <Frame {...props} headline={props.headline ?? ''} qr={Boolean(props.qrUrl)}>{scale => <figure style={{margin: 0, display: 'flex', flexDirection: 'column', gap: 36 * scale}}>
    <blockquote style={{margin: 0, borderLeft: `5px solid ${p.gold}`, paddingLeft: 28 * scale, fontSize: 48 * scale, lineHeight: 1.3}}>“{props.quote}”</blockquote>
    <figcaption style={{fontSize: 30 * scale, lineHeight: 1.3, color: p.mute}}><strong style={{color: p.gold}}>{props.speaker}</strong><br /><cite>{props.work}</cite>{props.year !== undefined && ` (${props.year})`}</figcaption>
  </figure>}</Frame>;
}
