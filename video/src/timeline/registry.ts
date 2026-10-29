// The one place that maps timeline card names to components. When card imports
// move (e.g. to packages/cards), only this file changes.
import type {ComponentType} from 'react';
import {TimelineCard} from '../cards/TimelineCard';
import {StatBarsCard} from '../cards/StatBarsCard';
import {TitleCard} from '../cards/TitleCard';
import {CompareCard} from '../cards/CompareCard';
import {FreezeCallout} from '../cards/FreezeCallout';
import {PartsCard} from '../cards/PartsCard';

// Props are validated by ScriptCast before emit; the registry is untyped on purpose.
export const cardRegistry: Record<string, ComponentType<any>> = {
  TimelineCard, StatBarsCard, TitleCard, CompareCard, FreezeCallout, PartsCard,
};
