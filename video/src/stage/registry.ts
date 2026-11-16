import type React from 'react';
import type {Segment, Shot} from './types';
export type SceneProps = {segment: Segment; shots: Shot[]; shotAt: (ms: number) => Shot | undefined};
export type SceneRegistry = Record<string, React.FC<SceneProps>>;
