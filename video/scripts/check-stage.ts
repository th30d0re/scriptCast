import assert from 'node:assert/strict';
import {geoDistance} from 'd3-geo';
import {tween, easing, settle, progress, flyTo} from '../src/stage/motion';
import {lookupCue} from '../src/stage/plan';
import {growRect, resolveMode} from '../src/stage/Instrument';
import {createProjection} from '../src/stage/MapStage';
import {parseMathText} from '../src/stage/MathText';
for(const ease of Object.values(easing)){assert(Math.abs(ease(0))<1e-8);assert(Math.abs(ease(1)-1)<1e-8);assert.equal(tween(-10,[{at:0,value:3},{at:100,value:8}],ease),3);assert.equal(tween(110,[{at:0,value:3},{at:100,value:8}],ease),8);}
assert.deepEqual(tween(50,[{at:0,value:[0,10]},{at:100,value:[10,30]}]),[5,20]);
assert.equal(progress(5,10,20),0);assert.equal(progress(25,10,20),1);
for(const damping of [10,20,30]){assert.equal(settle(-1,{stiffness:100,damping}),0);assert(Math.abs(settle(10000,{stiffness:100,damping})-1)<1e-6);}
assert.equal(lookupCue({hello:100},'hello'),100);assert.throws(()=>lookupCue({hello:100},'absent'),/Unknown cue.*absent.*Known ids: hello/);assert.throws(()=>lookupCue({},'toString'),/Unknown cue/);
const timeline=[{at:1000,mode:'expanded' as const},{at:3000,mode:'compact' as const}];
assert.equal(resolveMode(0,timeline).weights.compact,1);assert.equal(resolveMode(1200,timeline).weights.expanded,.5);assert.equal(resolveMode(2000,timeline).weights.expanded,1);assert.equal(resolveMode(4000,timeline).weights.compact,1);
const interrupted=[{at:0,mode:'expanded' as const},{at:200,mode:'settings' as const}];assert.equal(resolveMode(200,interrupted).weights.expanded,.5);assert.equal(resolveMode(600,interrupted).weights.settings,1);
const world:[number,number]=[0,0], portugal:[number,number]=[-8,39.5];
const flight=flyTo(800,[{at:0,camera:{center:world,zoom:1}},{at:1600,camera:{center:portugal,zoom:8}}]);
const corridorError=geoDistance(world,flight.center)+geoDistance(flight.center,portugal)-geoDistance(world,portugal);
{const keys=[{at:0,camera:{center:world,zoom:1}},{at:1600,camera:{center:portugal,zoom:8}}];const low=Math.min(...Array.from({length:33},(_,i)=>flyTo(i*50,keys).zoom));assert(low>=0.95,`flyTo zoomed out past the world view: min zoom ${low}`);}
assert(corridorError<1e-9,`flyTo left the great-circle corridor by ${corridorError}`);assert(flight.zoom<=8);assert.deepEqual(flyTo(0,[{at:0,camera:{center:world,zoom:1}},{at:1600,camera:{center:portugal,zoom:8}}]),{center:world,zoom:1});
for(const kind of ['naturalEarth','equirectangular','orthographic','mercator'] as const){const p=createProjection({center:[-8,39.5],zoom:8},kind,1920,1080);const [x,y]=p([-9.1393,38.7223])!;assert(x>0&&x<1920&&y>0&&y<1080,`${kind}: Lisbon ${x},${y}`);}
const fitted=createProjection({center:[0,0],pxPerDeg:10},'equirectangular',1920,1080);assert(Math.abs(fitted([1,0])![0]-fitted([0,0])![0]-10)<1e-8);
assert.deepEqual(growRect({x:100,y:100,width:40,height:20},{x:0,y:0,width:100,height:80},'left-up'),{x:40,y:40,width:100,height:80});
const mathCases:Record<string,ReturnType<typeof parseMathText>>={
  'ρ_τ':[{kind:'text',text:'ρ'},{kind:'sub',text:'τ'}],
  'Φ_{load}':[{kind:'text',text:'Φ'},{kind:'sub',text:'load'}],
  'O_{racialized}':[{kind:'text',text:'O'},{kind:'sub',text:'racialized'}],
  'W = jψ_s':[{kind:'text',text:'W = jψ'},{kind:'sub',text:'s'}],
  'ψ_m + jψ_s':[{kind:'text',text:'ψ'},{kind:'sub',text:'m'},{kind:'text',text:' + jψ'},{kind:'sub',text:'s'}]
};
for(const [source,expected] of Object.entries(mathCases))assert.deepEqual(parseMathText(source),expected,source);
assert.deepEqual(parseMathText('x^2 + y^{10}'),[{kind:'text',text:'x'},{kind:'sup',text:'2'},{kind:'text',text:' + y'},{kind:'sup',text:'10'}]);
console.log('PASS: motion/fly-to, springs, cues, instrument growth, four map projections + fit, and MathText parsing.');
