#!/usr/bin/env node
import {mkdirSync, readFileSync, renameSync, rmSync, writeFileSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
import {spawnSync} from 'node:child_process';

const [type, candidatePath, outPath] = process.argv.slice(2);
const usage = 'Usage: archify-layout.mjs <architecture|workflow> <candidate.json> <out.layout.json>';
const intersects = (a, b) => a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y;

function assertNoLabelOverlap(layout) {
  const clearance = 8 / Math.min(1700 / layout.viewBox[0], 700 / layout.viewBox[1]);
  for (const edge of layout.edges || []) {
    if (!edge.labelBox) continue;
    for (const node of layout.nodes || []) {
      const protectedNode = {x: node.x - clearance + 1e-7, y: node.y - clearance + 1e-7, w: node.w + 2 * clearance - 2e-7, h: node.h + 2 * clearance - 2e-7};
      if (intersects(edge.labelBox, protectedNode)) {
        throw new Error(`Edge label ${JSON.stringify(edge.label)} (${edge.id}) intersects node ${node.id}.`);
      }
    }
  }
}

// Adapter QA can exercise the normalized overlap guard without invoking Archify.
if (type === '--check-layout' && candidatePath && !outPath) {
  try {
    assertNoLabelOverlap(JSON.parse(readFileSync(candidatePath, 'utf8')));
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
  process.exit(0);
}
if (!['architecture', 'workflow'].includes(type) || !candidatePath || !outPath) {
  console.error(usage);
  process.exit(2);
}

const bin = process.env.ARCHIFY_BIN || '/Users/emmanuel/Dev/Tools/vendor/archify-a7ed6fd/archify/bin/archify.mjs';
let candidate;
try {
  candidate = JSON.parse(readFileSync(candidatePath, 'utf8'));
} catch (error) {
  console.error(`Cannot read candidate: ${error.message}`);
  process.exit(1);
}

const result = spawnSync(process.execPath, [bin, 'validate', type, candidatePath, '--layout-json'], {encoding: 'utf8'});
if (result.status !== 0) {
  process.stderr.write(result.stderr || result.stdout || 'Archify validation failed.\n');
  process.exit(result.status || 1);
}

let exported;
try {
  exported = JSON.parse(result.stdout);
} catch (error) {
  console.error(`Archify returned invalid layout JSON: ${error.message}`);
  process.exit(1);
}

const sourceNodes = new Map((candidate.nodes || candidate.components || []).map((node) => [node.id, node]));
const labelsByEdge = new Map((exported.labels || []).filter((label) => label.edge).map((label) => [label.edge, label]));
const nodeSource = type === 'workflow' ? exported.nodes : exported.components;
const edgeSource = type === 'workflow' ? exported.edges : exported.connections;
const groupSource = type === 'architecture' ? exported.boundaries : [];
const nodes = (nodeSource || []).map((node) => ({
  id: node.id,
  label: node.label || sourceNodes.get(node.id)?.label || node.id,
  x: node.x,
  y: node.y,
  w: node.width,
  h: node.height,
  ...(node.type ? {kind: node.type} : {}),
}));
const labelSegment = (points, anchor) => {
  let best;
  for (let index = 1; index < points.length; index += 1) {
    const [ax, ay] = points[index - 1], [bx, by] = points[index];
    const horizontal = ay === by;
    const vertical = ax === bx;
    if (!horizontal && !vertical) continue;
    const distance = horizontal ? Math.abs(anchor.y - ay) : Math.abs(anchor.x - ax);
    if (!best || distance < best.distance) best = {horizontal, ax, ay, bx, by, span: Math.abs((horizontal ? bx : by) - (horizontal ? ax : ay)), distance};
  }
  return best;
};
const scale = Math.min(1700 / exported.viewBox[0], 700 / exported.viewBox[1]);
const clearance = 8 / scale;
function placeLabel(edge, label) {
  if (!label) return {};
  const segment = labelSegment(edge.points || [], label);
  if (!segment) throw new Error(`Label ${label.label} on edge ${edge.id}: no straight segment available.`);
  const text = label.label || label.text || edge.label;
  let available = segment.span;
  const fit = (size) => {
    // IBM Plex Mono has a 0.6em advance. Include the entire background mask.
    const w = text.length * .6 * size + 6, h = size + 4;
    const y = label.y - size - 2;
    if (!segment.horizontal) {
      const box = {x: label.x - w / 2, y, w, h};
      return nodes.some(n => intersects(box, {x:n.x-clearance,y:n.y-clearance,w:n.w+2*clearance,h:n.h+2*clearance})) ? null : box;
    }
    let intervals = [[Math.min(segment.ax, segment.bx), Math.max(segment.ax, segment.bx)]];
    for (const n of nodes) {
      if (y + h <= n.y - clearance || y >= n.y + n.h + clearance) continue;
      const left = n.x - clearance, right = n.x + n.w + clearance;
      intervals = intervals.flatMap(([a,b]) => right <= a || left >= b ? [[a,b]] : [[a,Math.max(a,left)],[Math.min(b,right),b]].filter(([l,r]) => r > l));
    }
    const free = intervals.sort((a,b) => (b[1]-b[0])-(a[1]-a[0]))[0];
    available = free ? free[1]-free[0] : 0;
    if (!free || free[1]-free[0] + 1e-8 < w) return null;
    return {x:(free[0]+free[1]-w)/2,y,w,h};
  };
  let low = 12 / scale, high = segment.horizontal ? 15 : Math.min(15, 8 * segment.span / label.width);
  if (!fit(low)) throw new Error(`Label ${JSON.stringify(text)} on edge ${edge.id}: cannot fit 12px rendered; available space ${(available * scale).toFixed(2)} rendered px after node clearance (padding requires ${(6 * scale).toFixed(2)}px).`);
  for (let i=0; i<50; i++) { const mid=(low+high)/2; if (fit(mid)) low=mid; else high=mid; }
  return {labelBox:fit(low), labelSize:low, labelAt:[fit(low).x+fit(low).w/2, label.y]};
}
const edges = (edgeSource || []).map((edge, index) => {
  const label = labelsByEdge.get(edge.id);
  const placed = placeLabel(edge, label);
  return {
    id: edge.id || `${edge.from}-${edge.to}-${index}`,
    from: edge.from,
    to: edge.to,
    points: edge.points,
    ...(edge.label || label?.label || label?.text ? {label: edge.label || label?.label || label?.text} : {}),
    ...placed,
  };
});
const layout = {
  viewBox: exported.viewBox,
  nodes,
  edges,
  groups: groupSource.map((group) => ({label: group.label, x: group.x, y: group.y, w: group.width, h: group.height})),
};

if (!Array.isArray(layout.viewBox) || layout.viewBox.length !== 2 || !nodes.every((node) => Number.isFinite(node.x) && Number.isFinite(node.y) && Number.isFinite(node.w) && Number.isFinite(node.h)) || !edges.every((edge) => Array.isArray(edge.points))) {
  console.error('Archify layout is missing required geometry; no layout was written.');
  process.exit(1);
}

try {
  assertNoLabelOverlap(layout);
} catch (error) {
  console.error(`Archify layout label overlap: ${error.message} No layout was written.`);
  process.exit(1);
}

mkdirSync(dirname(resolve(outPath)), {recursive: true});
const temporary = `${outPath}.tmp-${process.pid}`;
try {
  writeFileSync(temporary, `${JSON.stringify(layout, null, 2)}\n`);
  renameSync(temporary, outPath);
} finally {
  rmSync(temporary, {force: true});
}
