# ScriptCast site

The ScriptCast cards (`packages/cards`) as a web page. Scroll drives each card's
`progress` (0 to 1) instead of the video frame clock.

- `#` (default): one sticky full-viewport section per card. `src/useScrollProgress.ts`
  turns the section's scroll position into `progress`. This path does not import `remotion`.
- `#player`: the `Timeline` composition in `@remotion/player` with
  `video/examples/timeline.example.json`, so you can scrub it.

Card assets come from `video/public` (see `vite.config.ts`).

```bash
npm install          # from the repo root (npm workspaces)
npm run dev -w site
npm run build -w site
```
