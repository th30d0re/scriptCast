import React, {createContext, useContext, useId, useMemo} from 'react';
import {geoEquirectangular, geoGraticule, geoGraticule10, geoInterpolate, geoMercator, geoNaturalEarth1, geoOrthographic, geoPath, type GeoProjection} from 'd3-geo';
import {feature, mesh} from 'topojson-client';
import type {Topology, GeometryCollection} from 'topojson-specification';
import atlas from 'world-atlas/countries-50m.json';
import {useTokens} from './tokens';
import {clamp} from './motion';

export type Camera = {center: [number, number]; zoom: number};
export type MapFit = {center: [number, number]; pxPerDeg: number};
export type ProjectionName = 'naturalEarth' | 'equirectangular' | 'orthographic' | 'mercator';
export type LandStyle = {fill: string; stroke: string; strokeWidth: number};
export type LineStyle = {stroke: string; strokeWidth: number; opacity: number};
export type GraticuleStyle = LineStyle & {step: number | [number, number]};
export type RhumbStyle = LineStyle & {centres: [number, number][]; spokes: number};
export type MapClip = {x: number; y: number; width: number; height: number};

function projectionFor(kind: ProjectionName) {
  return kind === 'naturalEarth' ? geoNaturalEarth1()
    : kind === 'orthographic' ? geoOrthographic()
    : kind === 'mercator' ? geoMercator()
    : geoEquirectangular();
}

function orientProjection(projection: GeoProjection, center: [number, number]) {
  if (projection.clipAngle() != null) projection.rotate([-center[0], -center[1]]);
  else projection.center(center);
}

/**
 * Zoom mode preserves the original "zoom 1 fits the world" contract. Fit mode
 * makes `pxPerDeg` exact at the equator for equirectangular and Mercator. For
 * Natural Earth and orthographic it is the local screen-space distance of one
 * longitude degree at `center`, which keeps authored regional diagrams stable.
 */
export function createProjection(view: Camera | MapFit, kind: ProjectionName, width: number, height: number): GeoProjection {
  if ('zoom' in view && view.zoom <= 0) throw new Error('Map zoom must be positive');
  if ('pxPerDeg' in view && view.pxPerDeg <= 0) throw new Error('Map pxPerDeg must be positive');
  const projection = projectionFor(kind).translate([width/2, height/2]);
  if ('zoom' in view) {
    projection.fitExtent([[8, 8], [width-8, height-8]], {type: 'Sphere'});
    projection.scale(projection.scale()*view.zoom);
    orientProjection(projection, view.center);
    return projection.translate([width/2, height/2]);
  }
  orientProjection(projection, view.center);
  if (kind === 'equirectangular' || kind === 'mercator') {
    projection.scale(view.pxPerDeg*180/Math.PI);
  } else {
    projection.scale(1);
    const left = projection([view.center[0]-0.5, view.center[1]]);
    const right = projection([view.center[0]+0.5, view.center[1]]);
    const localPxPerDeg = left && right ? Math.hypot(right[0]-left[0], right[1]-left[1]) : 0;
    if (!(localPxPerDeg > 0)) throw new Error(`Cannot fit ${kind} projection at ${view.center.join(',')}`);
    projection.scale(view.pxPerDeg/localPxPerDeg);
  }
  return projection.translate([width/2, height/2]);
}

const MapContext = createContext<GeoProjection | null>(null);
function useProjection() {const projection = useContext(MapContext); if (!projection) throw new Error('Map children require MapStage'); return projection;}
export function useProject(): (point: [number, number]) => [number, number] {const projection = useProjection(); return point => projection(point) ?? [NaN, NaN];}

const world = atlas as unknown as Topology<{countries: GeometryCollection}>;
const countries = feature(world, world.objects.countries).features;
const countryBorders = mesh(world, world.objects.countries, (a, b) => a !== b);

type MapStageBaseProps = {
  projection?: ProjectionName;
  highlight?: {id: string; color?: string; amount: number}[];
  land?: LandStyle;
  borders?: LineStyle;
  graticule?: boolean | GraticuleStyle;
  rhumb?: RhumbStyle;
  clip?: MapClip;
  width?: number;
  height?: number;
};
export type MapStageProps = React.PropsWithChildren<MapStageBaseProps & (
  {camera: Camera; fit?: never} | {camera?: never; fit: MapFit}
)>;

function rayEnd(x: number, y: number, angle: number, width: number, height: number): [number, number] {
  const dx = Math.cos(angle), dy = Math.sin(angle);
  const tx = dx > 0 ? (width-x)/dx : dx < 0 ? -x/dx : Infinity;
  const ty = dy > 0 ? (height-y)/dy : dy < 0 ? -y/dy : Infinity;
  const distance = Math.max(0, Math.min(tx, ty));
  return [x+dx*distance, y+dy*distance];
}

export function MapStage({camera, fit, projection = 'naturalEarth', highlight = [], land, borders, graticule = false, rhumb, clip, width = 1920, height = 1080, children}: MapStageProps) {
  const tokens = useTokens();
  const view = camera ?? fit;
  if (!view) throw new Error('MapStage requires camera or fit');
  const center0 = view.center[0], center1 = view.center[1];
  const viewMode = 'zoom' in view ? 'zoom' : 'fit';
  const scale = viewMode === 'zoom' ? (view as Camera).zoom : (view as MapFit).pxPerDeg;
  const projected = useMemo(() => createProjection(view, projection, width, height), [center0, center1, scale, viewMode, projection, width, height]);
  const path = geoPath(projected);
  const clipId = `stage-map-clip-${useId().replace(/:/g, '')}`;
  const graticuleGeometry = typeof graticule === 'object'
    ? geoGraticule().step(Array.isArray(graticule.step) ? graticule.step : [graticule.step, graticule.step])()
    : geoGraticule10();
  const landStyle = land ?? {fill: tokens.colours.land, stroke: tokens.colours.border, strokeWidth: tokens.strokes.map};
  return <MapContext.Provider value={projected}><svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} style={{position: 'absolute', inset: 0, background: clip ? 'transparent' : tokens.colours.ocean}}>
    {clip && <defs><clipPath id={clipId}><rect {...clip} /></clipPath></defs>}
    <g clipPath={clip ? `url(#${clipId})` : undefined}>
      <path d={path({type: 'Sphere'}) ?? ''} fill={tokens.colours.ocean} />
      {countries.map(country => {
        const emphasis = highlight.find(item => Number(item.id) === Number(country.id));
        return <g key={country.id}><path d={path(country) ?? ''} fill={landStyle.fill} stroke={landStyle.stroke} strokeWidth={landStyle.strokeWidth} />{emphasis && <path d={path(country) ?? ''} fill={emphasis.color ?? tokens.colours.accent} opacity={clamp(emphasis.amount)} />}</g>;
      })}
      {borders && <path d={path(countryBorders) ?? ''} fill="none" stroke={borders.stroke} strokeWidth={borders.strokeWidth} opacity={borders.opacity} />}
      {graticule && <path d={path(graticuleGeometry) ?? ''} fill="none" stroke={typeof graticule === 'object' ? graticule.stroke : tokens.colours.muted} opacity={typeof graticule === 'object' ? graticule.opacity : 0.25} strokeWidth={typeof graticule === 'object' ? graticule.strokeWidth : tokens.strokes.map} />}
      {rhumb && rhumb.centres.flatMap((centre, centreIndex) => {
        const start = projected(centre);
        if (!start || rhumb.spokes <= 0) return [];
        return Array.from({length: Math.floor(rhumb.spokes)}, (_, spokeIndex) => {
          const end = rayEnd(start[0], start[1], 2*Math.PI*spokeIndex/rhumb.spokes, width, height);
          return <line key={`${centreIndex}:${spokeIndex}`} x1={start[0]} y1={start[1]} x2={end[0]} y2={end[1]} stroke={rhumb.stroke} strokeWidth={rhumb.strokeWidth} opacity={rhumb.opacity} />;
        });
      })}
      {children}
    </g>
  </svg></MapContext.Provider>;
}

export function MapMarker({at, label, radius = 8}: {at: [number, number]; label?: string; radius?: number}) {
  const [x, y] = useProject()(at), tokens = useTokens();
  return <g transform={`translate(${x},${y})`}><circle r={radius} fill={tokens.colours.accent} />{label && <text x={radius+8} fill={tokens.colours.text} fontSize={tokens.sizes.label}>{label}</text>}</g>;
}

export function MapRoute({from, to, draw, color}: {from: [number, number]; to: [number, number]; draw: number; color?: string}) {
  const projection = useProjection(), tokens = useTokens(), interpolate = geoInterpolate(from, to);
  const amount = clamp(draw), points = Array.from({length: 65}, (_, i) => interpolate(amount*i/64));
  return <path d={geoPath(projection)({type: 'LineString', coordinates: points}) ?? ''} fill="none" stroke={color ?? tokens.colours.accent} strokeWidth={tokens.strokes.route} opacity={amount > 0 ? 1 : 0} />;
}

export type CompassRoseProps = {
  centre?: [number, number];
  center?: [number, number];
  radius: number;
  stroke?: string;
  fill?: string;
  labelColor?: string;
};
export function CompassRose({centre, center, radius, stroke, fill = 'none', labelColor}: CompassRoseProps) {
  const tokens = useTokens(), [x, y] = centre ?? center ?? [0, 0];
  const line = stroke ?? tokens.colours.text, text = labelColor ?? line;
  const points = Array.from({length: 32}, (_, i) => {
    const angle = -Math.PI/2+i*Math.PI/16;
    const pointRadius = i%2 === 0 ? radius : radius*0.32;
    return `${x+Math.cos(angle)*pointRadius},${y+Math.sin(angle)*pointRadius}`;
  }).join(' ');
  return <g aria-label="Compass rose">
    <circle cx={x} cy={y} r={radius} fill="none" stroke={line} strokeWidth={tokens.strokes.map} />
    <circle cx={x} cy={y} r={radius*0.58} fill="none" stroke={line} strokeWidth={tokens.strokes.map} />
    <polygon points={points} fill={fill} stroke={line} strokeWidth={tokens.strokes.border} />
    <text x={x} y={y-radius-10} textAnchor="middle" fill={text} fontFamily={tokens.fonts.sans} fontSize={tokens.sizes.label} fontWeight={600}>N</text>
  </g>;
}

type Point = [number, number];
type Cubic = {from: Point; c1: Point; c2: Point; to: Point};
function splineSegments(points: Point[], tension: number): Cubic[] {
  const factor = (1-clamp(tension))/6;
  return points.slice(0, -1).map((from, i) => {
    const before = points[Math.max(0, i-1)], to = points[i+1], after = points[Math.min(points.length-1, i+2)];
    return {from, c1: [from[0]+(to[0]-before[0])*factor, from[1]+(to[1]-before[1])*factor], c2: [to[0]-(after[0]-from[0])*factor, to[1]-(after[1]-from[1])*factor], to};
  });
}
function cubicPoint(segment: Cubic, t: number): Point {
  const u = 1-t;
  return [u**3*segment.from[0]+3*u*u*t*segment.c1[0]+3*u*t*t*segment.c2[0]+t**3*segment.to[0], u**3*segment.from[1]+3*u*u*t*segment.c1[1]+3*u*t*t*segment.c2[1]+t**3*segment.to[1]];
}
export function catmullRomPath(points: Point[], tension = 0): {d: string; length: number} {
  if (!points.length) return {d: '', length: 0};
  if (points.length === 1) return {d: `M${points[0][0]},${points[0][1]}`, length: 0};
  const segments = splineSegments(points, tension);
  const d = `M${points[0][0]},${points[0][1]}${segments.map(segment => ` C${segment.c1[0]},${segment.c1[1]} ${segment.c2[0]},${segment.c2[1]} ${segment.to[0]},${segment.to[1]}`).join('')}`;
  let length = 0;
  for (const segment of segments) {
    let previous = segment.from;
    for (let sample = 1; sample <= 32; sample++) {
      const point = cubicPoint(segment, sample/32);
      length += Math.hypot(point[0]-previous[0], point[1]-previous[1]);
      previous = point;
    }
  }
  return {d, length};
}

export type RouteProps = {
  waypoints: [number, number][];
  tension?: number;
  draw: number;
  markers?: number[];
  markerIndices?: number[];
  stroke?: string;
  strokeWidth?: number;
  markerRadius?: number;
};
export function Route({waypoints, tension = 0, draw, markers, markerIndices, stroke, strokeWidth, markerRadius = 6}: RouteProps) {
  const projection = useProjection(), tokens = useTokens();
  const points = waypoints.map(point => projection(point)).filter((point): point is [number, number] => point != null);
  const route = catmullRomPath(points, tension), amount = clamp(draw), chosen = markers ?? markerIndices ?? [];
  const color = stroke ?? tokens.colours.accent;
  return <g>
    <path d={route.d} fill="none" stroke={color} strokeWidth={strokeWidth ?? tokens.strokes.route} strokeDasharray={`${route.length} ${route.length}`} strokeDashoffset={route.length*(1-amount)} opacity={amount > 0 ? 1 : 0} />
    {chosen.map(index => points[index] ? <circle key={index} cx={points[index][0]} cy={points[index][1]} r={markerRadius} fill={color} /> : null)}
  </g>;
}
