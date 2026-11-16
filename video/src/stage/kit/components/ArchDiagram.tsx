import React from 'react';
import {color} from '../theme/tokens';
import {DrawLine, DrawRect} from '../motion';
import type {ArchLayout} from './types';

type Box = {x: number; y: number; w: number; h: number};
type Props = {layout: ArchLayout; box: Box; progress: number};

const stagger = (progress: number, start: number, span: number) => Math.max(0, Math.min(1, (progress - start) / span));
// The adapter uses these same wing/depth values when reserving label clearance.
const chevronGeometry = (points: readonly [number, number][]) => {
  const end = points[points.length - 1], previous = points[points.length - 2];
  const length = Math.hypot(end[0] - previous[0], end[1] - previous[1]) || 1;
  const ux = (end[0] - previous[0]) / length, uy = (end[1] - previous[1]) / length;
  const wing = 8, depth = 13;
  const vertices: [number, number][] = [[end[0] - ux * depth - uy * wing, end[1] - uy * depth + ux * wing], end, [end[0] - ux * depth + uy * wing, end[1] - uy * depth - ux * wing]];
  const xs = vertices.map(([vertexX]) => vertexX), ys = vertices.map(([, vertexY]) => vertexY);
  return {points: vertices.map(([vertexX, vertexY]) => `${vertexX},${vertexY}`).join(' '), box: {x: Math.min(...xs), y: Math.min(...ys), w: Math.max(...xs) - Math.min(...xs), h: Math.max(...ys) - Math.min(...ys)}};
};

/** Draws Archify's checked routing data in the Stage kit's visual language. */
export function ArchDiagram({layout, box, progress}: Props) {
  const [sourceW, sourceH] = layout.viewBox;
  const scale = Math.min(box.w / sourceW, box.h / sourceH);
  const x = box.x + (box.w - sourceW * scale) / 2;
  const y = box.y + (box.h - sourceH * scale) / 2;
  const groupProgress = stagger(progress, 0, .2);
  const nodeProgress = stagger(progress, .14, .42);
  const edgeProgress = stagger(progress, .5, .5);
  const reveal = (index: number, count: number, base: number) => stagger(base, count ? index / count * .42 : 0, .58);

  return <div style={{position: 'absolute', left: x, top: y, width: sourceW, height: sourceH, transform: `scale(${scale})`, transformOrigin: 'top left', pointerEvents: 'none'}}>
    {layout.groups.map((group, index) => {
      const p = reveal(index, layout.groups.length, groupProgress);
      return p > 0 ? <React.Fragment key={`${group.label}-${index}`}>
        <svg style={{position: 'absolute', left: group.x, top: group.y, overflow: 'visible'}} width={group.w} height={group.h} viewBox={`0 0 ${group.w} ${group.h}`}>
          <rect x="0" y="0" width={group.w} height={group.h} fill="none" stroke={color('color/line/hairline-strong')} strokeWidth="1" strokeDasharray="5 5" opacity={p}/>
        </svg>
        <div style={{position: 'absolute', left: group.x + 10, top: group.y - 17, padding: '0 5px', background: color('color/bg/field'), color: color('color/text/muted'), fontFamily: 'IBM Plex Mono', fontSize: 13, opacity: p}}>{group.label}</div>
      </React.Fragment> : null;
    })}
    {layout.nodes.map((node, index) => {
      const p = reveal(index, layout.nodes.length, nodeProgress);
      return <React.Fragment key={node.id}>
        <DrawRect box={{x: node.x, y: node.y, w: node.w, h: node.h}} progress={p} strokeToken="color/line/wire" fillToken="color/surface/1" glowToken="color/fill/gold" seed={index + 17}/>
        {p > 0 && <div style={{position: 'absolute', left: node.x + 10, top: node.y + 7, width: node.w - 20, height: node.h - 14, display: 'grid', placeItems: 'center', overflow: 'hidden', color: color('color/text/primary'), fontFamily: 'IBM Plex Mono', fontSize: 15, fontWeight: 500, lineHeight: 1.08, textAlign: 'center', opacity: p}}>{node.label}</div>}
      </React.Fragment>;
    })}
    {layout.edges.map((edge, index) => {
      const p = reveal(index, layout.edges.length, edgeProgress);
      const lineProgress = Math.min(1, p / .85);
      const finish = stagger(p, .85, .15);
      const chevron = chevronGeometry(edge.points);
      return <React.Fragment key={edge.id}>
        <DrawLine points={edge.points} end="none" progress={lineProgress} strokeToken="color/line/wire" glowToken="color/fill/gold" seed={index + 301}/>
        {finish > 0 && <svg style={{position: 'absolute', inset: 0, overflow: 'visible', pointerEvents: 'none'}} width={sourceW} height={sourceH}><polyline points={chevron.points} data-chevron-bounds={`${chevron.box.x},${chevron.box.y},${chevron.box.w},${chevron.box.h}`} fill="none" stroke={color('color/line/wire')} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" opacity={finish}/></svg>}
        {edge.label && edge.labelBox && edge.labelAt && finish > 0 && (() => {
          const size = edge.labelSize ?? 8;
          const [anchorX, baselineY] = edge.labelAt;
          return <svg style={{position: 'absolute', inset: 0, overflow: 'visible', pointerEvents: 'none'}} width={sourceW} height={sourceH} opacity={finish}>
            <rect x={edge.labelBox.x} y={edge.labelBox.y} width={edge.labelBox.w} height={edge.labelBox.h} fill={color('color/bg/field')}/>
            <text x={anchorX} y={baselineY} fill={color('color/text/muted')} fontFamily="IBM Plex Mono" fontSize={size} textAnchor="middle">{edge.label}</text>
          </svg>;
        })()}
      </React.Fragment>;
    })}
  </div>;
}
