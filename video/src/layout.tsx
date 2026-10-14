import {createContext, useContext} from 'react';
import {useVideoConfig} from 'remotion';
import {layoutFromDimensions, type Layout} from './safeZone';

export const LayoutContext = createContext<Layout | null>(null);
export function useLayout(): Layout {
  const explicit = useContext(LayoutContext);
  let inferred: Layout = 'portrait';
  try {
    const {width, height} = useVideoConfig();
    inferred = layoutFromDimensions(width, height);
  } catch { /* Static copy checks use the portrait default. */ }
  return explicit ?? inferred;
}
