import React from 'react';
import {useTokens} from './tokens';

export type GlassPanelProps = React.HTMLAttributes<HTMLDivElement> & {
  fill?: string;
  blur?: number;
  radius?: number;
  padding?: number | string;
};

export function GlassPanel({fill, blur = 18, radius, padding = 24, style, children, ...props}: GlassPanelProps) {
  const tokens = useTokens();
  return <div {...props} style={{background: fill ?? tokens.colours.glass, border: `1px solid ${tokens.colours.border}`, borderRadius: radius ?? tokens.radii.panel, padding, backdropFilter: `blur(${blur}px)`, WebkitBackdropFilter: `blur(${blur}px)`, boxSizing: 'border-box', ...style}}>{children}</div>;
}
