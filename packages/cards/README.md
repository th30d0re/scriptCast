# @scriptcast/cards

The ScriptCast card components (Timeline, StatBars, Title, Compare, FreezeCallout, Parts),
plus the theme and the 1080×1920 safe zone. The same components render in the Remotion
video (`video/`) and on a website. This package does not import `remotion`.

## The `progress` contract

Every card takes its content props plus one number, `progress`:

- `progress` goes from `0` (nothing has entered) to `1` (the card is complete).
  Values outside that range are clamped. If you omit it, the card is drawn complete.
- The card reads no clock. It does not call `useCurrentFrame`, `useVideoConfig` or
  any timer. The same `progress` always gives the same picture.
- `progress` covers the card's **entrance** only (headline rule, staggered item fades,
  bar growth). After `1`, the card does not change.
- Inside a card, motion is written in *beats*: one beat is one frame at
  `ENTRANCE_FPS` (30). Each card exports its entrance length in beats
  (`timelineBeats(props)`, `statBarsBeats(props)`, `titleBeats`, `compareBeats`,
  `freezeBeats`, `partsBeats`). The length depends on the item count, so a five-item
  timeline has a longer entrance than a two-item one.

The caller picks the clock:

| Host    | `progress` |
|---------|------------|
| Remotion (`video/src/cards.tsx`) | `frame / (beats * fps / ENTRANCE_FPS)`, clamped; `1` in a one-frame still |
| Website | for example, how far the card has scrolled into view, 0 to 1 |

An animated SVG (`art.animate`) has its own CSS keyframes. It is held at
`art.atSeconds` (default `0`). The caller advances that value if it wants the SVG to play.

## The host

Cards get images, asset URLs and render holds from a `CardHost` in React context:

```ts
type CardHost = {
  asset: (src: string) => string;        // 'assets/x.svg' -> URL
  Image: ComponentType<img props>;       // the <img> to draw with
  hold: (label?: string) => () => void;  // wait for a fetch or a fit pass; call the result when done
};
```

The default host (`defaultHost`) uses `/<src>`, a plain `<img>` and no holds. That works
on a website that serves `video/public/` at its root. Remotion wraps each card in
`<CardHostProvider value={...}>` with `staticFile`, `Img` and `delayRender`.

## Use

```tsx
import {TimelineCard} from '@scriptcast/cards';

<TimelineCard {...content} progress={scrollProgress} />
```

Cards are 1080×1920 px. Scale the container (CSS `transform: scale(...)`) to fit a page.
`Frame` measures its content in the browser and lowers the type scale until it fits, so
render the cards at full size inside the scaled container.

## Build

Bundlers (Remotion's webpack, Vite) read `src/` directly. `npm run build -w @scriptcast/cards`
writes a CommonJS copy to `dist/` for Node scripts (the video copy check runs it).
