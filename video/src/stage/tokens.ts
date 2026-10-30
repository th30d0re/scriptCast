import {createContext, useContext} from 'react';
import {palette, fontFamily} from '../theme';
export type Tokens = {
  colours: {background: string; surface: string; text: string; muted: string; accent: string; border: string; land: string; ocean: string};
  fonts: {body: string; heading: string}; sizes: {body: number; title: number; label: number};
  radii: {panel: number}; strokes: {border: number; map: number; route: number};
};
export const defaultTokens: Tokens = {colours: {background: palette.navy, surface: palette.navy2, text: palette.cream, muted: palette.mute, accent: palette.gold, border: palette.mute, land: palette.navy2, ocean: palette.navy}, fonts: {body: fontFamily, heading: fontFamily}, sizes: {body: 28, title: 80, label: 22}, radii: {panel: 16}, strokes: {border: 1, map: 0.8, route: 3}};
export const TokensContext = createContext<Tokens>(defaultTokens);
export const useTokens = () => useContext(TokensContext);
