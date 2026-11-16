/**Render one Stage scene at a given ms to static markup.

Invoked as a bundled script. The bundle imports the episode's ``scenes`` registry
and this file's ``renderFrames``. Python passes a job file:

    {plan, frames: [{scene, cue, ms, out}]}
*/
import React from 'react';
import {renderToStaticMarkup} from 'react-dom/server';
import fs from 'node:fs';
import {ClockContext, PlanContext, type SceneRegistry, type StagePlan} from '@stage';

export function renderFrames(scenes: SceneRegistry, plan: StagePlan, frames: {scene: string; ms: number; out: string}[]) {
  for (const frame of frames) {
    const Scene = scenes[frame.scene];
    if (!Scene) throw new Error(`scene ${frame.scene} is not in the episode registry`);
    const segment = plan.segments.find(item => item.scene === frame.scene);
    if (!segment) throw new Error(`scene ${frame.scene} is not in the plan`);
    const html = renderToStaticMarkup(
      <PlanContext.Provider value={plan}>
        <ClockContext.Provider value={{ms: frame.ms, fps: plan.fps, interactive: false}}>
          <Scene
            segment={segment}
            shots={segment.shots}
            shotAt={value => segment.shots.find(shot => value >= shot.start_ms && value < shot.end_ms)}
          />
        </ClockContext.Provider>
      </PlanContext.Provider>,
    );
    fs.mkdirSync(frame.out.slice(0, frame.out.lastIndexOf('/')), {recursive: true});
    fs.writeFileSync(frame.out, html);
  }
}

export function main(scenes: SceneRegistry) {
  const jobPath = process.argv[2];
  const job = JSON.parse(fs.readFileSync(jobPath, 'utf8')) as {plan: string; frames: {scene: string; ms: number; out: string}[]};
  const plan = JSON.parse(fs.readFileSync(job.plan, 'utf8')) as StagePlan;
  renderFrames(scenes, plan, job.frames);
}
