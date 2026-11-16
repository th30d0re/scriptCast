# Stage frame marks

`stage boxes PLAN --shot ID --at FRACTION -o boxes.json` extracts the visible
`data-text` and `data-shape` marks for the same frame calculation used by
`stage still`. `stage still ... --boxes` writes the companion
`<still>.boxes.json` after rendering its PNG.

The extractor uses Remotion 4.0.354's bundler and renderer browser. It serves
the generated bundle locally and drives its rendering API: `setPropsAndEnv`,
evaluation bundle mode, `waitForReady`, `remotion_calculateComposition`,
composition bundle mode, then `seekToFrame`. Evaluation mode mounts the root
and registers compositions before metadata is requested.
The seek waits for `remotion_renderReady` and fonts before collecting marks.
`StageMarks` calls `collectMarks()` directly; it does not repeat
the DOM/visibility or `data-box` logic in Node or Python.
The plan is injected before navigation using Remotion Page's
`evaluateOnNewDocument()` and `window.remotion_inputProps` (a JSON string).
The URL contains no props; composition and frame are set through the rendering
API. The internal helpers are resolved alongside the renderer's public entry
point because package exports hide dist subpaths. These calls are pinned to
4.0.354 and should be checked when upgrading Remotion. Failures print the last
200 captured browser console messages and page errors to stderr.
Dependency aliases use canonical physical paths and webpack symlink resolution,
matching remotion.config.ts even with a symlinked worktree node_modules.
Browser cleanup errors are reported separately without throwing over the
original failure. The script resolves the video workspace relative to itself,
so it can also be invoked from the repository root.

The emitted form is:

```json
{"frame": 54, "marks": [{"id": "title", "kind": "text", "box": {"x": 20, "y": 30, "w": 400, "h": 80}}]}
```

`scriptcast.tools.stage_critic` automatically uses matching sidecars next to
stills, or use `--boxes-dir DIR` to point at another directory. Their boxes are
normalised for tier 1 and their IDs are preserved in `critic.json`; a sidecar
with no marks intentionally does not fall back to the synthetic grid.
