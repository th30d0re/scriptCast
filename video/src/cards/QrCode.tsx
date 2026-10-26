import qrcode from 'qrcode-generator';
import {palette as p} from '../theme';

// A real, scannable QR code drawn as inline SVG (dark modules on a light tile with a
// two-module quiet zone). Error correction is L to keep the module count low, since the
// code is read off a phone-sized video frame.
export const QR_MAX_URL_LENGTH = 130;
export function QrCode({url, size = 150}: {url: string; size?: number}) {
  if (!/^https?:\/\//.test(url) || url.length > QR_MAX_URL_LENGTH) {
    throw new Error(`qrUrl must be an http(s) URL of at most ${QR_MAX_URL_LENGTH} characters: ${url}`);
  }
  const qr = qrcode(0, 'L');
  qr.addData(url);
  qr.make();
  const n = qr.getModuleCount();
  const quiet = 2;
  let path = '';
  for (let r = 0; r < n; r++) for (let c = 0; c < n; c++) if (qr.isDark(r, c)) path += `M${c + quiet} ${r + quiet}h1v1h-1z`;
  const total = n + 2 * quiet;
  return <svg data-qr={url} width={size} height={size} viewBox={`0 0 ${total} ${total}`} shapeRendering="crispEdges" style={{display: 'block', borderRadius: 8}}>
    <rect width={total} height={total} fill={p.cream} />
    <path d={path} fill={p.navy} />
  </svg>;
}
