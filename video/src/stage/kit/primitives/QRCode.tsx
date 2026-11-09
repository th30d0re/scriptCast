import React, {useMemo} from 'react';
import qrcode from 'qrcode-generator';
import {useTiming, type Timing} from '../motion/core';
import {color} from '../theme/tokens';

export type QRCodeProps = Timing & {
  /** The URL encoded in the code. */
  url: string;
  /** Rendered square size in scene pixels. */
  size: number;
  /** Foreground colour token. */
  fg?: string;
  /** Background colour token. */
  bg?: string;
};

type Run = {x: number; y: number; width: number};

function qrRuns(url: string, reveal: number): {modules: number; runs: Run[]} {
  const code = qrcode(0, 'M');
  code.addData(url);
  code.make();
  const modules = code.getModuleCount();
  const revealed = Math.ceil(modules * modules * reveal);
  const runs: Run[] = [];

  // Merge adjacent modules in each row. Besides keeping the SVG small, integer
  // module coordinates let the renderer retain a reliably scannable grid.
  for (let y = 0; y < modules; y++) {
    let start = -1;
    for (let x = 0; x <= modules; x++) {
      const visible = x < modules && y * modules + x < revealed && code.isDark(y, x);
      if (visible && start < 0) start = x;
      if (!visible && start >= 0) {
        runs.push({x: start, y, width: x - start});
        start = -1;
      }
    }
  }
  return {modules, runs};
}

/** A crisp, self-contained QR SVG with a two-module quiet zone. */
export function QRCode({url, size, fg = 'color/text/primary', bg = 'color/fill/panel-render', progress, startFrame, durationFrames}: QRCodeProps) {
  const timed = progress !== undefined || startFrame !== undefined || durationFrames !== undefined;
  const {p} = useTiming({progress, startFrame, durationFrames}, 12);
  const {modules, runs} = useMemo(() => qrRuns(url, timed ? p : 1), [url, timed, p]);
  const quietZone = 2;
  const total = modules + quietZone * 2;

  return <svg width={size} height={size} viewBox={`0 0 ${total} ${total}`} role="img" aria-label={`QR code for ${url}`} shapeRendering="crispEdges">
    <rect width={total} height={total} fill={color(bg)}/>
    <g fill={color(fg)}>{runs.map((run) => <rect key={`${run.x}:${run.y}`} x={run.x + quietZone} y={run.y + quietZone} width={run.width} height={1}/>)}</g>
  </svg>;
}
