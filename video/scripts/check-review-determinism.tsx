import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import path from 'node:path';
import React from 'react';
import {renderToStaticMarkup} from 'react-dom/server';
import {DemoTitle} from '../episodes/_demo/DemoTitle';
import {ClockContext} from '../src/stage/clock';
import {PlanContext} from '../src/stage/plan';
import type {StagePlan} from '../src/stage/types';
import {nearestCue, sceneAt} from '../src/stage/review/cue';
import {nearestMark, parseBox, type Mark} from '../src/stage/review/hit';
import {parseNotes} from '../src/stage/review/schema';
import planJson from '../episodes/_demo/stage_plan.json';

const video = path.resolve(__dirname, '../..');
const plan = planJson as StagePlan;
const segment = plan.segments[0];
assert.equal(segment.scene, 'DemoTitle');

function renderScene(ms: number) {
  return renderToStaticMarkup(
    <PlanContext.Provider value={plan}>
      <ClockContext.Provider value={{ms, fps: plan.fps, interactive: false}}>
        <DemoTitle segment={segment} shots={segment.shots} shotAt={time => segment.shots.find(shot => time >= shot.start_ms && time < shot.end_ms)} />
      </ClockContext.Provider>
    </PlanContext.Provider>,
  );
}

const html = renderScene(2000);
const again = renderScene(2000);
assert.equal(again, html);
assert.doesNotMatch(html, /data-review|ReviewSession|ReviewHitSurface|ReviewChrome|review\/notes/);
const goldenPath = path.join(video, 'scripts/fixtures/demo-title-ms2000.html');
assert.equal(readFileSync(goldenPath, 'utf8'), html, 'render-path markup changed');

const renderPath = [
  'src/stage/StageEpisode.tsx',
  'src/stage/index.ts',
  'src/stage/Root.tsx',
  'src/stage/api.ts',
  'src/stage/clock.tsx',
  'src/stage/plan.tsx',
  'src/index.ts',
  'remotion.config.ts',
];
for (const file of renderPath) {
  const source = readFileSync(path.join(video, file), 'utf8');
  assert.doesNotMatch(source, /stage\/review|ReviewSession|ReviewHitSurface|ReviewChrome|ReviewMode/, `${file} reaches review mode`);
}
const player = readFileSync(path.join(video, 'site/main.tsx'), 'utf8');
assert.match(player, /ReviewSession/);
assert.match(player, /ReviewHitSurface/);
assert.doesNotMatch(readFileSync(path.join(video, 'src/stage/StageEpisode.tsx'), 'utf8'), /ReviewHitSurface/);

const notes = parseNotes(JSON.parse(readFileSync(path.join(video, '../review/notes.json'), 'utf8')));
assert.ok(notes);
assert.equal(notes.length, 2);
assert.deepEqual(notes.map(note => note.scene), ['APAGroupDefinitions', 'APAGroupDefinitions']);
assert.deepEqual(notes.map(note => note.status), ['open', 'open']);
assert.deepEqual(new Set(notes.map(note => note.element)).size, 2);

const card: Mark = {id: 'definition-term', kind: 'text', box: {x: 148, y: 224, w: 574, h: 48}};
const tray: Mark = {id: 'allocation-tray', kind: 'shape', box: {x: 830, y: 690, w: 960, h: 220}};
const field: Mark = {id: 'apa-field', kind: 'shape', box: {x: 830, y: 130, w: 960, h: 520}};
assert.equal(nearestMark([field, card], 200, 240)?.id, 'definition-term');
assert.equal(nearestMark([field, tray], 100, 100)?.id, 'apa-field');
assert.equal(nearestMark([], 0, 0), null);
assert.deepEqual(parseBox('10 20 30 40'), {x: 10, y: 20, w: 30, h: 40});
assert.equal(parseBox('nope'), null);

assert.equal(nearestCue({t21Bias: 255890, 'S00-21.t21Bias': 255890, t21Behavior: 263890}, 255890), 't21Bias');
assert.equal(nearestCue({t21Bias: 255890, t21Behavior: 263890}, 263000), 't21Behavior');
assert.equal(nearestCue({}, 0), '');
assert.equal(sceneAt(plan, 2000), 'DemoTitle');
assert.equal(sceneAt(plan, 4000), 'DemoMap');

const hash = createHash('sha256').update(html).digest('hex');
console.log(`PASS: review determinism ${html.length} bytes sha256 ${hash}`);
