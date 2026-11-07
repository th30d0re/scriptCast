import React, {useLayoutEffect, useRef} from 'react';
import {useCurrentFrame} from 'remotion';
import {StageEpisode} from './StageEpisode';
import {collectMarks} from './review/ReviewMode';
import type {StagePlan} from './types';

declare global {
  interface Window {
    __scriptcastStageMarks?: {frame: number; marks: ReturnType<typeof collectMarks>};
    __scriptcastCollectStageMarks?: () => {frame: number; marks: ReturnType<typeof collectMarks>};
  }
}

/**
 * A render-only composition for headless inspection.  Keeping this alongside
 * StageEpisode means the boxes come from precisely the review-mode collector,
 * including declared ``data-box`` values and visibility filtering.
 */
export function StageMarks(plan: StagePlan) {
  const frame = useCurrentFrame();
  const root = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    window.__scriptcastCollectStageMarks = () => ({
      frame, marks: root.current ? collectMarks(root.current, plan.width, plan.height) : [],
    });
    if (root.current) {
      window.__scriptcastStageMarks = {
        frame,
        marks: collectMarks(root.current, plan.width, plan.height),
      };
    }
    return () => {delete window.__scriptcastCollectStageMarks;};
  }, [frame, plan.height, plan.width]);
  return <div ref={root} data-stage-marks-root style={{position: 'relative', width: plan.width, height: plan.height}}>
    <StageEpisode {...plan} />
  </div>;
}
