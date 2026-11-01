import React from 'react';
import {MapStage, MapMarker, MapRoute, Instrument, InstrumentRail, flyTo, cubicInOut, useStageMs, useCue, usePlan, WordHighlight} from '@stage';
export function DemoMap() {
  const ms = useStageMs(), zoom = useCue('portugal').at, expand = useCue('expand').at, collapse = useCue('collapse').at, plan = usePlan();
  const camera = flyTo(ms, [{at:zoom,camera:{center:[0,0],zoom:1}}, {at:zoom+1600,camera:{center:[-8,39.5],zoom:8}}], cubicInOut);
  return <><MapStage width={plan.width} height={plan.height} camera={camera} highlight={[{id:'620',amount:1}]} graticule><MapMarker at={[-9.1393,38.7223]} label="Lisbon" /><MapRoute from={[-9.1393,38.7223]} to={[2.35,48.85]} draw={Math.min(1,Math.max(0,(ms-zoom)/1600))} /></MapStage>
    <InstrumentRail width={plan.width} height={plan.height} slots={{topRight: compact => <Instrument grow="left-down" anchor={{compact, expanded:{x:0,y:0,width:600,height:360}}} modeTimeline={[{at:expand,mode:'expanded'},{at:collapse,mode:'compact'}]} compact={() => <strong>Portugal · 620</strong>} expanded={() => <><h2>A closer look</h2><WordHighlight turnId="map" /></>} settings={() => <><h3>Map settings</h3><p>Explore Portugal and its place in the world.</p></>} />}} />
  </>;
}
