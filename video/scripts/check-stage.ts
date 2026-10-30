import assert from 'node:assert/strict';
import {tween, easing, settle, progress} from '../src/stage/motion';
import {lookupCue} from '../src/stage/plan';
import {resolveMode} from '../src/stage/Instrument';
import {createProjection} from '../src/stage/MapStage';
for(const ease of Object.values(easing)){assert(Math.abs(ease(0))<1e-8);assert(Math.abs(ease(1)-1)<1e-8);assert.equal(tween(-10,[{at:0,value:3},{at:100,value:8}],ease),3);assert.equal(tween(110,[{at:0,value:3},{at:100,value:8}],ease),8);}
assert.deepEqual(tween(50,[{at:0,value:[0,10]},{at:100,value:[10,30]}]),[5,20]);
assert.equal(progress(5,10,20),0);assert.equal(progress(25,10,20),1);
for(const damping of [10,20,30]){assert.equal(settle(-1,{stiffness:100,damping}),0);assert(Math.abs(settle(10000,{stiffness:100,damping})-1)<1e-6);}
assert.equal(lookupCue({hello:100},'hello'),100);assert.throws(()=>lookupCue({hello:100},'absent'),/Unknown cue.*absent.*Known ids: hello/);assert.throws(()=>lookupCue({},'toString'),/Unknown cue/);
const timeline=[{at:1000,mode:'expanded' as const},{at:3000,mode:'compact' as const}];
assert.equal(resolveMode(0,timeline).weights.compact,1);assert.equal(resolveMode(1200,timeline).weights.expanded,.5);assert.equal(resolveMode(2000,timeline).weights.expanded,1);assert.equal(resolveMode(4000,timeline).weights.compact,1);
const interrupted=[{at:0,mode:'expanded' as const},{at:200,mode:'settings' as const}];assert.equal(resolveMode(200,interrupted).weights.expanded,.5);assert.equal(resolveMode(600,interrupted).weights.settings,1);
for(const kind of ['naturalEarth','equirectangular','orthographic'] as const){const p=createProjection({center:[-8,39.5],zoom:8},kind,1920,1080);const [x,y]=p([-9.1393,38.7223])!;assert(x>0&&x<1920&&y>0&&y<1080,`${kind}: Lisbon ${x},${y}`);}
console.log('PASS: easing/tween endpoints, vectors, spring convergence, cue errors, timeline transitions, Lisbon projections (3 types).');
