#!/usr/bin/env node
import {mkdirSync, readFileSync, renameSync, rmSync, writeFileSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
import {spawnSync} from 'node:child_process';

const [type, candidatePath, outPath] = process.argv.slice(2);
const usage = 'Usage: archify-layout.mjs <architecture|workflow> <candidate.json> <out.layout.json>';
const intersects = (a, b) => a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y;
// Keep this in lockstep with ArchDiagram's terminal, open chevron.
const chevronBox = (points) => {
  if (!points || points.length < 2) return null;
  const end = points[points.length - 1], previous = points[points.length - 2];
  const length = Math.hypot(end[0] - previous[0], end[1] - previous[1]) || 1;
  const ux = (end[0] - previous[0]) / length, uy = (end[1] - previous[1]) / length;
  const wing = 8, depth = 13;
  const vertices = [[end[0] - ux * depth - uy * wing, end[1] - uy * depth + ux * wing], end, [end[0] - ux * depth + uy * wing, end[1] - uy * depth - ux * wing]];
  const xs = vertices.map(([x]) => x), ys = vertices.map(([, y]) => y);
  return {x: Math.min(...xs), y: Math.min(...ys), w: Math.max(...xs) - Math.min(...xs), h: Math.max(...ys) - Math.min(...ys)};
};
const expand = (box, amount) => ({x: box.x - amount, y: box.y - amount, w: box.w + 2 * amount, h: box.h + 2 * amount});

function assertNoLabelOverlap(layout) {
  const clearance = 8 / Math.min(1700 / layout.viewBox[0], 700 / layout.viewBox[1]);
  const chevrons = (layout.edges || []).map((edge) => ({edge, box: chevronBox(edge.points)})).filter(({box}) => box);
  for (const edge of layout.edges || []) {
    if (!edge.labelBox) continue;
    for (const node of layout.nodes || []) {
      const protectedNode = expand(node, clearance - 1e-7);
      if (intersects(edge.labelBox, protectedNode)) {
        throw new Error(`Edge label ${JSON.stringify(edge.label)} (${edge.id}) intersects node ${node.id}.`);
      }
    }
    for (const {edge: chevronEdge, box} of chevrons) {
      if (intersects(edge.labelBox, expand(box, clearance - 1e-7))) {
        throw new Error(`Edge label ${JSON.stringify(edge.label)} (${edge.id}) intersects chevron ${chevronEdge.id}.`);
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
const chevrons = (edgeSource || []).map((edge) => ({id: edge.id, box: chevronBox(edge.points)})).filter(({box}) => box);
function placeLabel(edge, label) {
  if (!label) return {};
  const segment = labelSegment(edge.points || [], label);
  if (!segment) throw new Error(`Label ${label.label} on edge ${edge.id}: no straight segment available.`);
  const text = label.label || label.text || edge.label;
  let available = segment.span;
  const fit = (size) => {
    // IBM Plex Mono has a 0.6em advance. Include the entire background mask.
    const w = text.length * .6 * size + 6, h = size + 4;
    let baseline = label.y;
    let y = baseline - size - 2;
    if (!segment.horizontal) {
      const box = {x: label.x - w / 2, y, w, h};
      return nodes.some(n => intersects(box, {x:n.x-clearance,y:n.y-clearance,w:n.w+2*clearance,h:n.h+2*clearance})) ? null : box;
    }
    // A horizontal label sits above its line. If its patch would touch a chevron,
    // lift it just enough to retain the requested chevron clearance; this keeps
    // narrow endpoint gaps usable before the 12 rendered-pixel floor is reached.
    for (const {box} of chevrons) {
      const protectedChevron = expand(box, clearance);
      const candidate = {x: Math.min(segment.ax, segment.bx), y, w: segment.span, h};
      if (intersects(candidate, protectedChevron)) {
        // Keep a tiny source-space separation so a later interval subtraction
        // does not turn an exact-clearance contact into a floating-point overlap.
        y = Math.min(y, protectedChevron.y - h - 1e-7);
        baseline = y + size + 2;
      }
    }
    // The patch must also remain at least 4 rendered px above its own line.
    y = Math.min(y, segment.ay - 4 / scale - h);
    baseline = y + size + 2;
    let intervals = [[Math.min(segment.ax, segment.bx), Math.max(segment.ax, segment.bx)]];
    for (const protectedBox of [...nodes.map((n) => expand(n, clearance)), ...chevrons.map(({box}) => expand(box, clearance))]) {
      if (y + h <= protectedBox.y || y >= protectedBox.y + protectedBox.h) continue;
      const left = protectedBox.x, right = protectedBox.x + protectedBox.w;
      intervals = intervals.flatMap(([a,b]) => right <= a || left >= b ? [[a,b]] : [[a,Math.max(a,left)],[Math.min(b,right),b]].filter(([l,r]) => r > l));
    }
    const free = intervals.sort((a,b) => (b[1]-b[0])-(a[1]-a[0]))[0];
    available = free ? free[1]-free[0] : 0;
    if (!free || free[1]-free[0] + 1e-8 < w) return null;
    return {x:(free[0]+free[1]-w)/2,y,w,h,baseline};
  };
  let low = 12 / scale, high = segment.horizontal ? 15 : Math.min(15, 8 * segment.span / label.width);
  if (!fit(low)) throw new Error(`Label ${JSON.stringify(text)} on edge ${edge.id}: cannot fit 12px rendered; available space ${(available * scale).toFixed(2)} rendered px after node and chevron clearance (padding requires ${(6 * scale).toFixed(2)}px).`);
  for (let i=0; i<50; i++) { const mid=(low+high)/2; if (fit(mid)) low=mid; else high=mid; }
  const placed = fit(low);
  return {labelBox:{x:placed.x,y:placed.y,w:placed.w,h:placed.h}, labelSize:low, labelAt:[placed.x+placed.w/2, segment.horizontal ? placed.baseline : label.y]};
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
