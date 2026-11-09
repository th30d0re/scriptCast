#!/usr/bin/env node
/** Render looping Kit Review videos into video/public/kit-review. */
import {mkdir, readFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {bundle} from '@remotion/bundler';
import {renderMedia, selectComposition} from '@remotion/renderer';

const here = path.dirname(fileURLToPath(import.meta.url));
const video = path.resolve(here, '..');
const args = process.argv.slice(2);
const outDir = args.shift();
if (!outDir) throw new Error('Usage: node scripts/render-kit-videos.mjs <outDir> [--only id,id] [--browser-executable path]');

let only;
let browserExecutable;
while (args.length) {
  const arg = args.shift();
  if (arg === '--only') only = args.shift()?.split(',').filter(Boolean);
  else if (arg.startsWith('--only=')) only = arg.slice('--only='.length).split(',').filter(Boolean);
  else if (arg === '--browser-executable') browserExecutable = args.shift();
  else if (arg.startsWith('--browser-executable=')) browserExecutable = arg.slice('--browser-executable='.length);
  else throw new Error(`Unknown argument: ${arg}`);
}

// Keep this sourced from KitSheetIndex without requiring Node to load TSX.
const sheetSource = await readFile(path.join(video, 'src/stage/kit/KitSheet.tsx'), 'utf8');
const items = [...sheetSource.matchAll(/\['([^']+)',\s*'[^']+'(?:,\s*(true))?\]/g)]
  .map(([, id, motion]) => ({id, motion: motion === 'true'}));
const selected = only ? items.filter(({id}) => only.includes(id)) : items;
const missing = only?.filter((id) => !items.some((item) => item.id === id)) ?? [];
if (missing.length) throw new Error(`Unknown KitSheet item(s): ${missing.join(', ')}`);

const output = path.resolve(outDir);
await mkdir(output, {recursive: true});
const serveUrl = await bundle({entryPoint: path.join(video, 'src/index.ts')});
// Select per render: the composition carries its props, so one shared selection renders the same item every time.
const render = async (id, variant) => { const inputProps = {item: id, live: true, ...(variant ? {variant} : {})}; return renderMedia({
  composition: await selectComposition({serveUrl, id: 'KitSheetLive', inputProps, browserExecutable}),
  serveUrl,
  codec: 'h264',
  crf: 18,
  concurrency: 2,
  browserExecutable,
  outputLocation: path.join(output, `${id}${variant ? `.${variant}` : ''}.mp4`),
  inputProps,
}); };

for (const {id, motion} of selected) {
  await render(id);
  if (motion) for (const variant of ['opus', 'argon', 'astra', 'kimi']) await render(id, variant);
}
