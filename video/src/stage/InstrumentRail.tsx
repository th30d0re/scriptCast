import React from 'react';
import type {Rect} from './Instrument';
export type RailSlot = 'topLeft' | 'top' | 'topRight' | 'left' | 'right' | 'bottomLeft' | 'bottom' | 'bottomRight';
export function railLayout(width: number, height: number, panelWidth = 300, panelHeight = 140, margin = 32, gap = 24): Record<RailSlot, Rect> {
  const w = Math.min(panelWidth, (width-2*margin-2*gap)/3), h = Math.min(panelHeight, (height-2*margin-2*gap)/3);
  const xs = [margin, (width-w)/2, width-margin-w], ys = [margin, (height-h)/2, height-margin-h];
  const slots: [RailSlot, number, number][] = [['topLeft',0,0],['top',1,0],['topRight',2,0],['left',0,1],['right',2,1],['bottomLeft',0,2],['bottom',1,2],['bottomRight',2,2]];
  return Object.fromEntries(slots.map(([name,x,y]) => [name,{x:xs[x], y:ys[y], width:w, height:h}])) as Record<RailSlot,Rect>;
}
export function InstrumentRail({width = 1920, height = 1080, margin = 32, gap = 24, panelWidth = 300, panelHeight = 140, slots, children}: React.PropsWithChildren<{width?: number; height?: number; margin?: number; gap?: number; panelWidth?: number; panelHeight?: number; slots: Partial<Record<RailSlot, (rect: Rect) => React.ReactNode>>}>) {
  const layout = railLayout(width,height,panelWidth,panelHeight,margin,gap);
  return <>{children}{(Object.keys(slots) as RailSlot[]).map(name => <React.Fragment key={name}>{slots[name]?.(layout[name])}</React.Fragment>)}</>;
}
