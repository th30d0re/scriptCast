import React, {createContext, useContext} from 'react';
import {useCurrentFrame, useVideoConfig} from 'remotion';
export const ClockContext = createContext({ms: 0, fps: 30, interactive: false});
export const useStageMs = () => useContext(ClockContext).ms;
export function RemotionClock({startMs, children}: React.PropsWithChildren<{startMs: number}>) {
  const frame = useCurrentFrame(), {fps} = useVideoConfig();
  return <ClockContext.Provider value={{ms: startMs + frame * 1000 / fps, fps, interactive: false}}>{children}</ClockContext.Provider>;
}
export function ManualClock({ms, fps = 30, interactive = true, children}: React.PropsWithChildren<{ms: number; fps?: number; interactive?: boolean}>) {
  return <ClockContext.Provider value={{ms, fps, interactive}}>{children}</ClockContext.Provider>;
}
