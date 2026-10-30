import React, {createContext, useContext, useMemo} from 'react';
import {geoEquirectangular, geoNaturalEarth1, geoOrthographic, geoPath, geoGraticule10, geoInterpolate, type GeoProjection} from 'd3-geo';
import {feature} from 'topojson-client';
import type {Topology, GeometryCollection} from 'topojson-specification';
import atlas from 'world-atlas/countries-50m.json';
import {useTokens} from './tokens';
import {clamp} from './motion';
export type Camera = {center: [number, number]; zoom: number};
export type ProjectionName = 'naturalEarth' | 'equirectangular' | 'orthographic';
export function createProjection(camera: Camera, kind: ProjectionName, width: number, height: number): GeoProjection {
  const p = kind === 'naturalEarth' ? geoNaturalEarth1() : kind === 'orthographic' ? geoOrthographic() : geoEquirectangular();
  p.fitExtent([[8, 8], [width-8, height-8]], {type: 'Sphere'});
  p.scale(p.scale()*camera.zoom);
  if (kind === 'orthographic') p.rotate([-camera.center[0], -camera.center[1]]);
  else p.center(camera.center);
  return p.translate([width/2, height/2]);
}
const MapContext = createContext<GeoProjection | null>(null);
function useProjection() {const p = useContext(MapContext); if (!p) throw new Error('Map children require MapStage'); return p;}
export function useProject(): (point: [number, number]) => [number, number] {const p = useProjection(); return point => p(point) ?? [NaN, NaN];}
const world = atlas as unknown as Topology<{countries: GeometryCollection}>;
const countries = feature(world, world.objects.countries).features;
export function MapStage({camera, projection = 'naturalEarth', highlight = [], graticule = false, width = 1920, height = 1080, children}: React.PropsWithChildren<{camera: Camera; projection?: ProjectionName; highlight?: {id: string; color?: string; amount: number}[]; graticule?: boolean; width?: number; height?: number}>) {
  const tokens = useTokens(), p = useMemo(() => createProjection(camera, projection, width, height), [camera.center[0], camera.center[1], camera.zoom, projection, width, height]), path = geoPath(p);
  return <MapContext.Provider value={p}><svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} style={{position: 'absolute', inset: 0, background: tokens.colours.ocean}}>
    <path d={path({type: 'Sphere'}) ?? ''} fill={tokens.colours.ocean} />
    {countries.map(c => {const h = highlight.find(h => Number(h.id) === Number(c.id)); return <g key={c.id}><path d={path(c) ?? ''} fill={tokens.colours.land} stroke={tokens.colours.border} strokeWidth={tokens.strokes.map} />{h && <path d={path(c) ?? ''} fill={h.color ?? tokens.colours.accent} opacity={clamp(h.amount)} />}</g>;})}
    {graticule && <path d={path(geoGraticule10()) ?? ''} fill="none" stroke={tokens.colours.muted} opacity={0.25} strokeWidth={tokens.strokes.map} />}{children}
  </svg></MapContext.Provider>;
}
export function MapMarker({at, label, radius = 8}: {at: [number, number]; label?: string; radius?: number}) {
  const [x, y] = useProject()(at), tokens = useTokens();
  return <g transform={`translate(${x},${y})`}><circle r={radius} fill={tokens.colours.accent} />{label && <text x={radius+8} fill={tokens.colours.text} fontSize={tokens.sizes.label}>{label}</text>}</g>;
}
export function MapRoute({from, to, draw, color}: {from: [number, number]; to: [number, number]; draw: number; color?: string}) {
  const p = useProjection(), tokens = useTokens(), interpolate = geoInterpolate(from, to);
  const amount = clamp(draw), points = Array.from({length: 65}, (_, i) => interpolate(amount*i/64));
  return <path d={geoPath(p)({type: 'LineString', coordinates: points}) ?? ''} fill="none" stroke={color ?? tokens.colours.accent} strokeWidth={tokens.strokes.route} opacity={amount > 0 ? 1 : 0} />;
}
