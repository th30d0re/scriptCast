import {createContext, useContext, type ComponentType, type ImgHTMLAttributes} from 'react';

// What a card needs from the page that draws it. Remotion supplies staticFile, Img and
// delayRender; a website can keep the defaults (plain URLs, <img>, nothing to wait for).
export type CardHost = {
  // Public path under the host's static root (for example 'assets/x.svg') to a URL.
  asset: (src: string) => string;
  Image: ComponentType<ImgHTMLAttributes<HTMLImageElement> & {src: string}>;
  // Ask the host to wait (for a fetch or a layout pass); call the returned function when done.
  hold: (label?: string) => () => void;
};

export const defaultHost: CardHost = {
  asset: src => `/${src}`,
  Image: props => <img {...props} />,
  hold: () => () => {},
};

const HostContext = createContext<CardHost>(defaultHost);
export const CardHostProvider = HostContext.Provider;
export const useCardHost = () => useContext(HostContext);
